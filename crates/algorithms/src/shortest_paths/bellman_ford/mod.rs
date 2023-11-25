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

use self::{error::ShortestPathFasterError, iter::ShortestPathFasterIter};
use super::{
    common::{
        connections::outgoing_connections,
        cost::{DefaultCost, GraphCost},
    },
    DirectRoute, Route, ShortestDistance, ShortestPath,
};
use crate::polyfill::IteratorExt;

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum SPFACandidateOrder {
    #[default]
    SmallFirst,
    LargeLast,
}

pub struct ShortestPathFaster<D, E> {
    direction: D,
    edge_cost: E,
    candidate_order: SPFACandidateOrder,
}

impl ShortestPathFaster<Directed, DefaultCost> {
    pub fn directed() -> Self {
        Self {
            direction: Directed,
            edge_cost: DefaultCost,
            candidate_order: Default::default(),
        }
    }
}

impl ShortestPathFaster<Undirected, DefaultCost> {
    pub fn undirected() -> Self {
        Self {
            direction: Undirected,
            edge_cost: DefaultCost,
            candidate_order: Default::default(),
        }
    }
}

impl<D, E> ShortestPathFaster<D, E>
where
    D: GraphDirectionality,
{
    pub fn with_edge_cost<S, F>(self, edge_cost: F) -> ShortestPathFaster<D, F>
    where
        S: GraphStorage,
        F: GraphCost<S>,
    {
        ShortestPathFaster {
            direction: self.direction,
            edge_cost,
            candidate_order: Default::default(),
        }
    }

    pub fn with_candidate_order(self, candidate_order: SPFACandidateOrder) -> Self {
        Self {
            direction: self.direction,
            edge_cost: self.edge_cost,
            candidate_order,
        }
    }
}

impl<S, E> ShortestPath<S> for ShortestPathFaster<Undirected, E>
where
    S: GraphStorage,
    S::NodeId: PartialEq + Eq + Hash,
    E: GraphCost<S>,
    for<'a> E::Value: PartialOrd + Ord + Zero + Bounded + Clone + 'a,
    for<'a> &'a E::Value: Add<Output = E::Value>,
{
    type Cost = E::Value;
    type Error = ShortestPathFasterError;

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

impl<S, E> ShortestDistance<S> for ShortestPathFaster<Undirected, E>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    E: GraphCost<S>,
    for<'a> E::Value: PartialOrd + Ord + Zero + Zero + Bounded + Clone + 'a,
    for<'a> &'a E::Value: Add<Output = E::Value>,
{
    type Cost = E::Value;
    type Error = ShortestPathFasterError;

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

impl<S, E> ShortestPath<S> for ShortestPathFaster<Directed, E>
where
    S: DirectedGraphStorage,
    S::NodeId: Eq + Hash,
    E: GraphCost<S>,
    for<'a> E::Value: PartialOrd + Ord + Zero + Zero + Bounded + Clone + 'a,
    for<'a> &'a E::Value: Add<Output = E::Value>,
{
    type Cost = E::Value;
    type Error = ShortestPathFasterError;

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

impl<S, E> ShortestDistance<S> for ShortestPathFaster<Directed, E>
where
    S: DirectedGraphStorage,
    S::NodeId: Eq + Hash,
    E: GraphCost<S>,
    for<'a> E::Value: PartialOrd + Ord + Zero + Zero + Bounded + Clone + 'a,
    for<'a> &'a E::Value: Add<Output = E::Value>,
{
    type Cost = E::Value;
    type Error = ShortestPathFasterError;

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
