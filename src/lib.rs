
//! **petgraph** is a graph data structure library.
//!
//! The most prominent type is [`Graph`](./graph/struct.Graph.html) which is
//! an adjacency list graph with undirected or directed edges and arbitrary
//! associated data.
//!
//! Petgraph also provides [`GraphMap`](./graphmap/struct.GraphMap.html) which
//! is an hashmap-backed graph with undirected edges and only allows simple node
//! identifiers (such as integers or references).

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
#[cfg(feature = "generate")]
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
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum EdgeDirection {
    /// An `Outgoing` edge is an outward edge *from* the current node.
    Outgoing = 0,
    /// An `Incoming` edge is an inbound edge *to* the current node.
    Incoming = 1
}

impl EdgeDirection {
    /// Return the opposite `EdgeDirection`.
    #[inline]
    pub fn opposite(&self) -> EdgeDirection {
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
pub trait IntoWeightedEdge<E> {
    type NodeId;
    fn into_weighted_edge(self) -> (Self::NodeId, Self::NodeId, E);
}

impl<Ix, E> IntoWeightedEdge<E> for (Ix, Ix)
    where E: Default
{
    type NodeId = Ix;

    fn into_weighted_edge(self) -> (Ix, Ix, E) {
        let (s, t) = self;
        (s, t, E::default())
    }
}

impl<Ix, E> IntoWeightedEdge<E> for (Ix, Ix, E)
{
    type NodeId = Ix;
    fn into_weighted_edge(self) -> (Ix, Ix, E) {
        self
    }
}

impl<'a, Ix, E> IntoWeightedEdge<E> for (Ix, Ix, &'a E)
    where E: Clone
{
    type NodeId = Ix;
    fn into_weighted_edge(self) -> (Ix, Ix, E) {
        let (a, b, c) = self;
        (a, b, c.clone())
    }
}

impl<'a, Ix, E> IntoWeightedEdge<E> for &'a (Ix, Ix)
    where Ix: Copy, E: Default
{
    type NodeId = Ix;
    fn into_weighted_edge(self) -> (Ix, Ix, E) {
        let (s, t) = *self;
        (s, t, E::default())
    }
}

impl<'a, Ix, E> IntoWeightedEdge<E> for &'a (Ix, Ix, E)
    where Ix: Copy, E: Clone
{
    type NodeId = Ix;
    fn into_weighted_edge(self) -> (Ix, Ix, E) {
        self.clone()
    }
}
