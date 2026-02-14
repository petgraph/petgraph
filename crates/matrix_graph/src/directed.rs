use core::{cmp, hash::BuildHasher, marker::PhantomData};
use std::iter::Chain;

use itertools::Either;
use petgraph_core::{
    edge::{Edge, EdgeMut, EdgeRef},
    graph::{Cardinality, DensityHint, DirectedGraph, Graph},
    id::Id,
    node::{NodeMut, NodeRef},
};

use crate::{
    Directed, EdgeId, EdgeIterator, EdgeIteratorMut, MatrixGraph, MatrixGraphExtras, NodeId,
    Nullable, ensure_len, private::Sealed,
};

pub type DirEdgeId = EdgeId<Directed>;

impl EdgeId<Directed> {
    pub fn new_directed(source: NodeId, target: NodeId) -> Self {
        EdgeId {
            node1: source,
            node2: target,
            direction: PhantomData,
        }
    }
}

impl PartialEq for DirEdgeId {
    fn eq(&self, other: &Self) -> bool {
        self.node1 == other.node1 && self.node2 == other.node2
    }
}

impl Eq for DirEdgeId {}

impl PartialOrd for DirEdgeId {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match self.node1.partial_cmp(&other.node1) {
            Some(cmp::Ordering::Equal) => self.node2.partial_cmp(&other.node2),
            non_eq => non_eq,
        }
    }
}

impl Ord for DirEdgeId {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Id for DirEdgeId {}

impl<N, E, S, Null: Nullable<Wrapped = E>> Sealed for MatrixGraph<N, E, S, Null, Directed> {}

impl<N, E, S: BuildHasher, Null: Nullable<Wrapped = E>> MatrixGraphExtras<N>
    for MatrixGraph<N, E, S, Null, Directed>
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
        to_flat_square_matrix_position(a.0, b.0, self.node_capacity)
    }

    #[inline]
    fn extend_capacity_for_node(&mut self, new_node_capacity: usize, exact: bool) {
        let old_node_capacity = self.node_capacity;

        if old_node_capacity >= new_node_capacity {
            return;
        }

        self.node_capacity = extend_flat_square_matrix(
            &mut self.node_adjacencies,
            old_node_capacity,
            new_node_capacity,
            exact,
        );
    }

    #[inline]
    fn remove_node(&mut self, a: NodeId) -> N {
        for (id, _) in self.nodes.iter() {
            let position = self.to_edge_position(a, id);
            if let Some(pos) = position {
                self.node_adjacencies[pos] = Default::default();
            }

            let position = self.to_edge_position(id, a);
            if let Some(pos) = position {
                self.node_adjacencies[pos] = Default::default();
            }
        }

        self.nodes.remove(a.0)
    }
}

#[inline]
fn to_flat_square_matrix_position(row: usize, column: usize, width: usize) -> usize {
    row * width + column
}

// TODO: Double check this function
#[inline]
fn extend_flat_square_matrix<T: Default>(
    node_adjacencies: &mut Vec<T>,
    old_node_capacity: usize,
    new_node_capacity: usize,
    exact: bool,
) -> usize {
    // Grow the capacity by exponential steps to avoid repeated allocations.
    // Disabled for the with_capacity constructor.
    let new_node_capacity = if exact {
        new_node_capacity
    } else {
        const MIN_CAPACITY: usize = 4;
        cmp::max(new_node_capacity.next_power_of_two(), MIN_CAPACITY)
    };

    // Optimization: when resizing the matrix this way we skip the first few grows to make
    // small matrices a bit faster to work with.

    ensure_len(node_adjacencies, new_node_capacity.pow(2));
    for c in (1..old_node_capacity).rev() {
        let pos = c * old_node_capacity;
        let new_pos = c * new_node_capacity;
        // Move the slices directly if they do not overlap with their new position
        if pos + old_node_capacity <= new_pos {
            debug_assert!(pos + old_node_capacity < node_adjacencies.len());
            debug_assert!(new_pos + old_node_capacity < node_adjacencies.len());
            let ptr = node_adjacencies.as_mut_ptr();
            // SAFETY: pos + old_node_capacity <= new_pos, so this won't overlap
            unsafe {
                let old = ptr.add(pos);
                let new = ptr.add(new_pos);
                core::ptr::swap_nonoverlapping(old, new, old_node_capacity);
            }
        } else {
            for i in (0..old_node_capacity).rev() {
                node_adjacencies.as_mut_slice().swap(pos + i, new_pos + i);
            }
        }
    }

    new_node_capacity
}

