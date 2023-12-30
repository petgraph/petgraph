//! # Shortest Path Module
//!
//! This module contains traits and implementations for shortest path and shortest distance
//! algorithms.
//! These algorithms are used to find the shortest path or distance between two nodes in a graph.
//!
//! ## Traits
//!
//! - [`ShortestPath`]: Any implementation of this trait can be used to find the shortest path,
//!   depending on algorithm some restrictions may apply.
//! - [`ShortestDistance`]: Any implementation of this trait can be used to find the shortest
//!   distance, depending on algorithm some restrictions may apply.
//!
//! ## Implementations
//!
//! These are the algorithms implemented in `petgraph` itself, companion crates may implement other
//! algorithms.
//!
//! - [`AStar`]: An implementation of the A* shortest path algorithm.
//! - [`BellmanFord`]: An implementation of the Bellman-Ford shortest path algorithm.
//! - [`Dijkstra`]: An implementation of the Dijkstra's shortest path algorithm.
//! - [`FloydWarshall`]: An implementation of the Floyd-Warshall shortest path algorithm.
//!
//! ## Usage
//!
//! Each algorithm provides methods to find the shortest path or distance from a source node to a
//! target node, from a source node to all other nodes, and between all pairs of nodes.
//!
//! The [`ShortestPath`] trait provides methods to find paths, while the [`ShortestDistance`] trait
//! provides methods to find distances.
//! The difference between a path and a distance is that a path includes the sequence of nodes and
//! edges, while a distance is just the sum of the weights of the edges.
//!
//! ## Example
//!
//! ```rust
//! use petgraph_algorithms::shortest_paths::{Dijkstra, ShortestPath};
//! use petgraph_dino::DiDinoGraph;
//!
//! let mut graph = DiDinoGraph::new();
//! let a = *graph.insert_node("A").id();
//! let b = *graph.insert_node("B").id();
//! graph.insert_edge(7, &a, &b);
//!
//! let dijkstra = Dijkstra::directed();
//! let path = dijkstra.path_between(&graph, &a, &b);
//! assert!(path.is_some());
//!
//! let path = dijkstra.path_between(&graph, &b, &a);
//! assert!(path.is_none());
//! ```

pub mod astar;
pub mod bellman_ford;
mod common;
pub mod dijkstra;
pub mod floyd_warshall;

use error_stack::{Context, Result};
use petgraph_core::{Graph, GraphStorage};

pub use self::{
    astar::AStar,
    bellman_ford::BellmanFord,
    common::{
        cost::{Cost, GraphCost},
        path::Path,
        route::{DirectRoute, Route},
    },
    dijkstra::Dijkstra,
    floyd_warshall::FloydWarshall,
};

