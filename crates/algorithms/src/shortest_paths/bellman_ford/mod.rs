mod error;
mod r#impl;
mod measure;
#[cfg(test)]
mod tests;

use alloc::vec::Vec;
use core::hash::Hash;

use error_stack::Result;
use petgraph_core::{
    edge::marker::{Directed, Undirected},
    DirectedGraphStorage, Graph, GraphDirectionality, GraphStorage, Node,
};

use self::r#impl::ShortestPathFasterImpl;
pub use self::{error::BellmanFordError, measure::BellmanFordMeasure};
use super::{
    common::{
        connections::outgoing_connections,
        cost::{DefaultCost, GraphCost},
        transit::PredecessorMode,
    },
    Cost, DirectRoute, Route, ShortestDistance, ShortestPath,
};
use crate::polyfill::IteratorExt;

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum CandidateOrder {
    #[default]
    SmallFirst,
    LargeLast,
}

pub struct BellmanFord<D, E> {
    direction: D,
    edge_cost: E,
    candidate_order: CandidateOrder,
    negative_cycle_heuristics: bool,
}

impl BellmanFord<Directed, DefaultCost> {
    pub fn directed() -> Self {
        Self {
            direction: Directed,
            edge_cost: DefaultCost,
            candidate_order: CandidateOrder::default(),
            negative_cycle_heuristics: false,
        }
    }
}

impl BellmanFord<Undirected, DefaultCost> {
    pub fn undirected() -> Self {
        Self {
            direction: Undirected,
            edge_cost: DefaultCost,
            candidate_order: CandidateOrder::default(),
            negative_cycle_heuristics: false,
        }
    }
}

impl<D, E> BellmanFord<D, E>
where
    D: GraphDirectionality,
{
    pub fn with_edge_cost<S, F>(self, edge_cost: F) -> BellmanFord<D, F>
    where
        S: GraphStorage,
        F: GraphCost<S>,
    {
        BellmanFord {
            direction: self.direction,
            edge_cost,
            candidate_order: self.candidate_order,
            negative_cycle_heuristics: self.negative_cycle_heuristics,
        }
    }

    pub fn with_candidate_order(self, candidate_order: CandidateOrder) -> Self {
        Self {
            direction: self.direction,
            edge_cost: self.edge_cost,
            candidate_order,
            negative_cycle_heuristics: self.negative_cycle_heuristics,
        }
    }

    pub fn with_negative_cycle_heuristics(self, negative_cycle_heuristics: bool) -> Self {
        Self {
            direction: self.direction,
            edge_cost: self.edge_cost,
            candidate_order: self.candidate_order,
            negative_cycle_heuristics,
        }
    }
}

impl<S, E> ShortestPath<S> for BellmanFord<Undirected, E>
where
    S: GraphStorage,
    S::NodeId: PartialEq + Eq + Hash,
    E: GraphCost<S>,
    E::Value: BellmanFordMeasure,
{
    type Cost = E::Value;
    type Error = BellmanFordError;

    fn path_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: &'graph <S as GraphStorage>::NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>>, Self::Error> {
        ShortestPathFasterImpl::new(
            graph,
            &self.edge_cost,
            Node::<'graph, S>::connections as fn(&Node<'graph, S>) -> _,
            source,
            PredecessorMode::Record,
            self.candidate_order,
            self.negative_cycle_heuristics,
        )
        .map(ShortestPathFasterImpl::all)
    }

    fn path_to<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        target: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>>, Self::Error> {
        let iter = self.path_from(graph, target)?;

        Ok(iter.map(Route::reverse))
    }

    fn path_between<'graph>(
        &self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
        target: &'graph S::NodeId,
    ) -> Option<Route<'graph, S, Self::Cost>> {
        ShortestPathFasterImpl::new(
            graph,
            &self.edge_cost,
            Node::<'graph, S>::connections as fn(&Node<'graph, S>) -> _,
            source,
            PredecessorMode::Record,
            self.candidate_order,
            self.negative_cycle_heuristics,
        )
        .ok()?
        .between(target)
    }

    fn every_path<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter: Vec<_> = graph
            .nodes()
            .map(move |node| self.path_from(graph, node.id()))
            .collect_reports()?;

        Ok(iter.into_iter().flatten())
    }
}

