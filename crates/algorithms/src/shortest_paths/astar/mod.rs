mod error;
mod heuristic;
mod r#impl;
#[cfg(test)]
mod tests;

use alloc::vec::Vec;
use core::{hash::Hash, marker::PhantomData, ops::Add};

use error_stack::Result;
use num_traits::Zero;
use petgraph_core::{
    edge::{
        marker::{Directed, Undirected},
        Direction,
    },
    DirectedGraphStorage, Edge, Graph, GraphDirectionality, GraphStorage, Node,
};

use self::{error::AStarError, r#impl::AStarImpl};
use super::{
    common::transit::PredecessorMode, Cost, DirectRoute, Route, ShortestDistance, ShortestPath,
};
use crate::{
    polyfill::IteratorExt,
    shortest_paths::{
        astar::heuristic::GraphHeuristic,
        common::{
            connections::Connections,
            cost::{DefaultCost, GraphCost},
        },
    },
};

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
    edge_cost: E,
    heuristic: H,

    direction: PhantomData<fn() -> *const D>,
}

impl AStar<Directed, DefaultCost, ()> {
    pub fn directed() -> Self {
        Self {
            edge_cost: DefaultCost,
            heuristic: (),

            direction: PhantomData,
        }
    }
}

impl AStar<Undirected, DefaultCost, ()> {
    pub fn undirected() -> Self {
        Self {
            edge_cost: DefaultCost,
            heuristic: (),

            direction: PhantomData,
        }
    }
}

impl<D, E, H> AStar<D, E, H>
where
    D: GraphDirectionality,
{
    pub fn with_edge_cost<S, F>(self, edge_cost: F) -> AStar<D, F, H>
    where
        S: GraphStorage,
        F: GraphCost<S>,
    {
        AStar {
            edge_cost,
            heuristic: self.heuristic,

            direction: PhantomData,
        }
    }

    pub fn with_heuristic<S, I>(self, heuristic: I) -> AStar<D, E, I>
    where
        S: GraphStorage,
        I: GraphHeuristic<S>,
    {
        AStar {
            edge_cost: self.edge_cost,
            heuristic,

            direction: PhantomData,
        }
    }
}

impl<E, H> AStar<Directed, E, H> {
    fn call<'graph: 'this, 'this, S>(
        &'this self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
        target: &'graph S::NodeId,
        intermediates: PredecessorMode,
    ) -> Result<AStarImpl<'graph, 'this, S, E, H, impl Connections<'graph, S> + 'this>, AStarError>
    where
        S: DirectedGraphStorage,
        S::NodeId: Eq + Hash,
        E: GraphCost<S>,
        E::Value: PartialOrd + Ord + Zero + Clone + 'graph,
        for<'a> &'a E::Value: Add<Output = E::Value>,
        H: GraphHeuristic<S, Value = E::Value>,
    {
        AStarImpl::new(
            graph,
            &self.edge_cost,
            &self.heuristic,
            outgoing_connections as fn(&Node<'graph, S>) -> _,
            source,
            target,
            intermediates,
        )
    }
}

impl<E, H> AStar<Undirected, E, H> {
    fn call<'graph: 'this, 'this, S>(
        &'this self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
        target: &'graph S::NodeId,
        intermediates: PredecessorMode,
    ) -> Result<AStarImpl<'graph, 'this, S, E, H, impl Connections<'graph, S> + 'this>, AStarError>
    where
        S: GraphStorage,
        S::NodeId: Eq + Hash,
        E: GraphCost<S>,
        E::Value: PartialOrd + Ord + Zero + Clone + 'graph,
        for<'a> &'a E::Value: Add<Output = E::Value>,
        H: GraphHeuristic<S, Value = E::Value>,
    {
        AStarImpl::new(
            graph,
            &self.edge_cost,
            &self.heuristic,
            Node::<'graph, S>::connections as fn(&Node<'graph, S>) -> _,
            source,
            target,
            intermediates,
        )
    }
}

// in theory we could consolidate this even more, but there's a "problem" in that
// `outgoing_connections` is only available on directed graph storages.
// For now, while more code this is more flexible and more readable to the reader.
impl<S, E, H> ShortestPath<S> for AStar<Undirected, E, H>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    E: GraphCost<S>,
    for<'a> E::Value: PartialOrd + Ord + Zero + Clone + 'a,
    for<'a> &'a E::Value: Add<Output = E::Value>,
    H: GraphHeuristic<S, Value = E::Value>,
{
    type Cost = E::Value;
    type Error = AStarError;

    fn path_to<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        target: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let sources = graph.nodes().map(|node| node.id());

        let iter = sources
            .map(move |source| self.call(graph, source, target, PredecessorMode::Record))
            .collect_reports::<Vec<_>>()?;

        Ok(AStarImpl::find_all(iter))
    }

    fn path_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let targets = graph.nodes().map(|node| node.id());

        let iter = targets
            .map(move |target| self.call(graph, source, target, PredecessorMode::Record))
            .collect_reports::<Vec<_>>()?;

        Ok(AStarImpl::find_all(iter))
    }

    fn path_between<'graph>(
        &self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
        target: &'graph S::NodeId,
    ) -> Option<Route<'graph, S, Self::Cost>> {
        self.call(graph, source, target, PredecessorMode::Record)
            .ok()?
            .find()
    }

    fn every_path<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let sources = graph.nodes().map(|node| node.id());

        let iter = sources
            .flat_map(move |source| {
                graph
                    .nodes()
                    .map(|node| node.id())
                    .map(move |target| self.call(graph, source, target, PredecessorMode::Record))
            })
            .collect_reports::<Vec<_>>()?;

        Ok(AStarImpl::find_all(iter))
    }
}

