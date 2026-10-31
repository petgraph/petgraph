use core::{cmp, marker::PhantomData};

use fixedbitset::FixedBitSet;
use petgraph_core::{
    edge::{EdgeMut, EdgeRef},
    graph::{
        Cardinality, DensityHint, DirectedGraph, Graph,
        storable::{DataContainer, StorableGraph, VisitContainer, containers::DefaultVec},
    },
    id::Id,
    node::{NodeMut, NodeRef},
};

use crate::{
    Directed, Either, MatrixGraph, MatrixGraphEdgeDataContainer, MatrixGraphExtras,
    MatrixGraphNodeId, MatrixGraphVisitMap, NicheWrapper, private::Sealed,
};

/// A [`MatrixGraph`] with directed edges.
pub type DiMatrixGraph<N, E, Null = Option<E>> = MatrixGraph<N, E, Directed, Null>;

/// Edge identifier type for a directed [`MatrixGraph`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiMatrixEdgeId {
    pub source: MatrixGraphNodeId,
    pub target: MatrixGraphNodeId,
    _private: PhantomData<()>,
}

impl DiMatrixEdgeId {
    #[must_use]
    pub const fn new(source: MatrixGraphNodeId, target: MatrixGraphNodeId) -> Self {
        Self {
            source,
            target,
            _private: PhantomData::<()>,
        }
    }
}

impl core::fmt::Display for DiMatrixEdgeId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "DiMatrixEdgeId({}, {})", self.source.0, self.target.0)
    }
}

impl Id for DiMatrixEdgeId {}

impl<N, E, Null: NicheWrapper<Wrapped = E>> Sealed for DiMatrixGraph<N, E, Null> {}

impl<N, E, Null: NicheWrapper<Wrapped = E>> MatrixGraphExtras<N> for DiMatrixGraph<N, E, Null> {
    #[inline]
    fn to_edge_position(
        &self,
        node_a: MatrixGraphNodeId,
        node_b: MatrixGraphNodeId,
    ) -> Option<usize> {
        if node_a.0 >= self.edge_capacity_in_nodes || node_b.0 >= self.edge_capacity_in_nodes {
            None
        } else {
            Some(self.to_edge_position_unchecked(node_a, node_b))
        }
    }

    #[inline]
    fn to_edge_position_unchecked(
        &self,
        node_a: MatrixGraphNodeId,
        node_b: MatrixGraphNodeId,
    ) -> usize {
        to_flat_square_matrix_position(node_a.0, node_b.0, self.edge_capacity_in_nodes)
    }

    #[inline]
    fn extend_capacity_for_node(&mut self, new_edge_capacity_in_nodes: usize, exact: bool) {
        let old_edge_capacity_in_nodes = self.edge_capacity_in_nodes;

        if old_edge_capacity_in_nodes >= new_edge_capacity_in_nodes {
            return;
        }

        self.edge_capacity_in_nodes = extend_flat_square_matrix(
            &mut self.flattened_edge_data,
            old_edge_capacity_in_nodes,
            new_edge_capacity_in_nodes,
            exact,
        );
    }

    #[inline]
    fn remove_node_internal(&mut self, node: MatrixGraphNodeId) -> Option<N> {
        if !self.node_data.contains(node.0) {
            return None;
        }

        // If the node capacity is larger or equal to the node id, then there cannot exist any edges
        // incident to the node. But the node might still be in the node_data (i.e. the
        // graph) as an isolated node
        if node.0 < self.edge_capacity_in_nodes {
            for (id, _) in &self.node_data {
                let position = self.to_edge_position(node, MatrixGraphNodeId(id));
                if let Some(pos) = position {
                    let entry = &mut self.flattened_edge_data[pos];
                    if !entry.is_null() {
                        *entry = Default::default();
                        self.edge_count -= 1;
                    }
                }

                let position = self.to_edge_position(MatrixGraphNodeId(id), node);
                if let Some(pos) = position {
                    let entry = &mut self.flattened_edge_data[pos];
                    if !entry.is_null() {
                        *entry = Default::default();
                        self.edge_count -= 1;
                    }
                }
            }
        }

        Some(self.node_data.remove(node.0))
    }

    #[inline]
    fn edge_id_to_source(edge_id: Self::EdgeId) -> MatrixGraphNodeId {
        edge_id.source
    }

    #[inline]
    fn edge_id_to_target(edge_id: Self::EdgeId) -> MatrixGraphNodeId {
        edge_id.target
    }

    #[inline]
    fn edge_id_from_source_target(
        source: MatrixGraphNodeId,
        target: MatrixGraphNodeId,
    ) -> Self::EdgeId {
        DiMatrixEdgeId::new(source, target)
    }
}

#[inline]
const fn to_flat_square_matrix_position(row: usize, column: usize, width: usize) -> usize {
    row * width + column
}

