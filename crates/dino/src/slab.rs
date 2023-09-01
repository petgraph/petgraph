//! Adapted implementation of the excellent `alot` crate
use alloc::vec::Vec;
use core::{
    fmt::{Debug, Formatter},
    mem,
    num::{NonZeroU16, NonZeroUsize},
};
use std::hash::Hash;

const MAXIMUM_INDEX: usize = usize::MAX << 16 >> 16;

pub(crate) trait Key:
    Copy + Clone + PartialEq + Eq + PartialOrd + Ord + Hash + Debug
{
    fn from_id(id: EntryId) -> Self;
    fn into_id(self) -> EntryId;
}

/// A `EntryId` is a single `usize`, encoding generation information in the top
/// 1/4 of the bits, and index information in the remaining bits. This table
/// shows the breakdown for supported target platforms:
///
/// | `target_pointer_width` | generation bits | index bits |
/// |------------------------|-----------------|------------|
/// | 16                     | 4               | 12         |
/// | 32                     | 8               | 24         |
/// | 64                     | 16              | 48         |
///
/// Each time a lot is allocated, its generation is incremented. When retrieving
/// values using a `EntryId`, the generation is validated as a safe guard against
/// returning a value. Because the generation isn't significantly wide, the
/// generation can wrap and is not a perfect protection against stale data,
/// although the likelihood of improper access is greatly reduced.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntryId(NonZeroUsize);

impl Debug for EntryId {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("EntryId")
            .field("index", &self.index())
            .field("generation", &self.generation())
            .finish()
    }
}

impl EntryId {
    #[allow(clippy::cast_possible_truncation)]

    const GENERATION_MAX: u16 = (usize::MAX >> Self::INDEX_BITS) as u16;
    const INDEX_BITS: u32 = usize::BITS / 4 * 3;
    const INDEX_MASK: usize = usize::MAX << Self::INDEX_BITS >> Self::INDEX_BITS;

    #[inline]
    #[cfg_attr(target_pointer_width = "64", allow(clippy::absurd_extreme_comparisons))]
    fn new(generation: Generation, index: usize) -> Option<Self> {
        let is_valid = generation.get() <= Self::GENERATION_MAX && index <= Self::INDEX_MASK;

        is_valid.then(|| {
            Self(
                NonZeroUsize::new((generation.get() as usize) << Self::INDEX_BITS | index)
                    .expect("generation is non-zero"),
            )
        })
    }

    #[inline]
    #[must_use]
    const fn index(self) -> usize {
        self.0.get() & Self::INDEX_MASK
    }

    #[inline]
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    fn generation(self) -> Generation {
        Generation(
            NonZeroU16::new((self.0.get() >> Self::INDEX_BITS) as u16).expect("invalid generation"),
        )
    }
}

impl Key for EntryId {
    fn from_id(id: EntryId) -> Self {
        id
    }

