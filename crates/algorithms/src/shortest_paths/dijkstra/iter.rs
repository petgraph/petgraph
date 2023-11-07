use alloc::vec::Vec;
use core::{hash::Hash, ops::Add};

use error_stack::{Report, Result};
use fxhash::FxBuildHasher;
use hashbrown::HashMap;
use num_traits::Zero;
use petgraph_core::{Graph, GraphStorage, Node};

use crate::shortest_paths::{
    common::{
        connections::Connections,
        cost::{Cost, GraphCost},
        path::Path,
		route::Route,
        priority_queue::PriorityQueue,
        transit::{reconstruct_path_to, PredecessorMode},
    },
    dijkstra::DijkstraError,
};

pub(super) struct DijkstraIter<'graph: 'parent, 'parent, S, E, G>
where
    S: GraphStorage,
    E: GraphCost<S>,
    E::Value: Ord,
{
    queue: PriorityQueue<'graph, S, E::Value>,

    edge_cost: &'parent E,
    connections: G,

    source: Node<'graph, S>,

    num_nodes: usize,

    init: bool,
    next: Option<Node<'graph, S>>,

    predecessor_mode: PredecessorMode,

    distances: HashMap<&'graph S::NodeId, E::Value, FxBuildHasher>,
    predecessors: HashMap<&'graph S::NodeId, Option<Node<'graph, S>>, FxBuildHasher>,
}

impl<'graph: 'parent, 'parent, S, E, G> DijkstraIter<'graph, 'parent, S, E, G>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    E: GraphCost<S>,
    E::Value: PartialOrd + Ord + Zero + Clone + 'graph,
    for<'a> &'a E::Value: Add<Output = E::Value>,
    G: Connections<'graph, S>,
{
    pub(super) fn new(
        graph: &'graph Graph<S>,

        edge_cost: &'parent E,
        connections: G,

        source: &'graph S::NodeId,

        intermediates: PredecessorMode,
    ) -> Result<Self, DijkstraError> {
        let source_node = graph
            .node(source)
            .ok_or_else(|| Report::new(DijkstraError::NodeNotFound))?;

        let mut queue = PriorityQueue::new();

        let mut distances = HashMap::with_hasher(FxBuildHasher::default());
        distances.insert(source, E::Value::zero());

        let mut predecessors = HashMap::with_hasher(FxBuildHasher::default());
        if intermediates == PredecessorMode::Record {
            predecessors.insert(source, None);
        }

        Ok(Self {
            queue,
            edge_cost,
            connections,
            source: source_node,
            num_nodes: graph.num_nodes(),
            init: true,
            next: None,
            predecessor_mode: intermediates,
            distances,
            predecessors,
        })
    }
}

impl<'graph: 'parent, 'parent, S, E, G> Iterator for DijkstraIter<'graph, 'parent, S, E, G>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    E: GraphCost<S>,
    E::Value: PartialOrd + Ord + Zero + Clone + 'graph,
    for<'a> &'a E::Value: Add<Output = E::Value>,
    G: Connections<'graph, S>,
{
    type Item = Route<'graph, S, E::Value>;

    fn next(&mut self) -> Option<Self::Item> {
        // the first iteration is special, as we immediately return the source node
        // and then begin with the actual iteration loop.
        if self.init {
            self.init = false;
            self.next = Some(self.source);

            return Some(Route::new(
                Path::new(self.source, Vec::new(), self.source),
                Cost::new(E::Value::zero()),
            ));
        }

        // Process the neighbours from the node we determined in the last iteration.
        // Reasoning behind this see below.
        let node = self.next?;
        let connections = self.connections.connections(&node);

        for edge in connections {
            let (u, v) = edge.endpoints();
            let target = if v.id() == node.id() { u } else { v };

            let alternative = &self.distances[node.id()] + self.edge_cost.cost(edge).as_ref();

            if let Some(distance) = self.distances.get(target.id()) {
                // do not insert the updated distance if it is not strictly better than the current
                // one
                if alternative >= *distance {
                    continue;
                }
            }

            self.distances.insert(target.id(), alternative.clone());

            if self.predecessor_mode == PredecessorMode::Record {
                self.predecessors.insert(target.id(), Some(node));
            }

            self.queue.decrease_priority(target, alternative);
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
        let Some(node) = self.queue.pop_min() else {
            self.next = None;
            return None;
        };

        self.next = Some(node);

        // we're currently visiting the node that has the shortest distance, therefore we know
        // that the distance is the shortest possible
        let distance = self.distances[node.id()].clone();
        let transit = if self.predecessor_mode == PredecessorMode::Discard {
            Vec::new()
        } else {
            reconstruct_path_to(&self.predecessors, node.id())
        };

        let path = Path::new(self.source, transit, node);

        Some(Route::new(path, Cost::new(distance)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.num_nodes))
    }
}
