mod error;
mod iter;
#[cfg(test)]
mod tests;

use alloc::vec;
use core::{hash::Hash, ops::Add};

use error_stack::Result;
use num_traits::Zero;
use petgraph_core::{
    base::MaybeOwned,
    edge::{
        marker::{Directed, Undirected},
        Direction,
    },
    DirectedGraphStorage, Edge, Graph, GraphDirectionality, GraphStorage, Node,
};

pub(crate) use self::error::DijkstraError;
use self::iter::{DijkstraIter, Intermediates};
use crate::shortest_paths::{DirectRoute, Route, ShortestDistance, ShortestPath};

macro_rules! fold {
    ($iter:expr => flatten) => {
        $iter
            .fold(Ok(vec![]), |acc, value| match (acc, value) {
                (Ok(mut acc), Ok(value)) => {
                    acc.extend(value);
                    Ok(acc)
                }
                (Err(mut acc), Err(error)) => {
                    acc.extend_one(error);
                    Err(acc)
                }
                (Err(err), _) | (_, Err(err)) => Err(err),
            })
            .map(|value| value.into_iter())
    };
}

fn outgoing_connections<'a, S>(node: &Node<'a, S>) -> impl Iterator<Item = Edge<'a, S>> + 'a
where
    S: DirectedGraphStorage,
{
    node.directed_connections(Direction::Outgoing)
}

pub struct Dijkstra<D, F>
where
    D: GraphDirectionality,
{
    direction: D,

    edge_cost: F,
}

impl Dijkstra<Directed, ()> {
    pub fn directed() -> Self {
        Self {
            direction: Directed,
            edge_cost: (),
        }
    }
}

impl Dijkstra<Undirected, ()> {
    pub fn undirected() -> Self {
        Self {
            direction: Undirected,
            edge_cost: (),
        }
    }
}

impl<D, F> Dijkstra<D, F>
where
    D: GraphDirectionality,
{
    pub fn with_edge_cost<S, F2, T>(self, edge_cost: F2) -> Dijkstra<D, F2>
    where
        F2: Fn(Edge<S>) -> MaybeOwned<T>,
    {
        Dijkstra {
            direction: self.direction,
            edge_cost,
        }
    }

    pub fn without_edge_cost(self) -> Dijkstra<D, ()> {
        Dijkstra {
            direction: self.direction,
            edge_cost: (),
        }
    }
}

impl<S> ShortestPath<S> for Dijkstra<Undirected, ()>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    S::EdgeWeight: PartialOrd + Ord + Zero + Clone,
    for<'a> &'a S::EdgeWeight: Add<Output = S::EdgeWeight>,
{
    type Cost = S::EdgeWeight;
    type Error = DijkstraError;

    fn path_from<'a>(
        &self,
        graph: &'a Graph<S>,
        source: &'a S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'a, S, Self::Cost>>, Self::Error> {
        DijkstraIter::new(
            graph,
            |edge| MaybeOwned::Borrowed(edge.weight()),
            Node::<'a, S>::connections as fn(&Node<'a, S>) -> _,
            source,
            Intermediates::Record,
        )
    }

    fn every_path<'a>(
        &self,
        graph: &'a Graph<S>,
    ) -> Result<impl Iterator<Item = Route<'a, S, Self::Cost>>, Self::Error> {
        let iter = graph
            .nodes()
            .map(move |node| self.path_from(graph, node.id()));

        fold!(iter => flatten)
    }
}