impl<S, E> ShortestDistance<S> for BellmanFord<Undirected, E>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    E: GraphCost<S>,
    E::Value: BellmanFordMeasure,
{
    type Cost = E::Value;
    type Error = BellmanFordError;

    fn distance_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: &'graph <S as GraphStorage>::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter = ShortestPathFasterImpl::new(
            graph,
            &self.edge_cost,
            Node::<'graph, S>::connections as fn(&Node<'graph, S>) -> _,
            source,
            PredecessorMode::Discard,
            self.candidate_order,
            self.negative_cycle_heuristics,
        )?;

        Ok(iter.all().map(Into::into))
    }

    fn distance_to<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        target: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>>, Self::Error> {
        let iter = self.distance_from(graph, target)?;

        Ok(iter.map(DirectRoute::reverse))
    }

    fn distance_between<'graph>(
        &self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
        target: &'graph S::NodeId,
    ) -> Option<Cost<Self::Cost>> {
        ShortestPathFasterImpl::new(
            graph,
            &self.edge_cost,
            Node::<'graph, S>::connections as fn(&Node<'graph, S>) -> _,
            source,
            PredecessorMode::Discard,
            self.candidate_order,
            self.negative_cycle_heuristics,
        )
        .ok()?
        .between(target)
        .map(Route::into_cost)
    }

    fn every_distance<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter: Vec<_> = graph
            .nodes()
            .map(move |node| self.distance_from(graph, node.id()))
            .collect_reports()?;

        Ok(iter.into_iter().flatten())
    }
}

impl<S, E> ShortestPath<S> for BellmanFord<Directed, E>
where
    S: DirectedGraphStorage,
    S::NodeId: Eq + Hash,
    E: GraphCost<S>,
    E::Value: BellmanFordMeasure,
{
    type Cost = E::Value;
    type Error = BellmanFordError;

    fn path_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        ShortestPathFasterImpl::new(
            graph,
            &self.edge_cost,
            outgoing_connections as fn(&Node<'graph, S>) -> _,
            source,
            PredecessorMode::Record,
            self.candidate_order,
            self.negative_cycle_heuristics,
        )
        .map(ShortestPathFasterImpl::all)
    }

    fn path_between<'graph>(
        &self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
        target: &'graph S::NodeId,
    ) -> Option<Route<'graph, S, Self::Cost>> {
        ShortestPathFasterImpl::new(
            graph,
            &self.edge_cost,
            outgoing_connections as fn(&Node<'graph, S>) -> _,
            source,
            PredecessorMode::Record,
            self.candidate_order,
            self.negative_cycle_heuristics,
        )
        .ok()?
        .between(target)
    }

    fn every_path<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter: Vec<_> = graph
            .nodes()
            .map(move |node| self.path_from(graph, node.id()))
            .collect_reports()?;

        Ok(iter.into_iter().flatten())
    }
}

impl<S, E> ShortestDistance<S> for BellmanFord<Directed, E>
where
    S: DirectedGraphStorage,
    S::NodeId: Eq + Hash,
    E: GraphCost<S>,
    E::Value: BellmanFordMeasure,
{
    type Cost = E::Value;
    type Error = BellmanFordError;

    fn distance_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: &'graph <S as GraphStorage>::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>>, Self::Error> {
        let iter = ShortestPathFasterImpl::new(
            graph,
            &self.edge_cost,
            outgoing_connections as fn(&Node<'graph, S>) -> _,
            source,
            PredecessorMode::Discard,
            self.candidate_order,
            self.negative_cycle_heuristics,
        )?;

        Ok(iter.all().map(Into::into))
    }

    fn distance_between<'graph>(
        &self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
        target: &'graph S::NodeId,
    ) -> Option<Cost<Self::Cost>> {
        ShortestPathFasterImpl::new(
            graph,
            &self.edge_cost,
            outgoing_connections as fn(&Node<'graph, S>) -> _,
            source,
            PredecessorMode::Discard,
            self.candidate_order,
            self.negative_cycle_heuristics,
        )
        .ok()?
        .between(target)
        .map(Route::into_cost)
    }

    fn every_distance<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter: Vec<_> = graph
            .nodes()
            .map(move |node| self.distance_from(graph, node.id()))
            .collect_reports()?;

        Ok(iter.into_iter().flatten())
    }
}
