use core::hash::{BuildHasher, Hash};

use hashbrown::HashMap;

use crate::graph::storable::DataContainer;

pub mod directed;
pub mod undirected;

pub struct DefaultMap<Id, Data, S = foldhash::fast::RandomState> {
    map: HashMap<Id, Data, S>,
    default: Data,
}

impl<Id: Eq + Hash, Data: Default, S: BuildHasher> DataContainer<Id, Data>
    for DefaultMap<Id, Data, S>
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
        self.map.into_iter()
    }
}
