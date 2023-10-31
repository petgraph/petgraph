mod error;
mod r#impl;
mod matrix;
#[cfg(test)]
mod tests;

use core::marker::PhantomData;

use error_stack::Result;
use num_traits::{CheckedAdd, Zero};
use petgraph_core::{
    edge::marker::{Directed, Undirected},
    id::LinearGraphId,
    Graph, GraphStorage,
};

use crate::shortest_paths::{
    common::{
        cost::{DefaultCost, GraphCost},
        route::{DirectRoute, Route},
        transit::PredecessorMode,
    },
    floyd_warshall::{
        error::FloydWarshallError,
        r#impl::{
            init_directed_edge_distance, init_directed_edge_predecessor,
            init_undirected_edge_distance, init_undirected_edge_predecessor, FloydWarshallImpl,
        },
    },
    ShortestDistance, ShortestPath,
};

pub struct FloydWarshall<D, E> {
    edge_cost: E,

    direction: PhantomData<fn() -> *const D>,
}

impl FloydWarshall<Directed, DefaultCost> {
    pub fn directed() -> Self {
        Self {
            edge_cost: DefaultCost,
            direction: PhantomData,
        }
    }
}

impl FloydWarshall<Undirected, DefaultCost> {
    pub fn undirected() -> Self {
        Self {
            edge_cost: DefaultCost,
            direction: PhantomData,
        }
    }
}

impl<D, E> FloydWarshall<D, E> {
    pub fn with_edge_cost<S, F>(self, edge_cost: F) -> FloydWarshall<D, F>
    where
        S: GraphStorage,
        F: GraphCost<S>,
    {
        FloydWarshall {
            edge_cost,
            direction: self.direction,
        }
    }
}

impl<S, E> ShortestPath<S> for FloydWarshall<Undirected, E>
where
    S: GraphStorage,
    S::NodeId: LinearGraphId<S> + Clone + Send + Sync + 'static,
    E: GraphCost<S>,
    for<'a> E::Value: PartialOrd + CheckedAdd + Zero + Clone + 'a,
{
    type Cost = E::Value;
    type Error = FloydWarshallError;

    fn path_to<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        target: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        FloydWarshallImpl::new(
            graph,
            &self.edge_cost,
            PredecessorMode::Record,
            init_undirected_edge_distance::<S, E>,
            init_undirected_edge_predecessor::<S>,
        )
        .map(move |r#impl| r#impl.filter(move |_, target_node| target_node.id() == target))
    }

    fn path_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        FloydWarshallImpl::new(
            graph,
            &self.edge_cost,
            PredecessorMode::Record,
            init_undirected_edge_distance::<S, E>,
            init_undirected_edge_predecessor::<S>,
        )
        .map(move |r#impl| r#impl.filter(move |source_node, _| source_node.id() == source))
    }

    fn path_between<'graph>(
        &self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
        target: &'graph S::NodeId,
    ) -> Option<Route<'graph, S, Self::Cost>> {
        let r#impl = FloydWarshallImpl::new(
            graph,
            &self.edge_cost,
            PredecessorMode::Record,
            init_undirected_edge_distance::<S, E>,
            init_undirected_edge_predecessor::<S>,
        )
        .ok()?;

        r#impl
            .filter(|source_node, target_node| {
                source_node.id() == source && target_node.id() == target
            })
            .next()
    }

    fn every_path<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        FloydWarshallImpl::new(
            graph,
            &self.edge_cost,
            PredecessorMode::Record,
            init_undirected_edge_distance::<S, E>,
            init_undirected_edge_predecessor::<S>,
        )
        .map(|r#impl| r#impl.filter(|_, _| true))
    }
}

impl<S, E> ShortestDistance<S> for FloydWarshall<Undirected, E>
where
    S: GraphStorage,
    S::NodeId: LinearGraphId<S> + Clone + Send + Sync + 'static,
    E: GraphCost<S>,
    for<'a> E::Value: PartialOrd + CheckedAdd + Zero + Clone + 'a,
{
    type Cost = E::Value;
    type Error = FloydWarshallError;

    fn every_distance<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter = FloydWarshallImpl::new(
            graph,
            &self.edge_cost,
            PredecessorMode::Discard,
            init_undirected_edge_distance::<S, E>,
            init_undirected_edge_predecessor::<S>,
        )?;

        Ok(iter.filter(|_, _| true).map(From::from))
    }
}

impl<S, E> ShortestPath<S> for FloydWarshall<Directed, E>
where
    S: GraphStorage,
    S::NodeId: LinearGraphId<S> + Clone + Send + Sync + 'static,
    E: GraphCost<S>,
    for<'a> E::Value: PartialOrd + CheckedAdd + Zero + Clone + 'a,
{
    type Cost = E::Value;
    type Error = FloydWarshallError;

    fn path_to<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        target: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        FloydWarshallImpl::new(
            graph,
            &self.edge_cost,
            PredecessorMode::Record,
            init_directed_edge_distance::<S, E>,
            init_directed_edge_predecessor::<S>,
        )
        .map(move |r#impl| r#impl.filter(move |_, target_node| target_node.id() == target))
    }

    fn path_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        FloydWarshallImpl::new(
            graph,
            &self.edge_cost,
            PredecessorMode::Record,
            init_directed_edge_distance::<S, E>,
            init_directed_edge_predecessor::<S>,
        )
        .map(move |r#impl| r#impl.filter(move |source_node, _| source_node.id() == source))
    }

    // TODO: benchmark if the filter has an actual impact on performance
    fn path_between<'graph>(
        &self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
        target: &'graph S::NodeId,
    ) -> Option<Route<'graph, S, Self::Cost>> {
        let r#impl = FloydWarshallImpl::new(
            graph,
            &self.edge_cost,
            PredecessorMode::Record,
            init_directed_edge_distance::<S, E>,
            init_directed_edge_predecessor::<S>,
        )
        .ok()?;

        r#impl
            .filter(|source_node, target_node| {
                source_node.id() == source && target_node.id() == target
            })
            .next()
    }

    fn every_path<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        FloydWarshallImpl::new(
            graph,
            &self.edge_cost,
            PredecessorMode::Record,
            init_directed_edge_distance::<S, E>,
            init_directed_edge_predecessor::<S>,
        )
        .map(|r#impl| r#impl.filter(|_, _| true))
    }
}

impl<S, E> ShortestDistance<S> for FloydWarshall<Directed, E>
where
    S: GraphStorage,
    S::NodeId: LinearGraphId<S> + Clone + Send + Sync + 'static,
    E: GraphCost<S>,
    for<'a> E::Value: PartialOrd + CheckedAdd + Zero + Clone + 'a,
{
    type Cost = E::Value;
    type Error = FloydWarshallError;

    fn every_distance<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter = FloydWarshallImpl::new(
            graph,
            &self.edge_cost,
            PredecessorMode::Discard,
            init_directed_edge_distance::<S, E>,
            init_directed_edge_predecessor::<S>,
        )?;

        Ok(iter.filter(|_, _| true).map(From::from))
    }
}
