use crate::{edge::EdgeMut, graph::Graph, node::NodeMut, storage::RetainableGraphStorage};

impl<S> Graph<S>
where
    S: RetainableGraphStorage,
{
    pub fn retain(
        &mut self,
        nodes: impl FnMut(NodeMut<'_, S>) -> bool,
        edges: impl FnMut(EdgeMut<'_, S>) -> bool,
    ) {
        self.storage.retain(nodes, edges);
    }

    pub fn retain_nodes(&mut self, f: impl FnMut(NodeMut<'_, S>) -> bool) {
        self.storage.retain_nodes(f);
    }

    pub fn retain_edges(&mut self, f: impl FnMut(EdgeMut<'_, S>) -> bool) {
        self.storage.retain_edges(f);
    }
}
