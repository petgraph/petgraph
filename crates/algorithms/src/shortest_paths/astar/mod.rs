mod error;
mod r#impl;
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

use self::{error::AStarError, r#impl::AStarImpl};
use super::{
    common::intermediates::Intermediates, Cost, DirectRoute, Route, ShortestDistance, ShortestPath,
};
use crate::polyfill::IteratorExt;

fn outgoing_connections<'a, S>(node: &Node<'a, S>) -> impl Iterator<Item = Edge<'a, S>> + 'a
where
    S: DirectedGraphStorage,
{
    node.directed_connections(Direction::Outgoing)
}

pub struct AStar<D, E, H>
where
    D: GraphDirectionality,
{
    direction: D,

    edge_cost: E,
    heuristic: H,
}

impl AStar<Directed, (), ()> {
    pub fn directed() -> Self {
        Self {
            direction: Directed,
            edge_cost: (),
            heuristic: (),
        }
    }
}

impl AStar<Undirected, (), ()> {
    pub fn undirected() -> Self {
        Self {
            direction: Undirected,
            edge_cost: (),
            heuristic: (),
        }
    }
}

impl<D, E, H> AStar<D, E, H>
where
    D: GraphDirectionality,
{
    pub fn with_edge_cost<E2>(self, edge_cost: E2) -> AStar<D, E2, H> {
        AStar {
            direction: self.direction,
            edge_cost,
            heuristic: self.heuristic,
        }
    }

    pub fn without_edge_cost(self) -> AStar<D, (), H> {
        AStar {
            direction: self.direction,
            edge_cost: (),
            heuristic: self.heuristic,
        }
    }

    pub fn with_heuristic<H2>(self, heuristic: H2) -> AStar<D, E, H2> {
        AStar {
            direction: self.direction,
            edge_cost: self.edge_cost,
            heuristic,
        }
    }
}

macro_rules! call {
    (@impl edge($self:ident)default) => {
        |edge| MaybeOwned::Borrowed(edge.weight())
    };
    (@impl edge($self:ident)fn) => {
        &$self.edge_cost
    };
    (@impl direction($a:lifetime)undirected) => {
        Node::<$a, S>::connections as fn(&Node<$a, S>) -> _,
    };
    (@impl direction($self:ident)directed) => {
        outgoing_connections as fn(&Node<$a, S>) -> _
    };
    (
        $a:lifetime,
        $self:ident,
        $graph:ident,
        $source:ident,
        $target:ident,edge = $edge:ident,direction = $direction:ident,intermediates =
        $intermediates:ident
    ) => {{
        AStarImpl::new(
            $graph,
            call!(@impl edge($self) $edge),
            &$self.heuristic,
            call!(@impl direction($a) $direction),
            $source,
            $target,
            Intermediates::$intermediates,
        )
    }};
}

impl<S, H> ShortestPath<S> for AStar<Undirected, (), H>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    S::EdgeWeight: PartialOrd + Ord + Zero + Clone,
    for<'a> &'a S::EdgeWeight: Add<Output = S::EdgeWeight>,
    H: for<'a> Fn(Node<'a, S>, Node<'a, S>) -> MaybeOwned<'a, S::EdgeWeight>,
{
    type Cost = S::EdgeWeight;
    type Error = AStarError;

    fn path_to<'a>(
        &self,
        graph: &'a Graph<S>,
        target: &'a <S as GraphStorage>::NodeId,
    ) -> Result<impl Iterator<Item = Route<'a, S, Self::Cost>>, Self::Error> {
        let sources = graph.nodes().map(|node| node.id());

        let iter = sources
            .map(move |source| {
                call!(
                    'a,
                    self,
                    graph,
                    source,
                    target,
                    edge = default,
                    direction = undirected,
                    intermediates = Record
                )
            })
            .collect_reports::<Vec<_>>()?;

        Ok(iter.into_iter().filter_map(|r#impl| r#impl.find()))
    }

    fn path_from<'a>(
        &self,
        graph: &'a Graph<S>,
        source: &'a <S as GraphStorage>::NodeId,
    ) -> Result<impl Iterator<Item = Route<'a, S, Self::Cost>>, Self::Error> {
        let targets = graph.nodes().map(|node| node.id());

        let iter = targets
            .map(move |target| {
                call!(
                    'a,
                    self,
                    graph,
                    source,
                    target,
                    edge = default,
                    direction = undirected,
                    intermediates = Record
                )
            })
            .collect_reports::<Vec<_>>()?;

        Ok(iter.into_iter().filter_map(|r#impl| r#impl.find()))
    }

    fn path_between<'a>(
        &self,
        graph: &'a Graph<S>,
        source: &'a S::NodeId,
        target: &'a S::NodeId,
    ) -> Option<Route<'a, S, Self::Cost>> {
        call!(
            'a,
            self,
            graph,
            source,
            target,
            edge = default,
            direction = undirected,
            intermediates = Discard
        )?
        .find()
    }

    fn every_path<'a>(
        &self,
        graph: &'a Graph<S>,
    ) -> Result<impl Iterator<Item = Route<'a, S, Self::Cost>>, Self::Error> {
        let sources = graph.nodes().map(|node| node.id());

        let iter = sources
            .flat_map(move |source| {
                graph.nodes().map(|node| node.id()).map(move |target| {
                    call!(
                        'a,
                        self,
                        graph,
                        source,
                        target,
                        edge = default,
                        direction = undirected,
                        intermediates = Record
                    )
                })
            })
            .collect_reports::<Vec<_>>()?;

        Ok(iter.into_iter().filter_map(|r#impl| r#impl.find()))
    }
}