impl<S, E, H> ShortestDistance<S> for AStar<Undirected, E, H>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    E: GraphCost<S>,
    for<'a> E::Value: PartialOrd + Ord + Zero + Clone + 'a,
    for<'a> &'a E::Value: Add<Output = E::Value>,
    H: GraphHeuristic<S, Value = E::Value>,
{
    type Cost = E::Value;
    type Error = AStarError;

    fn distance_to<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        target: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let sources = graph.nodes().map(|node| node.id());

        let iter = sources
            .map(move |source| self.call(graph, source, target, PredecessorMode::Discard))
            .collect_reports::<Vec<_>>()?;

        Ok(AStarImpl::find_all_direct(iter))
    }

    fn distance_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let targets = graph.nodes().map(|node| node.id());

        let iter = targets
            .map(move |target| self.call(graph, source, target, PredecessorMode::Discard))
            .collect_reports::<Vec<_>>()?;

        Ok(AStarImpl::find_all_direct(iter))
    }

    fn distance_between<'graph>(
        &self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
        target: &'graph S::NodeId,
    ) -> Option<Cost<Self::Cost>> {
        self.call(graph, source, target, PredecessorMode::Discard)
            .ok()?
            .find()
            .map(|route| route.cost)
    }

    fn every_distance<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let sources = graph.nodes().map(|node| node.id());

        let iter = sources
            .flat_map(move |source| {
                graph
                    .nodes()
                    .map(|node| node.id())
                    .map(move |target| self.call(graph, source, target, PredecessorMode::Discard))
            })
            .collect_reports::<Vec<_>>()?;

        Ok(AStarImpl::find_all_direct(iter))
    }
}

impl<S, E, H> ShortestPath<S> for AStar<Directed, E, H>
where
    S: DirectedGraphStorage,
    S::NodeId: Eq + Hash,
    E: GraphCost<S>,
    for<'a> E::Value: PartialOrd + Ord + Zero + Clone + 'a,
    for<'a> &'a E::Value: Add<Output = E::Value>,
    H: GraphHeuristic<S, Value = E::Value>,
{
    type Cost = E::Value;
    type Error = AStarError;

    fn path_to<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        target: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let sources = graph.nodes().map(|node| node.id());

        let iter = sources
            .map(move |source| self.call(graph, source, target, PredecessorMode::Record))
            .collect_reports::<Vec<_>>()?;

        Ok(AStarImpl::find_all(iter))
    }

    fn path_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let targets = graph.nodes().map(|node| node.id());

        let iter = targets
            .map(move |target| self.call(graph, source, target, PredecessorMode::Record))
            .collect_reports::<Vec<_>>()?;

        Ok(AStarImpl::find_all(iter))
    }

    fn path_between<'graph>(
        &self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
        target: &'graph S::NodeId,
    ) -> Option<Route<'graph, S, Self::Cost>> {
        self.call(graph, source, target, PredecessorMode::Record)
            .ok()?
            .find()
    }

    fn every_path<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let sources = graph.nodes().map(|node| node.id());

        let iter = sources
            .flat_map(move |source| {
                graph
                    .nodes()
                    .map(|node| node.id())
                    .map(move |target| self.call(graph, source, target, PredecessorMode::Record))
            })
            .collect_reports::<Vec<_>>()?;

        Ok(AStarImpl::find_all(iter))
    }
}

impl<S, E, H> ShortestDistance<S> for AStar<Directed, E, H>
where
    S: DirectedGraphStorage,
    S::NodeId: Eq + Hash,
    E: GraphCost<S>,
    for<'a> E::Value: PartialOrd + Ord + Zero + Clone + 'a,
    for<'a> &'a E::Value: Add<Output = E::Value>,
    H: GraphHeuristic<S, Value = E::Value>,
{
    type Cost = E::Value;
    type Error = AStarError;

    fn distance_to<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        target: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let sources = graph.nodes().map(|node| node.id());

        let iter = sources
            .map(move |source| self.call(graph, source, target, PredecessorMode::Discard))
            .collect_reports::<Vec<_>>()?;

        Ok(AStarImpl::find_all_direct(iter))
    }

    fn distance_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let targets = graph.nodes().map(|node| node.id());

        let iter = targets
            .map(move |target| self.call(graph, source, target, PredecessorMode::Discard))
            .collect_reports::<Vec<_>>()?;

        Ok(AStarImpl::find_all_direct(iter))
    }

    fn distance_between<'graph>(
        &self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
        target: &'graph S::NodeId,
    ) -> Option<Cost<Self::Cost>> {
        self.call(graph, source, target, PredecessorMode::Discard)
            .ok()?
            .find()
            .map(|route| route.cost)
    }

    fn every_distance<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let sources = graph.nodes().map(|node| node.id());

        let iter = sources
            .flat_map(move |source| {
                graph
                    .nodes()
                    .map(|node| node.id())
                    .map(move |target| self.call(graph, source, target, PredecessorMode::Discard))
            })
            .collect_reports::<Vec<_>>()?;

        Ok(AStarImpl::find_all_direct(iter))
    }
}