/// # Shortest Path
///
/// A shortest path algorithm is an algorithm that finds a path between two nodes in a graph such
/// that the sum of the weights of its constituent edges is minimized.
///
/// Different algorithms exist to solve this problem, each with its own trade-offs, refer to the
/// different algorithms for more information.
///
/// ## Shortest Path vs Shortest Distance
///
/// The shortest path is the path between two nodes that has the lowest cost.
/// The shortest distance is the cost of the shortest path.
/// You should prefer shortest distance algorithms over shortest path algorithms if you are only
/// interested in the cost of the shortest path.
///
/// # Example
///
/// ```rust
/// use petgraph_algorithms::shortest_paths::{Dijkstra, ShortestPath};
/// use petgraph_dino::DiDinoGraph;
///
/// let mut graph = DiDinoGraph::new();
/// let a = *graph.insert_node("A").id();
/// let b = *graph.insert_node("B").id();
/// let c = *graph.insert_node("C").id();
///
/// graph.insert_edge(7, &a, &b);
/// graph.insert_edge(5, &b, &c);
/// graph.insert_edge(3, &a, &c);
///
/// let dijkstra = Dijkstra::directed();
/// let path = dijkstra.path_between(&graph, &a, &c).expect("path exists");
///
/// assert_eq!(path.cost().into_value(), 3);
///
/// assert_eq!(path.path().source().id(), &a);
/// assert_eq!(path.path().target().id(), &c);
///
/// assert!(path.path().transit().is_empty());
/// ```
pub trait ShortestPath<S>
where
    S: GraphStorage,
{
    /// The cost of the shortest path.
    type Cost;

    /// The error that can occur when computing the shortest path.
    type Error: Context;

    /// Returns the shortest path from the source to the target.
    ///
    /// Returns an iterator over all routes in the graph that end at the given target node.
    // TODO: example?
    fn path_to<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        target: S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter = self.every_path(graph)?;

        Ok(iter.filter(move |route| route.path().target().id() == target))
    }

    /// Returns the shortest path from the source to the target.
    ///
    /// This is also known as `SSSP` (Single Source Shortest Path) problem.
    ///
    /// Returns an iterator over all routes in the graph that start at the given source node.
    fn path_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter = self.every_path(graph)?;

        Ok(iter.filter(move |route| route.path().source().id() == source))
    }

    /// Returns the shortest path from the source to the target.
    ///
    /// This will return [`None`] if no path exists between the source and the target.
    fn path_between<'graph>(
        &self,
        graph: &'graph Graph<S>,
        source: S::NodeId,
        target: S::NodeId,
    ) -> Option<Route<'graph, S, Self::Cost>> {
        self.path_from(graph, source)
            .ok()?
            .find(|route| route.path().target().id() == target)
    }

    /// Returns an iterator over all shortest paths in the graph.
    ///
    /// This is also known as `APSP` (All Pairs Shortest Path) problem.
    ///
    /// Returns an iterator over all routes in the graph.
    fn every_path<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error>;
}

/// # Shortest Distance
///
/// A shortest distance algorithm is an algorithm that finds the distance between two nodes in a
/// graph such that the sum of the weights of its constituent edges is minimized.
///
/// Different algorithms exist to solve this problem, each with its own trade-offs, refer to the
/// different algorithms for more information.
///
/// ## Shortest Path vs Shortest Distance
///
/// The shortest path is the path between two nodes that has the lowest cost.
/// The shortest distance is the cost of the shortest path.
/// You should prefer shortest distance algorithms over shortest path algorithms if you are only
/// interested in the cost of the shortest path.
// TODO: example?
pub trait ShortestDistance<S>
where
    S: GraphStorage,
{
    /// The cost of the shortest path.
    type Cost;

    /// The error that can occur when computing the shortest path.
    type Error: Context;

    /// Returns the shortest distance from the source to the target.
    ///
    /// Returns an iterator over all direct routes in the graph that end at the given target node.
    /// A [`DirectRoute`] does not contain the nodes traversed, only the source and target nodes.
    fn distance_to<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        target: S::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter = self.every_distance(graph)?;

        Ok(iter.filter(move |route| route.target().id() == target))
    }

    /// Returns the shortest distance from the source to the target.
    ///
    /// This is also known as `SSSP` (Single Source Shortest Path) problem.
    ///
    /// Returns an iterator over all direct routes in the graph that start at the given source node.
    /// A [`DirectRoute`] does not contain the nodes traversed, only the source and target nodes.
    fn distance_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: S::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter = self.every_distance(graph)?;

        Ok(iter.filter(move |route| route.source().id() == source))
    }

    /// Returns the shortest distance from the source to the target.
    ///
    /// This will return [`None`] if no path exists between the source and the target.
    fn distance_between(
        &self,
        graph: &Graph<S>,
        source: S::NodeId,
        target: S::NodeId,
    ) -> Option<Cost<Self::Cost>> {
        self.every_distance(graph)
            .ok()?
            .find(move |route| route.source().id() == source && route.target().id() == target)
            .map(DirectRoute::into_cost)
    }

    /// Returns an iterator over all shortest distances in the graph.
    ///
    /// This is also known as `APSP` (All Pairs Shortest Path) problem.
    ///
    /// Returns an iterator over all direct routes in the graph.
    /// A [`DirectRoute`] does not contain the nodes traversed, only the source and target nodes.
    fn every_distance<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error>;
}
