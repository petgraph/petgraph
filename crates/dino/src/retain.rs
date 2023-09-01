use petgraph_core::{edge::EdgeMut, node::NodeMut, storage::RetainGraphStorage};

use crate::DinosaurStorage;

impl<N, E, D> RetainGraphStorage for DinosaurStorage<N, E, D> {
    fn retain(
        &mut self,
        mut nodes: impl FnMut(NodeMut<'_, Self>) -> bool,
        mut edges: impl FnMut(EdgeMut<'_, Self>) -> bool,
    ) {
        self.nodes.retain(|id, value| {
            let node = NodeMut::new(id, &mut value.weight);

            nodes(node)
        });

        self.edges.retain(|id, value| {
            let edge = EdgeMut::new(id, &value.source, &value.target, &mut value.weight);

            edges(edge)
        });

        self.closures.refresh(&self.nodes, &self.edges);
    }

    fn retain_nodes(&mut self, mut f: impl FnMut(NodeMut<'_, Self>) -> bool) {
        self.nodes.retain(|id, value| {
            let node = NodeMut::new(id, &mut value.weight);

            f(node)
        });

        self.closures.refresh(&self.nodes, &self.edges);
    }

    fn retain_edges(&mut self, mut f: impl FnMut(EdgeMut<'_, Self>) -> bool) {
        self.edges.retain(|id, value| {
            let edge = EdgeMut::new(id, &value.source, &value.target, &mut value.weight);

            f(edge)
        });

        self.closures.refresh(&self.nodes, &self.edges);
    }
}
