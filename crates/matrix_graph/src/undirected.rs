use core::{hash::BuildHasher, marker::PhantomData};

use petgraph_core::{
    edge::{EdgeMut, EdgeRef},
    graph::{Cardinality, DensityHint, Graph, UndirectedGraph},
    id::Id,
    node::{NodeMut, NodeRef},
};

use crate::{
    MatrixGraph, MatrixGraphExtras, NicheWrapper, NodeId, Undirected, ensure_len, private::Sealed,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnMatrixEdgeId {
    pub source: NodeId,
    pub target: NodeId,
    _private: PhantomData<()>,
}

impl UnMatrixEdgeId {
    #[must_use]
    pub const fn new(source: NodeId, target: NodeId) -> Self {
        if source.0 < target.0 {
            Self {
                source,
                target,
                _private: PhantomData::<()>,
            }
        } else {
            Self {
                source: target,
                target: source,
                _private: PhantomData::<()>,
            }
        }
    }
}

impl core::fmt::Display for UnMatrixEdgeId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "UnMatrixEdgeId{{{}, {}}}", self.source.0, self.target.0)
    }
}

impl Id for UnMatrixEdgeId {}

impl<N, E, S, Null: NicheWrapper<Wrapped = E>> Sealed for MatrixGraph<N, E, S, Null, Undirected> {}

impl<N, E, S: BuildHasher, Null: NicheWrapper<Wrapped = E>> MatrixGraphExtras<N>
    for MatrixGraph<N, E, S, Null, Undirected>
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
        to_lower_triangular_matrix_position(node_a.0, node_b.0)
    }

    #[inline]
    fn extend_capacity_for_node(&mut self, new_node_capacity: usize, _exact: bool) {
        let old_node_capacity = self.node_capacity;

        if old_node_capacity >= new_node_capacity {
            return;
        }

        self.node_capacity =
            extend_lower_triangular_matrix(&mut self.flattened_edge_data, new_node_capacity);
    }

    #[inline]
    fn remove_node(&mut self, node: NodeId) -> N {
        for (id, _) in self.node_data.iter() {
            let position = self.to_edge_position(node, NodeId(id));
            if let Some(pos) = position {
                let entry = &mut self.flattened_edge_data[pos];
                if !entry.is_null() {
                    *entry = Default::default();
                    self.edge_count -= 1;
                }
            }
        }

        self.node_data.remove(node.0)
    }
}

#[inline]
const fn to_lower_triangular_matrix_position(row: usize, column: usize) -> usize {
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

impl<N, E, S: BuildHasher, Null: NicheWrapper<Wrapped = E>> Graph
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
    type EdgeId = UnMatrixEdgeId;
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

impl<N, E, S: BuildHasher, Null: NicheWrapper<Wrapped = E>> UndirectedGraph
    for MatrixGraph<N, E, S, Null, Undirected>
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
            next_edge_tuple: (0, 0),
        }
        .map(|(source, target, data)| EdgeRef::<Self> {
            id: UnMatrixEdgeId::new(source, target),
            source,
            target,
            data,
        })
    }

    #[inline]
    fn edges_mut(&mut self) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        EdgeIterMut {
            edges: self.flattened_edge_data.iter_mut(),
            next_edge_tuple: (0, 0),
        }
        .map(|(source, target, data)| EdgeMut::<Self> {
            id: UnMatrixEdgeId::new(source, target),
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
    fn degree(&self, node: Self::NodeId) -> usize {
        NeighborIterator::new(&self.flattened_edge_data, node, self.node_capacity).count()
    }

    // Incidence
    #[inline]
    fn incident_edges(&self, node: Self::NodeId) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        NeighborIterator::new(&self.flattened_edge_data, node, self.node_capacity).map(
            move |(neighbor, data)| EdgeRef::<Self> {
                id: UnMatrixEdgeId::new(node, neighbor),
                source: node,
                target: neighbor,
                data,
            },
        )
    }

    #[inline]
    fn incident_edges_mut(
        &mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        NeighborIterMut::new(
            &mut self.flattened_edge_data,
            node,
            NodeId(0),
            self.node_capacity,
        )
        .map(move |(neighbor, data)| EdgeMut::<Self> {
            id: UnMatrixEdgeId::new(node, neighbor),
            source: node,
            target: neighbor,
            data,
        })
    }

    // Adjacency
    #[inline]
    fn adjacencies(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        NeighborIterator::new(&self.flattened_edge_data, node, self.node_capacity)
            .map(|(neighbor, _)| neighbor)
    }

    // Edges between nodes
    #[inline]
    fn edges_connecting(
        &self,
        lhs: Self::NodeId,
        rhs: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.to_edge_position(lhs, rhs).map_or_else(
            || None.into_iter(),
            |edge_index| {
                self.flattened_edge_data
                    .get(edge_index)
                    .unwrap()
                    .as_ref()
                    .map(|data| EdgeRef::<Self> {
                        id: UnMatrixEdgeId::new(lhs, rhs),
                        source: lhs,
                        target: rhs,
                        data,
                    })
                    .into_iter()
            },
        )
    }

    #[inline]
    fn edges_connecting_mut(
        &mut self,
        lhs: Self::NodeId,
        rhs: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        if let Some(edge_index) = self.to_edge_position(lhs, rhs) {
            self.flattened_edge_data
                .get_mut(edge_index)
                .unwrap()
                .as_mut()
                .map(|data| EdgeMut::<Self> {
                    id: UnMatrixEdgeId::new(lhs, rhs),
                    source: lhs,
                    target: rhs,
                    data,
                })
                .into_iter()
        } else {
            None.into_iter()
        }
    }

    // Existence checks
    #[inline]
    fn contains_node(&self, node: Self::NodeId) -> bool {
        self.node_data.get(node.0).is_some()
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
        self.contains_edge(UnMatrixEdgeId::new(source, target))
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.node_count() == 0
    }
}

