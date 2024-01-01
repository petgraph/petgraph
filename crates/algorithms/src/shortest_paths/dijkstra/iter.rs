use alloc::vec::Vec;
use core::mem;

use error_stack::{Report, Result};
use numi::num::{identity::Zero, ops::AddRef};
use petgraph_core::{
    node::NodeId,
    storage::{
        auxiliary::{FrequencyHint, Hints, OccupancyHint, PerformanceHint, SecondaryGraphStorage},
        AuxiliaryGraphStorage,
    },
    Graph, GraphStorage, Node,
};

use crate::shortest_paths::{
    common::{
        connections::Connections,
        cost::{Cost, GraphCost},
        path::Path,
        queue::priority::{PriorityQueue, PriorityQueueItem},
        route::Route,
        transit::{reconstruct_path_to, PredecessorMode},
    },
    dijkstra::{measure::DijkstraMeasure, DijkstraError},
};

pub(super) struct DijkstraIter<'graph: 'parent, 'parent, S, E, G>
where
    S: AuxiliaryGraphStorage,
    E: GraphCost<S>,
    E::Value: DijkstraMeasure,
{
    graph: &'graph Graph<S>,
    queue: PriorityQueue<'graph, S, E::Value>,

    edge_cost: &'parent E,
    connections: G,

    source: Node<'graph, S>,

    num_nodes: usize,

    init: bool,
    next: Option<PriorityQueueItem<E::Value>>,

    predecessor_mode: PredecessorMode,

    distances: S::SecondaryNodeStorage<'graph, E::Value>,
    predecessors: S::SecondaryNodeStorage<'graph, Option<NodeId>>,
}

impl<'graph: 'parent, 'parent, S, E, G> DijkstraIter<'graph, 'parent, S, E, G>
where
    S: AuxiliaryGraphStorage,
    E: GraphCost<S>,
    E::Value: DijkstraMeasure,
    G: Connections<'graph, S>,
{
    pub(super) fn new(
        graph: &'graph Graph<S>,

        edge_cost: &'parent E,
        connections: G,

        source: NodeId,

        predecessor_mode: PredecessorMode,
    ) -> Result<Self, DijkstraError> {
        let source_node = graph
            .node(source)
            .ok_or_else(|| Report::new(DijkstraError::NodeNotFound))?;

        let queue = PriorityQueue::new(graph.storage());

        let mut distances = graph.storage().secondary_node_storage(Hints {
            performance: PerformanceHint {
                read: FrequencyHint::Frequent,
                write: FrequencyHint::Frequent,
            },
            occupancy: OccupancyHint::Dense,
        });
        distances.set(source, E::Value::zero());

        let mut predecessors = graph.storage().secondary_node_storage(Hints {
            performance: PerformanceHint {
                read: FrequencyHint::Infrequent,
                write: FrequencyHint::Frequent,
            },
            occupancy: OccupancyHint::Dense,
        });
        if predecessor_mode == PredecessorMode::Record {
            predecessors.set(source, None);
        }

        Ok(Self {
            graph,
            queue,
            edge_cost,
            connections,
            source: source_node,
            num_nodes: graph.num_nodes(),
            init: true,
            next: None,
            predecessor_mode,
            distances,
            predecessors,
        })
    }
}

impl<'graph: 'parent, 'parent, S, E, G> Iterator for DijkstraIter<'graph, 'parent, S, E, G>
where
    S: AuxiliaryGraphStorage,
    E: GraphCost<S>,
    E::Value: DijkstraMeasure,
    G: Connections<'graph, S>,
{
    type Item = Route<'graph, S, E::Value>;

    fn next(&mut self) -> Option<Self::Item> {
        // the first iteration is special, as we immediately return the source node
        // and then begin with the actual iteration loop.
        if self.init {
            self.init = false;
            self.next = Some(PriorityQueueItem {
                node: self.source.id(),
                priority: E::Value::zero(),
            });
            self.queue.visit(self.source.id());

            return Some(Route::new(
                Path::new(self.source, Vec::new(), self.source),
                Cost::new(E::Value::zero()),
            ));
        }

        // Process the neighbours from the node we determined in the last iteration.
        // Reasoning behind this see below.
        let PriorityQueueItem {
            node,
            priority: cost,
        } = mem::take(&mut self.next)?;

        let connections = self.connections.connections(node);
        for edge in connections {
            let (u, v) = edge.endpoint_ids();
            let target = if u == node { v } else { u };

            // do not pursue edges that have already been processed.
            if self.queue.has_been_visited(target) {
                continue;
            }

            let alternative = cost.add_ref(self.edge_cost.cost(edge).as_ref());

            // TODO: Entry API
            if let Some(distance) = self.distances.get_mut(target) {
                // do not insert the updated distance if it is not strictly better than the
                // current one
                if *distance <= alternative {
                    continue;
                }

                *distance = alternative.clone();
            } else {
                self.distances.set(target, alternative.clone());
            }

            if self.predecessor_mode == PredecessorMode::Record {
                self.predecessors.set(target, Some(node));
            }

            // if let Some(target) = self.graph.node(target) {
            self.queue.decrease_priority(target, alternative);
            // }
        }

        // this is what makes this special: instead of getting the next node as the start of next
        // (which would make sense, right?) we get the next node at the end of the last iteration.
        // The reason behind this is simple: imagine we want to know the shortest path
        // between A -> B. If we would get the next node at the beginning of the iteration
        // (instead of at the end of the last iteration, like we do here), even though we
        // only need `A -> B`, we would still explore all edges from `B` to any other node and only
        // then return the path (and distance) between A and B. While the difference in
        // performance is minimal for small graphs, time savings are substantial for dense graphs.
        // You can kind of imagine it like this:
        // ```
        // let node = get_next();
        // yield node;
        // for neighbour in get_neighbours() { ... }
        // ```
        // Only difference is that we do not have generators in stable Rust (yet).
        let Some(item) = self.queue.pop_min() else {
            // next is already `None`
            return None;
        };

        let node = item.node;
        let cost = item.priority.clone();
        self.next = Some(item);

        // we're currently visiting the node that has the shortest distance, therefore we know
        // that the distance is the shortest possible
        let distance = cost;
        let transit = if self.predecessor_mode == PredecessorMode::Discard {
            Vec::new()
        } else {
            // TODO: cache via dedicated structure
            reconstruct_path_to::<S>(&self.predecessors, node)
                .into_iter()
                .filter_map(|id| self.graph.node(id))
                .collect()
        };

        let node = self.graph.node(node)?;

        Some(Route::new(
            Path::new(self.source, transit, node),
            Cost::new(distance),
        ))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.num_nodes))
    }
}
