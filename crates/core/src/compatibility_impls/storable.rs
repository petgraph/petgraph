use petgraph_old::{csr::IndexType, graph::NodeIndex};

use crate::graph::storable::{DataContainer, containers::DefaultVec};

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
