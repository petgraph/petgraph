use alloc::{collections::VecDeque, vec::Vec};
#[cfg(feature = "std")]
use std::collections::{HashMap as StdHashMap, HashSet as StdHashSet};

#[cfg(feature = "hashbrown")]
use hashbrown::{HashMap as HashbrownHashMap, HashSet as HashbrownHashSet};

use super::{DataContainer, QueueContainer, StackContainer, VisitContainer};
use crate::id::IndexId;

impl<Id: IndexId, Data: Default> DataContainer<Id, Data> for Vec<Data> {
    fn get(&self, node_id: Id) -> &Data {
        &self[node_id.as_usize()]
    }

    fn get_mut(&mut self, node_id: Id) -> &mut Data {
        &mut self[node_id.as_usize()]
    }

    fn insert(&mut self, node_id: Id, data: Data) {
        self[node_id.as_usize()] = data;
    }

    fn remove(&mut self, node_id: Id) {
        self[node_id.as_usize()] = Data::default();
    }

    fn clear(&mut self) {
        for data in self.iter_mut() {
            *data = Data::default();
        }
    }

    fn into_iter(self) -> impl Iterator<Item = (Id, Data)> {
        IntoIterator::into_iter(self)
            .enumerate()
            .map(|(index, data)| (Id::from_usize(index), data))
    }
}

impl<Id: IndexId> VisitContainer<Id> for Vec<Option<()>> {
    fn mark_visited(&mut self, node_id: Id) {
        self[node_id.as_usize()] = Some(());
    }

    fn mark_unvisited(&mut self, node_id: Id) {
        self[node_id.as_usize()] = None;
    }

    fn is_visited(&self, node_id: Id) -> bool {
        self[node_id.as_usize()].is_some()
    }

    fn clear(&mut self) {
        for entry in self.iter_mut() {
            *entry = None;
        }
    }
}

impl<Data> StackContainer<Data> for Vec<Data> {
    fn push(&mut self, data: Data) {
        self.push(data);
    }

    fn pop(&mut self) -> Option<Data> {
        self.pop()
    }

    fn peek_last(&self) -> Option<&Data> {
        self.last()
    }

    fn clear(&mut self) {
        self.clear();
    }
}

impl<Data> QueueContainer<Data> for VecDeque<Data> {
    fn push_back(&mut self, data: Data) {
        self.push_back(data);
    }

    fn pop_front(&mut self) -> Option<Data> {
        self.pop_front()
    }

    fn peek_first(&self) -> Option<&Data> {
        self.front()
    }

    fn clear(&mut self) {
        self.clear();
    }
}

#[cfg(feature = "std")]
pub struct DefaultMapStd<Id, Data, S = std::hash::RandomState> {
    map: StdHashMap<Id, Data, S>,
    default: Data,
}

#[cfg(feature = "std")]
impl<Id: core::hash::Hash + core::cmp::Eq, Data: Default, S: core::hash::BuildHasher>
    DataContainer<Id, Data> for DefaultMapStd<Id, Data, S>
{
    fn get(&self, node_id: Id) -> &Data {
        self.map.get(&node_id).unwrap_or(&self.default)
    }

    fn get_mut(&mut self, node_id: Id) -> &mut Data {
        self.map.entry(node_id).or_default()
    }

    fn insert(&mut self, node_id: Id, data: Data) {
        self.map.insert(node_id, data);
    }

    fn remove(&mut self, node_id: Id) {
        self.map.remove(&node_id);
    }

    fn clear(&mut self) {
        self.map.clear();
    }

    fn into_iter(self) -> impl Iterator<Item = (Id, Data)> {
        IntoIterator::into_iter(self.map)
    }
}

#[cfg(feature = "std")]
impl<Id: core::hash::Hash + core::cmp::Eq, S: core::hash::BuildHasher> VisitContainer<Id>
    for StdHashSet<Id, S>
{
    fn mark_visited(&mut self, node_id: Id) {
        self.insert(node_id);
    }

    fn mark_unvisited(&mut self, node_id: Id) {
        self.remove(&node_id);
    }

    fn is_visited(&self, node_id: Id) -> bool {
        self.contains(&node_id)
    }

    fn clear(&mut self) {
        self.clear();
    }
}

#[cfg(feature = "hashbrown")]
pub struct DefaultMapHashbrown<Id, Data, S = hashbrown::DefaultHashBuilder> {
    map: HashbrownHashMap<Id, Data, S>,
    default: Data,
}

#[cfg(feature = "hashbrown")]
impl<Id: core::hash::Hash + core::cmp::Eq, Data: Default, S: core::hash::BuildHasher>
    DataContainer<Id, Data> for DefaultMapHashbrown<Id, Data, S>
{
    fn get(&self, node_id: Id) -> &Data {
        self.map.get(&node_id).unwrap_or(&self.default)
    }

    fn get_mut(&mut self, node_id: Id) -> &mut Data {
        self.map.entry(node_id).or_default()
    }

    fn insert(&mut self, node_id: Id, data: Data) {
        self.map.insert(node_id, data);
    }

    fn remove(&mut self, node_id: Id) {
        self.map.remove(&node_id);
    }

    fn clear(&mut self) {
        self.map.clear();
    }

    fn into_iter(self) -> impl Iterator<Item = (Id, Data)> {
        IntoIterator::into_iter(self.map)
    }
}

#[cfg(feature = "hashbrown")]
impl<Id: core::hash::Hash + core::cmp::Eq, S: core::hash::BuildHasher> VisitContainer<Id>
    for HashbrownHashSet<Id, S>
{
    fn mark_visited(&mut self, node_id: Id) {
        self.insert(node_id);
    }

    fn mark_unvisited(&mut self, node_id: Id) {
        self.remove(&node_id);
    }

    fn is_visited(&self, node_id: Id) -> bool {
        self.contains(&node_id)
    }

    fn clear(&mut self) {
        self.clear();
    }
}
