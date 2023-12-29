use alloc::vec::Vec;
use core::iter;

use bitvec::{boxed::BitBox, prelude::BitVec};
use numi::borrow::Moo;
use petgraph_core::id::{AttributeStorage, FlagStorage};

use crate::slab::{EntryId, Generation, Key, Slab};

pub struct SlabFlagStorage<'a> {
    flags: BitBox,
    _marker: core::marker::PhantomData<&'a ()>,
}

impl<'a> SlabFlagStorage<'a> {
    pub(crate) fn new<K, V>(slab: &'a Slab<K, V>) -> Self
    where
        K: Key,
    {
        let length = slab.total_len();

        Self {
            flags: BitVec::repeat(false, length).into_boxed_bitslice(),
            _marker: core::marker::PhantomData,
        }
    }
}

impl<Id> FlagStorage<Id> for SlabFlagStorage<'_>
where
    Id: Key,
{
    #[inline]
    fn get(&self, id: &Id) -> Option<bool> {
        let index = id.into_id().index();

        self.flags.get(index).map(|bit| *bit)
    }

    #[inline]
    fn index(&self, id: &Id) -> bool {
        let index = id.into_id().index();

        self.flags[index]
    }

    #[inline]
    fn set(&mut self, id: &Id, flag: bool) -> Option<bool> {
        let index = id.into_id().index();

        let value = self.flags.replace(index, flag);

        Some(value)
    }
}

pub struct SlabAttributeStorageIter<'a, K, T> {
    iter: iter::Enumerate<core::slice::Iter<'a, Option<(Generation, T)>>>,
    _marker: core::marker::PhantomData<&'a K>,
}

impl<'a, K, T> Iterator for SlabAttributeStorageIter<'a, K, T>
where
    K: Key,
{
    type Item = (Moo<'a, K>, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (index, item) = self.iter.next()?;

            if let Some((generation, value)) = item.as_ref() {
                let id = EntryId::new(*generation, index)?;
                let id = K::from_id(id);

                return Some((Moo::Owned(id), value));
            }
        }
    }
}

pub struct SlabAttributeStorage<'a, K, T> {
    // generation is needed for iter
    items: Vec<Option<(Generation, T)>>,

    _slab: core::marker::PhantomData<&'a ()>,
    _key: core::marker::PhantomData<fn() -> *const K>,
}

impl<'a, K, T> SlabAttributeStorage<'a, K, T> {
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

impl<K, T> AttributeStorage<K, T> for SlabAttributeStorage<'_, K, T>
where
    K: Key,
{
    type Iter<'a> = SlabAttributeStorageIter<'a, K, T> where
        K: 'a,
        T: 'a,
        Self: 'a,;

    fn get(&self, id: &K) -> Option<&T> {
        let index = id.into_id().index();

        self.items
            .get(index)
            .and_then(|item| item.as_ref())
            .map(|(_, value)| value)
    }

    fn get_mut(&mut self, id: &K) -> Option<&mut T> {
        let index = id.into_id().index();

        self.items
            .get_mut(index)
            .and_then(|item| item.as_mut())
            .map(|(_, value)| value)
    }

    fn set(&mut self, id: &K, value: T) -> Option<T> {
        let index = id.into_id().index();
        let generation = id.into_id().generation();

        self.items
            .get_mut(index)
            .and_then(|item| item.replace((generation, value)))
            .map(|(_, value)| value)
    }

    fn remove(&mut self, id: &K) -> Option<T> {
        let index = id.into_id().index();

        self.items
            .get_mut(index)
            .and_then(Option::take)
            .map(|(_, value)| value)
    }

    fn iter(&self) -> Self::Iter<'_> {
        SlabAttributeStorageIter {
            iter: self.items.iter().enumerate(),
            _marker: core::marker::PhantomData,
        }
    }
}
