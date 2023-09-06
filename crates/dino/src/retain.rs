use petgraph_core::{
    edge::{marker::GraphDirection, EdgeMut},
    node::NodeMut,
    storage::RetainableGraphStorage,
};

use crate::DinosaurStorage;

impl<N, E, D> RetainableGraphStorage for DinosaurStorage<N, E, D>
where
    D: GraphDirection,
{
    fn retain(
        &mut self,
        mut nodes: impl FnMut(NodeMut<'_, Self>) -> bool,
        mut edges: impl FnMut(EdgeMut<'_, Self>) -> bool,
    ) {
        self.nodes.retain(|_, value| {
            let node = NodeMut::new(&value.id, &mut value.weight);

            nodes(node)
        });

        self.edges.retain(|_, value| {
            let edge = EdgeMut::new(&value.id, &mut value.weight, &value.source, &value.target);

            edges(edge)
        });

        self.closures.refresh(&self.nodes, &self.edges);
    }

    fn retain_nodes(&mut self, mut f: impl FnMut(NodeMut<'_, Self>) -> bool) {
        self.nodes.retain(|_, value| {
            let node = NodeMut::new(&value.id, &mut value.weight);

            f(node)
        });

        self.closures.refresh(&self.nodes, &self.edges);
    }

    fn retain_edges(&mut self, mut f: impl FnMut(EdgeMut<'_, Self>) -> bool) {
        self.edges.retain(|_, value| {
            let edge = EdgeMut::new(&value.id, &mut value.weight, &value.source, &value.target);

            f(edge)
        });

        self.closures.refresh(&self.nodes, &self.edges);
    }
}
