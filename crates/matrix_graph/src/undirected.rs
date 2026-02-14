use core::{cmp, hash::BuildHasher, marker::PhantomData};

use petgraph_core::{
    edge::{EdgeMut, EdgeRef},
    graph::{Cardinality, DensityHint, DirectedGraph, Graph},
    id::Id,
    node::{NodeMut, NodeRef},
};

use crate::{
    EdgeId, EdgeIteratorMut, MatrixGraph, MatrixGraphExtras, NodeId, Nullable, Undirected,
    ensure_len, private::Sealed,
};
pub type UndirEdgeId = EdgeId<Undirected>;

impl UndirEdgeId {
    pub fn new_undirected(node1: NodeId, node2: NodeId) -> Self {
        EdgeId {
            node1,
            node2,
            direction: PhantomData,
        }
    }
}

impl PartialEq for UndirEdgeId {
    fn eq(&self, other: &Self) -> bool {
        (self.node1 == other.node1 && self.node2 == other.node2)
            || (self.node1 == other.node2 && self.node2 == other.node1)
    }
}

impl Eq for UndirEdgeId {}

impl PartialOrd for UndirEdgeId {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let self_min = cmp::min(self.node1, self.node2);
        let self_max = cmp::max(self.node1, self.node2);
        let other_min = cmp::min(other.node1, other.node2);
        let other_max = cmp::max(other.node1, other.node2);

        match self_min.partial_cmp(&other_min) {
            Some(cmp::Ordering::Equal) => self_max.partial_cmp(&other_max),
            non_eq => non_eq,
        }
    }
}

impl Ord for UndirEdgeId {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Id for UndirEdgeId {}

impl<N, E, S, Null: Nullable<Wrapped = E>> Sealed for MatrixGraph<N, E, S, Null, Undirected> {}

impl<N, E, S: BuildHasher, Null: Nullable<Wrapped = E>> MatrixGraphExtras<N>
    for MatrixGraph<N, E, S, Null, Undirected>
{
    #[inline]
    fn to_edge_position(&self, a: NodeId, b: NodeId) -> Option<usize> {
        if a.0 >= self.node_capacity || b.0 >= self.node_capacity {
            None
        } else {
            Some(self.to_edge_position_unchecked(a, b))
        }
    }

    #[inline]
    fn to_edge_position_unchecked(&self, a: NodeId, b: NodeId) -> usize {
        to_lower_triangular_matrix_position(a.0, b.0)
    }

    #[inline]
    fn extend_capacity_for_node(&mut self, new_node_capacity: usize, _exact: bool) {
        let old_node_capacity = self.node_capacity;

        if old_node_capacity >= new_node_capacity {
            return;
        }

        self.node_capacity =
            extend_lower_triangular_matrix(&mut self.node_adjacencies, new_node_capacity);
    }

    #[inline]
    fn remove_node(&mut self, a: NodeId) -> N {
        for (id, _) in self.nodes.iter() {
            let position = self.to_edge_position(a, id);
            if let Some(pos) = position {
                self.node_adjacencies[pos] = Default::default();
            }
        }

        self.nodes.remove(a.0)
    }
}

#[inline]
fn to_lower_triangular_matrix_position(row: usize, column: usize) -> usize {
    let (row, column) = if row > column {
        (row, column)
    } else {
        (column, row)
    };
    (row * (row + 1)) / 2 + column
}

#[inline]
fn extend_lower_triangular_matrix<T: Default>(
    node_adjacencies: &mut Vec<T>,
    new_capacity: usize,
) -> usize {
    let max_node = new_capacity - 1;
    let max_pos = to_lower_triangular_matrix_position(max_node, max_node);
    ensure_len(node_adjacencies, max_pos + 1);
    new_capacity
}

impl<N, E, S: BuildHasher, Null: Nullable<Wrapped = E>> Graph
    for MatrixGraph<N, E, S, Null, Undirected>
{
    type EdgeData<'graph>
        = E
    where
        Self: 'graph;
    type EdgeDataMut<'graph>
        = &'graph mut E
    where
        Self: 'graph;
    type EdgeDataRef<'graph>
        = &'graph E
    where
        Self: 'graph;
    type EdgeId = UndirEdgeId;
    type NodeData<'graph>
        = N
    where
        Self: 'graph;
    type NodeDataMut<'graph>
        = &'graph mut N
    where
        Self: 'graph;
    type NodeDataRef<'graph>
        = &'graph N
    where
        Self: 'graph;
    type NodeId = NodeId;
}
