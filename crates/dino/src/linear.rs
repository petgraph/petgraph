use petgraph_core::{
    edge::EdgeId, node::NodeId, storage::linear::LinearGraphStorage, GraphDirectionality,
};

use crate::{slab::SlabIndexMapper, DinoStorage};

impl<N, E, D> LinearGraphStorage for DinoStorage<N, E, D>
where
    D: GraphDirectionality,
{
    type EdgeIndexMapper<'graph> = SlabIndexMapper<'graph, EdgeId> where Self: 'graph;
    type NodeIndexMapper<'graph> = SlabIndexMapper<'graph, NodeId> where Self: 'graph;

    fn node_index_mapper(&self) -> Self::NodeIndexMapper<'_> {
        SlabIndexMapper::new(&self.nodes)
    }

    fn edge_index_mapper(&self) -> Self::EdgeIndexMapper<'_> {
        SlabIndexMapper::new(&self.edges)
    }
}