impl<S> ShortestDistance<S> for Dijkstra<Undirected, ()>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    S::EdgeWeight: PartialOrd + Ord + Zero + Clone,
    for<'a> &'a S::EdgeWeight: Add<Output = S::EdgeWeight>,
{
    type Cost = S::EdgeWeight;
    type Error = DijkstraError;

    fn distance_from<'a>(
        &self,
        graph: &'a Graph<S>,
        source: &'a <S as GraphStorage>::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'a, S, Self::Cost>>, Self::Error> {
        let iter = DijkstraIter::new(
            graph,
            |edge| MaybeOwned::Borrowed(edge.weight()),
            Node::<'a, S>::connections as fn(&Node<'a, S>) -> _,
            source,
            Intermediates::Discard,
        )?;

        Ok(iter.map(|route| DirectRoute {
            source: route.path.source,
            target: route.path.target,
            distance: route.distance,
        }))
    }

    fn every_distance<'a>(
        &self,
        graph: &'a Graph<S>,
    ) -> Result<impl Iterator<Item = DirectRoute<'a, S, Self::Cost>>, Self::Error> {
        let iter = graph
            .nodes()
            .map(move |node| self.distance_from(graph, node.id()));

        fold!(iter => flatten)
    }
}

impl<S> ShortestPath<S> for Dijkstra<Directed, ()>
where
    S: DirectedGraphStorage,
    S::NodeId: Eq + Hash,
    S::EdgeWeight: PartialOrd + Ord + Zero + Clone,
    for<'a> &'a S::EdgeWeight: Add<Output = S::EdgeWeight>,
{
    type Cost = S::EdgeWeight;
    type Error = DijkstraError;

    fn path_from<'a>(
        &self,
        graph: &'a Graph<S>,
        source: &'a S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'a, S, Self::Cost>>, Self::Error> {
        DijkstraIter::new(
            graph,
            |edge| MaybeOwned::Borrowed(edge.weight()),
            outgoing_connections as fn(&Node<'a, S>) -> _,
            source,
            Intermediates::Record,
        )
    }

    fn every_path<'a>(
        &self,
        graph: &'a Graph<S>,
    ) -> Result<impl Iterator<Item = Route<'a, S, Self::Cost>>, Self::Error> {
        let iter = graph
            .nodes()
            .map(move |node| self.path_from(graph, node.id()));

        fold!(iter => flatten)
    }
}

impl<S> ShortestDistance<S> for Dijkstra<Directed, ()>
where
    S: DirectedGraphStorage,
    S::NodeId: Eq + Hash,
    S::EdgeWeight: PartialOrd + Ord + Zero + Clone,
    for<'a> &'a S::EdgeWeight: Add<Output = S::EdgeWeight>,
{
    type Cost = S::EdgeWeight;
    type Error = DijkstraError;

    fn distance_from<'a>(
        &self,
        graph: &'a Graph<S>,
        source: &'a <S as GraphStorage>::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'a, S, Self::Cost>>, Self::Error> {
        let iter = DijkstraIter::new(
            graph,
            |edge| MaybeOwned::Borrowed(edge.weight()),
            outgoing_connections as fn(&Node<'a, S>) -> _,
            source,
            Intermediates::Discard,
        )?;

        Ok(iter.map(|route| DirectRoute {
            source: route.path.source,
            target: route.path.target,
            distance: route.distance,
        }))
    }

    fn every_distance<'a>(
        &self,
        graph: &'a Graph<S>,
    ) -> Result<impl Iterator<Item = DirectRoute<'a, S, Self::Cost>>, Self::Error> {
        let iter = graph
            .nodes()
            .map(move |node| self.distance_from(graph, node.id()));

        fold!(iter => flatten)
    }
}

impl<S, F, T> ShortestPath<S> for Dijkstra<Undirected, F>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    F: Fn(Edge<S>) -> MaybeOwned<T>,
    for<'a> T: PartialOrd + Ord + Zero + Clone + 'a,
    for<'a> &'a T: Add<Output = T>,
{
    type Cost = T;
    type Error = DijkstraError;

    fn path_from<'a>(
        &self,
        graph: &'a Graph<S>,
        source: &'a S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'a, S, Self::Cost>>, Self::Error> {
        DijkstraIter::new(
            graph,
            &self.edge_cost,
            Node::<'a, S>::connections as fn(&Node<'a, S>) -> _,
            source,
            Intermediates::Record,
        )
    }

    fn every_path<'a>(
        &self,
        graph: &'a Graph<S>,
    ) -> Result<impl Iterator<Item = Route<'a, S, Self::Cost>>, Self::Error> {
        let iter = graph
            .nodes()
            .map(move |node| self.path_from(graph, node.id()));

        fold!(iter => flatten)
    }
}

