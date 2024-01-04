use petgraph_core::{
    edge::EdgeId, node::NodeId, storage::sequential::SequentialGraphStorage, GraphDirectionality,
};

use crate::{slab::SlabIndexMapper, DinoStorage};

impl<N, E, D> SequentialGraphStorage for DinoStorage<N, E, D>
where
    D: GraphDirectionality,
{
    type EdgeIdBijection<'graph> = SlabIndexMapper<'graph, EdgeId> where Self: 'graph;
    type NodeIdBijection<'graph> = SlabIndexMapper<'graph, NodeId> where Self: 'graph;

    fn node_id_bijection(&self) -> Self::NodeIdBijection<'_> {
        SlabIndexMapper::new(&self.nodes)
    }

    fn edge_id_bijection(&self) -> Self::EdgeIdBijection<'_> {
        SlabIndexMapper::new(&self.edges)
    }
}
