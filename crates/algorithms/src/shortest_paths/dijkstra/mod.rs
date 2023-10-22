mod error;
mod iter;
#[cfg(test)]
mod tests;

use alloc::vec::Vec;
use core::{hash::Hash, marker::PhantomData, ops::Add};

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
use self::iter::DijkstraIter;
use super::common::intermediates::Intermediates;
use crate::{
    polyfill::IteratorExt,
    shortest_paths::{
        common::cost::{DefaultCost, GraphCost},
        DirectRoute, Route, ShortestDistance, ShortestPath,
    },
};

fn outgoing_connections<'a, S>(node: &Node<'a, S>) -> impl Iterator<Item = Edge<'a, S>> + 'a
where
    S: DirectedGraphStorage,
{
    node.directed_connections(Direction::Outgoing)
}

pub struct Dijkstra<D, E>
where
    D: GraphDirectionality,
{
    edge_cost: E,

    direction: PhantomData<fn() -> *const D>,
}

impl Dijkstra<Directed, DefaultCost> {
    pub fn directed() -> Self {
        Self {
            direction: PhantomData,
            edge_cost: DefaultCost,
        }
    }
}

impl Dijkstra<Undirected, DefaultCost> {
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

macro_rules! methods {
    (@impl any from($self:ident)edge default) => {
        |edge| MaybeOwned::Borrowed(edge.weight())
    };
    (@impl any from($self:ident)edge fn) => {
        &$self.edge_cost
    };
    (@impl any from($a:lifetime)connection undirected) => {
        Node::<$a, S>::connections as fn(&Node<$a, S>) -> _
    };
    (@impl any from($a:lifetime)connection directed) => {
        outgoing_connections as fn(&Node<$a, S>) -> _
    };
    (@impl any from(edge: $edge:ident, direction: $direction:ident, record: $record:ident, $a:lifetime, $self:ident, $graph:ident, $source:ident) body) => {
        DijkstraIter::new(
            $graph,
            methods!(@impl any from($self) edge $edge),
            methods!(@impl any from($a) connection $direction),
            $source,
            Intermediates::$record,
        )
    };
    (@impl $variant:ident to(edge: $edge:ident, direction: directed)) => {
        // TODO: make use of graph views
    };
    (@impl path to(edge: $edge:ident, direction: undirected)) => {
        fn path_to<'a: 'b, 'b>(
            &'b self,
            graph: &'a Graph<S>,
            target: &'a S::NodeId,
        ) -> Result<impl Iterator<Item = Route<'a, S, Self::Cost>>, Self::Error> {
            let iter = self.path_from(graph, target)?;

            Ok(iter.map(Route::reverse))
        }
    };
    (@impl path from(edge: $edge:ident, direction: $direction:ident)) => {
        fn path_from<'a: 'b, 'b>(
            &'b self,
            graph: &'a Graph<S>,
            source: &'a S::NodeId,
        ) -> Result<impl Iterator<Item = Route<'a, S, Self::Cost>> + 'b, Self::Error> {
            methods!(@impl any from(edge: $edge, direction: $direction, record: Record, 'a, self, graph, source) body)
        }
    };
    (@impl distance to(edge: $edge:ident, direction: undirected)) => {
        fn distance_to<'a>(
            &self,
            graph: &'a Graph<S>,
            target: &'a S::NodeId,
        ) -> Result<impl Iterator<Item = DirectRoute<'a, S, Self::Cost>>, Self::Error> {
            let iter = self.distance_from(graph, target)?;

            Ok(iter.map(DirectRoute::reverse))
        }
    };
    (@impl distance from(edge: $edge:ident, direction: $direction:ident)) => {
        fn distance_from<'a>(
            &self,
            graph: &'a Graph<S>,
            source: &'a <S as GraphStorage>::NodeId,
        ) -> Result<impl Iterator<Item = DirectRoute<'a, S, Self::Cost>>, Self::Error> {
            let iter = methods!(@impl any from(edge: $edge, direction: $direction, record: Discard, 'a, self, graph, source) body)?;

            Ok(iter.map(|route| DirectRoute {
                source: route.path.source,
                target: route.path.target,
                cost: route.cost,
            }))
        }
    };
    (@impl path every) => {
        fn every_path<'a: 'b, 'b>(
            &'b self,
            graph: &'a Graph<S>,
        ) -> Result<impl Iterator<Item = Route<'a, S, Self::Cost>> + 'b, Self::Error> {
            let iter = graph
                .nodes()
                .map(move |node| self.path_from(graph, node.id()))
                .collect_reports::<Vec<_>>()?;

            Ok(iter.into_iter().flatten())
        }
    };
    (@impl distance every) => {
        fn every_distance<'a>(
            &self,
            graph: &'a Graph<S>,
        ) -> Result<impl Iterator<Item = DirectRoute<'a, S, Self::Cost>>, Self::Error> {
            let iter = graph
                .nodes()
                .map(move |node| self.distance_from(graph, node.id()))
                .collect_reports::<Vec<_>>()?;

            Ok(iter.into_iter().flatten())
        }
    };
    ($variant:ident {
        edge: $edge:ident,
        direction: $direction:ident
    }) => {
        methods!(@impl $variant from(edge: $edge, direction: $direction));
        methods!(@impl $variant to(edge: $edge, direction: $direction));
        methods!(@impl $variant every);
    };
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
            Intermediates::Record,
        )
    }

    fn path_to<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        target: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>>, Self::Error> {
        let iter = self.path_from(graph, target)?;

        Ok(iter.map(Route::reverse))
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
            Intermediates::Discard,
        )?;

        Ok(iter.map(|route| DirectRoute {
            source: route.path.source,
            target: route.path.target,
            cost: route.cost,
        }))
    }

    fn distance_to<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        target: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>>, Self::Error> {
        let iter = self.distance_from(graph, target)?;

        Ok(iter.map(DirectRoute::reverse))
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
            Intermediates::Record,
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
            Intermediates::Discard,
        )?;

        Ok(iter.map(|route| DirectRoute {
            source: route.path.source,
            target: route.path.target,
            cost: route.cost,
        }))
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
