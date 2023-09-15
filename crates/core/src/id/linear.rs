#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::{id::GraphId, storage::GraphStorage};

pub trait IndexMapper<From, To> {
    const CONTINUOUS: bool;

    fn map(&mut self, from: &From) -> To;

    // TODO: contemplate this returning `Cow` (or something similar) to avoid cloning.
    fn reverse(&mut self, to: &To) -> Option<From>;
}

pub trait LinearGraphId<S>: GraphId + Sized
where
    S: GraphStorage,
{
    fn index_mapper(storage: &S) -> impl IndexMapper<Self, usize>;
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
    const CONTINUOUS: bool = true;

    fn map(&mut self, from: &T) -> usize {
        if I::CONTINUOUS {
            self.inner.map(from)
        } else if let Some(index) = self.lookup.iter().position(|v| v == from) {
            index
        } else {
            self.lookup.push(from.clone());
            self.lookup.len() - 1
        }
    }

    fn reverse(&mut self, to: &usize) -> Option<T> {
        if I::CONTINUOUS {
            self.inner.reverse(to)
        } else {
            self.lookup.get(*to).cloned()
        }
    }
}
