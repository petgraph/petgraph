
//! **petgraph** is a graph data structure library.
//!
//! The most prominent type is [`Graph`](./graph/struct.Graph.html) which is
//! a directed or undirected graph with arbitrary mutable node and edge weights.
//! It is based on rustc's graph implementation.
//!
//! Petgraph also provides [`GraphMap`](./graphmap/struct.GraphMap.html) which
//! is an undirected hashmap-backed graph which only allows simple node identifiers
//! (such as integers or references).

extern crate fixedbitset;

use std::cmp::Ordering;
use std::hash::{self, Hash};
use std::fmt;
use std::ops::{Deref};

pub use scored::MinScored;
pub use graphmap::GraphMap;
pub use graph::Graph;

pub use self::EdgeDirection::{Outgoing, Incoming};
pub use visit::{
    Bfs,
    BfsIter,
    Dfs,
    DfsIter,
};

mod scored;
pub mod algo;
#[doc(hidden)] // Not for public consumption -- only for testing
pub mod generate;
pub mod graphmap;
pub mod graph;
pub mod visit;
pub mod unionfind;
mod dijkstra;
mod isomorphism;
mod traits_graph;

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
pub struct Directed;

/// Marker type for an undirected graph.
#[derive(Copy, Clone, Debug)]
pub struct Undirected;

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


/// A reference that is hashed and compared by its pointer value.
pub struct Ptr<'b, T: 'b>(pub &'b T);

impl<'b, T> Copy for Ptr<'b, T> {}
impl<'b, T> Clone for Ptr<'b, T>
{
    fn clone(&self) -> Self { *self }
}

fn ptr_eq<T>(a: *const T, b: *const T) -> bool {
    a == b
}

impl<'b, T> PartialEq for Ptr<'b, T>
{
    /// Ptr compares by pointer equality, i.e if they point to the same value
    fn eq(&self, other: &Ptr<'b, T>) -> bool {
        ptr_eq(self.0, other.0)
    }
}

impl<'b, T> PartialOrd for Ptr<'b, T>
{
    fn partial_cmp(&self, other: &Ptr<'b, T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'b, T> Ord for Ptr<'b, T>
{
    /// Ptr is ordered by pointer value, i.e. an arbitrary but stable and total order.
    fn cmp(&self, other: &Ptr<'b, T>) -> Ordering {
        let a = self.0 as *const _;
        let b = other.0 as *const _;
        a.cmp(&b)
    }
}

impl<'b, T> Deref for Ptr<'b, T> {
    type Target = T;
    fn deref<'a>(&'a self) -> &'a T {
        self.0
    }
}

impl<'b, T> Eq for Ptr<'b, T> {}

impl<'b, T> Hash for Ptr<'b, T>
{
    fn hash<H: hash::Hasher>(&self, st: &mut H)
    {
        let ptr = (self.0) as *const T;
        ptr.hash(st)
    }
}

impl<'b, T: fmt::Debug> fmt::Debug for Ptr<'b, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}