/// An iterator over the edges of a directed graph which yields the source and target of each edge
/// along with a reference to the edge data.
struct EdgeIter<'a, It: Iterator<Item = &'a Null>, Null: NicheWrapper + 'a> {
    edges: It,
    next_edge_tuple: (usize, usize),
}

impl<'a, It: Iterator<Item = &'a Null>, Null: NicheWrapper> Iterator for EdgeIter<'a, It, Null> {
    type Item = (NodeId, NodeId, &'a Null::Wrapped);

    fn next(&mut self) -> Option<Self::Item> {
        for edge in self.edges.by_ref() {
            let current_edge_tuple = self.next_edge_tuple;
            self.next_edge_tuple.1 += 1;
            if self.next_edge_tuple.1 > self.next_edge_tuple.0 {
                self.next_edge_tuple.0 += 1;
                self.next_edge_tuple.1 = 0;
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
    next_edge_tuple: (usize, usize),
}

impl<'a, It: Iterator<Item = &'a mut Null>, Null: NicheWrapper> Iterator
    for EdgeIterMut<'a, It, Null>
{
    type Item = (NodeId, NodeId, &'a mut Null::Wrapped);

    fn next(&mut self) -> Option<Self::Item> {
        for edge in self.edges.by_ref() {
            let current_edge_tuple = self.next_edge_tuple;
            self.next_edge_tuple.1 += 1;
            if self.next_edge_tuple.1 > self.next_edge_tuple.0 {
                self.next_edge_tuple.0 += 1;
                self.next_edge_tuple.1 = 0;
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

struct NeighborIterator<'a, Null: NicheWrapper + 'a> {
    edges: &'a [Null],
    start_node: NodeId,
    next_other_node: NodeId,
    node_capacity: usize,
}

impl<'a, Null: NicheWrapper + 'a> NeighborIterator<'a, Null> {
    const fn new(edges: &'a [Null], start_node: NodeId, node_capacity: usize) -> Self {
        if start_node.0 >= node_capacity {
            // This iterator will be empty, since a node with an index greater than or equal to
            // the node capacity cannot have any neighbors.
            Self {
                edges,
                start_node,
                next_other_node: start_node,
                node_capacity,
            }
        } else {
            Self {
                edges,
                start_node,
                next_other_node: NodeId(0),
                node_capacity,
            }
        }
    }
}

impl<'a, Null: NicheWrapper> Iterator for NeighborIterator<'a, Null> {
    type Item = (NodeId, &'a Null::Wrapped);

    fn next(&mut self) -> Option<Self::Item> {
        while self.next_other_node.0 < self.node_capacity {
            let this_other_node = self.next_other_node;

            let edge_index =
                to_lower_triangular_matrix_position(self.start_node.0, this_other_node.0);
            if let Some(data) = self.edges[edge_index].as_ref() {
                self.next_other_node.0 += 1;
                return Some((this_other_node, data));
            }
            self.next_other_node.0 += 1;
        }

        None
    }
}

/// An iterator over the neighbors of a node (`source_node`) in a directed graph which yields the
/// neighbor along with a mutable reference to the edge data for the edge connecting the source node
/// to the neighbor.
struct NeighborIterMut<'a, Null> {
    /// The remaining slice of the flattened edge data that has not yet been consumed by the
    /// iterator.
    remaining_edge_data: &'a mut [Null],
    /// Index in the flattened edge data of the edge sitting currently at `remaining_edge_data[0]`,
    consumed: usize,
    source_node: NodeId,
    next_node: NodeId,
    node_capacity: usize,
}

impl<'a, Null> NeighborIterMut<'a, Null> {
    #[inline]
    const fn new(
        flattened_edge_data: &'a mut [Null],
        source_node: NodeId,
        next_node: NodeId,
        node_capacity: usize,
    ) -> Self {
        if source_node.0 >= node_capacity {
            // This iterator will be empty, since a node with an index greater than or equal to
            // the node capacity cannot have any neighbors.
            Self {
                remaining_edge_data: flattened_edge_data,
                consumed: 0,
                source_node,
                next_node: source_node,
                node_capacity,
            }
        } else {
            Self {
                remaining_edge_data: flattened_edge_data,
                consumed: 0,
                source_node,
                next_node,
                node_capacity,
            }
        }
    }
}

impl<'a, Null: NicheWrapper> Iterator for NeighborIterMut<'a, Null> {
    type Item = (NodeId, &'a mut Null::Wrapped);

    fn next(&mut self) -> Option<Self::Item> {
        while self.next_node.0 < self.node_capacity {
            let this_node = self.next_node;
            self.next_node.0 += 1;

            let index = to_lower_triangular_matrix_position(self.source_node.0, this_node.0);
            let skip = index - self.consumed;

            let rest = core::mem::take(&mut self.remaining_edge_data);
            let (head, tail) = rest.get_mut(skip..)?.split_first_mut()?;
            self.remaining_edge_data = tail;
            self.consumed = index + 1;

            if let Some(data) = head.as_mut() {
                return Some((this_node, data));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use petgraph_core::test_undirected_graph;

    use super::{super::*, *};

    #[test]
    fn test_default() {
        let graph =
            MatrixGraph::<i32, i32, foldhash::fast::RandomState, Option<i32>, Undirected>::default(
            );
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_with_capacity() {
        let graph = MatrixGraph::<i32, i32, foldhash::fast::RandomState, Option<i32>, Undirected>::with_capacity(10);
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_remove_node() {
        let mut graph: MatrixGraph<char, (), foldhash::fast::RandomState, Option<()>, Undirected> =
            MatrixGraph::default();
        let node_a = graph.add_node('a');

        graph.remove_node(node_a);

        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_add_edge() {
        let mut graph: MatrixGraph<char, (), foldhash::fast::RandomState, Option<()>, Undirected> =
            MatrixGraph::default();
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
        let mut graph = UnMatrix::<u8, ()>::default();
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
        assert!(graph.contains_edge(UnMatrixEdgeId::new(node_2, node_1)));
        assert!(graph.contains_edge(UnMatrixEdgeId::new(node_2, node_3)));
        assert!(graph.contains_edge(UnMatrixEdgeId::new(node_2, node_4)));
    }

    #[test]
    fn test_matrix_resize() {
        let mut graph = UnMatrix::<u8, ()>::with_capacity(3);
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
        assert!(graph.contains_edge(UnMatrixEdgeId::new(node_1, node_0)));
        assert!(graph.contains_edge(UnMatrixEdgeId::new(node_1, node_1)));
        assert!(graph.contains_edge(UnMatrixEdgeId::new(node_2, node_3)));
    }

    #[test]
    fn test_add_edge_with_data() {
        let mut graph = MatrixGraph::new_undirected();
        let node_a = graph.add_node('a');
        let node_b = graph.add_node('b');
        let node_c = graph.add_node('c');
        graph.add_edge(node_a, node_b, true);
        graph.add_edge(node_b, node_c, false);
        assert!(
            graph
                .edge(UnMatrixEdgeId::new(node_a, node_b))
                .unwrap()
                .data
        );
        assert!(
            !*graph
                .edge(UnMatrixEdgeId::new(node_b, node_c))
                .unwrap()
                .data
        );
    }

    #[test]
    fn test_clear() {
        let mut graph = MatrixGraph::new_undirected();
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

        assert_eq!(graph.adjacencies(node_a).collect::<Vec<_>>(), vec![]);
        assert_eq!(graph.adjacencies(node_b).collect::<Vec<_>>(), vec![]);
        assert_eq!(graph.adjacencies(node_c).collect::<Vec<_>>(), vec![]);
    }

    #[test]
    fn test_alternative_null_type() {
        let mut graph: MatrixGraph<(), i32, foldhash::fast::RandomState, NotZero<i32>, Undirected> =
            MatrixGraph::default();

        let node_a = graph.add_node(());
        let node_b = graph.add_node(());

        assert!(!graph.contains_edge(UnMatrixEdgeId::new(node_a, node_b)));
        assert_eq!(graph.edge_count(), 0);

        graph.add_edge(node_a, node_b, 12);

        assert!(graph.contains_edge(UnMatrixEdgeId::new(node_a, node_b)));
        assert_eq!(graph.edge_count(), 1);
        assert_eq!(
            graph
                .edge(UnMatrixEdgeId::new(node_a, node_b))
                .unwrap()
                .data,
            &12
        );

        graph.remove_edge(node_a, node_b);

        assert!(!graph.contains_edge(UnMatrixEdgeId::new(node_a, node_b)));
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    #[should_panic = "assertion failed: !value.is_zero()"]
    fn test_not_zero_asserted() {
        let mut graph: MatrixGraph<(), i32, foldhash::fast::RandomState, NotZero<i32>, Undirected> =
            MatrixGraph::default();

        let node_a = graph.add_node(());
        let node_b = graph.add_node(());

        graph.add_edge(node_a, node_b, 0);
    }

    #[test]
    fn test_not_zero_float() {
        let mut graph: MatrixGraph<(), f32, foldhash::fast::RandomState, NotZero<f32>, Undirected> =
            MatrixGraph::default();

        let node_a = graph.add_node(());
        let node_b = graph.add_node(());

        assert!(!graph.contains_edge(UnMatrixEdgeId::new(node_a, node_b)));
        assert_eq!(graph.edge_count(), 0);

        let val = 12.0;
        graph.add_edge(node_a, node_b, val);

        assert!(graph.contains_edge(UnMatrixEdgeId::new(node_a, node_b)));
        assert_eq!(graph.edge_count(), 1);
        assert!(
            (graph
                .edge(UnMatrixEdgeId::new(node_a, node_b))
                .unwrap()
                .data
                - val)
                .abs()
                <= 0.0
        );

        graph.remove_edge(node_a, node_b);

        assert!(!graph.contains_edge(UnMatrixEdgeId::new(node_a, node_b)));
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    #[should_panic(expected = "called `Option::unwrap()` on a `None` value")]
    fn test_remove_edge() {
        let mut graph = MatrixGraph::<char, u32>::new_directed();
        let node_a = graph.add_node('a');
        let node_b = graph.add_node('b');
        let node_c = graph.add_node('c');
        graph.add_edge(node_a, node_b, 1);
        graph.add_edge(node_b, node_c, 2);
        assert_eq!(graph.remove_edge(node_a, node_b), 1);
        assert_eq!(graph.remove_edge(node_a, node_b), 0);
    }

    fn remove_node(
        graph: &mut MatrixGraph<(), (), foldhash::fast::RandomState, Option<()>, Undirected>,
        node: NodeId,
    ) {
        graph.remove_node(node);
    }

    fn add_edge(
        graph: &mut MatrixGraph<(), (), foldhash::fast::RandomState, Option<()>, Undirected>,
        source: NodeId,
        target: NodeId,
        _: (),
    ) -> UnMatrixEdgeId {
        graph.add_edge(source, target, ());
        UnMatrixEdgeId::new(source, target)
    }

    fn remove_edge(
        graph: &mut MatrixGraph<(), (), foldhash::fast::RandomState, Option<()>, Undirected>,
        edge_id: UnMatrixEdgeId,
    ) {
        graph.remove_edge(edge_id.source, edge_id.target);
    }

    test_undirected_graph!(
        MatrixGraph::<(), (), foldhash::fast::RandomState, Option<()>, Undirected>::new_undirected,
        MatrixGraph::<(), (), foldhash::fast::RandomState, Option<()>, Undirected>::add_node,
        remove_node,
        add_edge,
        remove_edge
    );
}
