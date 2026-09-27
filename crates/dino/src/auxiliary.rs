use petgraph_core::{
    edge::EdgeId,
    node::NodeId,
    storage::{auxiliary::Hints, AuxiliaryGraphStorage},
    GraphDirectionality,
};

use crate::{
    slab::secondary::{SlabBooleanStorage, SlabSecondaryStorage},
    DinoStorage,
};

impl<N, E, D> AuxiliaryGraphStorage for DinoStorage<N, E, D>
where
    D: GraphDirectionality,
{
    type BooleanEdgeStorage<'graph> = SlabBooleanStorage<'graph, EdgeId> where Self: 'graph;
    type BooleanNodeStorage<'graph> = SlabBooleanStorage<'graph, NodeId>  where Self: 'graph;
    type SecondaryEdgeStorage<'graph, V> = SlabSecondaryStorage<'graph, EdgeId, V>  where Self: 'graph;
    type SecondaryNodeStorage<'graph, V> = SlabSecondaryStorage<'graph, NodeId, V> where Self: 'graph;

    fn secondary_node_storage<V>(&self, _: Hints) -> Self::SecondaryNodeStorage<'_, V> {
        SlabSecondaryStorage::new(&self.nodes)
    }

    fn secondary_edge_storage<V>(&self, _: Hints) -> Self::SecondaryEdgeStorage<'_, V> {
        SlabSecondaryStorage::new(&self.edges)
    }

    fn boolean_node_storage(&self, _: Hints) -> Self::BooleanNodeStorage<'_> {
        SlabBooleanStorage::new(&self.nodes)
    }

    fn boolean_edge_storage(&self, _: Hints) -> Self::BooleanEdgeStorage<'_> {
        SlabBooleanStorage::new(&self.edges)
    }
}
