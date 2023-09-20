#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::{id::GraphId, owned::MaybeOwned, storage::GraphStorage};

mod sealed {
    pub trait Sealed: Copy {}

    impl Sealed for super::Continuous {}
    impl Sealed for super::Discrete {}
}

pub trait Continuity: sealed::Sealed {
    const CONTINUOUS: bool;

    fn is_continuous() -> bool {
        Self::CONTINUOUS
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Continuous;

impl Continuity for Continuous {
    const CONTINUOUS: bool = true;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Discrete;

impl Continuity for Discrete {
    const CONTINUOUS: bool = false;
}

pub trait IndexMapper<From, To> {
    // we cannot use `const` in trait bounds (yet), so we need to use marker traits and types
    type Continuity: Continuity;

    fn map(&mut self, from: &From) -> To;

    fn lookup(&self, from: &From) -> Option<To>;

    fn reverse(&mut self, to: &To) -> Option<MaybeOwned<From>>;
}

pub trait LinearGraphId<S>: GraphId + Sized
where
    S: GraphStorage,
{
    type Mapper<'a>: IndexMapper<Self, usize>
    where
        Self: 'a,
        S: 'a;

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
