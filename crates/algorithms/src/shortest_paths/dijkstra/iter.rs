use alloc::vec::Vec;
use core::{
    hash::{BuildHasher, Hash},
    ops::Add,
};

use error_stack::{Report, Result};
use fxhash::FxBuildHasher;
use hashbrown::HashMap;
use num_traits::Zero;
use petgraph_core::{base::MaybeOwned, Edge, Graph, GraphStorage, Node};

use crate::shortest_paths::{
    dijkstra::{queue::Queue, DijkstraError},
    Distance, Path, Route,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) enum Intermediates {
    Discard,
    Record,
}

fn reconstruct_intermediates<'a, S, H>(
    previous: &HashMap<&'a S::NodeId, Option<Node<'a, S>>, H>,
    target: &'a S::NodeId,
) -> Vec<Node<'a, S>>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    H: BuildHasher,
{
    let mut current = target;

    let mut path = Vec::new();

    while let Some(node) = previous[current] {
        path.push(node);
        current = node.id();
    }

    // remove the source node (last one)
    path.pop();
    path.reverse();

    path
}

trait ConnectionFn<'a, S>
where
    S: GraphStorage,
{
    fn connections(&self, node: &Node<'a, S>) -> impl Iterator<Item = Edge<'a, S>> + 'a;
}

impl<'a, S, I> ConnectionFn<'a, S> for fn(&Node<'a, S>) -> I
where
    S: GraphStorage,
    I: Iterator<Item = Edge<'a, S>> + 'a,
{
    fn connections(&self, node: &Node<'a, S>) -> impl Iterator<Item = Edge<'a, S>> + 'a {
        (*self)(node)
    }
}

pub(super) struct DijkstraIter<'a, S, T, F, G>
where
    S: GraphStorage,
    T: Ord,
{
    queue: Queue<'a, S, T>,

    edge_cost: F,
    connections: G,

    source: Node<'a, S>,

    intermediates: Intermediates,

    distances: HashMap<&'a S::NodeId, T, FxBuildHasher>,
    previous: HashMap<&'a S::NodeId, Option<Node<'a, S>>, FxBuildHasher>,
}

impl<'a, S, T, F, G> DijkstraIter<'a, S, T, F, G>
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
    ) -> Result<Self, DijkstraError> {
        let source_node = graph
            .node(source)
            .ok_or_else(|| Report::new(DijkstraError::NodeNotFound))?;

        let mut distances = HashMap::with_hasher(FxBuildHasher::default());
        let mut previous = HashMap::with_hasher(FxBuildHasher::default());

        let mut queue = Queue::new();

        distances.insert(source, T::zero());
        if intermediates == Intermediates::Record {
            previous.insert(source, None);
        }

        queue.push(source_node, T::zero());

        Ok(Self {
            queue,
            edge_cost,
            connections,
            source: source_node,
            intermediates,
            distances,
            previous,
        })
    }
}

impl<'a, S, T, F, G> Iterator for DijkstraIter<'a, S, T, F, G>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    F: Fn(Edge<'a, S>) -> MaybeOwned<'a, T>,
    T: PartialOrd + Ord + Zero + Clone + 'a,
    for<'b> &'b T: Add<Output = T>,
    G: ConnectionFn<'a, S>,
{
    type Item = Route<'a, S, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.queue.pop_min()?;

        let connections = self.connections.connections(&node);

        for edge in connections {
            let (u, v) = edge.endpoints();
            let target = if v.id() == node.id() { u } else { v };

            let alternative = &self.distances[node.id()] + (self.edge_cost)(edge).as_ref();

            let mut insert = false;
            if let Some(distance) = self.distances.get(target.id()) {
                if alternative < *distance {
                    insert = true;
                }
            } else {
                insert = true;
            }

            if insert {
                self.distances.insert(target.id(), alternative.clone());

                if self.intermediates == Intermediates::Record {
                    self.previous.insert(target.id(), Some(node));
                }

                self.queue.decrease_priority(target, alternative);
            }
        }

        // we're currently visiting the node that has the shortest distance, therefore we know
        // that the distance is the shortest possible
        let distance = self.distances[node.id()].clone();
        let intermediates = if self.intermediates == Intermediates::Discard {
            vec![]
        } else {
            reconstruct_intermediates(&self.previous, node.id())
        };

        let path = Path {
            source: self.source,
            target: node,
            intermediates,
        };

        Some(Route {
            path,
            distance: Distance { value: distance },
        })
    }
}