// TODO: Double check this function
#[inline]
fn extend_flat_square_matrix<T: Default>(
    node_adjacencies: &mut Vec<T>,
    old_edge_capacity_in_nodes: usize,
    new_edge_capacity_in_nodes: usize,
    exact: bool,
) -> usize {
    // Grow the capacity by exponential steps to avoid repeated allocations.
    // Disabled for the with_capacity constructor.
    let new_edge_capacity_in_nodes = if exact {
        new_edge_capacity_in_nodes
    } else {
        const MIN_CAPACITY: usize = 4;
        cmp::max(
            new_edge_capacity_in_nodes
                .checked_next_power_of_two()
                .unwrap_or_else(|| panic!("Matrix capacity overflow")),
            MIN_CAPACITY,
        )
    };

    // Optimization: when resizing the matrix this way we skip the first few grows to make
    // small matrices a bit faster to work with.

    node_adjacencies.resize_with(
        new_edge_capacity_in_nodes
            .checked_pow(2)
            .unwrap_or_else(|| panic!("Matrix capacity overflow")),
        || T::default(),
    );
    for current in (1..old_edge_capacity_in_nodes).rev() {
        let pos = current * old_edge_capacity_in_nodes;
        let new_pos = current
            .checked_mul(new_edge_capacity_in_nodes)
            .unwrap_or_else(|| panic!("Matrix capacity overflow"));
        // Move the slices directly if they do not overlap with their new position
        if pos + old_edge_capacity_in_nodes <= new_pos {
            debug_assert!(pos + old_edge_capacity_in_nodes < node_adjacencies.len());
            debug_assert!(new_pos + old_edge_capacity_in_nodes < node_adjacencies.len());
            let ptr = node_adjacencies.as_mut_ptr();
            // SAFETY: pos + old_edge_capacity_in_nodes <= new_pos, so this won't overlap
            unsafe {
                let old = ptr.add(pos);
                let new = ptr.add(new_pos);
                core::ptr::swap_nonoverlapping(old, new, old_edge_capacity_in_nodes);
            }
        } else {
            for i in (0..old_edge_capacity_in_nodes).rev() {
                node_adjacencies.swap(pos + i, new_pos + i);
            }
        }
    }

    new_edge_capacity_in_nodes
}

impl<N, E, Null: NicheWrapper<Wrapped = E>> Graph for DiMatrixGraph<N, E, Null> {
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
    type EdgeId = DiMatrixEdgeId;
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
    type NodeId = MatrixGraphNodeId;
}

