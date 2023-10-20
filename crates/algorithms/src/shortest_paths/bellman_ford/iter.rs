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

        let mut distances = HashMap::with_hasher(FxBuildHasher::default());
        let mut previous = HashMap::with_hasher(FxBuildHasher::default());

        distances.insert(source, T::zero());
        if intermediates == Intermediates::Record {
            previous.insert(source, None);
        }

        Ok(Self {
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

    fn next(&mut self) -> Option<Self::Item> {
        // go on then do the thing with the paths

        todo!();
    }
}
