//! A* shortest path algorithm.
mod error;
mod heuristic;
mod r#impl;
mod measure;
#[cfg(test)]
mod tests;

use alloc::vec::Vec;
use core::{hash::Hash, marker::PhantomData, ops::Add};

use error_stack::Result;
use petgraph_core::{
    edge::marker::{Directed, Undirected},
    DirectedGraphStorage, Graph, GraphDirectionality, GraphStorage, Node,
};

use self::r#impl::AStarImpl;
pub use self::{error::AStarError, heuristic::GraphHeuristic, measure::AStarMeasure};
use super::{
    common::{
        connections::{outgoing_connections, Connections},
        cost::{Cost, DefaultCost, GraphCost},
        route::{DirectRoute, Route},
        transit::PredecessorMode,
    },
    ShortestDistance, ShortestPath,
};
use crate::polyfill::IteratorExt;

/// A* shortest path algorithm.
///
/// A* is a shortest path algorithm that uses a heuristic to guide the search.
/// It is an extension of Dijkstra's algorithm, and is guaranteed to find the shortest path if the
/// heuristic is admissible.
/// A heuristic is admissible if it never overestimates the cost to reach the goal.
///
/// This implementation of A* is generic over the graph directionality, edge cost, and heuristic.
///
/// Edge weights need to implement [`AStarMeasure`], a trait that is automatically implemented for
/// all types that satisfy the constraints of the algorithm.
pub struct AStar<D, E, H>
where
    D: GraphDirectionality,
{
    edge_cost: E,
    heuristic: H,

    direction: PhantomData<fn() -> *const D>,
}

impl AStar<Directed, DefaultCost, ()> {
    /// Create a new A* instance with the default edge cost and no heuristic.
    ///
    /// You won't be able to run A* without providing a heuristic using [`Self::with_heuristic`].
    ///
    /// If instantiated for a directed graph, [`AStar`] will not implement [`ShortestPath`] and
    /// [`ShortestDistance`] on undirected graphs.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_algorithms::shortest_paths::AStar;
    /// use petgraph_core::{base::MaybeOwned, edge::marker::Directed, GraphStorage, Node};
    /// use petgraph_dino::{DiDinoGraph, DinoStorage};
    ///
    /// // TODO: heuristic utils
    /// fn heuristic<S>(source: Node<S>, target: Node<S>) -> MaybeOwned<i32>
    /// where
    ///     S: GraphStorage<NodeWeight = (i32, i32), EdgeWeight = i32>,
    /// {
    ///     let source = source.weight();
    ///     let target = target.weight();
    ///
    ///     MaybeOwned::Owned((source.0 - target.0).abs() + (source.1 - target.1).abs())
    /// }
    ///
    /// let algorithm = AStar::directed().with_heuristic(heuristic);
    ///
    /// let mut graph = DiDinoGraph::new();
    /// let a = *graph.insert_node((0, 1)).id();
    /// let b = *graph.insert_node((2, 2)).id();
    ///
    /// graph.insert_edge(5, &a, &b);
    /// ```
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
        E::Value: AStarMeasure,
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
        E::Value: AStarMeasure,
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
    E::Value: AStarMeasure,
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
    E::Value: AStarMeasure,
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
            .map(|route| route.into_cost())
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
    E::Value: AStarMeasure,
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
    E::Value: AStarMeasure,
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
            .map(|route| route.into_cost())
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
