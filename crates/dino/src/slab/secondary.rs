use alloc::vec::Vec;
use core::iter;

use bitvec::{boxed::BitBox, prelude::BitVec};
use petgraph_core::storage::auxiliary::{BooleanGraphStorage, SecondaryGraphStorage};

use crate::slab::{EntryId, Generation, Key, Slab};

pub struct SlabBooleanStorage<'a, K> {
    flags: BitBox,

    _slab: core::marker::PhantomData<&'a ()>,
    _key: core::marker::PhantomData<fn() -> *const K>,
}

impl<'a, K> SlabBooleanStorage<'a, K>
where
    K: Key,
{
    pub(crate) fn new<V>(slab: &'a Slab<K, V>) -> Self {
        let length = slab.total_len();

        Self {
            flags: BitVec::repeat(false, length).into_boxed_bitslice(),

            _slab: core::marker::PhantomData,
            _key: core::marker::PhantomData,
        }
    }
}

impl<K> BooleanGraphStorage<K> for SlabBooleanStorage<'_, K>
where
    K: Key,
{
    #[inline]
    fn get(&self, id: K) -> Option<bool> {
        let index = id.into_id().index();

        self.flags.get(index).map(|bit| *bit)
    }

    #[inline]
    fn set(&mut self, id: K, flag: bool) -> Option<bool> {
        let index = id.into_id().index();

        let value = self.flags.replace(index, flag);

        Some(value)
    }

    #[inline]
    fn fill(&mut self, flag: bool) {
        self.flags.fill(flag);
    }
}

pub struct SlabSecondaryStorageIter<'a, K, T> {
    iter: iter::Enumerate<core::slice::Iter<'a, Option<(Generation, T)>>>,
    _marker: core::marker::PhantomData<&'a K>,
}

impl<'a, K, T> Iterator for SlabSecondaryStorageIter<'a, K, T>
where
    K: Key,
{
    type Item = (K, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (index, item) = self.iter.next()?;

            if let Some((generation, value)) = item.as_ref() {
                let id = EntryId::new(*generation, index)?;
                let id = K::from_id(id);

                return Some((id, value));
            }
        }
    }
}

pub struct SlabSecondaryStorage<'a, K, T> {
    // generation is needed for iter
    items: Vec<Option<(Generation, T)>>,

    _slab: core::marker::PhantomData<&'a ()>,
    _key: core::marker::PhantomData<fn() -> *const K>,
}

impl<'a, K, T> SlabSecondaryStorage<'a, K, T> {
    pub(crate) fn new<V>(slab: &'a Slab<K, V>) -> Self
    where
        K: Key,
    {
        let length = slab.total_len();

        Self {
            items: iter::repeat_with(|| None).take(length).collect::<Vec<_>>(),

            _slab: core::marker::PhantomData,
            _key: core::marker::PhantomData,
        }
    }
}

impl<K, T> SecondaryGraphStorage<K, T> for SlabSecondaryStorage<'_, K, T>
where
    K: Key,
{
    type Iter<'a> = SlabSecondaryStorageIter<'a, K, T> where
        K: 'a,
        T: 'a,
        Self: 'a,;

    fn get(&self, id: K) -> Option<&T> {
        let index = id.into_id().index();

        self.items
            .get(index)
            .and_then(|item| item.as_ref())
            .map(|(_, value)| value)
    }

    fn get_mut(&mut self, id: K) -> Option<&mut T> {
        let index = id.into_id().index();

        self.items
            .get_mut(index)
            .and_then(|item| item.as_mut())
            .map(|(_, value)| value)
    }

    fn set(&mut self, id: K, value: T) -> Option<T> {
        let index = id.into_id().index();
        let generation = id.into_id().generation();

        self.items
            .get_mut(index)
            .and_then(|item| item.replace((generation, value)))
            .map(|(_, value)| value)
    }

    fn remove(&mut self, id: K) -> Option<T> {
        let index = id.into_id().index();

        self.items
            .get_mut(index)
            .and_then(Option::take)
            .map(|(_, value)| value)
    }

    fn iter(&self) -> Self::Iter<'_> {
        SlabSecondaryStorageIter {
            iter: self.items.iter().enumerate(),
            _marker: core::marker::PhantomData,
        }
    }
}