    fn into_id(self) -> EntryId {
        self
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Generation(NonZeroU16);

impl Generation {
    pub const fn first() -> Self {
        Self(match NonZeroU16::new(1) {
            Some(n) => n,
            None => unreachable!(),
        })
    }

    #[cfg_attr(target_pointer_width = "64", allow(clippy::absurd_extreme_comparisons))]
    const fn next(self) -> Self {
        match self.0.checked_add(1) {
            Some(next) if next.get() <= EntryId::GENERATION_MAX => Self(next),
            _ => Self::first(),
        }
    }

    pub const fn get(self) -> u16 {
        self.0.get()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Entry<V> {
    Occupied { value: V, generation: Generation },
    Vacant { generation: Generation },
}

impl<V> Entry<V> {
    const fn new() -> Self {
        Self::Vacant {
            generation: Generation::first(),
        }
    }

    const fn generation(&self) -> Generation {
        match self {
            Self::Vacant { generation } | Self::Occupied { generation, .. } => *generation,
        }
    }

    const fn is_occupied(&self) -> bool {
        match self {
            Self::Occupied { .. } => true,
            Self::Vacant { .. } => false,
        }
    }

    fn remove(&mut self) -> Option<V> {
        if !self.is_occupied() {
            return None;
        }

        mem::replace(self, Self::Vacant {
            generation: self.generation().next(),
        })
        .into_inner()
    }

    fn insert(&mut self, value: V) {
        // we increment the generation on remove!
        let generation = self.generation();

        *self = Self::Occupied { value, generation };
    }

    const fn get(&self) -> Option<&V> {
        match self {
            Self::Occupied { value, .. } => Some(value),
            Self::Vacant { .. } => None,
        }
    }

    fn get_mut(&mut self) -> Option<&mut V> {
        match self {
            Self::Occupied { value, .. } => Some(value),
            Self::Vacant { .. } => None,
        }
    }

    fn into_inner(self) -> Option<V> {
        match self {
            Self::Occupied { value, .. } => Some(value),
            Self::Vacant { .. } => None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct FreeIndex(usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Slab<K, V>
where
    K: Key,
{
    entries: Vec<Entry<V>>,
    free: Vec<FreeIndex>,

    _marker: core::marker::PhantomData<K>,
}

impl<K, V> Slab<K, V>
where
    K: Key,
{
    pub(crate) fn new() -> Self {
        Self::with_capacity(None)
    }

    pub(crate) fn with_capacity(capacity: Option<usize>) -> Self {
        Self {
            entries: Vec::with_capacity(capacity.unwrap_or(0)),
            free: Vec::new(),
            _marker: core::marker::PhantomData,
        }
    }

    pub(crate) fn grow_up_to(&mut self, capacity: usize) {
        if capacity <= self.entries.len() {
            return;
        }

        let current = self.entries.len();
        self.entries.reserve(capacity);

        for index in current..capacity {
            self.entries.push(Entry::new());
            self.free.push(FreeIndex(index));
        }
    }

    pub(crate) fn insert(&mut self, value: V) -> K {
        let index = if let Some(free_index) = self.free.pop() {
            free_index.0
        } else {
            assert_ne!(self.entries.len(), MAXIMUM_INDEX, "slab is full");

            self.entries.push(Entry::new());
            self.entries.len() - 1
        };

        let generation = self.entries[index].generation();
        self.entries[index].insert(value);

        K::from_id(EntryId::new(generation, index).expect("invalid entry id"))
    }

    fn insert_with_generation(&mut self, value: V, generation: Generation) -> K {
        let index = if let Some(free_index) = self.free.pop() {
            free_index.0
        } else {
            assert_ne!(self.entries.len(), MAXIMUM_INDEX, "slab is full");

            self.entries.push(Entry::new());
            self.entries.len() - 1
        };

        self.entries[index] = Entry::Occupied { value, generation };

        K::from_id(EntryId::new(generation, index).expect("invalid entry id"))
    }

    pub(crate) fn next_key(&self) -> K {
        let index = self
            .free
            .last()
            .map_or(self.entries.len(), |free_index| free_index.0);

        let generation = self
            .entries
            .get(index)
            .map_or(Generation::first(), Entry::generation);

        K::from_id(EntryId::new(generation, index).expect("invalid entry id"))
    }

    pub(crate) fn remove(&mut self, key: K) -> Option<V> {
        let id = key.into_id();

        let generation = id.generation();
        let index = id.index();

        let entry = self.entries.get_mut(index)?;

        if entry.generation() != generation {
            return None;
        }

        let value = entry.remove();
        self.free.push(FreeIndex(index));
        value
    }

    pub(crate) fn get(&self, key: K) -> Option<&V> {
        let id = key.into_id();

        let generation = id.generation();
        let index = id.index();

        let entry = self.entries.get(index)?;

        if entry.generation() != generation {
            return None;
        }

        entry.get()
    }

    pub(crate) fn get_mut(&mut self, key: K) -> Option<&mut V> {
        let id = key.into_id();

        let generation = id.generation();
        let index = id.index();

        let entry = self.entries.get_mut(index)?;

        if entry.generation() != generation {
            return None;
        }

        entry.get_mut()
    }

    pub(crate) fn clear(&mut self) {
        self.entries.clear();
        self.free.clear();
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &V> {
        self.entries.iter().filter_map(Entry::get)
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.entries.iter_mut().filter_map(Entry::get_mut)
    }

    pub(crate) fn into_iter(self) -> impl Iterator<Item = V> {
        self.entries.into_iter().filter_map(Entry::into_inner)
    }

    pub(crate) fn entries(&self) -> impl Iterator<Item = (K, &V)> + '_ {
        self.entries
            .iter()
            .enumerate()
            .filter_map(move |(index, entry)| {
                let id = EntryId::new(entry.generation(), index)?;
                let key = K::from_id(id);
                let value = entry.get()?;

                Some((key, value))
            })
    }

    pub(crate) fn entries_mut(&mut self) -> impl Iterator<Item = (K, &mut V)> + '_ {
        self.entries
            .iter_mut()
            .enumerate()
            .filter_map(move |(index, entry)| {
                let id = EntryId::new(entry.generation(), index)?;
                let key = K::from_id(id);
                let value = entry.get_mut()?;

                Some((key, value))
            })
    }

    pub(crate) fn into_entries(self) -> impl Iterator<Item = (K, V)> {
        self.entries
            .into_iter()
            .enumerate()
            .filter_map(move |(index, entry)| {
                let id = EntryId::new(entry.generation(), index)?;
                let key = K::from_id(id);
                let value = entry.into_inner()?;

                Some((key, value))
            })
    }

    // TODO: drain

    pub(crate) fn len(&self) -> usize {
        self.entries.len() - self.free.len()
    }
}

impl<K, V> FromIterator<(K, V)> for Slab<K, V>
where
    K: Key,
{
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let iter = iter.into_iter();

        let mut slab = Self::with_capacity(Some(iter.size_hint().0));

        for (key, value) in iter {
            let id = key.into_id();
            let index = id.index();
            let generation = id.generation();

            slab.grow_up_to(index + 1);
            slab.insert_with_generation(value, generation);
        }

        slab
    }
}

#[cfg(test)]
mod tests {
    use super::EntryId;
    use crate::slab::Slab;

    #[test]
    fn size_of_entry_id() {
        assert_eq!(
            core::mem::size_of::<EntryId>(),
            core::mem::size_of::<usize>()
        );
    }

    #[test]
    fn size_of_entry() {
        assert_eq!(
            core::mem::size_of::<crate::slab::Entry<u16>>(),
            core::mem::size_of::<u16>() + core::mem::size_of::<u16>()
        );
    }

    #[test]
    fn insert() {
        let mut slab = Slab::<EntryId, u16>::new();

        let a = slab.insert(42);
        assert_eq!(slab.get(a), Some(&42));

        assert_eq!(slab.len(), 1);

        let b = slab.insert(43);
        assert_eq!(slab.get(b), Some(&43));

        assert_eq!(slab.len(), 2);
    }

    #[test]
    fn remove() {
        let mut slab = Slab::<EntryId, u16>::new();

        let a = slab.insert(42);
        assert_eq!(slab.get(a), Some(&42));

        assert_eq!(slab.len(), 1);

        let b = slab.insert(43);
        assert_eq!(slab.get(b), Some(&43));

        assert_eq!(slab.len(), 2);

        assert_eq!(slab.remove(a), Some(42));
        assert_eq!(slab.get(a), None);

        assert_eq!(slab.entries.len(), 2);
        assert_eq!(slab.free.len(), 1);

        assert_eq!(slab.free[0].0, 0);
        assert!(!slab.entries[0].is_occupied());
    }

    #[test]
    fn reuse() {
        let mut slab = Slab::<EntryId, u16>::new();

        let a = slab.insert(42);
        assert_eq!(slab.get(a), Some(&42));

        assert_eq!(slab.len(), 1);

        let b = slab.insert(43);
        assert_eq!(slab.get(b), Some(&43));

        assert_eq!(slab.len(), 2);

        assert_eq!(slab.remove(a), Some(42));

        assert_eq!(slab.entries.len(), 2);
        assert_eq!(slab.free.len(), 1);

        let c = slab.insert(44);
        assert_eq!(slab.get(c), Some(&44));

        assert_eq!(slab.len(), 2);
        assert_eq!(slab.entries.len(), 2);
        assert_eq!(slab.free.len(), 0);

        assert_eq!(slab.get(a), None);
    }

    #[test]
    fn from_iter() {
        let mut slab = Slab::<EntryId, u16>::new();
        slab.insert(42);
        let b = slab.insert(43);
        slab.insert(44);

        let mut d = slab.insert(45);
        for _ in 0..16 {
            slab.remove(d);
            d = slab.insert(45);
        }

        let e = slab.insert(46);

        slab.remove(b);
        slab.remove(e);

        let entries = slab
            .entries()
            .map(|(key, value)| (key, *value))
            .collect::<Vec<_>>();

        let copy = Slab::from_iter(entries);

        assert_eq!(slab, copy);
    }
}
