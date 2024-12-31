use alloc::collections::BTreeSet;

use petgraph_core::{
    edge::{EdgeMut, marker::GraphDirectionality},
    node::NodeMut,
    storage::GraphStoragePrune,
};

use crate::{DinoStorage, closure::Closures};

impl<N, E, D> GraphStoragePrune for DinoStorage<N, E, D>
where
    D: GraphDirectionality,
{
    fn retain(
        &mut self,
        mut nodes: impl FnMut(NodeMut<'_, Self>) -> bool,
        mut edges: impl FnMut(EdgeMut<'_, Self>) -> bool,
    ) {
        let mut removed = BTreeSet::new();

        self.nodes.retain(|_, value| {
            let node = NodeMut::new(value.id, &mut value.weight);

            let retain = nodes(node);

            if !retain {
                removed.insert(value.id);
            }

            retain
        });

        self.edges.retain(|_, value| {
            if removed.contains(&value.source) || removed.contains(&value.target) {
                return false;
            }

            let edge = EdgeMut::new(value.id, &mut value.weight, value.source, value.target);

            edges(edge)
        });

        Closures::refresh(&mut self.nodes, &self.edges);
    }

    fn retain_nodes(&mut self, mut f: impl FnMut(NodeMut<'_, Self>) -> bool) {
        let mut removed = BTreeSet::new();

        self.nodes.retain(|_, value| {
            let node = NodeMut::new(value.id, &mut value.weight);

            let retain = f(node);

            if !retain {
                removed.insert(value.id);
            }

            retain
        });

        self.edges.retain(|_, value| {
            !removed.contains(&value.source) && !removed.contains(&value.target)
        });

        Closures::refresh(&mut self.nodes, &self.edges);
    }

    fn retain_edges(&mut self, mut f: impl FnMut(EdgeMut<'_, Self>) -> bool) {
        self.edges.retain(|_, value| {
            let edge = EdgeMut::new(value.id, &mut value.weight, value.source, value.target);

            f(edge)
        });

        Closures::refresh(&mut self.nodes, &self.edges);
    }
}
