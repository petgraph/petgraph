use crate::{edge::EdgeMut, node::NodeMut, storage::GraphStorage};

pub trait RetainableGraphStorage: GraphStorage {
    fn retain(
        &mut self,
        mut nodes: impl FnMut(NodeMut<'_, Self>) -> bool,
        mut edges: impl FnMut(EdgeMut<'_, Self>) -> bool,
    ) {
        self.retain_nodes(&mut nodes);
        self.retain_edges(&mut edges);
    }

    fn retain_nodes(&mut self, f: impl FnMut(NodeMut<'_, Self>) -> bool);

    fn retain_edges(&mut self, f: impl FnMut(EdgeMut<'_, Self>) -> bool);
}
