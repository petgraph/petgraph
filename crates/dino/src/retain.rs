use petgraph_core::{
    edge::{marker::GraphDirectionality, EdgeMut},
    node::NodeMut,
    storage::RetainableGraphStorage,
};
use roaring::RoaringBitmap;

use crate::{slab::Key, DinosaurStorage};

impl<N, E, D> RetainableGraphStorage for DinosaurStorage<N, E, D>
where
    D: GraphDirectionality,
{
    fn retain(
        &mut self,
        mut nodes: impl FnMut(NodeMut<'_, Self>) -> bool,
        mut edges: impl FnMut(EdgeMut<'_, Self>) -> bool,
    ) {
        let mut removed = RoaringBitmap::new();

        self.nodes.retain(|_, value| {
            let node = NodeMut::new(&value.id, &mut value.weight);

            let retain = nodes(node);

            if !retain {
                removed.insert(value.id.into_id().raw());
            }

            retain
        });

        self.edges.retain(|_, value| {
            if removed.contains(value.source.into_id().raw())
                || removed.contains(value.target.into_id().raw())
            {
                return false;
            }

            let edge = EdgeMut::new(&value.id, &mut value.weight, &value.source, &value.target);

            edges(edge)
        });

        self.closures.refresh(&self.nodes, &self.edges);
    }

    fn retain_nodes(&mut self, mut f: impl FnMut(NodeMut<'_, Self>) -> bool) {
        let mut removed = RoaringBitmap::new();

        self.nodes.retain(|_, value| {
            let node = NodeMut::new(&value.id, &mut value.weight);

            let retain = f(node);

            if !retain {
                removed.insert(value.id.into_id().raw());
            }

            retain
        });

        self.edges.retain(|_, value| {
            !removed.contains(value.source.into_id().raw())
                && !removed.contains(value.target.into_id().raw())
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