impl<S, F, T> ShortestDistance<S> for Dijkstra<Undirected, F>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    F: Fn(Edge<S>) -> MaybeOwned<T>,
    for<'a> T: PartialOrd + Ord + Zero + Clone + 'a,
    for<'a> &'a T: Add<Output = T>,
{
    type Cost = T;
    type Error = DijkstraError;

    fn distance_from<'a>(
        &self,
        graph: &'a Graph<S>,
        source: &'a <S as GraphStorage>::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'a, S, Self::Cost>>, Self::Error> {
        let iter = DijkstraIter::new(
            graph,
            &self.edge_cost,
            Node::<'a, S>::connections as fn(&Node<'a, S>) -> _,
            source,
            Intermediates::Discard,
        )?;

        Ok(iter.map(|route| DirectRoute {
            source: route.path.source,
            target: route.path.target,
            distance: route.distance,
        }))
    }

    fn every_distance<'a>(
        &self,
        graph: &'a Graph<S>,
    ) -> Result<impl Iterator<Item = DirectRoute<'a, S, Self::Cost>>, Self::Error> {
        let iter = graph
            .nodes()
            .map(move |node| self.distance_from(graph, node.id()));

        fold!(iter => flatten)
    }
}

impl<S, F, T> ShortestPath<S> for Dijkstra<Directed, F>
where
    S: DirectedGraphStorage,
    S::NodeId: Eq + Hash,
    F: Fn(Edge<S>) -> MaybeOwned<T>,
    for<'a> T: PartialOrd + Ord + Zero + Clone + 'a,
    for<'a> &'a T: Add<Output = T>,
{
    type Cost = T;
    type Error = DijkstraError;

    fn path_from<'a>(
        &self,
        graph: &'a Graph<S>,
        source: &'a S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'a, S, Self::Cost>>, Self::Error> {
        DijkstraIter::new(
            graph,
            &self.edge_cost,
            outgoing_connections as fn(&Node<'a, S>) -> _,
            source,
            Intermediates::Record,
        )
    }

    fn every_path<'a>(
        &self,
        graph: &'a Graph<S>,
    ) -> Result<impl Iterator<Item = Route<'a, S, Self::Cost>>, Self::Error> {
        let iter = graph
            .nodes()
            .map(move |node| self.path_from(graph, node.id()));

        fold!(iter => flatten)
    }
}

impl<S, F, T> ShortestDistance<S> for Dijkstra<Directed, F>
where
    S: DirectedGraphStorage,
    S::NodeId: Eq + Hash,
    F: Fn(Edge<S>) -> MaybeOwned<T>,
    for<'a> T: PartialOrd + Ord + Zero + Clone + 'a,
    for<'a> &'a T: Add<Output = T>,
{
    type Cost = T;
    type Error = DijkstraError;

    fn distance_from<'a>(
        &self,
        graph: &'a Graph<S>,
        source: &'a <S as GraphStorage>::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'a, S, Self::Cost>>, Self::Error> {
        let iter = DijkstraIter::new(
            graph,
            &self.edge_cost,
            outgoing_connections as fn(&Node<'a, S>) -> _,
            source,
            Intermediates::Discard,
        )?;

        Ok(iter.map(|route| DirectRoute {
            source: route.path.source,
            target: route.path.target,
            distance: route.distance,
        }))
    }

    fn every_distance<'a>(
        &self,
        graph: &'a Graph<S>,
    ) -> Result<impl Iterator<Item = DirectRoute<'a, S, Self::Cost>>, Self::Error> {
        let iter = graph
            .nodes()
            .map(move |node| self.distance_from(graph, node.id()));

        fold!(iter => flatten)
    }
}
