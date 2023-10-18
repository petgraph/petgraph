mod astar;
// TODO: this is currently pub because of `Paths`, I'd like to rename it and put it into this module
//  instead.
mod bellman_ford;
mod dijkstra;
mod floyd_warshall;
mod k_shortest_path_length;
mod measure;
mod total_ord;

use alloc::vec::Vec;

pub use astar::astar;
pub use bellman_ford::{bellman_ford, find_negative_cycle, Paths};
pub use dijkstra::dijkstra;
pub use floyd_warshall::floyd_warshall;
pub use k_shortest_path_length::k_shortest_path_length;
pub use measure::{BoundedMeasure, FloatMeasure, Measure};
use petgraph_core::{Graph, GraphStorage};
pub use total_ord::TotalOrd;

struct Distance<S>
where
    S: GraphStorage,
{
    value: S::EdgeWeight,
}

struct Path<'a, S>
where
    S: GraphStorage,
{
    source: &'a S::NodeId,
    target: &'a S::NodeId,

    intermediates: Vec<&'a S::NodeId>,
}

struct Route<'a, S>
where
    S: GraphStorage,
{
    path: Path<'a, S>,

    distance: Distance<S>,
}

struct DirectRoute<'a, S>
where
    S: GraphStorage,
{
    source: &'a S::NodeId,
    target: &'a S::NodeId,

    distance: Distance<S>,
}

pub trait ShortestPath<S>
where
    S: GraphStorage,
{
    fn to(self, graph: &Graph<S>, target: &S::NodeId) -> impl Iterator<Item = Route<S>> {
        self.every(graph)
            .filter(move |route| route.path.target == target)
    }
    fn from(self, graph: &Graph<S>, source: &S::NodeId) -> impl Iterator<Item = Route<S>> {
        self.every(graph)
            .filter(move |route| route.path.source == source)
    }
    fn between(self, graph: &Graph<S>, source: &S::NodeId, target: &S::NodeId) -> Option<Route<S>> {
        self.every(graph)
            .find(move |route| route.path.source == source && route.path.target == target)
    }
    fn every(self, graph: &Graph<S>) -> impl Iterator<Item = Route<S>>;
}

pub trait ShortestDistance<S>
where
    S: GraphStorage,
{
    fn to(self, graph: &Graph<S>, target: &S::NodeId) -> impl Iterator<Item = DirectRoute<S>> {
        self.every(graph)
            .filter(move |route| route.target == target)
    }
    fn from(self, graph: &Graph<S>, source: &S::NodeId) -> impl Iterator<Item = DirectRoute<S>> {
        self.every(graph)
            .filter(move |route| route.source == source)
    }
    fn between(
        self,
        graph: &Graph<S>,
        source: &S::NodeId,
        target: &S::NodeId,
    ) -> Option<Distance<S>> {
        self.every(graph)
            .find(move |route| route.source == source && route.target == target)
            .map(|route| route.distance)
    }
    fn every(self, graph: &Graph<S>) -> impl Iterator<Item = DirectRoute<S>>;
}
