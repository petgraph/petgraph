use fixedbitset::FixedBitSet;
use petgraph_old::{
    csr::IndexType,
    graph::{EdgeIndex, NodeIndex},
};

use crate::graph::storable::{DataContainer, VisitContainer, containers::DefaultVec};

impl<Ix: IndexType> VisitContainer<NodeIndex<Ix>> for FixedBitSet {
    fn mark_visited(&mut self, node_id: NodeIndex<Ix>) {
        self.set(node_id.index(), true);
    }

    fn mark_unvisited(&mut self, node_id: NodeIndex<Ix>) {
        self.set(node_id.index(), false);
    }

    fn is_visited(&self, node_id: NodeIndex<Ix>) -> bool {
        self.contains(node_id.index())
    }

    fn clear(&mut self) {
        self.clear();
    }
}

impl<Ix: IndexType, Data: Clone> DataContainer<NodeIndex<Ix>, Data> for DefaultVec<Data> {
    fn get(&self, id: NodeIndex<Ix>) -> &Data {
        &self.vec[id.index()]
    }

    fn get_mut(&mut self, id: NodeIndex<Ix>) -> &mut Data {
        &mut self.vec[id.index()]
    }

    fn insert(&mut self, id: NodeIndex<Ix>, data: Data) {
        self.vec[id.index()] = data;
    }

    fn remove(&mut self, id: NodeIndex<Ix>) {
        self.vec[id.index()] = self.default.clone();
    }

    fn clear(&mut self) {
        for data in &mut self.vec {
            *data = self.default.clone();
        }
    }
}

impl<Ix: IndexType> VisitContainer<EdgeIndex<Ix>> for FixedBitSet {
    fn mark_visited(&mut self, node_id: EdgeIndex<Ix>) {
        self.set(node_id.index(), true);
    }

    fn mark_unvisited(&mut self, node_id: EdgeIndex<Ix>) {
        self.set(node_id.index(), false);
    }

    fn is_visited(&self, node_id: EdgeIndex<Ix>) -> bool {
        self.contains(node_id.index())
    }

    fn clear(&mut self) {
        self.clear();
    }
}

impl<Ix: IndexType, Data: Clone> DataContainer<EdgeIndex<Ix>, Data> for DefaultVec<Data> {
    fn get(&self, id: EdgeIndex<Ix>) -> &Data {
        &self.vec[id.index()]
    }

    fn get_mut(&mut self, id: EdgeIndex<Ix>) -> &mut Data {
        &mut self.vec[id.index()]
    }

    fn insert(&mut self, id: EdgeIndex<Ix>, data: Data) {
        self.vec[id.index()] = data;
    }

    fn remove(&mut self, id: EdgeIndex<Ix>) {
        self.vec[id.index()] = self.default.clone();
    }

    fn clear(&mut self) {
        for data in &mut self.vec {
            *data = self.default.clone();
        }
    }
}
