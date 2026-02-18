use core::{cmp, hash::BuildHasher, marker::PhantomData};

use petgraph_core::{
    edge::{EdgeMut, EdgeRef},
    graph::{Cardinality, DensityHint, DirectedGraph, Graph},
    id::Id,
    node::{NodeMut, NodeRef},
};

use crate::{
    Directed, EdgeId, Either, MatrixGraph, MatrixGraphExtras, NicheWrapper, NodeId, ensure_len,
    private::Sealed,
};

pub type DirEdgeId = EdgeId<Directed>;

impl DirEdgeId {
    #[must_use]
    pub const fn new_directed(source: NodeId, target: NodeId) -> Self {
        Self {
            node_a: source,
            node_b: target,
            direction: PhantomData,
        }
    }
}

impl PartialEq for DirEdgeId {
    fn eq(&self, other: &Self) -> bool {
        self.node_a == other.node_a && self.node_b == other.node_b
    }
}

impl Eq for DirEdgeId {}

impl PartialOrd for DirEdgeId {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DirEdgeId {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.node_a.cmp(&other.node_a) {
            cmp::Ordering::Equal => self.node_b.cmp(&other.node_b),
            non_eq => non_eq,
        }
    }
}

impl Id for DirEdgeId {}

impl<N, E, S, Null: NicheWrapper<Wrapped = E>> Sealed for MatrixGraph<N, E, S, Null, Directed> {}

