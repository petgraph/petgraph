use core::{hash::Hash, ops::Add};

use error_stack::{Report, Result};
use fxhash::FxBuildHasher;
use hashbrown::HashMap;
use num_traits::Zero;
use petgraph_core::{base::MaybeOwned, Edge, Graph, GraphStorage, Node};

use super::error::BellmanFordError;
use crate::shortest_paths::{common::traits::ConnectionFn, Route};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) enum Intermediates {
    Discard,
    Record,
}

pub(super) struct BellmanFordIter<'a, S, T, F, G>
where
    S: GraphStorage,
    T: Ord,
{
    queue: Queue<'graph, S, E::Value>,

    edge_cost: F,
    connections: G,

    source: Node<'a, S>,

    num_nodes: usize,

    init: bool,
    next: Option<Node<'a, S>>,

    intermediates: Intermediates,

    distances: HashMap<&'a S::NodeId, T, FxBuildHasher>,
    predecessors: HashMap<&'a S::NodeId, Option<Node<'a, S>>, FxBuildHasher>,
}

impl<'a, S, T, F, G> BellmanFordIter<'a, S, T, F, G>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    F: Fn(Edge<'a, S>) -> MaybeOwned<'a, T>,
    T: PartialOrd + Ord + Zero + Clone + 'a,
    for<'b> &'b T: Add<Output = T>,
    G: ConnectionFn<'a, S>,
{
    pub(super) fn new(
        graph: &'a Graph<S>,

        edge_cost: F,
        connections: G,

        source: &'a S::NodeId,

        intermediates: Intermediates,
    ) -> Result<Self, BellmanFordError> {
        let source_node = graph
            .node(source)
            .ok_or_else(|| Report::new(BellmanFordError::NodeNotFound))?;

        let mut queue: Queue<'_, S, <E as GraphCost<S>>::Value> = Queue::new();
        queue.push(source_node, T::zero());

        let mut distances = HashMap::with_hasher(FxBuildHasher::default());
        distances.insert(source, T::zero());

        let mut predecessors = HashMap::with_hasher(FxBuildHasher::default());
        if intermediates == Intermediates::Record {
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
            intermediates,
            distances,
            predecessors,
        })
    }
}

impl<'a, S, T, F, G> Iterator for BellmanFordIter<'a, S, T, F, G>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    F: Fn(Edge<'a, S>) -> MaybeOwned<'a, T>,
    T: PartialOrd + Ord + Zero + Clone + 'a,
    for<'b> &'b T: Add<Output = T>,
    G: ConnectionFn<'a, S>,
{
    type Item = Route<'a, S, T>;

    // The concrete implementation is the SPFA (Shortest Path Faster Algorithm) algorithm, which is
    // a variant of Bellman-Ford that uses a queue to avoid unnecessary relaxation.
    // https://en.wikipedia.org/wiki/Shortest_path_faster_algorithm
    fn next(&mut self) -> Option<Self::Item> {
        // the first iteration is special, as we immediately return the source node
        // and then begin with the actual iteration loop.
        if self.init {
            self.init = false;
            self.next = Some(self.source);

            return Some(Route {
                path: Path {
                    source: self.source,
                    target: self.source,
                    intermediates: Vec::new(),
                },
                cost: Cost(E::Value::zero()),
            });
        }

        let node = self.next?;
        let connections = self.connections.connections(&node);

        for edge in connections {
            let (u, v) = edge.endpoints();
            let target = if v.id() == node.id() { u } else { v };

            let next_distance_cost = self.distances[&node.id()] + (self.edge_cost)(edge);

            if next_distance_cost < self.distances[&target.id()] {
                self.distances.insert(target.id(), next_distance_cost);

                if self.intermediates == Intermediates::Record {
                    self.predecessors.insert(target.id(), Some(node));
                }

                self.queue
                    .decrease_priority(target, self.distances[&target.id()]);
            }
        }

        let Some(node) = self.queue.pop_min() else {
            // No more elements in the queue, we're done.
            self.next = None;
            return None;
        };

        self.next = Some(node);

        // we're currently visiting the node that has the shortest distance, therefore we know
        // that the distance is the shortest possible
        let distance = self.distances[node.id()].clone();
        let intermediates = if self.intermediates == Intermediates::Discard {
            Vec::new()
        } else {
            reconstruct_intermediates(&self.predecessors, node.id())
        };

        let path = Path {
            source: self.source,
            target: node,
            intermediates,
        };

        Some(Route {
            path,
            cost: Cost(distance),
        })
    }
}
