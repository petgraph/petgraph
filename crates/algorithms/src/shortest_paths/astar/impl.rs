use alloc::vec::Vec;

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
    astar::{error::AStarError, heuristic::GraphHeuristic, AStarMeasure},
    common::{
        connections::Connections,
        cost::{Cost, GraphCost},
        queue::priority::{PriorityQueue, PriorityQueueItem},
        route::{DirectRoute, Route},
        transit::{reconstruct_path_to, PredecessorMode},
    },
    Path,
};

pub(super) struct AStarImpl<'graph: 'parent, 'parent, S, E, H, C>
where
    S: GraphStorage,
    E: GraphCost<S>,
    E::Value: Ord,
{
    graph: &'graph Graph<S>,
    queue: PriorityQueue<'graph, S, E::Value>,

    edge_cost: &'parent E,
    heuristic: &'parent H,
    connections: C,

    source: Node<'graph, S>,
    target: Node<'graph, S>,

    predecessor_mode: PredecessorMode,

    distances: S::SecondaryNodeStorage<'graph, E::Value>,
    estimates: S::SecondaryNodeStorage<'graph, E::Value>,
    predecessors: S::SecondaryNodeStorage<'graph, Option<NodeId>>,
}

impl<'graph: 'parent, 'parent, S, E, H, C> AStarImpl<'graph, 'parent, S, E, H, C>
where
    S: GraphStorage,
    E: GraphCost<S>,
    E::Value: AStarMeasure,
    H: GraphHeuristic<S, Value = E::Value>,
    C: Connections<'graph, S> + 'parent,
{
    pub(super) fn new(
        graph: &'graph Graph<S>,

        edge_cost: &'parent E,
        heuristic: &'parent H,
        connections: C,

        source: NodeId,
        target: NodeId,

        predecessor_mode: PredecessorMode,
    ) -> Result<Self, AStarError> {
        let source_node = graph
            .node(source)
            .ok_or_else(|| Report::new(AStarError::NodeNotFound))?;

        let target_node = graph
            .node(target)
            .ok_or_else(|| Report::new(AStarError::NodeNotFound))?;

        let estimate = heuristic.estimate(source_node, target_node);

        let mut queue = PriorityQueue::new(graph.storage());
        queue.check_admissibility = false;

        queue.push(source_node.id(), estimate.clone().into_owned());

        let mut distances = graph.storage().secondary_node_storage(Hints {
            performance: PerformanceHint {
                read: FrequencyHint::Frequent,
                write: FrequencyHint::Frequent,
            },
            occupancy: OccupancyHint::Dense,
        });
        distances.set(source, E::Value::zero());

        let estimates = graph.storage().secondary_node_storage(Hints {
            performance: PerformanceHint {
                read: FrequencyHint::Frequent,
                write: FrequencyHint::Infrequent,
            },
            occupancy: OccupancyHint::Dense,
        });

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
            heuristic,
            connections,

            source: source_node,
            target: target_node,

            predecessor_mode,

            distances,
            estimates,
            predecessors,
        })
    }

    pub(super) fn find(mut self) -> Option<Route<'graph, S, E::Value>> {
        while let Some(PriorityQueueItem {
            node,
            priority: current_estimate,
        }) = self.queue.pop_min()
        {
            if node == self.target.id() {
                let transit = if self.predecessor_mode == PredecessorMode::Record {
                    reconstruct_path_to::<S>(&self.predecessors, node)
                        .into_iter()
                        .filter_map(|id| self.graph.node(id))
                        .collect()
                } else {
                    Vec::new()
                };

                let distance = self.distances.get(node)?.clone();
                return Some(Route::new(
                    Path::new(self.source, transit, self.target),
                    Cost::new(distance),
                ));
            }

            if let Some(estimate) = self.estimates.get_mut(node) {
                // if the current estimate is better than the estimate in the queue, skip this node
                if *estimate <= current_estimate {
                    continue;
                }

                *estimate = current_estimate;
            } else {
                self.estimates.set(node, current_estimate);
            }

            let connections = self.connections.connections(node);
            for edge in connections {
                let source_distance = self.distances.get(node)?;
                let alternative = source_distance.add_ref(self.edge_cost.cost(edge).as_ref());

                let (u, v) = edge.endpoints();
                let neighbour = if u.id() == node { v } else { u };

                if let Some(distance) = self.distances.get(neighbour.id()) {
                    if alternative >= *distance {
                        continue;
                    }
                }

                let guess =
                    alternative.add_ref(self.heuristic.estimate(neighbour, self.target).as_ref());
                self.distances.set(neighbour.id(), alternative);

                if self.predecessor_mode == PredecessorMode::Record {
                    self.predecessors.set(neighbour.id(), Some(node));
                }

                self.queue.decrease_priority(neighbour.id(), guess);
            }
        }

        None
    }

    #[inline]
    pub(super) fn find_all(
        items: Vec<Self>,
    ) -> impl Iterator<Item = Route<'graph, S, E::Value>> + 'parent {
        items.into_iter().filter_map(Self::find)
    }

    #[inline]
    pub(super) fn find_all_direct(
        items: Vec<Self>,
    ) -> impl Iterator<Item = DirectRoute<'graph, S, E::Value>> + 'parent {
        Self::find_all(items).map(From::from)
    }
}
