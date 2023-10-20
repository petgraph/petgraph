mod error;
mod iter;

use core::{hash::Hash, ops::Add};

use num_traits::Zero;
use petgraph_core::{
    base::MaybeOwned,
    edge::marker::{Directed, Undirected},
    Edge, GraphDirectionality, GraphStorage, Node,
};

use self::{
    error::BellmanFordError,
    iter::{BellmanFordIter, Intermediates},
};
use super::ShortestPath;

pub struct BellmanFord<D, F> {
    direction: D,
    edge_cost: F,
}

impl BellmanFord<Directed, ()> {
    pub fn directed() -> Self {
        Self {
            direction: Directed,
            edge_cost: (),
        }
    }
}

impl BellmanFord<Undirected, ()> {
    pub fn directed() -> Self {
        Self {
            direction: Undirected,
            edge_cost: (),
        }
    }
}

impl<D, F> BellmanFord<D, F>
where
    D: GraphDirectionality,
{
    pub fn with_edge_cost<S, F2, T>(self, edge_cost: F2) -> BellmanFord<D, F2>
    where
        F2: Fn(Edge<S>) -> MaybeOwned<T>,
    {
        BellmanFord {
            direction: self.direction,
            edge_cost,
        }
    }

    pub fn without_edge_cost(self) -> BellmanFord<D, ()> {
        BellmanFord {
            direction: self.direction,
            edge_cost: (),
        }
    }
}

impl<S> ShortestPath<S> for BellmanFord<Undirected, ()>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    S::EdgeWeight: PartialOrd + Ord + Zero + Clone,
    for<'a> &'a S::EdgeWeight: Add<Output = S::EdgeWeight>,
{
    type Cost = S::EdgeWeight;
    type Error = BellmanFordError;

    fn path_from<'a>(
        &self,
        graph: &'a petgraph_core::Graph<S>,
        source: &'a <S as GraphStorage>::NodeId,
    ) -> error_stack::Result<impl Iterator<Item = super::Route<'a, S, Self::Cost>>, Self::Error>
    {
        BellmanFordIter::new(
            graph,
            |edge| MaybeOwned::Borrowed(edge.weight()),
            Node::<'a, S>::connections as fn(&Node<'a, S>) -> _,
            source,
            Intermediates::Record,
        )
    }

    fn every_path<'a>(
        &self,
        graph: &'a petgraph_core::Graph<S>,
    ) -> error_stack::Result<impl Iterator<Item = super::Route<'a, S, Self::Cost>>, Self::Error>
    {
        unimplemented!()
    }
}
