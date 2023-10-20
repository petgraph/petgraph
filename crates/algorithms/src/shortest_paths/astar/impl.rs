use alloc::vec::Vec;
use core::{hash::Hash, ops::Add};

use error_stack::{Report, Result};
use fxhash::FxBuildHasher;
use hashbrown::HashMap;
use num_traits::Zero;
use petgraph_core::{base::MaybeOwned, Edge, Graph, GraphStorage, Node};

use crate::shortest_paths::{
    astar::error::AStarError,
    common::{
        intermediates::{self, reconstruct_intermediates, Intermediates},
        queue::Queue,
        traits::ConnectionFn,
    },
    Cost, Path, Route,
};

pub(super) struct AStarImpl<'a, S, T, E, H, C>
where
    S: GraphStorage,
    T: Ord,
{
    queue: Queue<'a, S, T>,

    edge_cost: E,
    heuristic: H,
    connections: C,

    source: Node<'a, S>,
    target: Node<'a, S>,

    intermediates: Intermediates,

    distances: HashMap<&'a S::NodeId, T, FxBuildHasher>,
    previous: HashMap<&'a S::NodeId, Option<Node<'a, S>>, FxBuildHasher>,
}

impl<'a, S, T, E, H, C> AStarImpl<'a, S, T, E, H, C>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    T: PartialOrd + Ord + Zero + Clone + 'a,
    for<'b> &'b T: Add<Output = T>,
    E: Fn(Edge<'a, S>) -> MaybeOwned<'a, T>,
    H: Fn(Node<'a, S>, Node<'a, S>) -> MaybeOwned<'a, T>,
    C: ConnectionFn<'a, S>,
{
    pub(super) fn new(
        graph: &'a Graph<S>,

        edge_cost: E,
        heuristic: H,
        connections: C,

        source: &'a S::NodeId,
        target: &'a S::NodeId,

        intermediates: Intermediates,
    ) -> Result<Self, AStarError> {
        let source_node = graph
            .node(source)
            .ok_or_else(|| Report::new(AStarError::NodeNotFound))?;

        let target_node = graph
            .node(target)
            .ok_or_else(|| Report::new(AStarError::NodeNotFound))?;

        let mut queue = Queue::new();
        queue.push(
            source_node,
            heuristic(source_node, target_node).into_owned(),
        );

        let mut distances = HashMap::with_hasher(FxBuildHasher::default());
        distances.insert(source, T::zero());

        let mut previous = HashMap::with_hasher(FxBuildHasher::default());
        if intermediates == Intermediates::Record {
            previous.insert(source, None);
        }

        Ok(Self {
            queue,

            edge_cost,
            heuristic,
            connections,

            source: source_node,
            target: target_node,

            intermediates,

            distances,
            previous,
        })
    }

    pub(super) fn find(mut self) -> Option<Route<'a, S, T>> {
        while let Some(node) = self.queue.pop_min() {
            if node.id() == self.target.id() {
                let intermediates = if self.intermediates == Intermediates::Record {
                    reconstruct_intermediates(&self.previous, node.id())
                } else {
                    Vec::new()
                };

                let distance = self.distances[node.id()].clone();

                let path = Path {
                    source: self.source,
                    target: self.target,
                    intermediates,
                };

                return Some(Route {
                    path,
                    cost: Cost(distance),
                });
            }

            let connections = self.connections.connections(&node);
            for edge in connections {
                let alternative = &self.distances[node.id()] + (self.edge_cost)(edge).as_ref();

                let (u, v) = edge.endpoints();
                let neighbour = if u.id() == node.id() { v } else { u };

                if let Some(distance) = self.distances.get(neighbour.id()) {
                    if alternative >= *distance {
                        continue;
                    }
                }

                let guess = &alternative + (self.heuristic)(neighbour, self.target).as_ref();
                self.distances.insert(neighbour.id(), alternative);

                if self.intermediates == Intermediates::Record {
                    self.previous.insert(neighbour.id(), Some(node));
                }

                self.queue.decrease_priority(neighbour, guess);
            }
        }

        None
    }
}