impl<N, E, S: BuildHasher, Null: Nullable<Wrapped = E>> Graph
    for MatrixGraph<N, E, S, Null, Directed>
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
    type EdgeId = DirEdgeId;
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

impl<N, E, S: BuildHasher, Null: Nullable<Wrapped = E>> DirectedGraph
    for MatrixGraph<N, E, S, Null, Directed>
where
    MatrixGraph<N, E, S, Null, Directed>: MatrixGraphExtras<N>,
{
    #[inline]
    fn density_hint(&self) -> DensityHint {
        DensityHint::Dense
    }

    // Cardinality
    #[inline]
    fn cardinality(&self) -> Cardinality {
        Cardinality {
            order: self.node_count(),
            size: self.edge_count(),
        }
    }

    #[inline]
    fn node_count(&self) -> usize {
        self.nodes.len()
    }

    #[inline]
    fn edge_count(&self) -> usize {
        self.edge_count
    }

    // Node Iteration

    fn nodes(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        self.nodes
            .iter()
            .map(|(id, data)| NodeRef::<Self> { id, data })
    }

    fn nodes_mut(&mut self) -> impl Iterator<Item = NodeMut<'_, Self>> {
        self.nodes
            .iter_mut()
            .map(|(id, data)| NodeMut::<Self> { id, data })
    }

    /// Nodes with degree 0 (no incident edges).
    #[inline]
    fn isolated_nodes(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        todo!();
        self.nodes().filter(|node| self.degree(node.id) == 0)
    }

    // Edge iteration

    fn edges(&self) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        EdgeIterator {
            edges: self.node_adjacencies.iter(),
            current_edge_tuple: (0, 0),
            node_capacity: self.node_capacity,
        }
        .map(|(source, target, data)| EdgeRef::<Self> {
            id: EdgeId::<Directed>::new_directed(source, target),
            source,
            target,
            data,
        })
    }

    fn edges_mut(&mut self) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        EdgeIteratorMut {
            edges: self.node_adjacencies.iter_mut(),
            current_edge_tuple: (0, 0),
            node_capacity: self.node_capacity,
        }
        .map(|(source, target, data)| EdgeMut::<Self> {
            id: EdgeId::<Directed>::new_directed(source, target),
            source,
            target,
            data,
        })
    }

    // Lookup
    #[inline]
    fn node(&self, id: Self::NodeId) -> Option<NodeRef<'_, Self>> {
        self.nodes
            .get(id.0)
            .map(|data| NodeRef::<Self> { id, data })
    }

    #[inline]
    fn node_mut(&mut self, id: Self::NodeId) -> Option<NodeMut<'_, Self>> {
        self.nodes
            .get_mut(id.0)
            .map(|data| NodeMut::<Self> { id, data })
    }

    #[inline]
    fn edge(&self, id: Self::EdgeId) -> Option<EdgeRef<'_, Self>> {
        let edge_index = self.to_edge_position(id.node1, id.node2)?;
        self.node_adjacencies
            .get(edge_index)?
            .as_ref()
            .map(|data| EdgeRef::<Self> {
                id,
                source: id.node1,
                target: id.node2,
                data,
            })
    }

    #[inline]
    fn edge_mut(&mut self, id: Self::EdgeId) -> Option<EdgeMut<'_, Self>> {
        let edge_index = self.to_edge_position(id.node1, id.node2)?;
        self.node_adjacencies
            .get_mut(edge_index)?
            .as_mut()
            .map(|data| EdgeMut::<Self> {
                id,
                source: id.node1,
                target: id.node2,
                data,
            })
    }

    // Degree
    #[inline]
    fn in_degree(&self, node: Self::NodeId) -> usize {
        todo!();
        self.incoming_edges(node).count()
    }

    #[inline]
    fn out_degree(&self, node: Self::NodeId) -> usize {
        todo!();
        self.outgoing_edges(node).count()
    }

    #[inline]
    fn degree(&self, node: Self::NodeId) -> usize {
        todo!();
        self.in_degree(node) + self.out_degree(node)
    }

    // Edges by direction
    #[inline]
    fn incoming_edges(&self, node: Self::NodeId) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        todo!();
        self.edges().filter(move |edge| edge.target == node)
    }

    #[inline]
    fn incoming_edges_mut(
        &mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        todo!();
        self.edges_mut().filter(move |edge| edge.target == node)
    }

    #[inline]
    fn outgoing_edges(&self, node: Self::NodeId) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        todo!();
        self.edges().filter(move |edge| edge.source == node)
    }

    #[inline]
    fn outgoing_edges_mut(
        &mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        todo!();
        self.edges_mut().filter(move |edge| edge.source == node)
    }

    #[inline]
    fn incident_edges(&self, node: Self::NodeId) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        todo!();
        self.edges()
            .filter(move |edge| edge.source == node || edge.target == node)
    }

    #[inline]
    fn incident_edges_mut(
        &mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        todo!();
        self.edges_mut()
            .filter(move |edge| edge.source == node || edge.target == node)
    }

    // Adjacency
    #[inline]
    fn predecessors(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        todo!();
        self.incoming_edges(node).map(|edge| edge.source)
    }

    #[inline]
    fn successors(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        todo!();
        self.outgoing_edges(node).map(|edge| edge.target)
    }

    #[inline]
    fn adjacencies(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        todo!();
        self.predecessors(node).chain(self.successors(node))
    }

    // Edges between nodes
    #[inline]
    fn edges_between(
        &self,
        source: Self::NodeId,
        target: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        let edge_index = self.to_edge_position(source, target);
        if let Some(edge_index) = edge_index {
            self.node_adjacencies
                .get(edge_index)
                .unwrap()
                .as_ref()
                .map(|data| EdgeRef::<Self> {
                    id: EdgeId::new_directed(source, target),
                    source,
                    target,
                    data,
                })
                .into_iter()
        } else {
            None.into_iter()
        }
    }

    #[inline]
    fn edges_between_mut(
        &mut self,
        source: Self::NodeId,
        target: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        let edge_index = self.to_edge_position(source, target);
        if let Some(edge_index) = edge_index {
            self.node_adjacencies
                .get_mut(edge_index)
                .unwrap()
                .as_mut()
                .map(|data| EdgeMut::<Self> {
                    id: EdgeId::new_directed(source, target),
                    source,
                    target,
                    data,
                })
                .into_iter()
        } else {
            None.into_iter()
        }
    }

    #[inline]
    fn edges_connecting(
        &self,
        lhs: Self::NodeId,
        rhs: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        let edge_index_one = self.to_edge_position(lhs, rhs);
        let edge_index_two = self.to_edge_position(rhs, lhs);
        match (edge_index_one, edge_index_two) {
            (Some(edge_index_one), Some(edge_index_two)) => Either::Left(
                self.node_adjacencies
                    .get(edge_index_one)
                    .unwrap()
                    .as_ref()
                    .map(|data| EdgeRef::<Self> {
                        id: EdgeId::new_directed(lhs, rhs),
                        source: lhs,
                        target: rhs,
                        data,
                    })
                    .into_iter()
                    .chain(
                        self.node_adjacencies
                            .get(edge_index_two)
                            .unwrap()
                            .as_ref()
                            .map(|data| EdgeRef::<Self> {
                                id: EdgeId::new_directed(rhs, lhs),
                                source: rhs,
                                target: lhs,
                                data,
                            })
                            .into_iter(),
                    ),
            ),
            (Some(edge_index_one), None) => Either::Right(Either::Left(
                self.node_adjacencies
                    .get(edge_index_one)
                    .unwrap()
                    .as_ref()
                    .map(|data| EdgeRef::<Self> {
                        id: EdgeId::new_directed(lhs, rhs),
                        source: lhs,
                        target: rhs,
                        data,
                    })
                    .into_iter(),
            )),
            (None, Some(edge_index_two)) => Either::Right(Either::Right(
                self.node_adjacencies
                    .get(edge_index_two)
                    .unwrap()
                    .as_ref()
                    .map(|data| EdgeRef::<Self> {
                        id: EdgeId::new_directed(rhs, lhs),
                        source: rhs,
                        target: lhs,
                        data,
                    })
                    .into_iter(),
            )),
            (None, None) => Either::Right(Either::Left(None.into_iter())),
        }
    }

    #[inline]
    fn edges_connecting_mut(
        &mut self,
        lhs: Self::NodeId,
        rhs: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        let edge_index_one = self.to_edge_position(lhs, rhs);
        let edge_index_two = self.to_edge_position(rhs, lhs);
        match (edge_index_one, edge_index_two) {
            (Some(edge_index_one), Some(edge_index_two)) => {
                if edge_index_one < edge_index_two {
                    let (first_part, second_part) =
                        self.node_adjacencies.split_at_mut(edge_index_two);
                    let first_iter = first_part
                        .get_mut(edge_index_one)
                        .unwrap()
                        .as_mut()
                        .map(|data| EdgeMut::<Self> {
                            id: EdgeId::new_directed(lhs, rhs),
                            source: lhs,
                            target: rhs,
                            data,
                        })
                        .into_iter();
                    let second_iter = second_part
                        .get_mut(0)
                        .unwrap()
                        .as_mut()
                        .map(|data| EdgeMut::<Self> {
                            id: EdgeId::new_directed(rhs, lhs),
                            source: rhs,
                            target: lhs,
                            data,
                        })
                        .into_iter();
                    Either::Left(first_iter.chain(second_iter))
                } else {
                    let (first_part, second_part) =
                        self.node_adjacencies.split_at_mut(edge_index_one);
                    let first_iter = second_part
                        .get_mut(0)
                        .unwrap()
                        .as_mut()
                        .map(|data| EdgeMut::<Self> {
                            id: EdgeId::new_directed(rhs, lhs),
                            source: rhs,
                            target: lhs,
                            data,
                        })
                        .into_iter();
                    let second_iter = first_part
                        .get_mut(edge_index_two)
                        .unwrap()
                        .as_mut()
                        .map(|data| EdgeMut::<Self> {
                            id: EdgeId::new_directed(lhs, rhs),
                            source: lhs,
                            target: rhs,
                            data,
                        })
                        .into_iter();
                    Either::Left(first_iter.chain(second_iter))
                }
            }
            (Some(edge_index_one), None) => Either::Right(Either::Left(
                self.node_adjacencies
                    .get_mut(edge_index_one)
                    .unwrap()
                    .as_mut()
                    .map(|data| EdgeMut::<Self> {
                        id: EdgeId::new_directed(lhs, rhs),
                        source: lhs,
                        target: rhs,
                        data,
                    })
                    .into_iter(),
            )),
            (None, Some(edge_index_two)) => Either::Right(Either::Right(
                self.node_adjacencies
                    .get_mut(edge_index_two)
                    .unwrap()
                    .as_mut()
                    .map(|data| EdgeMut::<Self> {
                        id: EdgeId::new_directed(rhs, lhs),
                        source: rhs,
                        target: lhs,
                        data,
                    })
                    .into_iter(),
            )),
            (None, None) => Either::Right(Either::Left(None.into_iter())),
        }
    }

    // Existence checks
    #[inline]
    fn contains_node(&self, node: Self::NodeId) -> bool {
        self.nodes.get(node.0).is_some()
    }

    #[inline]
    fn contains_edge(&self, edge: Self::EdgeId) -> bool {
        if let Some(edge_index) = self.to_edge_position(edge.node1, edge.node2) {
            self.node_adjacencies.get(edge_index).is_some()
        } else {
            false
        }
    }

    #[inline]
    fn is_adjacent(&self, source: Self::NodeId, target: Self::NodeId) -> bool {
        self.contains_edge(EdgeId::new_directed(source, target))
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.node_count() == 0
    }

    // Sources and sinks

    /// Nodes with `in_degree = 0`.
    #[inline]
    fn sources(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        todo!();
        self.nodes().filter(|node| self.in_degree(node.id) == 0)
    }

    /// Nodes with `out_degree = 0`.
    #[inline]
    fn sinks(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        todo!();
        self.nodes().filter(|node| self.out_degree(node.id) == 0)
    }
}
