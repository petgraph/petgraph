
//! **petgraph** is a graph data structure library.
//!
//! The most prominent type is [`Graph`](./graph/struct.Graph.html) which is
//! a directed or undirected graph with arbitrary associated node and edge data.
//!
//! Petgraph also provides [`GraphMap`](./graphmap/struct.GraphMap.html) which
//! is an undirected hashmap-backed graph which only allows simple node identifiers
//! (such as integers or references).

extern crate fixedbitset;

pub use graph::Graph;
pub use graphmap::GraphMap;

pub use visit::{
    Bfs,
    BfsIter,
    Dfs,
    DfsIter,
};
pub use EdgeDirection::{Outgoing, Incoming};

mod scored;
pub mod algo;
#[doc(hidden)] // Not for public consumption -- only for testing
pub mod generate;
pub mod graphmap;
pub mod graph;
pub mod dot;
pub mod visit;
pub mod unionfind;
mod dijkstra;
mod isomorphism;
mod traits_graph;
#[cfg(feature = "quickcheck")]
pub mod quickcheck;

// Index into the NodeIndex and EdgeIndex arrays
/// Edge direction
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum EdgeDirection {
    /// An `Outgoing` edge is an outward edge *from* the current node.
    Outgoing = 0,
    /// An `Incoming` edge is an inbound edge *to* the current node.
    Incoming = 1
}

impl EdgeDirection {
    #[inline]
    fn opposite(&self) -> EdgeDirection {
        match *self {
            Outgoing => Incoming,
            Incoming => Outgoing,
        }
    }
}

/// Marker type for a directed graph.
#[derive(Copy, Clone, Debug)]
pub enum Directed { }

/// Marker type for an undirected graph.
#[derive(Copy, Clone, Debug)]
pub enum Undirected { }

/// A graph's edge type determines whether is has directed edges or not.
pub trait EdgeType {
    fn is_directed() -> bool;
}

impl EdgeType for Directed {
    #[inline]
    fn is_directed() -> bool { true }
}

impl EdgeType for Undirected {
    #[inline]
    fn is_directed() -> bool { false }
}


/// Convert an element like `(i, j)` or `(i, j, w)` into
/// a triple of source, target, edge weight.
///
/// For `Graph::from_edges` and `GraphMap::from_edges`.
pub trait IntoWeightedEdge<Ix, E> {
    fn into_weighted_edge(self) -> (Ix, Ix, E);
}

impl<Ix, E> IntoWeightedEdge<Ix, E> for (Ix, Ix)
    where E: Default
{
    fn into_weighted_edge(self) -> (Ix, Ix, E) {
        let (s, t) = self;
        (s, t, E::default())
    }
}

impl<Ix, E> IntoWeightedEdge<Ix, E> for (Ix, Ix, E)
{
    fn into_weighted_edge(self) -> (Ix, Ix, E) {
        self
    }
}

impl<'a, Ix, E> IntoWeightedEdge<Ix, E> for (Ix, Ix, &'a E)
    where E: Clone
{
    fn into_weighted_edge(self) -> (Ix, Ix, E) {
        let (a, b, c) = self;
        (a, b, c.clone())
    }
}

impl<'a, Ix, E> IntoWeightedEdge<Ix, E> for &'a (Ix, Ix)
    where Ix: Copy, E: Default
{
    fn into_weighted_edge(self) -> (Ix, Ix, E) {
        let (s, t) = *self;
        (s, t, E::default())
    }
}

impl<'a, Ix, E> IntoWeightedEdge<Ix, E> for &'a (Ix, Ix, E)
    where Ix: Copy, E: Clone
{
    fn into_weighted_edge(self) -> (Ix, Ix, E) {
        self.clone()
    }
}
