use core::slice::GetDisjointMutError;

use super::Graph;

pub trait DisjointMutGraph: Graph {
    // additional methods to a graph that allow for more performant mutable access

    fn get_disjoint_nodes_mut<const N: usize>(
        &mut self,
        ids: [Self::NodeId; N],
    ) -> Result<[Self::NodeMut<'_>; N], GetDisjointMutError>;

    fn get_disjoint_edges_mut<const N: usize>(
        &mut self,
        ids: [Self::EdgeId; N],
    ) -> Result<[Self::EdgeMut<'_>; N], GetDisjointMutError>;
}
