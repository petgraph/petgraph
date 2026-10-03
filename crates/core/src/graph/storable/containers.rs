use alloc::{collections::VecDeque, vec::Vec};
use core::hash::Hash;

use hashbrown::{HashMap, HashSet};

use super::{QueueContainer, StackContainer, VisitContainer};
use crate::{graph::DataContainer, id::IndexId};

impl<Id: Eq + Hash, Data: Default> DataContainer<Id, Data> for HashMap<Id, Data> {
    fn get(&mut self, node_id: Id) -> &Data {
        self.entry(node_id).or_insert_with(Default::default)
    }

    fn get_mut(&mut self, node_id: Id) -> &mut Data {
        self.entry(node_id).or_insert_with(Default::default)
    }

    fn insert(&mut self, node_id: Id, data: Data) {
        self.insert(node_id, data);
    }

    fn remove(&mut self, node_id: Id) {
        self.remove(&node_id);
    }

    fn clear(&mut self) {
        self.clear();
    }
}

impl<Id: IndexId, Data: Default> DataContainer<Id, Data> for Vec<(Id, Data)> {
    fn get(&mut self, node_id: Id) -> &Data {
        &self[node_id.as_usize()].1
    }

    fn get_mut(&mut self, node_id: Id) -> &mut Data {
        &mut self[node_id.as_usize()].1
    }

    fn insert(&mut self, node_id: Id, data: Data) {
        *&mut self[node_id.as_usize()].1 = data;
    }

    fn remove(&mut self, node_id: Id) {
        *&mut self[node_id.as_usize()].1 = Data::default();
    }

    fn clear(&mut self) {
        for (_, data) in self.iter_mut() {
            *data = Data::default();
        }
    }
}

impl<Id: Eq + Hash> VisitContainer<Id> for HashSet<Id> {
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

impl<Id: IndexId> VisitContainer<Id> for Vec<Option<()>> {
    fn mark_visited(&mut self, node_id: Id) {
        *&mut self[node_id.as_usize()] = Some(());
    }

    fn mark_unvisited(&mut self, node_id: Id) {
        *&mut self[node_id.as_usize()] = None;
    }

    fn is_visited(&self, node_id: Id) -> bool {
        self[node_id.as_usize()].is_some()
    }

    fn clear(&mut self) {
        for v in self.iter_mut() {
            *v = None;
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

    fn clear(&mut self) {
        self.clear();
    }
}

impl<Data> QueueContainer<Data> for VecDeque<Data> {
    fn enqueue(&mut self, data: Data) {
        self.push_back(data);
    }

    fn dequeue(&mut self) -> Option<Data> {
        self.pop_front()
    }

    fn clear(&mut self) {
        self.clear();
    }
}
