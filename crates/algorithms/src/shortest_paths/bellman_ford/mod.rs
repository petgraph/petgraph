mod error;
mod iter;

use core::{hash::Hash, ops::Add};

use error_stack::Result;
use num_traits::Zero;
use petgraph_core::{
    base::MaybeOwned,
    edge::marker::{Directed, Undirected},
    Edge, GraphDirectionality, GraphStorage, Node,
};

use self::{
    error::BellmanFordError,
    iter::{BellManFordCandidateOrder, BellmanFordIter, Intermediates},
};
use super::{ShortestDistance, ShortestPath};

pub struct BellmanFord<D, F> {
    direction: D,
    edge_cost: F,
    candidate_order: BellManFordCandidateOrder,
}

impl BellmanFord<Directed, ()> {
    pub fn directed() -> Self {
        Self {
            direction: Directed,
            edge_cost: (),
            candidate_order: Default::default(),
        }
    }
}

impl BellmanFord<Undirected, ()> {
    pub fn directed() -> Self {
        Self {
            direction: Undirected,
            edge_cost: (),
            candidate_order: Default::default(),
        }
    }
}

impl<D, F> BellmanFord<D, F>
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
            candidate_order: Default::default(),
        }
    }

    pub fn without_edge_cost(self) -> BellmanFord<D, ()> {
        BellmanFord {
            direction: self.direction,
            edge_cost: (),
            candidate_order: Default::default(),
        }
    }

    pub fn with_candidate_order(self, label_order: BellManFordCandidateOrder) -> Self {
        Self {
            direction: self.direction,
            edge_cost: self.edge_cost,
            candidate_order: label_order,
        }
    }
}

impl<S> ShortestPath<S> for BellmanFord<Undirected, ()>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    E: GraphCost<S>,
    for<'a> E::Value: PartialOrd + Ord + Zero + Clone + 'a,
    for<'a> &'a E::Value: Add<Output = E::Value>,
{
    type Cost = S::EdgeWeight;
    type Error = BellmanFordError;

    fn path_from<'a>(
        &self,
        graph: &'a petgraph_core::Graph<S>,
        source: &'a <S as GraphStorage>::NodeId,
    ) -> Result<impl Iterator<Item = super::Route<'a, S, Self::Cost>>, Self::Error> {
        BellmanFordIter::new(
            graph,
            &self.edge_cost,
            Node::<'graph, S>::connections as fn(&Node<'graph, S>) -> _,
            source,
            Intermediates::Record,
            self.candidate_order,
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

impl<S> ShortestDistance<S> for BellmanFord<Undirected, ()>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    E: GraphCost<S>,
    for<'a> E::Value: PartialOrd + Ord + Zero + Clone + 'a,
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
        let iter = BellmanFordIter::new(
            graph,
            &self.edge_cost,
            Node::<'graph, S>::connections as fn(&Node<'graph, S>) -> _,
            source,
            Intermediates::Record,
            self.candidate_order,
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
    for<'a> E::Value: PartialOrd + Ord + Zero + Clone + 'a,
    for<'a> &'a E::Value: Add<Output = E::Value>,
{
    type Cost = E::Value;
    type Error = BellmanFordError;

    fn path_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        BellmanFordIter::new(
            graph,
            &self.edge_cost,
            outgoing_connections as fn(&Node<'graph, S>) -> _,
            source,
            Intermediates::Record,
            self.candidate_order,
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
    for<'a> E::Value: PartialOrd + Ord + Zero + Clone + 'a,
    for<'a> &'a E::Value: Add<Output = E::Value>,
{
    type Cost = E::Value;
    type Error = BellmanFordError;

    fn distance_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: &'graph <S as GraphStorage>::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>>, Self::Error> {
        let iter = BellmanFordIter::new(
            graph,
            &self.edge_cost,
            outgoing_connections as fn(&Node<'graph, S>) -> _,
            source,
            Intermediates::Discard,
            self.candidate_order,
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
