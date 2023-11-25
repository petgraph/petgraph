mod error;
mod iter;
#[cfg(test)]
mod tests;

use core::{hash::Hash, ops::Add};

use error_stack::Result;
use num_traits::{Bounded, Zero};
use petgraph_core::{
    edge::marker::{Directed, Undirected},
    DirectedGraphStorage, Graph, GraphDirectionality, GraphStorage, Node,
};

use self::{error::BellmanFordError, iter::ShortestPathFasterIter};
use super::{
    common::{
        connections::outgoing_connections,
        cost::{DefaultCost, GraphCost},
    },
    DirectRoute, Route, ShortestDistance, ShortestPath,
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
    for<'a> E::Value: PartialOrd + Ord + Zero + Bounded + Clone + 'a,
    for<'a> &'a E::Value: Add<Output = E::Value>,
{
    type Cost = E::Value;
    type Error = BellmanFordError;

    fn path_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph petgraph_core::Graph<S>,
        source: &'graph <S as GraphStorage>::NodeId,
    ) -> Result<impl Iterator<Item = super::Route<'graph, S, Self::Cost>>, Self::Error> {
        ShortestPathFasterIter::new(
            graph,
            &self.edge_cost,
            Node::<'graph, S>::connections as fn(&Node<'graph, S>) -> _,
            source,
            // self.candidate_order,
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
        graph: &'graph petgraph_core::Graph<S>,
    ) -> Result<impl Iterator<Item = super::Route<'graph, S, Self::Cost>> + 'this, Self::Error>
    {
        let iter = graph
            .nodes()
            .map(move |node| self.path_from(graph, node.id()))
            .collect_reports::<Vec<_>>()?;

        Ok(iter.into_iter().flatten())
    }
}

impl<S, E> ShortestDistance<S> for BellmanFord<Undirected, E>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    E: GraphCost<S>,
    for<'a> E::Value: PartialOrd + Ord + Zero + Zero + Bounded + Clone + 'a,
    for<'a> &'a E::Value: Add<Output = E::Value>,
{
    type Cost = E::Value;
    type Error = BellmanFordError;

    fn distance_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph petgraph_core::Graph<S>,
        source: &'graph <S as GraphStorage>::NodeId,
    ) -> Result<impl Iterator<Item = super::DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error>
    {
        let iter = ShortestPathFasterIter::new(
            graph,
            &self.edge_cost,
            Node::<'graph, S>::connections as fn(&Node<'graph, S>) -> _,
            source,
            // self.candidate_order,
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
        graph: &'graph petgraph_core::Graph<S>,
    ) -> Result<impl Iterator<Item = super::DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error>
    {
        let iter = graph
            .nodes()
            .map(move |node| self.distance_from(graph, node.id()))
            .collect_reports::<Vec<_>>()?;

        Ok(iter.into_iter().flatten())
    }
}

impl<S, E> ShortestPath<S> for BellmanFord<Directed, E>
where
    S: DirectedGraphStorage,
    S::NodeId: Eq + Hash,
    E: GraphCost<S>,
    for<'a> E::Value: PartialOrd + Ord + Zero + Zero + Bounded + Clone + 'a,
    for<'a> &'a E::Value: Add<Output = E::Value>,
{
    type Cost = E::Value;
    type Error = BellmanFordError;

    fn path_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        ShortestPathFasterIter::new(
            graph,
            &self.edge_cost,
            outgoing_connections as fn(&Node<'graph, S>) -> _,
            source,
            // self.candidate_order,
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

impl<S, E> ShortestDistance<S> for BellmanFord<Directed, E>
where
    S: DirectedGraphStorage,
    S::NodeId: Eq + Hash,
    E: GraphCost<S>,
    for<'a> E::Value: PartialOrd + Ord + Zero + Zero + Bounded + Clone + 'a,
    for<'a> &'a E::Value: Add<Output = E::Value>,
{
    type Cost = E::Value;
    type Error = BellmanFordError;

    fn distance_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: &'graph <S as GraphStorage>::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>>, Self::Error> {
        let iter = ShortestPathFasterIter::new(
            graph,
            &self.edge_cost,
            outgoing_connections as fn(&Node<'graph, S>) -> _,
            source,
            // self.candidate_order,
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
