// mod astar;
// mod bellman_ford;
mod dijkstra;
// mod floyd_warshall;
// mod k_shortest_path_length;
// mod measure;
// mod total_ord;

use alloc::vec::Vec;

use error_stack::{Context, Result};
use petgraph_core::{Graph, GraphStorage, Node};

struct Distance<T> {
    value: T,
}

struct Path<'a, S>
where
    S: GraphStorage,
{
    source: Node<'a, S>,
    target: Node<'a, S>,

    intermediates: Vec<Node<'a, S>>,
}

struct Route<'a, S, T>
where
    S: GraphStorage,
{
    path: Path<'a, S>,

    distance: Distance<T>,
}

struct DirectRoute<'a, S, T>
where
    S: GraphStorage,
{
    source: Node<'a, S>,
    target: Node<'a, S>,

    distance: Distance<T>,
}

pub trait ShortestPath<S>
where
    S: GraphStorage,
{
    type Cost;
    type Error: Context;

    fn to<'a>(
        &self,
        graph: &'a Graph<S>,
        target: &'a S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'a, S, Self::Cost>>, Self::Error> {
        let iter = self.every(graph)?;

        Ok(iter.filter(move |route| route.path.target.id() == target))
    }
    fn from<'a>(
        &self,
        graph: &'a Graph<S>,
        source: &'a S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'a, S, Self::Cost>>, Self::Error> {
        let iter = self.every(graph)?;

        Ok(iter.filter(move |route| route.path.source.id() == source))
    }
    fn between<'a>(
        &self,
        graph: &'a Graph<S>,
        source: &'a S::NodeId,
        target: &'a S::NodeId,
    ) -> Option<Route<'a, S, Self::Cost>> {
        self.every(graph)
            .ok()?
            .find(move |route| route.path.source.id() == source && route.path.target.id() == target)
    }
    fn every<'a>(
        &self,
        graph: &'a Graph<S>,
    ) -> Result<impl Iterator<Item = Route<'a, S, Self::Cost>>, Self::Error>;
}

pub trait ShortestDistance<S>
where
    S: GraphStorage,
{
    type Cost;
    type Error: Context;

    fn to<'a>(
        &self,
        graph: &'a Graph<S>,
        target: &'a S::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'a, S, Self::Cost>>, Self::Error> {
        let iter = self.every(graph)?;

        Ok(iter.filter(move |route| route.target.id() == target))
    }
    fn from<'a>(
        &self,
        graph: &'a Graph<S>,
        source: &'a S::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'a, S, Self::Cost>>, Self::Error> {
        let iter = self.every(graph)?;

        Ok(iter.filter(move |route| route.source.id() == source))
    }
    fn between(
        &self,
        graph: &Graph<S>,
        source: &S::NodeId,
        target: &S::NodeId,
    ) -> Option<Distance<Self::Cost>> {
        self.every(graph)
            .ok()?
            .find(move |route| route.source.id() == source && route.target.id() == target)
            .map(|route| route.distance)
    }
    fn every<'a>(
        &self,
        graph: &'a Graph<S>,
    ) -> Result<impl Iterator<Item = DirectRoute<'a, S, Self::Cost>>, Self::Error>;
}
