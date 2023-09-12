//! Adapted implementation of the excellent `alot` crate
use alloc::vec::Vec;
use core::{
    fmt::{Debug, Formatter},
    hash::Hash,
    marker::PhantomData,
    mem,
    num::{NonZeroU16, NonZeroUsize},
};

use hashbrown::HashMap;
use petgraph_core::{id::LinearGraphId, storage::LinearIndexLookup};

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
/// | `target_pointer_width` | generation bits | index bits | unused bits |
/// |------------------------|-----------------|------------|-------------|
/// | 16                     | 4               | 12         | 0           |
/// | 32                     | 8               | 24         | 0           |
/// | 64                     | 16              | 32         | 16          |
///
/// Each time a lot is allocated, its generation is incremented. When retrieving
/// values using a `EntryId`, the generation is validated as a safe guard against
/// returning a value. Because the generation isn't significantly wide, the
/// generation can wrap and is not a perfect protection against stale data,
/// although the likelihood of improper access is greatly reduced.
///
/// These values are used as indices into a roaring bitmap, which has a limit of [`u32::MAX`]
/// entries. Therefore, even though the index bits are 48, the maximum bits available for the index
/// is 32.
///
/// `EntryId` has the following layout: `<generation> <unused> <index>`
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct EntryId(NonZeroUsize);

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

    const GENERATION_MAX: u16 = (usize::MAX >> Self::UNCLAMPED_INDEX_BITS) as u16;
    const GENERATION_OFFSET: u32 = Self::UNCLAMPED_INDEX_BITS;
    const INDEX_BITS: u32 = if Self::UNCLAMPED_INDEX_BITS > 32 {
        32
    } else {
        Self::UNCLAMPED_INDEX_BITS
    };
    /// # Explanation
    ///
    /// How does this magic work? Easy! (Let's assume 32-bit)
    /// We first take the maximum value of a usize (`0xFFFF_FFFF`)
    /// We then shift it right by 24 bits (`0x0000_00FF`)
    /// We then shift it left by 24 bits (`0xFF00_0000`)
    /// We now have the correct mask, but inverse.
    /// We then invert it (`0x00FF_FFFF`)
    const INDEX_MASK: usize = !(usize::MAX >> Self::INDEX_BITS << Self::INDEX_BITS);
    const UNCLAMPED_INDEX_BITS: u32 = usize::BITS / 4 * 3;

    #[inline]
    #[cfg_attr(target_pointer_width = "64", allow(clippy::absurd_extreme_comparisons))]
    fn new(generation: Generation, index: usize) -> Option<Self> {
        let is_valid = generation.get() <= Self::GENERATION_MAX && index <= Self::INDEX_MASK;

        is_valid.then(|| {
            Self(
                NonZeroUsize::new((generation.get() as usize) << Self::GENERATION_OFFSET | index)
                    .expect("generation is non-zero"),
            )
        })
    }

    // TODO: make it easier to create indices!
    pub(crate) fn new_unchecked(raw: u32) -> Self {
        Self(NonZeroUsize::new(raw as usize).expect("raw is zero"))
    }

    #[inline]
    #[must_use]
    const fn index(self) -> usize {
        self.0.get() & Self::INDEX_MASK
    }

    #[inline]
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub(crate) const fn index_u32(self) -> u32 {
        // we know that the index will never be larger than 32 bits
        self.index() as u32
    }

    #[inline]
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    fn generation(self) -> Generation {
        Generation(
            NonZeroU16::new((self.0.get() >> Self::GENERATION_OFFSET) as u16)
                .expect("invalid generation"),
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
pub(crate) struct Generation(NonZeroU16);

impl Generation {
    pub(crate) const fn first() -> Self {
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

    const fn get(self) -> u16 {
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

        mem::replace(
            self,
            Self::Vacant {
                generation: self.generation().next(),
            },
        )
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

    _marker: PhantomData<K>,
}

impl<K, V> Slab<K, V>
where
    K: Key,
{
    #[cfg(test)]
    pub(crate) fn new() -> Self {
        Self::with_capacity(None)
    }

    pub(crate) fn with_capacity(capacity: Option<usize>) -> Self {
        Self {
            entries: Vec::with_capacity(capacity.unwrap_or(0)),
            free: Vec::new(),
            _marker: PhantomData,
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

    fn insert_index(&mut self) -> usize {
        if let Some(free_index) = self.free.pop() {
            free_index.0
        } else {
            assert_ne!(self.entries.len(), MAXIMUM_INDEX, "slab is full");

            self.entries.push(Entry::new());
            self.entries.len() - 1
        }
    }

    pub(crate) fn insert(&mut self, value: V) -> K {
        let index = self.insert_index();

        let generation = self.entries[index].generation();
        self.entries[index].insert(value);

        K::from_id(EntryId::new(generation, index).expect("invalid entry id"))
    }

    fn insert_with_generation(&mut self, value: V, generation: Generation) -> K {
        let index = self.insert_index();

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

    pub(crate) fn contains_key(&self, key: K) -> bool {
        let id = key.into_id();

        let generation = id.generation();
        let index = id.index();

        let Some(entry) = self.entries.get(index) else {
            return false;
        };

        entry.generation() == generation
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

    pub(crate) fn reserve(&mut self, additional: usize) {
        self.entries.reserve(additional);
    }

    pub(crate) fn shrink_to_fit(&mut self) {
        // this is a bit more complicated, then just calling `shrink_to_fit` on the entries
        // we need, to remove all entries that are free from the back until we find an occupied
        // entry.

        while let Some(Entry::Vacant { .. }) = self.entries.last() {
            let index = self.entries.len() - 1;

            // the chance that we have a free index at the end is very high
            if let Some(FreeIndex(free_index)) = self.free.last() {
                if *free_index == index {
                    self.free.pop();
                } else {
                    // we have a free index, but it's not at the end, find it and remove it
                    let free_index = self
                        .free
                        .iter()
                        .position(|FreeIndex(free_index)| *free_index == index);

                    if let Some(free_index) = free_index {
                        self.free.remove(free_index);
                    }
                }
            }

            self.entries.pop();
        }

        self.entries.shrink_to_fit();
        self.free.shrink_to_fit();
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

    pub(crate) fn keys(&self) -> impl Iterator<Item = K> + '_ {
        self.entries
            .iter()
            .enumerate()
            .filter_map(move |(index, entry)| {
                if !entry.is_occupied() {
                    return None;
                }

                let id = EntryId::new(entry.generation(), index)?;
                let key = K::from_id(id);

                Some(key)
            })
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

    pub(crate) fn retain(&mut self, mut f: impl FnMut(K, &mut V) -> bool) {
        for (index, entry) in self.entries.iter_mut().enumerate() {
            let key =
                K::from_id(EntryId::new(entry.generation(), index).expect("invalid entry id"));

            if let Entry::Occupied { value, .. } = entry {
                let keep = f(key, value);

                if !keep {
                    entry.remove();
                    self.free.push(FreeIndex(index));
                }
            }
        }
    }

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

pub(crate) struct SlabLinearIndexLookup<'a, K>
where
    K: Key,
{
    lookup: HashMap<K, usize>,

    // we only have the lifetime here to ensure that the slab is not modified while we are mapping,
    // as that will invalidate indices.
    _lifetime: PhantomData<&'a ()>,
}

impl<'a, K> SlabLinearIndexLookup<'a, K>
where
    K: Key,
{
    pub(crate) fn new<V>(slab: &'a Slab<K, V>) -> Self {
        let map = slab
            .keys()
            .enumerate()
            .map(|(index, key)| (key, index))
            .collect();

        Self {
            lookup: map,
            _lifetime: PhantomData,
        }
    }
}

impl<K> LinearIndexLookup for SlabLinearIndexLookup<'_, K>
where
    K: Key + LinearGraphId,
{
    type GraphId = K;

    fn get(&self, id: &Self::GraphId) -> Option<usize> {
        self.lookup.get(id).copied()
    }
}

#[cfg(test)]
mod tests {
    use alloc::{vec, vec::Vec};

    use super::EntryId;
    use crate::slab::{FreeIndex, Slab};

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

        for (left, right) in copy.entries.iter().zip(slab.entries.iter()) {
            assert_eq!(left.is_occupied(), right.is_occupied());

            if left.is_occupied() {
                assert_eq!(left.generation(), right.generation());
                assert_eq!(left.get(), right.get());
            }
        }
    }

    #[test]
    fn shrink_to_fit() {
        let mut slab = Slab::<EntryId, u16>::new();

        let a = slab.insert(42);
        let b = slab.insert(43);
        let c = slab.insert(44);
        let d = slab.insert(45);

        slab.remove(d);
        slab.remove(b);

        assert_eq!(slab.free, vec![FreeIndex(3), FreeIndex(1)]);

        assert_eq!(slab.entries.len(), 4);

        slab.shrink_to_fit();

        assert_eq!(slab.entries.len(), 3);
        assert_eq!(slab.free, vec![FreeIndex(1)]);
    }

    #[test]
    fn retain() {
        let mut slab = Slab::<EntryId, u16>::new();

        let a = slab.insert(42);
        let b = slab.insert(43);
        let c = slab.insert(44);
        let d = slab.insert(45);

        slab.retain(|_, value| *value % 2 == 0);

        assert_eq!(slab.entries.len(), 4);
        assert_eq!(slab.free.len(), 2);

        assert_eq!(slab.get(a), Some(&42));
        assert_eq!(slab.get(b), None);
        assert_eq!(slab.get(c), Some(&44));
        assert_eq!(slab.get(d), None);

        // should be the same as:
        let mut equivalent = Slab::<EntryId, u16>::new();

        let a = equivalent.insert(42);
        let b = equivalent.insert(43);
        let c = equivalent.insert(44);
        let d = equivalent.insert(45);

        equivalent.remove(b);
        equivalent.remove(d);

        assert_eq!(slab, equivalent);
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn layout() {
        assert_eq!(EntryId::INDEX_MASK, 0x0000_0000_FFFF_FFFF);
        assert_eq!(EntryId::INDEX_BITS, 32);
        assert_eq!(EntryId::UNCLAMPED_INDEX_BITS, 48);
    }

    #[test]
    #[cfg(target_pointer_width = "32")]
    fn layout() {
        assert_eq!(EntryId::INDEX_MASK, 0x00FF_FFFF);
        assert_eq!(EntryId::INDEX_BITS, 24);
        assert_eq!(EntryId::UNCLAMPED_INDEX_BITS, 24);
    }

    #[test]
    #[cfg(target_pointer_width = "16")]
    fn layout() {
        assert_eq!(EntryId::INDEX_MASK, 0x0FFF);
        assert_eq!(EntryId::INDEX_BITS, 12);
        assert_eq!(EntryId::UNCLAMPED_INDEX_BITS, 12);
    }
}