impl<N, E, S: BuildHasher, Null: NicheWrapper<Wrapped = E>> MatrixGraphExtras<N>
    for MatrixGraph<N, E, S, Null, Directed>
{
    #[inline]
    fn to_edge_position(&self, node_a: NodeId, node_b: NodeId) -> Option<usize> {
        if node_a.0 >= self.node_capacity || node_b.0 >= self.node_capacity {
            None
        } else {
            Some(self.to_edge_position_unchecked(node_a, node_b))
        }
    }

    #[inline]
    fn to_edge_position_unchecked(&self, node_a: NodeId, node_b: NodeId) -> usize {
        to_flat_square_matrix_position(node_a.0, node_b.0, self.node_capacity)
    }

    #[inline]
    fn extend_capacity_for_node(&mut self, new_node_capacity: usize, exact: bool) {
        let old_node_capacity = self.node_capacity;

        if old_node_capacity >= new_node_capacity {
            return;
        }

        self.node_capacity = extend_flat_square_matrix(
            &mut self.flattened_edge_data,
            old_node_capacity,
            new_node_capacity,
            exact,
        );
    }

    #[inline]
    fn remove_node(&mut self, node: NodeId) -> N {
        for (id, _) in self.node_data.iter() {
            let position = self.to_edge_position(node, NodeId(id));
            if let Some(pos) = position {
                self.flattened_edge_data[pos] = Default::default();
            }

            let position = self.to_edge_position(NodeId(id), node);
            if let Some(pos) = position {
                self.flattened_edge_data[pos] = Default::default();
            }
        }

        self.node_data.remove(node.0)
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
    for current in (1..old_node_capacity).rev() {
        let pos = current * old_node_capacity;
        let new_pos = current * new_node_capacity;
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

impl<N, E, S: BuildHasher, Null: NicheWrapper<Wrapped = E>> Graph
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

impl<N, E, S: BuildHasher, Null: NicheWrapper<Wrapped = E>> DirectedGraph
    for MatrixGraph<N, E, S, Null, Directed>
where
    Self: MatrixGraphExtras<N>,
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
            id: NodeId(id),
            data,
        })
    }

    #[inline]
    fn nodes_mut(&mut self) -> impl Iterator<Item = NodeMut<'_, Self>> {
        self.node_data.iter_mut().map(|(id, data)| NodeMut::<Self> {
            id: NodeId(id),
            data,
        })
    }

    /// Nodes with degree 0 (no incident edges).
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
            node_capacity: self.node_capacity,
        }
        .map(|(source, target, data)| EdgeRef::<Self> {
            id: DirEdgeId::new_directed(source, target),
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
            node_capacity: self.node_capacity,
        }
        .map(|(source, target, data)| EdgeMut::<Self> {
            id: DirEdgeId::new_directed(source, target),
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
        let edge_index = self.to_edge_position(id.node_a, id.node_b)?;
        self.flattened_edge_data
            .get(edge_index)?
            .as_ref()
            .map(|data| EdgeRef::<Self> {
                id,
                source: id.node_a,
                target: id.node_b,
                data,
            })
    }

    #[inline]
    fn edge_mut(&mut self, id: Self::EdgeId) -> Option<EdgeMut<'_, Self>> {
        let edge_index = self.to_edge_position(id.node_a, id.node_b)?;
        self.flattened_edge_data
            .get_mut(edge_index)?
            .as_mut()
            .map(|data| EdgeMut::<Self> {
                id,
                source: id.node_a,
                target: id.node_b,
                data,
            })
    }

    // Degree
    #[inline]
    fn in_degree(&self, node: Self::NodeId) -> usize {
        incoming_neighbor_iter(&self.flattened_edge_data, node, self.node_capacity).count()
    }

    #[inline]
    fn out_degree(&self, node: Self::NodeId) -> usize {
        outgoing_neighbor_iter(&self.flattened_edge_data, node, self.node_capacity).count()
    }

    #[inline]
    fn degree(&self, node: Self::NodeId) -> usize {
        self.in_degree(node) + self.out_degree(node)
    }

    // Edges by direction
    #[inline]
    fn incoming_edges(&self, node: Self::NodeId) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        incoming_neighbor_iter(&self.flattened_edge_data, node, self.node_capacity).map(
            move |(source, data)| EdgeRef::<Self> {
                id: EdgeId::new_directed(source, node),
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
        incoming_neighbor_iter_mut(&mut self.flattened_edge_data, node, self.node_capacity).map(
            move |(source, data)| EdgeMut::<Self> {
                id: EdgeId::new_directed(source, node),
                source,
                target: node,
                data,
            },
        )
    }

    #[inline]
    fn outgoing_edges(&self, node: Self::NodeId) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        outgoing_neighbor_iter(&self.flattened_edge_data, node, self.node_capacity).map(
            move |(target, data)| EdgeRef::<Self> {
                id: EdgeId::new_directed(node, target),
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
        outgoing_neighbor_iter_mut(&mut self.flattened_edge_data, node, self.node_capacity).map(
            move |(target, data)| EdgeMut::<Self> {
                id: EdgeId::new_directed(node, target),
                source: node,
                target,
                data,
            },
        )
    }

    #[inline]
    fn incident_edges(&self, node: Self::NodeId) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        neighbor_iter(&self.flattened_edge_data, node, self.node_capacity).map(
            move |(source, target, data)| EdgeRef::<Self> {
                id: EdgeId::new_directed(source, target),
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
        neighbor_iter_mut(&mut self.flattened_edge_data, node, self.node_capacity).map(
            move |(source, target, data)| EdgeMut::<Self> {
                id: EdgeId::new_directed(source, target),
                source,
                target,
                data,
            },
        )
    }

    // Adjacency
    #[inline]
    fn predecessors(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        incoming_neighbor_iter(&self.flattened_edge_data, node, self.node_capacity)
            .map(|(source, _)| source)
    }

    #[inline]
    fn successors(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        outgoing_neighbor_iter(&self.flattened_edge_data, node, self.node_capacity)
            .map(|(target, _)| target)
    }

    #[inline]
    fn adjacencies(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        neighbor_iter(&self.flattened_edge_data, node, self.node_capacity)
            .map(|(neighbor, _, _)| neighbor)
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
                        id: EdgeId::new_directed(source, target),
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
                self.flattened_edge_data
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
                        self.flattened_edge_data
                            .get(edge_index_two)
                            .unwrap()
                            .as_ref()
                            .map(|data| EdgeRef::<Self> {
                                id: EdgeId::new_directed(rhs, lhs),
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
                        id: EdgeId::new_directed(lhs, rhs),
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
                        self.flattened_edge_data.split_at_mut(edge_index_two);
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
                        self.flattened_edge_data.split_at_mut(edge_index_one);
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
                self.flattened_edge_data
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
                self.flattened_edge_data
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
        self.node_data.get(node.0).is_some()
    }

    #[inline]
    fn contains_edge(&self, edge: Self::EdgeId) -> bool {
        self.to_edge_position(edge.node_a, edge.node_b)
            .is_some_and(|edge_index| {
                self.flattened_edge_data
                    .get(edge_index)
                    .is_some_and(|data| !data.is_null())
            })
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
        self.nodes().filter(|node| self.in_degree(node.id) == 0)
    }

    /// Nodes with `out_degree = 0`.
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
    node_capacity: usize,
}

impl<'a, It: Iterator<Item = &'a Null>, Null: NicheWrapper> Iterator for EdgeIter<'a, It, Null> {
    type Item = (NodeId, NodeId, &'a Null::Wrapped);

    fn next(&mut self) -> Option<Self::Item> {
        for edge in self.edges.by_ref() {
            let current_edge_tuple = self.current_edge_tuple;
            self.current_edge_tuple.1 += 1;
            if self.current_edge_tuple.1 == self.node_capacity {
                self.current_edge_tuple.0 += 1;
                self.current_edge_tuple.1 = 0;
            }
            if !edge.is_null() {
                return Some((
                    NodeId(current_edge_tuple.0),
                    NodeId(current_edge_tuple.1),
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
    node_capacity: usize,
}

impl<'a, It: Iterator<Item = &'a mut Null>, Null: NicheWrapper> Iterator
    for EdgeIterMut<'a, It, Null>
{
    type Item = (NodeId, NodeId, &'a mut Null::Wrapped);

    fn next(&mut self) -> Option<Self::Item> {
        for edge in self.edges.by_ref() {
            let current_edge_tuple = self.current_edge_tuple;
            self.current_edge_tuple.1 += 1;
            if self.current_edge_tuple.1 == self.node_capacity {
                self.current_edge_tuple.0 += 1;
                self.current_edge_tuple.1 = 0;
            }
            if !edge.is_null() {
                return Some((
                    NodeId(current_edge_tuple.0),
                    NodeId(current_edge_tuple.1),
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
    target: NodeId,
    num_nodes: usize,
) -> impl Iterator<Item = (NodeId, &'a <Null as NicheWrapper>::Wrapped)> {
    let start_index = target.0;
    node_adjacencies
        .iter()
        .skip(start_index)
        .step_by(num_nodes)
        .take(num_nodes)
        .enumerate()
        .filter_map(move |(i, adj)| adj.as_ref().map(|data| (NodeId(i), data)))
}

/// Returns an iterator over the neighbors of a node which correspond to incoming edges with the
/// associated edge data.
#[inline]
fn incoming_neighbor_iter_mut<'b, Null: NicheWrapper + 'b>(
    node_adjacencies: &'b mut [Null],
    target: NodeId,
    num_nodes: usize,
) -> impl Iterator<Item = (NodeId, &'b mut <Null as NicheWrapper>::Wrapped)> {
    let start_index = target.0 * num_nodes;
    node_adjacencies
        .iter_mut()
        .skip(start_index)
        .step_by(num_nodes)
        .take(num_nodes)
        .enumerate()
        .filter_map(move |(i, adj)| adj.as_mut().map(|data| (NodeId(i), data)))
}

/// Returns an iterator over the neighbors of a node which correspond to outgoing edges with the
/// associated edge data.
#[inline]
fn outgoing_neighbor_iter<'a, Null: NicheWrapper + 'a>(
    node_adjacencies: &'a [Null],
    source: NodeId,
    num_nodes: usize,
) -> impl Iterator<Item = (NodeId, &'a <Null as NicheWrapper>::Wrapped)> {
    let start_index = source.0 * num_nodes;
    node_adjacencies
        .iter()
        .skip(start_index)
        .take(num_nodes)
        .enumerate()
        .filter_map(move |(i, adj)| adj.as_ref().map(|data| (NodeId(i), data)))
}

/// Returns an iterator over the neighbors of a node which correspond to outgoing edges with the
/// associated edge data.
#[inline]
fn outgoing_neighbor_iter_mut<'b, Null: NicheWrapper + 'b>(
    node_adjacencies: &'b mut [Null],
    source: NodeId,
    num_nodes: usize,
) -> impl Iterator<Item = (NodeId, &'b mut <Null as NicheWrapper>::Wrapped)> {
    let start_index = source.0 * num_nodes;
    node_adjacencies
        .iter_mut()
        .skip(start_index)
        .take(num_nodes)
        .enumerate()
        .filter_map(move |(i, adj)| adj.as_mut().map(|data| (NodeId(i), data)))
}

/// Returns an iterator over the neighbors of a node with the edge data.
///
/// The item in the iterator is a tuple of the form `(source, target, edge_data)` where either
/// `source` or `target` (or both) is the given node and the other is the neighbor.
#[inline]
fn neighbor_iter<'a, Null: NicheWrapper + 'a>(
    node_adjacencies: &'a Vec<Null>,
    node: NodeId,
    num_nodes: usize,
) -> impl Iterator<Item = (NodeId, NodeId, &'a <Null as NicheWrapper>::Wrapped)> {
    let mut slice = node_adjacencies.as_slice();
    let first_iter = {
        if node.0 == 0 {
            None
        } else {
            let bound = (node.0 - 1) * num_nodes;
            let (start, end) = slice.split_at(bound);
            slice = end;
            Some(
                start
                    .iter()
                    .skip(node.0)
                    .step_by(num_nodes)
                    .enumerate()
                    .filter_map(move |(i, adj)| adj.as_ref().map(|data| (NodeId(i), node, data))),
            )
        }
    };
    let (start, end) = slice.split_at(num_nodes);

    let second_iter = start
        .iter()
        .enumerate()
        .filter_map(move |(i, adj)| adj.as_ref().map(|data| (node, NodeId(i), data)));

    let third_iter = end
        .iter()
        .skip(node.0)
        .step_by(num_nodes)
        .enumerate()
        .filter_map(move |(i, adj)| adj.as_ref().map(|data| (NodeId(i), node, data)));

    if let Some(first_iter) = first_iter {
        Either::Left(first_iter.chain(second_iter).chain(third_iter))
    } else {
        Either::Right(second_iter.chain(third_iter))
    }
}

/// Returns an iterator over the neighbors of a node with a mutable reference to the edge data.
///
/// The item in the iterator is a tuple of the form `(source, target, edge_data)` where either
/// `source` or `target` (or both) is the given node and the other is the neighbor.
#[inline]
fn neighbor_iter_mut<'a, Null: NicheWrapper + 'a>(
    node_adjacencies: &'a mut Vec<Null>,
    node: NodeId,
    num_nodes: usize,
) -> impl Iterator<Item = (NodeId, NodeId, &'a mut <Null as NicheWrapper>::Wrapped)> {
    let mut slice = node_adjacencies.as_mut_slice();
    let first_iter = {
        if node.0 == 0 {
            None
        } else {
            let bound = (node.0 - 1) * num_nodes;
            let (start, end) = slice.split_at_mut(bound);
            slice = end;
            Some(
                start
                    .iter_mut()
                    .skip(node.0)
                    .step_by(num_nodes)
                    .enumerate()
                    .filter_map(move |(i, adj)| adj.as_mut().map(|data| (NodeId(i), node, data))),
            )
        }
    };
    let (start, end) = slice.split_at_mut(num_nodes);

    let second_iter = start
        .iter_mut()
        .enumerate()
        .filter_map(move |(i, adj)| adj.as_mut().map(|data| (node, NodeId(i), data)));

    let third_iter = end
        .iter_mut()
        .skip(node.0)
        .step_by(num_nodes)
        .enumerate()
        .filter_map(move |(i, adj)| adj.as_mut().map(|data| (NodeId(i), node, data)));

    if let Some(first_iter) = first_iter {
        Either::Left(first_iter.chain(second_iter).chain(third_iter))
    } else {
        Either::Right(second_iter.chain(third_iter))
    }
}

#[cfg(test)]
mod tests {
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
    fn test_remove_node() {
        let mut graph: MatrixGraph<char, ()> = MatrixGraph::new();
        let node_a = graph.add_node('a');

        graph.remove_node(node_a);

        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_add_edge() {
        let mut graph = MatrixGraph::<_, _>::new();
        let node_a = graph.add_node('a');
        let node_b = graph.add_node('b');
        let node_c = graph.add_node('c');
        graph.add_edge(node_a, node_b, ());
        graph.add_edge(node_b, node_c, ());
        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 2);
    }

    #[test]
    /// Adds an edge that triggers a second extension of the matrix.
    /// From #425
    fn test_add_edge_with_extension() {
        let mut graph = DiMatrix::<u8, ()>::new();
        let _node_0 = graph.add_node(0);
        let node_1 = graph.add_node(1);
        let node_2 = graph.add_node(2);
        let node_3 = graph.add_node(3);
        let node_4 = graph.add_node(4);
        let _node_5 = graph.add_node(5);
        graph.add_edge(node_2, node_1, ());
        graph.add_edge(node_2, node_3, ());
        graph.add_edge(node_2, node_4, ());
        assert_eq!(graph.node_count(), 6);
        assert_eq!(graph.edge_count(), 3);
        assert!(graph.contains_edge(DirEdgeId::new_directed(node_2, node_1)));
        assert!(graph.contains_edge(DirEdgeId::new_directed(node_2, node_3)));
        assert!(graph.contains_edge(DirEdgeId::new_directed(node_2, node_4)));
    }

    #[test]
    fn test_matrix_resize() {
        let mut graph = DiMatrix::<u8, ()>::with_capacity(3);
        let node_0 = graph.add_node(0);
        let node_1 = graph.add_node(1);
        let node_2 = graph.add_node(2);
        let node_3 = graph.add_node(3);
        graph.add_edge(node_1, node_0, ());
        graph.add_edge(node_1, node_1, ());
        // Triggers a resize from capacity 3 to 4
        graph.add_edge(node_2, node_3, ());
        assert_eq!(graph.node_count(), 4);
        assert_eq!(graph.edge_count(), 3);
        assert!(graph.contains_edge(DirEdgeId::new_directed(node_1, node_0)));
        assert!(graph.contains_edge(DirEdgeId::new_directed(node_1, node_1)));
        assert!(graph.contains_edge(DirEdgeId::new_directed(node_2, node_3)));
    }

    #[test]
    fn test_add_edge_with_data() {
        let mut graph = MatrixGraph::<_, _>::new();
        let node_a = graph.add_node('a');
        let node_b = graph.add_node('b');
        let node_c = graph.add_node('c');
        graph.add_edge(node_a, node_b, true);
        graph.add_edge(node_b, node_c, false);
        assert!(
            graph
                .edge(DirEdgeId::new_directed(node_a, node_b))
                .unwrap()
                .data
        );
        assert!(
            !*graph
                .edge(DirEdgeId::new_directed(node_b, node_c))
                .unwrap()
                .data
        );
    }

    #[test]
    fn test_clear() {
        let mut graph = MatrixGraph::<_, _>::new();
        let node_a = graph.add_node('a');
        let node_b = graph.add_node('b');
        let node_c = graph.add_node('c');
        assert_eq!(graph.node_count(), 3);

        graph.add_edge(node_a, node_b, ());
        graph.add_edge(node_b, node_c, ());
        graph.add_edge(node_c, node_a, ());
        assert_eq!(graph.edge_count(), 3);

        graph.clear();

        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);

        let node_a = graph.add_node('a');
        let node_b = graph.add_node('b');
        let node_c = graph.add_node('c');
        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 0);

        assert_eq!(graph.predecessors(node_a).collect::<Vec<_>>(), vec![]);
        assert_eq!(graph.predecessors(node_b).collect::<Vec<_>>(), vec![]);
        assert_eq!(graph.predecessors(node_c).collect::<Vec<_>>(), vec![]);

        assert_eq!(graph.successors(node_a).collect::<Vec<_>>(), vec![]);
        assert_eq!(graph.successors(node_b).collect::<Vec<_>>(), vec![]);
        assert_eq!(graph.successors(node_c).collect::<Vec<_>>(), vec![]);
    }

    #[test]
    fn test_alternative_null_type() {
        let mut graph: MatrixGraph<(), i32, foldhash::fast::RandomState, NotZero<i32>, Directed> =
            MatrixGraph::default();

        let node_a = graph.add_node(());
        let node_b = graph.add_node(());

        assert!(!graph.contains_edge(DirEdgeId::new_directed(node_a, node_b)));
        assert_eq!(graph.edge_count(), 0);

        graph.add_edge(node_a, node_b, 12);

        assert!(graph.contains_edge(DirEdgeId::new_directed(node_a, node_b)));
        assert_eq!(graph.edge_count(), 1);
        assert_eq!(
            graph
                .edge(DirEdgeId::new_directed(node_a, node_b))
                .unwrap()
                .data,
            &12
        );

        graph.remove_edge(node_a, node_b);

        assert!(!graph.contains_edge(DirEdgeId::new_directed(node_a, node_b)));
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    #[should_panic = "assertion failed: !value.is_zero()"]
    fn test_not_zero_asserted() {
        let mut graph: MatrixGraph<(), i32, foldhash::fast::RandomState, NotZero<i32>, Directed> =
            MatrixGraph::default();

        let node_a = graph.add_node(());
        let node_b = graph.add_node(());

        graph.add_edge(node_a, node_b, 0);
    }

    #[test]
    fn test_not_zero_float() {
        let mut graph: MatrixGraph<(), f32, foldhash::fast::RandomState, NotZero<f32>, Directed> =
            MatrixGraph::default();

        let node_a = graph.add_node(());
        let node_b = graph.add_node(());

        assert!(!graph.contains_edge(DirEdgeId::new_directed(node_a, node_b)));
        assert_eq!(graph.edge_count(), 0);

        let val = 12.0;
        graph.add_edge(node_a, node_b, val);

        assert!(graph.contains_edge(DirEdgeId::new_directed(node_a, node_b)));
        assert_eq!(graph.edge_count(), 1);
        assert!(
            (graph
                .edge(DirEdgeId::new_directed(node_a, node_b))
                .unwrap()
                .data
                - val)
                .abs()
                <= 0.0
        );

        graph.remove_edge(node_a, node_b);

        assert!(!graph.contains_edge(DirEdgeId::new_directed(node_a, node_b)));
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    #[should_panic = "called `Option::unwrap()` on a `None` value"]
    fn test_remove_edge() {
        let mut graph = MatrixGraph::<char, u32>::new();
        let node_a = graph.add_node('a');
        let node_b = graph.add_node('b');
        let node_c = graph.add_node('c');
        graph.add_edge(node_a, node_b, 1);
        graph.add_edge(node_b, node_c, 2);
        assert_eq!(graph.remove_edge(node_a, node_b), 1);
        assert_eq!(graph.remove_edge(node_a, node_b), 0);
    }
}
