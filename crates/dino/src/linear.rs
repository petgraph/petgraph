use petgraph_core::{
    edge::marker::GraphDirection,
    storage::{LinearGraphStorage, LinearIndexLookup},
};

use crate::{slab::SlabLinearIndexLookup, DinosaurStorage};

impl<N, E, D> LinearGraphStorage for DinosaurStorage<N, E, D>
where
    D: GraphDirection,
{
    fn linear_edge_index_lookup(&self) -> impl LinearIndexLookup<GraphId = Self::EdgeId> + '_ {
        SlabLinearIndexLookup::new(&self.edges)
    }

    fn linear_node_index_lookup(&self) -> impl LinearIndexLookup<GraphId = Self::NodeId> + '_ {
        SlabLinearIndexLookup::new(&self.nodes)
    }
}
