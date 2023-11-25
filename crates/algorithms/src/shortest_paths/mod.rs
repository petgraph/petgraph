mod astar;
mod bellman_ford;
mod common;
mod dijkstra;
mod floyd_warshall;

use error_stack::{Context, Result};
use petgraph_core::{Graph, GraphStorage};

pub use self::{
    astar::AStar,
    common::{
        cost::Cost,
        path::Path,
        route::{DirectRoute, Route},
    },
    dijkstra::Dijkstra,
    floyd_warshall::FloydWarshall,
};

pub trait ShortestPath<S>
where
    S: GraphStorage,
{
    type Cost;
    type Error: Context;

    fn path_to<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        target: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter = self.every_path(graph)?;

        Ok(iter.filter(move |route| route.path().target().id() == target))
    }

    fn path_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter = self.every_path(graph)?;

        Ok(iter.filter(move |route| route.path().source().id() == source))
    }

    fn path_between<'graph>(
        &self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
        target: &'graph S::NodeId,
    ) -> Option<Route<'graph, S, Self::Cost>> {
        self.path_from(graph, source)
            .ok()?
            .find(|route| route.path().target().id() == target)
    }

    fn every_path<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error>;
}

pub trait ShortestDistance<S>
where
    S: GraphStorage,
{
    type Cost;
    type Error: Context;

    fn distance_to<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        target: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter = self.every_distance(graph)?;

        Ok(iter.filter(move |route| route.target().id() == target))
    }
    fn distance_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter = self.every_distance(graph)?;

        Ok(iter.filter(move |route| route.source().id() == source))
    }
    fn distance_between<'graph>(
        &self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
        target: &'graph S::NodeId,
    ) -> Option<Cost<Self::Cost>> {
        self.every_distance(graph)
            .ok()?
            .find(move |route| route.source().id() == source && route.target().id() == target)
            .map(|route| route.into_cost())
    }
    fn every_distance<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error>;
}
