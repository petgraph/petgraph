use alloc::vec;
use core::hash::Hash;

use petgraph_core::{
    storage::RetainableGraphStorage, EdgeMut, GraphDirectionality, GraphStorage, NodeMut,
};

use crate::EntryStorage;

impl<NK, NV, EK, EV, D> RetainableGraphStorage for EntryStorage<NK, NV, EK, EV, D>
where
    D: GraphDirectionality,
    NK: Hash,
    EK: Hash,
{
    fn retain_nodes(&mut self, f: impl FnMut(NodeMut<'_, Self>) -> bool) {
        let mut remove = vec![];

        for node in self.inner.nodes_mut() {
            let node = unsafe { node.change_storage_unchecked() };
            if !f(node) {
                remove.push(node.id());
            }
        }

        for id in remove {
            self.remove_node(id);
        }
    }

    fn retain_edges(&mut self, f: impl FnMut(EdgeMut<'_, Self>) -> bool) {
        let mut remove = vec![];
        for edge in self.inner.edges_mut() {
            let edge = unsafe { edge.change_storage_unchecked() };
            if !f(edge) {
                remove.push(edge.id());
            }
        }

        for id in remove {
            self.remove_edge(id);
        }
    }
}
