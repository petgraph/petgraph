//! Deprecated types and functions.
//!
//! These are functions and traits that have been used in petgraph prior to 0.7.0, are are scheduled
//! for removal in 0.8.0.
#![allow(deprecated)]
#![allow(missing_docs)]
#[macro_use]
pub mod visit;

pub mod macros;

pub mod index;
#[macro_use]
#[cfg(feature = "alloc")]
pub mod data;
pub mod edge;
pub mod utils;

/// Convert an element like `(i, j)` or `(i, j, w)` into
/// a triple of source, target, edge weight.
///
/// For `Graph::from_edges` and `GraphMap::from_edges`.
pub trait IntoWeightedEdge<E> {
    type NodeId;
    fn into_weighted_edge(self) -> (Self::NodeId, Self::NodeId, E);
}

impl<Ix, E> IntoWeightedEdge<E> for (Ix, Ix)
where
    E: Default,
{
    type NodeId = Ix;

    fn into_weighted_edge(self) -> (Ix, Ix, E) {
        let (s, t) = self;
        (s, t, E::default())
    }
}

impl<Ix, E> IntoWeightedEdge<E> for (Ix, Ix, E) {
    type NodeId = Ix;

    fn into_weighted_edge(self) -> (Ix, Ix, E) {
        self
    }
}

impl<'a, Ix, E> IntoWeightedEdge<E> for (Ix, Ix, &'a E)
where
    E: Clone,
{
    type NodeId = Ix;

    fn into_weighted_edge(self) -> (Ix, Ix, E) {
        let (a, b, c) = self;
        (a, b, c.clone())
    }
}

impl<'a, Ix, E> IntoWeightedEdge<E> for &'a (Ix, Ix)
where
    Ix: Copy,
    E: Default,
{
    type NodeId = Ix;

    fn into_weighted_edge(self) -> (Ix, Ix, E) {
        let (s, t) = *self;
        (s, t, E::default())
    }
}

impl<'a, Ix, E> IntoWeightedEdge<E> for &'a (Ix, Ix, E)
where
    Ix: Copy,
    E: Clone,
{
    type NodeId = Ix;

    fn into_weighted_edge(self) -> (Ix, Ix, E) {
        self.clone()
    }
}