impl<N, E, Null: NicheWrapper<Wrapped = E>> DirectedGraph for DiMatrixGraph<N, E, Null> {
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
        self.node_data.len()
    }

    #[inline]
    fn edge_count(&self) -> usize {
        self.edge_count
    }

    // Node Iteration

    #[inline]
    fn nodes(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        self.node_data.iter().map(|(id, data)| NodeRef::<Self> {
            id: MatrixGraphNodeId(id),
            data,
        })
    }

    #[inline]
    fn nodes_mut(&mut self) -> impl Iterator<Item = NodeMut<'_, Self>> {
        self.node_data.iter_mut().map(|(id, data)| NodeMut::<Self> {
            id: MatrixGraphNodeId(id),
            data,
        })
    }

    #[inline]
    fn isolated_nodes(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        self.nodes().filter(|node| self.degree(node.id) == 0)
    }

    // Edge iteration
    #[inline]
    fn edges(&self) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        EdgeIter {
            edges: self.flattened_edge_data.iter(),
            current_edge_tuple: (0, 0),
            edge_capacity_in_nodes: self.edge_capacity_in_nodes,
        }
        .map(|(source, target, data)| EdgeRef::<Self> {
            id: DiMatrixEdgeId::new(source, target),
            source,
            target,
            data,
        })
    }

    #[inline]
    fn edges_mut(&mut self) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        EdgeIterMut {
            edges: self.flattened_edge_data.iter_mut(),
            current_edge_tuple: (0, 0),
            edge_capacity_in_nodes: self.edge_capacity_in_nodes,
        }
        .map(|(source, target, data)| EdgeMut::<Self> {
            id: DiMatrixEdgeId::new(source, target),
            source,
            target,
            data,
        })
    }

    // Lookup
    #[inline]
    fn node(&self, id: Self::NodeId) -> Option<NodeRef<'_, Self>> {
        self.node_data
            .get(id.0)
            .map(|data| NodeRef::<Self> { id, data })
    }

    #[inline]
    fn node_mut(&mut self, id: Self::NodeId) -> Option<NodeMut<'_, Self>> {
        self.node_data
            .get_mut(id.0)
            .map(|data| NodeMut::<Self> { id, data })
    }

    #[inline]
    fn edge(&self, id: Self::EdgeId) -> Option<EdgeRef<'_, Self>> {
        let edge_index = self.to_edge_position(id.source, id.target)?;
        self.flattened_edge_data
            .get(edge_index)?
            .as_ref()
            .map(|data| EdgeRef::<Self> {
                id,
                source: id.source,
                target: id.target,
                data,
            })
    }

    #[inline]
    fn edge_mut(&mut self, id: Self::EdgeId) -> Option<EdgeMut<'_, Self>> {
        let edge_index = self.to_edge_position(id.source, id.target)?;
        self.flattened_edge_data
            .get_mut(edge_index)?
            .as_mut()
            .map(|data| EdgeMut::<Self> {
                id,
                source: id.source,
                target: id.target,
                data,
            })
    }

    // Degree
    #[inline]
    fn in_degree(&self, node: Self::NodeId) -> usize {
        incoming_neighbor_iter(&self.flattened_edge_data, node, self.edge_capacity_in_nodes).count()
    }

    #[inline]
    fn out_degree(&self, node: Self::NodeId) -> usize {
        outgoing_neighbor_iter(&self.flattened_edge_data, node, self.edge_capacity_in_nodes).count()
    }

    #[inline]
    fn degree(&self, node: Self::NodeId) -> usize {
        self.in_degree(node) + self.out_degree(node)
    }

    // Edges by direction
    #[inline]
    fn incoming_edges(&self, node: Self::NodeId) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        incoming_neighbor_iter(&self.flattened_edge_data, node, self.edge_capacity_in_nodes).map(
            move |(source, data)| EdgeRef::<Self> {
                id: DiMatrixEdgeId::new(source, node),
                source,
                target: node,
                data,
            },
        )
    }

    #[inline]
    fn incoming_edges_mut(
        &mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        incoming_neighbor_iter_mut(
            &mut self.flattened_edge_data,
            node,
            self.edge_capacity_in_nodes,
        )
        .map(move |(source, data)| EdgeMut::<Self> {
            id: DiMatrixEdgeId::new(source, node),
            source,
            target: node,
            data,
        })
    }

    #[inline]
    fn outgoing_edges(&self, node: Self::NodeId) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        outgoing_neighbor_iter(&self.flattened_edge_data, node, self.edge_capacity_in_nodes).map(
            move |(target, data)| EdgeRef::<Self> {
                id: DiMatrixEdgeId::new(node, target),
                source: node,
                target,
                data,
            },
        )
    }

    #[inline]
    fn outgoing_edges_mut(
        &mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        outgoing_neighbor_iter_mut(
            &mut self.flattened_edge_data,
            node,
            self.edge_capacity_in_nodes,
        )
        .map(move |(target, data)| EdgeMut::<Self> {
            id: DiMatrixEdgeId::new(node, target),
            source: node,
            target,
            data,
        })
    }

    #[inline]
    fn incident_edges(&self, node: Self::NodeId) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        neighbor_iter(&self.flattened_edge_data, node, self.edge_capacity_in_nodes).map(
            move |(source, target, data)| EdgeRef::<Self> {
                id: DiMatrixEdgeId::new(source, target),
                source,
                target,
                data,
            },
        )
    }

    #[inline]
    fn incident_edges_mut(
        &mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        neighbor_iter_mut(
            &mut self.flattened_edge_data,
            node,
            self.edge_capacity_in_nodes,
        )
        .map(move |(source, target, data)| EdgeMut::<Self> {
            id: DiMatrixEdgeId::new(source, target),
            source,
            target,
            data,
        })
    }

    // Adjacency
    #[inline]
    fn predecessors(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        incoming_neighbor_iter(&self.flattened_edge_data, node, self.edge_capacity_in_nodes)
            .map(|(source, _)| source)
    }

    #[inline]
    fn successors(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        outgoing_neighbor_iter(&self.flattened_edge_data, node, self.edge_capacity_in_nodes)
            .map(|(target, _)| target)
    }

    #[inline]
    fn adjacencies(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        neighbor_iter(&self.flattened_edge_data, node, self.edge_capacity_in_nodes)
            .map(move |(node_a, node_b, _)| if node == node_a { node_b } else { node_a })
    }

    // Edges between nodes
    #[inline]
    fn edges_between(
        &self,
        source: Self::NodeId,
        target: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        let edge_index = self.to_edge_position(source, target);
        edge_index.map_or_else(
            || None.into_iter(),
            |edge_index| {
                self.flattened_edge_data
                    .get(edge_index)
                    .unwrap()
                    .as_ref()
                    .map(|data| EdgeRef::<Self> {
                        id: DiMatrixEdgeId::new(source, target),
                        source,
                        target,
                        data,
                    })
                    .into_iter()
            },
        )
    }

    #[inline]
    fn edges_between_mut(
        &mut self,
        source: Self::NodeId,
        target: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        let edge_index = self.to_edge_position(source, target);
        if let Some(edge_index) = edge_index {
            self.flattened_edge_data
                .get_mut(edge_index)
                .unwrap()
                .as_mut()
                .map(|data| EdgeMut::<Self> {
                    id: DiMatrixEdgeId::new(source, target),
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
                self.flattened_edge_data
                    .get(edge_index_one)
                    .unwrap()
                    .as_ref()
                    .map(|data| EdgeRef::<Self> {
                        id: DiMatrixEdgeId::new(lhs, rhs),
                        source: lhs,
                        target: rhs,
                        data,
                    })
                    .into_iter()
                    .chain(
                        self.flattened_edge_data
                            .get(edge_index_two)
                            .unwrap()
                            .as_ref()
                            .map(|data| EdgeRef::<Self> {
                                id: DiMatrixEdgeId::new(rhs, lhs),
                                source: rhs,
                                target: lhs,
                                data,
                            }),
                    ),
            ),
            (Some(edge_index_one), None) => Either::Right(Either::Left(
                self.flattened_edge_data
                    .get(edge_index_one)
                    .unwrap()
                    .as_ref()
                    .map(|data| EdgeRef::<Self> {
                        id: DiMatrixEdgeId::new(lhs, rhs),
                        source: lhs,
                        target: rhs,
                        data,
                    })
                    .into_iter(),
            )),
            (None, Some(edge_index_two)) => Either::Right(Either::Right(
                self.flattened_edge_data
                    .get(edge_index_two)
                    .unwrap()
                    .as_ref()
                    .map(|data| EdgeRef::<Self> {
                        id: DiMatrixEdgeId::new(rhs, lhs),
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
        if lhs.0 >= self.edge_capacity_in_nodes || rhs.0 >= self.edge_capacity_in_nodes {
            return Either::Left(None.into_iter());
        }

        // We use to_edge_position_unchecked from now on, since we just checked that lhs and rhs are
        // within bounds.
        if rhs == lhs {
            let edge_index = self.to_edge_position_unchecked(lhs, rhs);

            return Either::Right(Either::Left(
                self.flattened_edge_data
                    .get_mut(edge_index)
                    .unwrap()
                    .as_mut()
                    .map(|data| EdgeMut::<Self> {
                        id: DiMatrixEdgeId::new(lhs, rhs),
                        source: lhs,
                        target: rhs,
                        data,
                    })
                    .into_iter(),
            ));
        }

        let (node_a, node_b) = if lhs < rhs { (lhs, rhs) } else { (rhs, lhs) };

        let edge_index_one = self.to_edge_position_unchecked(node_a, node_b);
        let edge_index_two = self.to_edge_position_unchecked(node_b, node_a);

        let (first_part, second_part) = self.flattened_edge_data.split_at_mut(edge_index_two);
        let first_iter = first_part
            .get_mut(edge_index_one)
            .unwrap()
            .as_mut()
            .map(|data| EdgeMut::<Self> {
                id: DiMatrixEdgeId::new(node_a, node_b),
                source: node_a,
                target: node_b,
                data,
            })
            .into_iter();
        let second_iter = second_part
            .get_mut(0)
            .unwrap()
            .as_mut()
            .map(|data| EdgeMut::<Self> {
                id: DiMatrixEdgeId::new(node_b, node_a),
                source: node_b,
                target: node_a,
                data,
            })
            .into_iter();
        Either::Right(Either::Right(first_iter.chain(second_iter)))
    }

    // Existence checks
    #[inline]
    fn contains_node(&self, node: Self::NodeId) -> bool {
        self.node_data.contains(node.0)
    }

    #[inline]
    fn contains_edge(&self, edge: Self::EdgeId) -> bool {
        self.to_edge_position(edge.source, edge.target)
            .is_some_and(|edge_index| {
                self.flattened_edge_data
                    .get(edge_index)
                    .is_some_and(|data| !data.is_null())
            })
    }

    #[inline]
    fn is_adjacent(&self, source: Self::NodeId, target: Self::NodeId) -> bool {
        self.contains_edge(DiMatrixEdgeId::new(source, target))
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.node_count() == 0
    }

    // Sources and sinks
    #[inline]
    fn sources(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        self.nodes().filter(|node| self.in_degree(node.id) == 0)
    }

    #[inline]
    fn sinks(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        self.nodes().filter(|node| self.out_degree(node.id) == 0)
    }
}

/// An iterator over the edges of a directed graph which yields the source and target of each edge
/// along with a reference to the edge data.
struct EdgeIter<'a, It: Iterator<Item = &'a Null>, Null: NicheWrapper + 'a> {
    edges: It,
    current_edge_tuple: (usize, usize),
    edge_capacity_in_nodes: usize,
}

impl<'a, It: Iterator<Item = &'a Null>, Null: NicheWrapper> Iterator for EdgeIter<'a, It, Null> {
    type Item = (MatrixGraphNodeId, MatrixGraphNodeId, &'a Null::Wrapped);

    fn next(&mut self) -> Option<Self::Item> {
        for edge in self.edges.by_ref() {
            let current_edge_tuple = self.current_edge_tuple;
            self.current_edge_tuple.1 += 1;
            if self.current_edge_tuple.1 == self.edge_capacity_in_nodes {
                self.current_edge_tuple.0 += 1;
                self.current_edge_tuple.1 = 0;
            }
            if !edge.is_null() {
                return Some((
                    MatrixGraphNodeId(current_edge_tuple.0),
                    MatrixGraphNodeId(current_edge_tuple.1),
                    edge.as_ref().unwrap(),
                ));
            }
        }
        None
    }
}

/// An iterator over the edges of a directed graph which yields the source and target of each edge
/// along with a mutable reference to the edge data.
struct EdgeIterMut<'a, It: Iterator<Item = &'a mut Null>, Null: NicheWrapper + 'a> {
    edges: It,
    current_edge_tuple: (usize, usize),
    edge_capacity_in_nodes: usize,
}

impl<'a, It: Iterator<Item = &'a mut Null>, Null: NicheWrapper> Iterator
    for EdgeIterMut<'a, It, Null>
{
    type Item = (MatrixGraphNodeId, MatrixGraphNodeId, &'a mut Null::Wrapped);

    fn next(&mut self) -> Option<Self::Item> {
        for edge in self.edges.by_ref() {
            let current_edge_tuple = self.current_edge_tuple;
            self.current_edge_tuple.1 += 1;
            if self.current_edge_tuple.1 == self.edge_capacity_in_nodes {
                self.current_edge_tuple.0 += 1;
                self.current_edge_tuple.1 = 0;
            }
            if !edge.is_null() {
                return Some((
                    MatrixGraphNodeId(current_edge_tuple.0),
                    MatrixGraphNodeId(current_edge_tuple.1),
                    edge.as_mut().unwrap(),
                ));
            }
        }
        None
    }
}

/// Returns an iterator over the neighbors of a node which correspond to incoming edges with the
/// associated edge data.
#[inline]
fn incoming_neighbor_iter<'a, Null: NicheWrapper + 'a>(
    node_adjacencies: &'a [Null],
    target: MatrixGraphNodeId,
    edge_capacity_in_nodes: usize,
) -> impl Iterator<Item = (MatrixGraphNodeId, &'a <Null as NicheWrapper>::Wrapped)> {
    let start_index = target.0;
    if edge_capacity_in_nodes == 0 || start_index >= edge_capacity_in_nodes {
        return Either::Left(None.into_iter());
    }
    Either::Right(
        node_adjacencies
            .iter()
            .skip(start_index)
            .step_by(edge_capacity_in_nodes)
            .take(edge_capacity_in_nodes)
            .enumerate()
            .filter_map(move |(i, adj)| adj.as_ref().map(|data| (MatrixGraphNodeId(i), data))),
    )
}

/// Returns an iterator over the neighbors of a node which correspond to incoming edges with the
/// associated edge data.
#[inline]
fn incoming_neighbor_iter_mut<'b, Null: NicheWrapper + 'b>(
    node_adjacencies: &'b mut [Null],
    target: MatrixGraphNodeId,
    edge_capacity_in_nodes: usize,
) -> impl Iterator<Item = (MatrixGraphNodeId, &'b mut <Null as NicheWrapper>::Wrapped)> {
    let start_index = target.0;
    if edge_capacity_in_nodes == 0 || start_index >= edge_capacity_in_nodes {
        return Either::Left(None.into_iter());
    }
    Either::Right(
        node_adjacencies
            .iter_mut()
            .skip(start_index)
            .step_by(edge_capacity_in_nodes)
            .take(edge_capacity_in_nodes)
            .enumerate()
            .filter_map(move |(i, adj)| adj.as_mut().map(|data| (MatrixGraphNodeId(i), data))),
    )
}

/// Returns an iterator over the neighbors of a node which correspond to outgoing edges with the
/// associated edge data.
#[inline]
fn outgoing_neighbor_iter<'a, Null: NicheWrapper + 'a>(
    node_adjacencies: &'a [Null],
    source: MatrixGraphNodeId,
    edge_capacity_in_nodes: usize,
) -> impl Iterator<Item = (MatrixGraphNodeId, &'a <Null as NicheWrapper>::Wrapped)> {
    let start_index = source.0 * edge_capacity_in_nodes;
    if edge_capacity_in_nodes == 0 || source.0 >= edge_capacity_in_nodes {
        return Either::Left(None.into_iter());
    }
    Either::Right(
        node_adjacencies
            .iter()
            .skip(start_index)
            .take(edge_capacity_in_nodes)
            .enumerate()
            .filter_map(move |(i, adj)| adj.as_ref().map(|data| (MatrixGraphNodeId(i), data))),
    )
}

/// Returns an iterator over the neighbors of a node which correspond to outgoing edges with the
/// associated edge data.
#[inline]
fn outgoing_neighbor_iter_mut<'b, Null: NicheWrapper + 'b>(
    node_adjacencies: &'b mut [Null],
    source: MatrixGraphNodeId,
    edge_capacity_in_nodes: usize,
) -> impl Iterator<Item = (MatrixGraphNodeId, &'b mut <Null as NicheWrapper>::Wrapped)> {
    let start_index = source.0 * edge_capacity_in_nodes;
    if edge_capacity_in_nodes == 0 || source.0 >= edge_capacity_in_nodes {
        return Either::Left(None.into_iter());
    }
    Either::Right(
        node_adjacencies
            .iter_mut()
            .skip(start_index)
            .take(edge_capacity_in_nodes)
            .enumerate()
            .filter_map(move |(i, adj)| adj.as_mut().map(|data| (MatrixGraphNodeId(i), data))),
    )
}

/// Returns an iterator over the neighbors of a node with the edge data.
///
/// The item in the iterator is a tuple of the form `(source, target, edge_data)` where either
/// `source` or `target` (or both) is the given node and the other is the neighbor.
#[inline]
fn neighbor_iter<'a, Null: NicheWrapper + 'a>(
    node_adjacencies: &'a [Null],
    node: MatrixGraphNodeId,
    edge_capacity_in_nodes: usize,
) -> impl Iterator<
    Item = (
        MatrixGraphNodeId,
        MatrixGraphNodeId,
        &'a <Null as NicheWrapper>::Wrapped,
    ),
> {
    // If the node index is out of bounds, return an empty iterator.
    if edge_capacity_in_nodes == 0 || node.0 >= edge_capacity_in_nodes {
        return Either::Left(None.into_iter());
    }

    let node_row_start = node.0 * edge_capacity_in_nodes;
    let (rows_before, remaining) = node_adjacencies.split_at(node_row_start);
    let (node_row, rows_after) = remaining.split_at(edge_capacity_in_nodes);

    let incoming_before = rows_before
        .iter()
        .skip(node.0)
        .step_by(edge_capacity_in_nodes)
        .enumerate()
        .filter_map(move |(source, adjacency)| {
            adjacency
                .as_ref()
                .map(|data| (MatrixGraphNodeId(source), node, data))
        });

    let outgoing = node_row
        .iter()
        .enumerate()
        .filter_map(move |(target, adjacency)| {
            adjacency
                .as_ref()
                .map(|data| (node, MatrixGraphNodeId(target), data))
        });

    let incoming_after = rows_after
        .iter()
        .skip(node.0)
        .step_by(edge_capacity_in_nodes)
        .enumerate()
        .filter_map(move |(offset, adjacency)| {
            let source = MatrixGraphNodeId(node.0 + 1 + offset);
            adjacency.as_ref().map(|data| (source, node, data))
        });

    Either::Right(incoming_before.chain(outgoing).chain(incoming_after))
}

/// Returns an iterator over the neighbors of a node with a mutable reference to the edge data.
///
/// The item in the iterator is a tuple of the form `(source, target, edge_data)` where either
/// `source` or `target` (or both) is the given node and the other is the neighbor.
#[inline]
fn neighbor_iter_mut<'a, Null: NicheWrapper + 'a>(
    node_adjacencies: &'a mut [Null],
    node: MatrixGraphNodeId,
    edge_capacity_in_nodes: usize,
) -> impl Iterator<
    Item = (
        MatrixGraphNodeId,
        MatrixGraphNodeId,
        &'a mut <Null as NicheWrapper>::Wrapped,
    ),
> {
    // If the node index is out of bounds, return an empty iterator.
    if edge_capacity_in_nodes == 0 || node.0 >= edge_capacity_in_nodes {
        return Either::Left(None.into_iter());
    }

    let node_row_start = node.0 * edge_capacity_in_nodes;
    let (rows_before, remaining) = node_adjacencies.split_at_mut(node_row_start);
    let (node_row, rows_after) = remaining.split_at_mut(edge_capacity_in_nodes);

    let incoming_before = rows_before
        .iter_mut()
        .skip(node.0)
        .step_by(edge_capacity_in_nodes)
        .enumerate()
        .filter_map(move |(source, adjacency)| {
            adjacency
                .as_mut()
                .map(|data| (MatrixGraphNodeId(source), node, data))
        });

    let outgoing = node_row
        .iter_mut()
        .enumerate()
        .filter_map(move |(target, adjacency)| {
            adjacency
                .as_mut()
                .map(|data| (node, MatrixGraphNodeId(target), data))
        });

    let incoming_after = rows_after
        .iter_mut()
        .skip(node.0)
        .step_by(edge_capacity_in_nodes)
        .enumerate()
        .filter_map(move |(offset, adjacency)| {
            let source = MatrixGraphNodeId(node.0 + 1 + offset);
            adjacency.as_mut().map(|data| (source, node, data))
        });

    Either::Right(incoming_before.chain(outgoing).chain(incoming_after))
}

impl VisitContainer<DiMatrixEdgeId> for MatrixGraphVisitMap<DiMatrixEdgeId> {
    fn mark_visited(&mut self, id: DiMatrixEdgeId) {
        let index =
            to_flat_square_matrix_position(id.source.0, id.target.0, self.edge_capacity_in_nodes);
        self.visit_map.set(index, true);
    }

    fn mark_unvisited(&mut self, id: DiMatrixEdgeId) {
        let index =
            to_flat_square_matrix_position(id.source.0, id.target.0, self.edge_capacity_in_nodes);
        self.visit_map.set(index, false);
    }

    fn is_visited(&self, id: DiMatrixEdgeId) -> bool {
        let index =
            to_flat_square_matrix_position(id.source.0, id.target.0, self.edge_capacity_in_nodes);
        self.visit_map.contains(index) && self.visit_map[index]
    }

    fn clear(&mut self) {
        self.visit_map.clear();
    }
}

impl<Data: Clone> DataContainer<DiMatrixEdgeId, Data>
    for MatrixGraphEdgeDataContainer<DiMatrixEdgeId, Data>
{
    fn get(&self, id: DiMatrixEdgeId) -> &Data {
        let index =
            to_flat_square_matrix_position(id.source.0, id.target.0, self.edge_capacity_in_nodes);
        &self.data[index]
    }

    fn get_mut(&mut self, id: DiMatrixEdgeId) -> &mut Data {
        let index =
            to_flat_square_matrix_position(id.source.0, id.target.0, self.edge_capacity_in_nodes);
        &mut self.data[index]
    }

    fn insert(&mut self, id: DiMatrixEdgeId, data: Data) {
        let index =
            to_flat_square_matrix_position(id.source.0, id.target.0, self.edge_capacity_in_nodes);
        self.data[index] = data;
    }

    fn remove(&mut self, id: DiMatrixEdgeId) {
        let index =
            to_flat_square_matrix_position(id.source.0, id.target.0, self.edge_capacity_in_nodes);
        self.data[index] = self.default_value.clone();
    }

    fn clear(&mut self) {
        self.data.fill(self.default_value.clone());
    }
}

impl<N, E, Null: NicheWrapper<Wrapped = E>> StorableGraph for MatrixGraph<N, E, Directed, Null> {
    type EdgeDataContainer<Data: Clone> = MatrixGraphEdgeDataContainer<DiMatrixEdgeId, Data>;
    type EdgeVisitContainer = MatrixGraphVisitMap<DiMatrixEdgeId>;
    type NodeDataContainer<Data: Clone> = DefaultVec<Data>;
    type NodeVisitContainer = FixedBitSet;

    fn node_visit_container(&self) -> Self::NodeVisitContainer {
        FixedBitSet::with_capacity(self.node_data.capacity())
    }

    fn node_data_container<Data: Clone>(
        &self,
        default_value: Data,
    ) -> Self::NodeDataContainer<Data> {
        DefaultVec::new_with_capacity(default_value, self.node_data.capacity())
    }

    fn edge_visit_container(&self) -> Self::EdgeVisitContainer {
        MatrixGraphVisitMap {
            visit_map: FixedBitSet::with_capacity(self.flattened_edge_data.len()),
            edge_capacity_in_nodes: self.edge_capacity_in_nodes,
            phantom: PhantomData,
        }
    }

    fn edge_data_container<Data: Clone>(
        &self,
        default_value: Data,
    ) -> Self::EdgeDataContainer<Data> {
        MatrixGraphEdgeDataContainer {
            data: vec![default_value.clone(); self.flattened_edge_data.len()],
            edge_capacity_in_nodes: self.edge_capacity_in_nodes,
            default_value,
            phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use petgraph_core::{
        test_directed_editable_graph, test_directed_graph, test_directed_storable_graph,
    };

    use super::{super::*, *};

    #[test]
    fn test_default() {
        let graph = MatrixGraph::<i32, i32>::default();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_with_capacity() {
        let graph = MatrixGraph::<i32, i32>::with_capacity(10);
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_new() {
        let graph = MatrixGraph::<i32, i32>::new_directed();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_clear() {
        let mut graph = MatrixGraph::<_, _>::new_directed();
        let node_a = graph.add_node('a').unwrap();
        let node_b = graph.add_node('b').unwrap();
        let node_c = graph.add_node('c').unwrap();
        assert_eq!(graph.node_count(), 3);

        graph.add_edge(node_a, node_b, ()).unwrap();
        graph.add_edge(node_b, node_c, ()).unwrap();
        graph.add_edge(node_c, node_a, ()).unwrap();
        assert_eq!(graph.edge_count(), 3);

        graph.clear();

        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);

        let node_a = graph.add_node('a').unwrap();
        let node_b = graph.add_node('b').unwrap();
        let node_c = graph.add_node('c').unwrap();
        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 0);

        assert_eq!(graph.adjacencies(node_a).collect::<Vec<_>>(), vec![]);
        assert_eq!(graph.adjacencies(node_b).collect::<Vec<_>>(), vec![]);
        assert_eq!(graph.adjacencies(node_c).collect::<Vec<_>>(), vec![]);

        graph.clear();

        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    #[should_panic = "capacity overflow"]
    fn test_overflow_vec() {
        let _ = MatrixGraph::<(), (), Directed, Option<()>>::with_capacity(usize::MAX);
    }

    #[test]
    #[should_panic = "Matrix capacity overflow"]
    fn test_overflow_node() {
        let mut graph = MatrixGraph::<(), (), Directed, Option<()>>::default();
        graph.extend_capacity_for_node(usize::MAX, false);
    }

    #[test]
    #[should_panic = "Matrix capacity overflow"]
    fn test_overflow_edge() {
        let mut graph = MatrixGraph::<(), (), Directed, Option<()>>::default();
        graph.extend_capacity_for_edge(MatrixGraphNodeId(usize::MAX), MatrixGraphNodeId(0));
    }

    #[test]
    fn test_alternative_null_type() {
        let mut graph: MatrixGraph<(), i32, Directed, NotZero<i32>> = MatrixGraph::default();

        let node_a = graph.add_node(()).unwrap();
        let node_b = graph.add_node(()).unwrap();

        assert!(!graph.contains_edge(DiMatrixEdgeId::new(node_a, node_b)));
        assert_eq!(graph.edge_count(), 0);

        graph.add_edge(node_a, node_b, 12).unwrap();

        assert!(graph.contains_edge(DiMatrixEdgeId::new(node_a, node_b)));
        assert_eq!(graph.edge_count(), 1);
        assert_eq!(
            graph
                .edge(DiMatrixEdgeId::new(node_a, node_b))
                .unwrap()
                .data,
            &12
        );

        graph
            .remove_edge(DiMatrixEdgeId::new(node_a, node_b))
            .unwrap();

        assert!(!graph.contains_edge(DiMatrixEdgeId::new(node_a, node_b)));
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    #[should_panic = "assertion failed: value != T::default()"]
    fn test_not_zero_asserted() {
        let mut graph: MatrixGraph<(), i32, Directed, NotZero<i32>> = MatrixGraph::default();

        let node_a = graph.add_node(()).unwrap();
        let node_b = graph.add_node(()).unwrap();

        graph.add_edge(node_a, node_b, 0).unwrap();
    }

    #[test]
    fn test_not_zero_float() {
        let mut graph: MatrixGraph<(), f32, Directed, NotZero<f32>> = MatrixGraph::default();

        let node_a = graph.add_node(()).unwrap();
        let node_b = graph.add_node(()).unwrap();

        assert!(!graph.contains_edge(DiMatrixEdgeId::new(node_a, node_b)));
        assert_eq!(graph.edge_count(), 0);

        let val = 12.0;
        graph.add_edge(node_a, node_b, val).unwrap();

        assert!(graph.contains_edge(DiMatrixEdgeId::new(node_a, node_b)));
        assert_eq!(graph.edge_count(), 1);
        assert!(
            (graph
                .edge(DiMatrixEdgeId::new(node_a, node_b))
                .unwrap()
                .data
                - val)
                .abs()
                <= 0.0
        );

        graph
            .remove_edge(DiMatrixEdgeId::new(node_a, node_b))
            .unwrap();

        assert!(!graph.contains_edge(DiMatrixEdgeId::new(node_a, node_b)));
        assert_eq!(graph.edge_count(), 0);
    }

    test_directed_graph!(MatrixGraph::<(), (), Directed, Option<()>>::new_directed,);

    test_directed_editable_graph!(MatrixGraph::<(), (), Directed, Option<()>>::new_directed,);

    test_directed_storable_graph!(MatrixGraph::<(), (), Directed, Option<()>>::new_directed,);
}