impl<S, H> ShortestDistance<S> for AStar<Undirected, (), H>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    S::EdgeWeight: PartialOrd + Ord + Zero + Clone,
    for<'a> &'a S::EdgeWeight: Add<Output = S::EdgeWeight>,
    H: for<'a> Fn(Node<'a, S>, Node<'a, S>) -> MaybeOwned<'a, S::EdgeWeight>,
{
    type Cost = S::EdgeWeight;
    type Error = AStarError;

    fn distance_to<'a>(
        &self,
        graph: &'a Graph<S>,
        target: &'a S::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'a, S, Self::Cost>>, Self::Error> {
        let sources = graph.nodes().map(|node| node.id());

        let iter = sources
            .map(move |source| {
                call!(
                    'a,
                    self,
                    graph,
                    source,
                    target,
                    edge = default,
                    direction = undirected,
                    intermediates = Record
                )
            })
            .collect_reports::<Vec<_>>()?;

        Ok(iter
            .into_iter()
            .filter_map(|r#impl| r#impl.find())
            .map(|route| DirectRoute {
                source: route.path.source,
                target: route.path.target,
                cost: route.cost,
            }))
    }

    fn distance_from<'a>(
        &self,
        graph: &'a Graph<S>,
        source: &'a S::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'a, S, Self::Cost>>, Self::Error> {
        let targets = graph.nodes().map(|node| node.id());

        let iter = targets
            .map(move |target| {
                call!(
                    'a,
                    self,
                    graph,
                    source,
                    target,
                    edge = default,
                    direction = undirected,
                    intermediates = Record
                )
            })
            .collect_reports::<Vec<_>>()?;

        Ok(iter
            .into_iter()
            .filter_map(|r#impl| r#impl.find())
            .map(|route| DirectRoute {
                source: route.path.source,
                target: route.path.target,
                cost: route.cost,
            }))
    }

    fn distance_between(
        &self,
        graph: &Graph<S>,
        source: &S::NodeId,
        target: &S::NodeId,
    ) -> Option<Cost<Self::Cost>> {
        call!(
            'a,
            self,
            graph,
            source,
            target,
            edge = default,
            direction = undirected,
            intermediates = Discard
        )?
        .find()
        .map(|route| route.cost)
    }

    fn every_distance<'a>(
        &self,
        graph: &'a Graph<S>,
    ) -> Result<impl Iterator<Item = DirectRoute<'a, S, Self::Cost>>, Self::Error> {
        let sources = graph.nodes().map(|node| node.id());

        let iter = sources
            .flat_map(move |source| {
                graph.nodes().map(|node| node.id()).map(move |target| {
                    call!(
                        'a,
                        self,
                        graph,
                        source,
                        target,
                        edge = default,
                        direction = undirected,
                        intermediates = Discard
                    )
                })
            })
            .collect_reports::<Vec<_>>()?;

        Ok(iter
            .into_iter()
            .filter_map(|r#impl| r#impl.find())
            .map(|route| DirectRoute {
                source: route.path.source,
                target: route.path.target,
                cost: route.cost,
            }))
    }
}
