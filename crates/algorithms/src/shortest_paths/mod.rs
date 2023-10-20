// mod astar;
mod astar;
mod bellman_ford;
mod common;
mod dijkstra;
// mod floyd_warshall;
// mod k_shortest_path_length;
// mod measure;
// mod total_ord;

use alloc::vec::{IntoIter, Vec};

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

impl<'a, S> Path<'a, S>
where
    S: GraphStorage,
{
    fn to_vec(self) -> Vec<Node<'a, S>> {
        let mut vec = Vec::with_capacity(self.intermediates.len() + 2);

        vec.push(self.source);
        vec.extend(self.intermediates);
        vec.push(self.target);

        vec
    }

    fn iter(&self) -> impl Iterator<Item = &Node<'a, S>> {
        let mut iter = self.intermediates.iter();

        iter.next_back();

        iter
    }
}

impl<'a, S> IntoIterator for Path<'a, S>
where
    S: GraphStorage,
{
    type IntoIter = IntoIter<Node<'a, S>>;
    type Item = Node<'a, S>;

    fn into_iter(self) -> Self::IntoIter {
        self.to_vec().into_iter()
    }
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

    fn path_to<'a>(
        &self,
        graph: &'a Graph<S>,
        target: &'a S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'a, S, Self::Cost>>, Self::Error> {
        let iter = self.every_path(graph)?;

        Ok(iter.filter(move |route| route.path.target.id() == target))
    }
    fn path_from<'a>(
        &self,
        graph: &'a Graph<S>,
        source: &'a S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'a, S, Self::Cost>>, Self::Error> {
        let iter = self.every_path(graph)?;

        Ok(iter.filter(move |route| route.path.source.id() == source))
    }
    fn path_between<'a>(
        &self,
        graph: &'a Graph<S>,
        source: &'a S::NodeId,
        target: &'a S::NodeId,
    ) -> Option<Route<'a, S, Self::Cost>> {
        self.every_path(graph)
            .ok()?
            .find(move |route| route.path.source.id() == source && route.path.target.id() == target)
    }
    fn every_path<'a>(
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

    fn distance_to<'a>(
        &self,
        graph: &'a Graph<S>,
        target: &'a S::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'a, S, Self::Cost>>, Self::Error> {
        let iter = self.every_distance(graph)?;

        Ok(iter.filter(move |route| route.target.id() == target))
    }
    fn distance_from<'a>(
        &self,
        graph: &'a Graph<S>,
        source: &'a S::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'a, S, Self::Cost>>, Self::Error> {
        let iter = self.every_distance(graph)?;

        Ok(iter.filter(move |route| route.source.id() == source))
    }
    fn distance_between(
        &self,
        graph: &Graph<S>,
        source: &S::NodeId,
        target: &S::NodeId,
    ) -> Option<Distance<Self::Cost>> {
        self.every_distance(graph)
            .ok()?
            .find(move |route| route.source.id() == source && route.target.id() == target)
            .map(|route| route.distance)
    }
    fn every_distance<'a>(
        &self,
        graph: &'a Graph<S>,
    ) -> Result<impl Iterator<Item = DirectRoute<'a, S, Self::Cost>>, Self::Error>;
}
