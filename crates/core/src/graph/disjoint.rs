use super::Graph;
use crate::{edge::EdgeMut, node::NodeMut};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GetDisjointMutError {
    /// An index provided was out-of-bounds for the slice.
    IndexOutOfBounds,
    /// Two indices provided were overlapping.
    OverlappingIndices,
}

pub trait DisjointMutGraph: Graph {
    // additional methods to a graph that allow for more performant mutable access

    fn get_disjoint_nodes_mut<const N: usize>(
        &mut self,
        ids: [Self::NodeId; N],
    ) -> Result<[NodeMut<'_, Self>; N], GetDisjointMutError>;

    fn get_disjoint_edges_mut<const N: usize>(
        &mut self,
        ids: [Self::EdgeId; N],
    ) -> Result<[EdgeMut<'_, Self>; N], GetDisjointMutError>;
}
