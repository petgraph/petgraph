mod error;
mod iter;
#[cfg(test)]
mod tests;

use alloc::vec::Vec;
use core::{hash::Hash, marker::PhantomData, ops::Add};

use error_stack::Result;
use num_traits::Zero;
use petgraph_core::{
    edge::marker::{Directed, Undirected},
    DirectedGraphStorage, Graph, GraphDirectionality, GraphStorage, Node,
};

pub use self::error::DijkstraError;
use self::iter::DijkstraIter;
use super::{
    common::{
        connections::outgoing_connections,
        cost::{DefaultCost, GraphCost},
        route::{DirectRoute, Route},
        transit::PredecessorMode,
    },
    ShortestDistance, ShortestPath,
};
use crate::polyfill::IteratorExt;

pub struct Dijkstra<D, E>
where
    D: GraphDirectionality,
{
    edge_cost: E,

    direction: PhantomData<fn() -> *const D>,
}

impl Dijkstra<Directed, DefaultCost> {
    #[must_use]
    pub fn directed() -> Self {
        Self {
            direction: PhantomData,
            edge_cost: DefaultCost,
        }
    }
}

impl Dijkstra<Undirected, DefaultCost> {
    #[must_use]
    pub fn undirected() -> Self {
        Self {
            direction: PhantomData,
            edge_cost: DefaultCost,
        }
    }
}

impl<D, E> Dijkstra<D, E>
where
    D: GraphDirectionality,
{
    pub fn with_edge_cost<S, F>(self, edge_cost: F) -> Dijkstra<D, F>
    where
        S: GraphStorage,
        F: GraphCost<S>,
    {
        Dijkstra {
            direction: PhantomData,
            edge_cost,
        }
    }
}

impl<S, E> ShortestPath<S> for Dijkstra<Undirected, E>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    E: GraphCost<S>,
    for<'a> E::Value: PartialOrd + Ord + Zero + Clone + 'a,
    for<'a> &'a E::Value: Add<Output = E::Value>,
{
    type Cost = E::Value;
    type Error = DijkstraError;

    fn path_to<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        target: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>>, Self::Error> {
        let iter = self.path_from(graph, target)?;

        Ok(iter.map(Route::reverse))
    }

    fn path_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        DijkstraIter::new(
            graph,
            &self.edge_cost,
            Node::<'graph, S>::connections as fn(&Node<'graph, S>) -> _,
            source,
            PredecessorMode::Record,
        )
    }

    fn every_path<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter = graph
            .nodes()
            .map(move |node| self.path_from(graph, node.id()))
            .collect_reports::<Vec<_>>()?;

        Ok(iter.into_iter().flatten())
    }
}

impl<S, E> ShortestDistance<S> for Dijkstra<Undirected, E>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    E: GraphCost<S>,
    for<'a> E::Value: PartialOrd + Ord + Zero + Clone + 'a,
    for<'a> &'a E::Value: Add<Output = E::Value>,
{
    type Cost = E::Value;
    type Error = DijkstraError;

    fn distance_to<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        target: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>>, Self::Error> {
        let iter = self.distance_from(graph, target)?;

        Ok(iter.map(DirectRoute::reverse))
    }

    fn distance_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: &'graph <S as GraphStorage>::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter = DijkstraIter::new(
            graph,
            &self.edge_cost,
            Node::<'graph, S>::connections as fn(&Node<'graph, S>) -> _,
            source,
            PredecessorMode::Discard,
        )?;

        Ok(iter.map(From::from))
    }

    fn every_distance<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter = graph
            .nodes()
            .map(move |node| self.distance_from(graph, node.id()))
            .collect_reports::<Vec<_>>()?;

        Ok(iter.into_iter().flatten())
    }
}

impl<S, E> ShortestPath<S> for Dijkstra<Directed, E>
where
    S: DirectedGraphStorage,
    S::NodeId: Eq + Hash,
    E: GraphCost<S>,
    for<'a> E::Value: PartialOrd + Ord + Zero + Clone + 'a,
    for<'a> &'a E::Value: Add<Output = E::Value>,
{
    type Cost = E::Value;
    type Error = DijkstraError;

    fn path_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        DijkstraIter::new(
            graph,
            &self.edge_cost,
            outgoing_connections as fn(&Node<'graph, S>) -> _,
            source,
            PredecessorMode::Record,
        )
    }

    fn every_path<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter = graph
            .nodes()
            .map(move |node| self.path_from(graph, node.id()))
            .collect_reports::<Vec<_>>()?;

        Ok(iter.into_iter().flatten())
    }
}

impl<S, E> ShortestDistance<S> for Dijkstra<Directed, E>
where
    S: DirectedGraphStorage,
    S::NodeId: Eq + Hash,
    E: GraphCost<S>,
    for<'a> E::Value: PartialOrd + Ord + Zero + Clone + 'a,
    for<'a> &'a E::Value: Add<Output = E::Value>,
{
    type Cost = E::Value;
    type Error = DijkstraError;

    fn distance_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: &'graph <S as GraphStorage>::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>>, Self::Error> {
        let iter = DijkstraIter::new(
            graph,
            &self.edge_cost,
            outgoing_connections as fn(&Node<'graph, S>) -> _,
            source,
            PredecessorMode::Discard,
        )?;

        Ok(iter.map(From::from))
    }

    fn every_distance<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter = graph
            .nodes()
            .map(move |node| self.distance_from(graph, node.id()))
            .collect_reports::<Vec<_>>()?;

        Ok(iter.into_iter().flatten())
    }
}
