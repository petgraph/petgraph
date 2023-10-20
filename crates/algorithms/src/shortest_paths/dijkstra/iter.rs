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
    common::{queue::Queue, traits::ConnectionFn},
    dijkstra::DijkstraError,
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

    loop {
        let Some(node) = previous[current] else {
            // this case should in theory _never_ happen, as the next statement
            // terminates if the next node is `None` (we're at a source node)
            // we do it this way, so that we don't need to push and then pop immediately.
            break;
        };

        if previous[node.id()].is_none() {
            // we have reached the source node
            break;
        }

        path.push(node);
        current = node.id();
    }

    path.reverse();

    path
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

    num_nodes: usize,

    init: bool,
    next: Option<Node<'a, S>>,

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

        let mut queue = Queue::new();
        queue.push(source_node, T::zero());

        let mut distances = HashMap::with_hasher(FxBuildHasher::default());
        distances.insert(source, T::zero());

        let mut previous = HashMap::with_hasher(FxBuildHasher::default());
        if intermediates == Intermediates::Record {
            previous.insert(source, None);
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
                distance: Distance { value: T::zero() },
            });
        }

        // Process the neighbours from the node we determined in the last iteration.
        // Reasoning behind this see below.
        let node = self.next?;
        let connections = self.connections.connections(&node);

        for edge in connections {
            let (u, v) = edge.endpoints();
            let target = if v.id() == node.id() { u } else { v };

            let alternative = &self.distances[node.id()] + (self.edge_cost)(edge).as_ref();

            if let Some(distance) = self.distances.get(target.id()) {
                // do not insert the updated distance if it is not strictly better than the current
                // one
                if alternative >= *distance {
                    continue;
                }
            }

            self.distances.insert(target.id(), alternative.clone());

            if self.intermediates == Intermediates::Record {
                self.previous.insert(target.id(), Some(node));
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
        let intermediates = if self.intermediates == Intermediates::Discard {
            Vec::new()
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.num_nodes))
    }
}
