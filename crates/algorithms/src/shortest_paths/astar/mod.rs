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
    edge::marker::{Directed, Undirected},
    Edge, Graph, GraphDirectionality, GraphStorage, Node,
};

use self::{error::AStarError, r#impl::AStarImpl};
use super::{common::intermediates::Intermediates, Route, ShortestPath};
use crate::polyfill::IteratorExt;

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

impl<H> AStar<Undirected, (), H> {
    fn between<'a, S>(
        &self,
        graph: &'a Graph<S>,
        source: &'a S::NodeId,
        target: &'a S::NodeId,
    ) -> Result<Option<Route<'a, S, S::EdgeWeight>>, AStarError>
    where
        S: GraphStorage,
        S::NodeId: Eq + Hash,
        S::EdgeWeight: PartialOrd + Ord + Zero + Clone,
        for<'b> &'b S::EdgeWeight: Add<Output = S::EdgeWeight>,
        H: Fn(Node<'a, S>, Node<'a, S>) -> MaybeOwned<'a, S::EdgeWeight>,
    {
        let r#impl = AStarImpl::new(
            graph,
            |edge| MaybeOwned::Borrowed(edge.weight()),
            &self.heuristic,
            Node::<'a, S>::connections as fn(&Node<'a, S>) -> _,
            source,
            target,
            Intermediates::Record,
        )?;

        Ok(r#impl.find())
    }
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
            .map(move |source| self.between(graph, source, target))
            .collect_reports::<Vec<_>>()?;

        Ok(iter.into_iter().filter_map(|route| route))
    }

    fn path_from<'a>(
        &self,
        graph: &'a Graph<S>,
        source: &'a <S as GraphStorage>::NodeId,
    ) -> Result<impl Iterator<Item = Route<'a, S, Self::Cost>>, Self::Error> {
        let targets = graph.nodes().map(|node| node.id());

        let iter = targets
            .map(move |target| self.between(graph, source, target))
            .collect_reports::<Vec<_>>()?;

        Ok(iter.into_iter().filter_map(|route| route))
    }

    fn path_between<'a>(
        &self,
        graph: &'a Graph<S>,
        source: &'a S::NodeId,
        target: &'a S::NodeId,
    ) -> Option<Route<'a, S, Self::Cost>> {
        match self.between(graph, source, target) {
            Ok(route) => route,
            Err(_) => None,
        }
    }

    fn every_path<'a>(
        &self,
        graph: &'a Graph<S>,
    ) -> Result<impl Iterator<Item = Route<'a, S, Self::Cost>>, Self::Error> {
        let sources = graph.nodes().map(|node| node.id());

        let iter = sources
            .flat_map(move |source| {
                let targets = graph.nodes().map(|node| node.id());

                targets.map(move |target| self.between(graph, source, target))
            })
            .collect_reports::<Vec<_>>()?;

        Ok(iter.into_iter().filter_map(|route| route))
    }
}

// TODO: all others... (quite a lot...) Do we want to delegate this to macros? Probably a good idea?
//  We could also merge all of them into one, but that would _very_ messy and unreadable in
//  documentation.
