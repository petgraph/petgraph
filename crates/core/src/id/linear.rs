#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::{id::GraphId, owned::MaybeOwned, storage::GraphStorage};

mod sealed {
    pub trait Sealed: Copy {}

    impl Sealed for super::Continuous {}
    impl Sealed for super::Discrete {}
}

/// Marker trait for the continuity property of a node or edge index.
///
/// This trait is sealed and cannot be implemented outside of this crate.
///
/// The type is implemented for two types: [`Continuous`] and [`Discrete`].
pub trait Continuity: sealed::Sealed {
    /// Whether the index is continuous.
    ///
    /// If this is `true`, then the index is continuous, otherwise it is discrete.
    const CONTINUOUS: bool;

    /// Whether the index is continuous.
    #[must_use]
    fn is_continuous() -> bool {
        Self::CONTINUOUS
    }
}

/// Marker type for a continuous index.
///
/// This type is ZST and is only really useful as a generic argument to specify the continuity of a
/// graph index.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Continuous;

impl Continuity for Continuous {
    const CONTINUOUS: bool = true;
}

/// Marker type for a discrete index.
///
/// This type is ZST and is only really useful as a generic argument to specify the continuity of a
/// graph index.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Discrete;

impl Continuity for Discrete {
    const CONTINUOUS: bool = false;
}

/// Index mapper for a graph.
///
/// The index mapper is a type, that maps a specific value (`From`), typically a [`LinearGraphId`],
/// into a different value (`To`), typically a [`usize`].
///
/// How this conversion is done is up to the implementation, but should be consistent, i.e. the same
/// input value should always map to the same output value.
/// Index lookup should also be (if possible) `O(1)` for `From -> To`, but not necessarily for `To
/// -> From`.
pub trait IndexMapper<From, To> {
    // we cannot use `const` in trait bounds (yet), so we need to use marker traits and types
    /// The continuity property of the index mapper.
    ///
    /// This is either [`Continuous`] or [`Discrete`], meaning that if [`Continuous`], the value
    /// returned from [`Self::map`] and [`Self::lookup`] will be continuous, with no holes in the
    /// `To` value range.
    type Continuity: Continuity;

    /// Map a value from `From` to `To`.
    ///
    /// This **must** be pure and **must** return a valid value for any `From` value.
    /// This might mutate the internal state of the mapper.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::id::{IndexMapper, LinearGraphId};
    /// use petgraph_dino::{DiDinoGraph, NodeId};
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    /// # let ab = graph.insert_edge("A → B", &a, &b);
    ///
    /// let mut mapper = NodeId::index_mapper(graph.storage());
    ///
    /// // The mapping is highly dependent on the implementation, but should be consistent.
    /// // The order for which node maps to which value is not guaranteed.
    /// assert_eq!(mapper.map(&a), 0);
    /// assert_eq!(mapper.map(&b), 1);
    /// ```
    fn map(&mut self, from: &From) -> To;

    /// Lookup a value from `From` to `To`.
    ///
    /// This **must** be pure, but **may** return `None` if the value has not been [`Self::map`]ped
    /// previously.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::id::{IndexMapper, LinearGraphId};
    /// use petgraph_dino::{DiDinoGraph, NodeId};
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    /// # let ab = graph.insert_edge("A → B", &a, &b);
    ///
    /// let mut mapper = NodeId::index_mapper(graph.storage());
    ///
    /// assert_eq!(mapper.map(&a), 0);
    /// assert_eq!(mapper.lookup(&a), Some(0));
    /// ```
    fn lookup(&self, from: &From) -> Option<To>;

    /// Reverse lookup a value from `To` to `From`.
    ///
    /// This **must** be pure, but **may** return `None` if the value has not been [`Self::map`]ped
    /// or if a reverse mapping does not exist (for example `To` is too large and therefore does not
    /// exist, or is a hole in the `To` value range).
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{
    ///     id::{IndexMapper, LinearGraphId},
    ///     owned::MaybeOwned,
    /// };
    /// use petgraph_dino::{DiDinoGraph, NodeId};
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    /// # let ab = graph.insert_edge("A → B", &a, &b);
    ///
    /// let mut mapper = NodeId::index_mapper(graph.storage());
    ///
    /// assert_eq!(mapper.reverse(&0), Some(MaybeOwned::Borrowed(&a)));
    /// ```
    fn reverse(&mut self, to: &To) -> Option<MaybeOwned<From>>;
}

/// Linear graph identifier.
///
/// A linear graph identifier is a graph identifier that has a linear mapping to a `usize` value,
/// that mapping _may_ be continuous or discrete.
pub trait LinearGraphId<S>: GraphId + Sized
where
    S: GraphStorage,
{
    /// The index mapper for this graph identifier.
    type Mapper<'a>: IndexMapper<Self, usize>
    where
        Self: 'a,
        S: 'a;

    /// Get the index mapper for this graph identifier.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::id::{IndexMapper, LinearGraphId};
    /// use petgraph_dino::{DiDinoGraph, NodeId};
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    /// # let ab = graph.insert_edge("A → B", &a, &b);
    ///
    /// let mapper = NodeId::index_mapper(graph.storage());
    /// ```
    fn index_mapper(storage: &S) -> Self::Mapper<'_>;
}

/// Continuous index mapper.
///
/// Implementation of an `IndexMapper` on-top of an existing `IndexMapper`, that if the inner
/// `IndexMapper` is not continuous, will create an internal lookup table to make it continuous.
/// The lookup table simply takes every value requested, looks it up in the internal lookup table,
/// returns the index or pushes a new index to the lookup table and returns that.
///
/// Wrapping an already continuous `IndexMapper` will simply return the value from the inner mapper
/// without any additional lookup or allocation.
#[cfg(feature = "alloc")]
pub struct ContinuousIndexMapper<I, T> {
    inner: I,
    lookup: Vec<T>,
}

#[cfg(feature = "alloc")]
impl<I, T> ContinuousIndexMapper<I, T> {
    pub fn new(inner: I) -> Self {
        Self {
            inner,
            lookup: Vec::new(),
        }
    }
}

#[cfg(feature = "alloc")]
impl<I, T> IndexMapper<T, usize> for ContinuousIndexMapper<I, T>
where
    I: IndexMapper<T, usize>,
    T: PartialEq + Clone,
{
    type Continuity = Continuous;

    fn map(&mut self, from: &T) -> usize {
        if I::Continuity::CONTINUOUS {
            self.inner.map(from)
        } else if let Some(index) = self.lookup.iter().position(|v| v == from) {
            index
        } else {
            self.lookup.push(from.clone());
            self.lookup.len() - 1
        }
    }

    fn lookup(&self, from: &T) -> Option<usize> {
        if I::Continuity::CONTINUOUS {
            self.inner.lookup(from)
        } else {
            self.lookup.iter().position(|v| v == from)
        }
    }

    fn reverse(&mut self, to: &usize) -> Option<MaybeOwned<T>> {
        if I::Continuity::CONTINUOUS {
            self.inner.reverse(to)
        } else {
            self.lookup.get(*to).map(|v| MaybeOwned::Borrowed(v))
        }
    }
}
