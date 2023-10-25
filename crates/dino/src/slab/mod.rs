//! Adapted implementation of the excellent `alot` crate
mod entry;
mod generation;
mod id;
mod key;

use alloc::{vec, vec::Vec};
use core::{fmt::Debug, hash::Hash, marker::PhantomData, ptr};

use hashbrown::HashMap;
use petgraph_core::{
    base::MaybeOwned,
    deprecated::index::IndexType,
    id::{Continuous, IndexMapper},
};

use crate::slab::entry::{Entry, State};
pub(crate) use crate::slab::{generation::Generation, id::EntryId, key::Key};

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
            assert_ne!(
                self.entries.len(),
                (EntryId::GENERATION_MAX as usize) + 1,
                "slab is full"
            );

            self.entries.push(Entry::new());
            self.entries.len() - 1
        }
    }

    pub(crate) fn insert(&mut self, value: V) -> K {
        let index = self.insert_index();

        let generation = self.entries[index].generation;
        self.entries[index].insert(value);

        K::from_id(EntryId::new(generation, index).expect("invalid entry id"))
    }

    fn insert_with_generation(&mut self, value: V, generation: Generation) -> K {
        let index = self.insert_index();

        self.entries[index] = Entry::from_parts(generation, State::Occupied(value));

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
            .map_or(Generation::first(), |entry| entry.generation);

        K::from_id(EntryId::new(generation, index).expect("invalid entry id"))
    }

    pub(crate) fn remove(&mut self, key: K) -> Option<V> {
        let id = key.into_id();

        let generation = id.generation();
        let index = id.index();

        let entry = self.entries.get_mut(index)?;

        if entry.generation != generation {
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

        entry.generation == generation
    }

    pub(crate) fn get(&self, key: K) -> Option<&V> {
        let id = key.into_id();

        let generation = id.generation();
        let index = id.index();

        let entry = self.entries.get(index)?;

        if entry.generation != generation {
            return None;
        }

        entry.get()
    }

    pub(crate) fn get_mut(&mut self, key: K) -> Option<&mut V> {
        let id = key.into_id();

        let generation = id.generation();
        let index = id.index();

        let entry = self.entries.get_mut(index)?;

        if entry.generation != generation {
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

        while self.entries.last().map_or(false, Entry::is_vacant) {
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

    // TODO: we might want to consider moving to `Option<&mut V>` if we need it.
    pub(crate) fn filter_mut(
        &mut self,
        indices: impl Iterator<Item = K>,
    ) -> impl Iterator<Item = &mut V> {
        let mut last = None;

        let indices = indices.filter(move |index| {
            if index.into_id().raw() <= last.unwrap_or(0) {
                return false;
            }

            last = Some(index.into_id().raw());

            true
        });

        indices.filter_map(|index| {
            let id = index.into_id();
            let generation = id.generation();
            let index = id.index();

            if index >= self.entries.len() {
                return None;
            }

            // SAFETY: `indices` is ordered. This is checked by the `indices` iterator.
            // This means that we can never access the same index twice.
            // The only other way we could try to access the same index twice, would be if
            // the index is the same, but the generation is different.
            // To ensure that we only access the same index once, we check (via a shared
            // reference) if the generation is the same.
            // We can access the generation and the value simultaneously, because they are
            // non-overlapping.
            let entry = unsafe { self.entries.as_mut_ptr().add(index) };

            // SAFETY: We need to access both the generation and the state separately.
            // The generation is accessed via a shared reference, while the state is
            // accessed via a mutable.
            // This is because we need to ensure that the generation is not the same.
            let entry_generation = unsafe { *ptr::addr_of!((*entry).generation) };

            if entry_generation != generation {
                return None;
            }

            // SAFETY: Access the entry information via a shared reference. We never mutate the
            // entry information so we take a shared reference, instead of a mutable reference (like
            // we do with the state).
            let entry_info = unsafe { &*ptr::addr_of!((*entry).info) };

            // SAFETY: We need to access both the generation and the state separately.
            // The previous check ensures that the generation is the same, so we can safely access
            // the state via a mutable reference.
            let entry_state = unsafe { &mut *ptr::addr_of_mut!((*entry).state) };

            if entry_info.is_vacant() {
                return None;
            }

            // SAFETY: The previous check ensures that we're no vacant, and therefore can access
            // the inner value safely (which is a union).
            Some(unsafe { &mut *entry_state.occupied })
        })
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

                let id = EntryId::new(entry.generation, index)?;
                let key = K::from_id(id);

                Some(key)
            })
    }

    pub(crate) fn entries(&self) -> impl Iterator<Item = (K, &V)> + '_ {
        self.entries
            .iter()
            .enumerate()
            .filter_map(move |(index, entry)| {
                let id = EntryId::new(entry.generation, index)?;
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
                let id = EntryId::new(entry.generation, index)?;
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
                let id = EntryId::new(entry.generation, index)?;
                let key = K::from_id(id);
                let value = entry.into_inner()?;

                Some((key, value))
            })
    }

    pub(crate) fn retain(&mut self, mut f: impl FnMut(K, &mut V) -> bool) {
        for (index, entry) in self.entries.iter_mut().enumerate() {
            let key = K::from_id(EntryId::new(entry.generation, index).expect("invalid entry id"));

            if let State::Occupied(value) = entry.state_mut() {
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

// TODO: test
pub struct SlabIndexMapper<'a, K>
where
    K: Key,
{
    lookup: Vec<Option<usize>>,
    reverse: Vec<Option<K>>,

    // we only have the lifetime here to ensure that the slab is not modified while we are mapping,
    // as that will invalidate indices.
    _lifetime: PhantomData<&'a ()>,
}

impl<'a, K> SlabIndexMapper<'a, K>
where
    K: Key,
{
    pub(crate) fn new<V>(slab: &'a Slab<K, V>) -> Self {
        // TODO: for more than 1024 elements (8 byte * 512 = 4096 bytes) we should use runtime
        //  length encoding instead.

        let mut lookup = vec![None; slab.entries.len()];
        let mut reverse = vec![None; slab.len()];

        for (index, key) in slab.keys().enumerate() {
            lookup[key.into_id().index()] = Some(index);
            reverse[index] = Some(key);
        }

        Self {
            lookup,
            reverse,
            _lifetime: PhantomData,
        }
    }
}

impl<K> IndexMapper<K> for SlabIndexMapper<'_, K>
where
    K: Key,
{
    fn max(&self) -> usize {
        self.reverse.len()
    }

    fn get(&self, from: &K) -> Option<usize> {
        self.lookup.get(from.into_id().index()).copied().flatten()
    }

    fn index(&self, from: &K) -> usize {
        self.lookup[from.into_id().index()].expect("tried to access vacant entry")
    }

    fn reverse(&self, to: usize) -> Option<MaybeOwned<K>> {
        self.reverse
            .get(to)
            .copied()
            .flatten()
            .map(MaybeOwned::from)
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
        // We could in theory remove 3 bytes by doing the following:
        //  1 byte: variant
        //  1 byte: generation
        // State: (union)
        //  2 bytes: state
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
                assert_eq!(left.generation, right.generation);
                assert_eq!(left.get(), right.get());
            }
        }
    }

    #[test]
    #[allow(unused_variables)]
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
    #[allow(unused_variables)]
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
        assert_eq!(EntryId::INDEX_MASK, 0x0000_0000_00FF_FFFF);
        assert_eq!(EntryId::INDEX_BITS, 24);
        assert_eq!(EntryId::GENERATION_MAX, 0xFF);
    }

    #[test]
    #[cfg(target_pointer_width = "32")]
    fn layout() {
        assert_eq!(EntryId::INDEX_MASK, 0x00FF_FFFF);
        assert_eq!(EntryId::INDEX_BITS, 24);
        assert_eq!(EntryId::GENERATION_MAX, 0xFF);
    }

    #[test]
    #[cfg(target_pointer_width = "16")]
    fn layout() {
        assert_eq!(EntryId::INDEX_MASK, 0x0FFF);
        assert_eq!(EntryId::INDEX_BITS, 12);
        assert_eq!(EntryId::GENERATION_MAX, 0xF);
    }

    #[test]
    fn filter_mut() {
        let mut slab = Slab::<EntryId, u16>::new();

        let a = slab.insert(42);
        let b = slab.insert(43);
        let c = slab.insert(44);

        let iter = slab.filter_mut(vec![a, b, c].into_iter());
        let output: Vec<_> = iter.collect();

        assert_eq!(output, vec![&mut 42, &mut 43, &mut 44]);
    }

    #[test]
    fn filter_mut_same_index_last_invalid() {
        let mut slab = Slab::<EntryId, u16>::new();

        let a = slab.insert(42);
        let a2 = EntryId::new(a.generation().next(), a.index()).unwrap();

        let iter = slab.filter_mut(vec![a, a2].into_iter());
        let output: Vec<_> = iter.collect();

        assert_eq!(output, vec![&mut 42]);
    }

    #[test]
    fn filter_mut_same_index_first_invalid() {
        let mut slab = Slab::<EntryId, u16>::new();

        let a = slab.insert(42);
        slab.remove(a);
        let a2 = slab.insert(42);

        let iter = slab.filter_mut(vec![a, a2].into_iter());
        let output: Vec<_> = iter.collect();

        assert_eq!(output, vec![&mut 42]);
    }

    #[test]
    fn filter_mut_not_sorted() {
        let mut slab = Slab::<EntryId, u16>::new();

        let a = slab.insert(42);
        let b = slab.insert(43);
        let c = slab.insert(44);

        let iter = slab.filter_mut(vec![b, a, c].into_iter());
        let output: Vec<_> = iter.collect();

        assert_eq!(output, vec![&mut 43, &mut 44]);
    }

    #[test]
    fn filter_mut_duplicate() {
        let mut slab = Slab::<EntryId, u16>::new();

        let a = slab.insert(42);

        let iter = slab.filter_mut(vec![a, a].into_iter());
        let output: Vec<_> = iter.collect();

        assert_eq!(output, vec![&mut 42]);
    }
}
