//! Marker traits for the directional property of a graph.
//!
//! This module defines the marker trait [`GraphDirectionality`] and two marker structs [`Directed`]
//! and [`Undirected`].
//!
//! You are unable to implement the trait [`GraphDirectionality`] for your own types, as it is
//! sealed.
//!
//! The core [`Graph`] makes no use of these types, but these types are provided to ease development
//! (and unify the API surface) of implementations, that might need to indicate the directionality
//! of their implementation.
//!
//! [`Graph`]: crate::graph::Graph
mod sealed {
    pub trait Sealed: Copy + 'static {}

    impl Sealed for super::Undirected {}
    impl Sealed for super::Directed {}
}

/// Marker trait for the directional property of a graph.
///
/// This trait is sealed and cannot be implemented for types outside of `petgraph_core`.
///
/// The type is implemented for two types: [`Undirected`] and [`Directed`].
pub trait GraphDirectionality: sealed::Sealed {
    /// Directional property of the graph.
    ///
    /// `true` if the graph is directed, `false` if undirected.
    const DIRECTED: bool;

    /// Returns `true` if the graph is directed.
    ///
    /// This is equivalent to [`Self::DIRECTED`].
    #[must_use]
    fn is_directed() -> bool {
        Self::DIRECTED
    }
}

/// Marker struct for undirected edges.
///
/// This type is ZST and is only really useful as a generic argument to specify the directionality
/// of a graph (undirected vs directed).
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Undirected;

impl GraphDirectionality for Undirected {
    const DIRECTED: bool = false;
}

/// Marker struct for directed edges.
///
/// This type is ZST and is only really useful as a generic argument to specify the directionality
/// of a graph (undirected vs directed).
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Directed;

impl GraphDirectionality for Directed {
    const DIRECTED: bool = true;
}
