// mod astar;
// mod bellman_ford;
mod astar;
mod common;
mod dijkstra;
// mod floyd_warshall;
// mod k_shortest_path_length;
// mod measure;
// mod total_ord;

use alloc::vec::{IntoIter, Vec};

use error_stack::{Context, Result};
use petgraph_core::{Graph, GraphStorage, Node};

pub use self::{astar::AStar, dijkstra::Dijkstra};

pub struct Cost<T>(T);

impl<T> Cost<T> {
    fn value(&self) -> &T {
        &self.0
    }

    fn into_value(self) -> T {
        self.0
    }
}

pub struct Path<'a, S>
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
    #[must_use]
    pub fn to_vec(self) -> Vec<Node<'a, S>> {
        let mut vec = Vec::with_capacity(self.intermediates.len() + 2);

        vec.push(self.source);
        vec.extend(self.intermediates);
        vec.push(self.target);

        vec
    }

    pub fn iter(&self) -> impl Iterator<Item = &Node<'a, S>> {
        let mut iter = self.intermediates.iter();

        iter.next_back();

        iter
    }

    fn reverse(self) -> Self {
        let mut intermediates = self.intermediates;

        intermediates.reverse();

        Self {
            source: self.target,
            target: self.source,
            intermediates,
        }
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

pub struct Route<'a, S, T>
where
    S: GraphStorage,
{
    path: Path<'a, S>,

    cost: Cost<T>,
}

impl<'a, S, T> Route<'a, S, T>
where
    S: GraphStorage,
{
    fn reverse(self) -> Self {
        Self {
            path: self.path.reverse(),
            cost: self.cost,
        }
    }
}

pub struct DirectRoute<'a, S, T>
where
    S: GraphStorage,
{
    source: Node<'a, S>,
    target: Node<'a, S>,

    cost: Cost<T>,
}

impl<'a, S, T> DirectRoute<'a, S, T>
where
    S: GraphStorage,
{
    fn reverse(self) -> Self {
        Self {
            source: self.target,
            target: self.source,
            cost: self.cost,
        }
    }
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
        self.path_from(graph, source)
            .ok()?
            .find(|route| route.path.target.id() == target)
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
    fn distance_between<'a>(
        &self,
        graph: &'a Graph<S>,
        source: &'a S::NodeId,
        target: &'a S::NodeId,
    ) -> Option<Cost<Self::Cost>> {
        self.every_distance(graph)
            .ok()?
            .find(move |route| route.source.id() == source && route.target.id() == target)
            .map(|route| route.cost)
    }
    fn every_distance<'a>(
        &self,
        graph: &'a Graph<S>,
    ) -> Result<impl Iterator<Item = DirectRoute<'a, S, Self::Cost>>, Self::Error>;
}
