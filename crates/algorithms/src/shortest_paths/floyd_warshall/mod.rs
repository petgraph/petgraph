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
        intermediates::Intermediates,
    },
    floyd_warshall::{
        error::FloydWarshallError,
        r#impl::{
            init_directed_edge_distance, init_directed_edge_predecessor,
            init_undirected_edge_distance, init_undirected_edge_predecessor, FloydWarshallImpl,
        },
    },
    DirectRoute, Route, ShortestDistance, ShortestPath,
};

struct FloydWarshall<D, E> {
    edge_cost: E,

    direction: PhantomData<fn() -> *const D>,
}

impl FloydWarshall<Directed, DefaultCost> {
    fn directed() -> Self {
        Self {
            edge_cost: DefaultCost,
            direction: PhantomData,
        }
    }
}

impl FloydWarshall<Undirected, DefaultCost> {
    fn undirected() -> Self {
        Self {
            edge_cost: DefaultCost,
            direction: PhantomData,
        }
    }
}

impl<D, E> FloydWarshall<D, E> {
    fn with_edge_cost<S, F>(self, edge_cost: F) -> FloydWarshall<D, F>
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

    fn every_path<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter = FloydWarshallImpl::new(
            graph,
            &self.edge_cost,
            Intermediates::Record,
            init_undirected_edge_distance::<S, E>,
            init_undirected_edge_predecessor::<S>,
        )?
        .iter();

        Ok(iter)
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
            Intermediates::Discard,
            init_undirected_edge_distance::<S, E>,
            init_undirected_edge_predecessor::<S>,
        )?;

        Ok(iter.iter().map(|route| DirectRoute {
            source: route.path.source,
            target: route.path.target,
            cost: route.cost,
        }))
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

    fn every_path<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter = FloydWarshallImpl::new(
            graph,
            &self.edge_cost,
            Intermediates::Record,
            init_directed_edge_distance::<S, E>,
            init_directed_edge_predecessor::<S>,
        )?;

        Ok(iter.iter())
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
            Intermediates::Discard,
            init_directed_edge_distance::<S, E>,
            init_directed_edge_predecessor::<S>,
        )?;

        Ok(iter.iter().map(|route| DirectRoute {
            source: route.path.source,
            target: route.path.target,
            cost: route.cost,
        }))
    }
}
