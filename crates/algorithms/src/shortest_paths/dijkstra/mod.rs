mod error;
mod iter;
#[cfg(test)]
mod tests;

use alloc::vec::Vec;
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
use self::iter::DijkstraIter;
use super::common::intermediates::Intermediates;
use crate::{
    polyfill::IteratorExt,
    shortest_paths::{DirectRoute, Route, ShortestDistance, ShortestPath},
};

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
        fn path_to<'a>(
            &self,
            graph: &'a Graph<S>,
            target: &'a <S as GraphStorage>::NodeId,
        ) -> Result<impl Iterator<Item = Route<'a, S, Self::Cost>>, Self::Error> {
            let iter = self.path_from(graph, target)?;

            Ok(iter.map(Route::reverse))
        }
    };
    (@impl path from(edge: $edge:ident, direction: $direction:ident)) => {
        fn path_from<'a>(
            &self,
            graph: &'a Graph<S>,
            source: &'a S::NodeId,
        ) -> Result<impl Iterator<Item = Route<'a, S, Self::Cost>>, Self::Error> {
            methods!(@impl any from(edge: $edge, direction: $direction, record: Record, 'a, self, graph, source) body)
        }
    };
    (@impl distance to(edge: $edge:ident, direction: undirected)) => {
        fn distance_to<'a>(
            &self,
            graph: &'a Graph<S>,
            target: &'a <S as GraphStorage>::NodeId,
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
        fn every_path<'a>(
            &self,
            graph: &'a Graph<S>,
        ) -> Result<impl Iterator<Item = Route<'a, S, Self::Cost>>, Self::Error> {
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

impl<S> ShortestPath<S> for Dijkstra<Undirected, ()>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    S::EdgeWeight: PartialOrd + Ord + Zero + Clone,
    for<'a> &'a S::EdgeWeight: Add<Output = S::EdgeWeight>,
{
    type Cost = S::EdgeWeight;
    type Error = DijkstraError;

    methods!(path {
        edge: default,
        direction: undirected
    });
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

    methods!(distance {
        edge: default,
        direction: undirected
    });
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

    methods!(path {
        edge: default,
        direction: directed
    });
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

    methods!(distance {
        edge: default,
        direction: directed
    });
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

    methods!(path {
        edge: fn,
        direction: undirected
    });
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

    methods!(distance {
        edge: fn,
        direction: undirected
    });
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

    methods!(path {
        edge: fn,
        direction: directed
    });
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

    methods!(distance {
        edge: fn,
        direction: directed
    });
}
