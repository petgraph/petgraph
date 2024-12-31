use alloc::vec;
use core::hash::Hash;

use petgraph_core::{
    EdgeMut, GraphDirectionality, GraphStorage, NodeMut, storage::GraphStoragePrune,
};

use crate::EntryStorage;

impl<NK, NV, EK, EV, D> GraphStoragePrune for EntryStorage<NK, NV, EK, EV, D>
where
    D: GraphDirectionality,
    NK: Hash,
    EK: Hash,
{
    fn retain_nodes(&mut self, mut f: impl FnMut(NodeMut<'_, Self>) -> bool) {
        let mut remove = vec![];

        for node in self.inner.nodes_mut() {
            let node = node.change_storage_unchecked();
            let id = node.id();

            if !f(node) {
                remove.push(id);
            }
        }

        for id in remove {
            self.remove_node(id);
        }
    }

    fn retain_edges(&mut self, mut f: impl FnMut(EdgeMut<'_, Self>) -> bool) {
        let mut remove = vec![];
        for edge in self.inner.edges_mut() {
            let edge = edge.change_storage_unchecked();
            let id = edge.id();

            if !f(edge) {
                remove.push(id);
            }
        }

        for id in remove {
            self.remove_edge(id);
        }
    }
}
