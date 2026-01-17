use core::{fmt::Display, hash::BuildHasher};

use petgraph_old::{
    Directed, Direction, Undirected,
    csr::IndexType,
    matrix_graph::{MatrixGraph, NodeIndex, Nullable},
    visit::IntoNodeReferences,
};

use crate::{
    edge::{EdgeMut, EdgeRef},
    graph::{Cardinality, DensityHint, DirectedGraph, Graph as NewGraph, UndirectedGraph},
    id::Id,
    node::{NodeMut, NodeRef},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MatrixGraphEdgeId<Ix> {
    pub source: NodeIndex<Ix>,
    pub target: NodeIndex<Ix>,
}

impl<Ix: Display> Display for MatrixGraphEdgeId<Ix> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Edge({}, {})", self.source, self.target)
    }
}

impl<Ix: Display + IndexType> Id for MatrixGraphEdgeId<Ix> {}

impl<N, E, S, Ty, Null: Nullable<Wrapped = E>, Ix: IndexType + Display> NewGraph
    for MatrixGraph<N, E, S, Ty, Null, Ix>
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
    type EdgeId = MatrixGraphEdgeId<Ix>;
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
    type NodeId = NodeIndex<Ix>;
}

impl<N, E, S: BuildHasher, Null: Nullable<Wrapped = E>, Ix: IndexType + Display> DirectedGraph
    for MatrixGraph<N, E, S, Directed, Null, Ix>
{
    #[inline]
    fn density_hint(&self) -> DensityHint {
        DensityHint::Dense
    }

    #[inline]
    fn cardinality(&self) -> Cardinality {
        Cardinality {
            order: self.node_count(),
            size: self.edge_count(),
        }
    }

    #[inline]
    fn node_count(&self) -> usize {
        MatrixGraph::node_count(self)
    }

    #[inline]
    fn edge_count(&self) -> usize {
        MatrixGraph::edge_count(self)
    }

    // Node Iteration

    fn nodes(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        self.node_references()
            .map(|(id, data)| NodeRef::<'_, Self> { id, data })
    }

    fn nodes_mut<'a>(&'a mut self) -> impl Iterator<Item = NodeMut<'a, Self>> {
        self.all_nodes_mut()
            .map(|(id, data)| NodeMut::<'a, Self> { id, data })
    }

    /// Nodes with degree 0 (no incident edges).
    ///
    /// Due to restrictions in the API of [`MatrixGraph`](petgraph_old::matrix_graph::MatrixGraph)
    /// this is implemented by filtering [Self::nodes], which may be inefficient for large graphs.
    #[inline]
    fn isolated_nodes(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        self.nodes().filter(|node| self.degree(node.id) == 0)
    }

    // Edge iteration

    fn edges(&self) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.all_edges()
            .map(|(source, target, data)| EdgeRef::<'_, Self> {
                id: MatrixGraphEdgeId { source, target },
                source,
                target,
                data,
            })
    }

    fn edges_mut(&mut self) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.all_edges_mut()
            .map(|(source, target, data)| EdgeMut::<'_, Self> {
                id: MatrixGraphEdgeId { source, target },
                source,
                target,
                data,
            })
    }

    // Lookup
    #[inline]
    fn node(&self, id: Self::NodeId) -> Option<NodeRef<'_, Self>> {
        if !self.has_node(id) {
            return None;
        }

        Some(NodeRef::<'_, Self> {
            id,
            data: self.node_weight(id),
        })
    }

    #[inline]
    fn node_mut(&mut self, id: Self::NodeId) -> Option<NodeMut<'_, Self>> {
        if !self.has_node(id) {
            return None;
        }

        Some(NodeMut::<'_, Self> {
            id,
            data: self.node_weight_mut(id),
        })
    }

    #[inline]
    fn edge(&self, id: Self::EdgeId) -> Option<EdgeRef<'_, Self>> {
        if !self.has_node(id.source) || !self.has_node(id.target) {
            return None;
        }

        if !self.has_edge(id.source, id.target) {
            return None;
        }

        Some(EdgeRef::<'_, Self> {
            id,
            source: id.source,
            target: id.target,
            data: self.edge_weight(id.source, id.target),
        })
    }

    #[inline]
    fn edge_mut(&mut self, id: Self::EdgeId) -> Option<EdgeMut<'_, Self>> {
        if !self.has_node(id.source) || !self.has_node(id.target) {
            return None;
        }

        if !self.has_edge(id.source, id.target) {
            return None;
        }

        Some(EdgeMut::<'_, Self> {
            id,
            source: id.source,
            target: id.target,
            data: self.edge_weight_mut(id.source, id.target),
        })
    }

    // Degree

    /// Number of incoming edges.
    ///
    /// Performance note: Due to restrictions in the API of [`Graph`](petgraph_old::Graph) this
    /// iterates over the incoming neighbors to count them, which may be inefficient for large
    /// graphs.
    #[inline]
    fn in_degree(&self, node: Self::NodeId) -> usize {
        self.neighbors_directed(node, Direction::Incoming).count()
    }

    /// Number of outgoing edges.
    ///
    /// Performance note: Due to restrictions in the API of [`Graph`](petgraph_old::Graph) this
    /// iterates over the outgoing neighbors to count them, which may be inefficient for large
    /// graphs.
    #[inline]
    fn out_degree(&self, node: Self::NodeId) -> usize {
        self.neighbors_directed(node, Direction::Outgoing).count()
    }

    /// Number of incident edges.
    ///
    /// Performance note: Due to restrictions in the API of [`Graph`](petgraph_old::Graph) this
    /// iterates over all neighbors to count them, which may be inefficient for large graphs.
    #[inline]
    fn degree(&self, node: Self::NodeId) -> usize {
        self.neighbors_directed(node, Direction::Incoming).count()
            + self.neighbors_directed(node, Direction::Outgoing).count()
    }

    // Edges by direction
    #[inline]
    fn incoming_edges(&self, node: Self::NodeId) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.edges_directed(node, Direction::Incoming)
            .map(|(source, target, data)| EdgeRef::<'_, Self> {
                id: MatrixGraphEdgeId { source, target },
                source,
                target,
                data,
            })
    }

    /// Mutable iterator over incoming edges. That is, edges where `target == node`.
    ///
    /// Performance note: Due to restrictions in the API of
    /// [`MatrixGraph`](petgraph_old::matrix_graph::MatrixGraph) this is implemented by
    /// filtering [Self::edges_mut], which may be inefficient for large graphs.
    #[inline]
    fn incoming_edges_mut(
        &mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.edges_mut().filter(move |edge| edge.target == node)
    }

    #[inline]
    fn outgoing_edges(&self, node: Self::NodeId) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.edges_directed(node, Direction::Outgoing)
            .map(|(source, target, data)| EdgeRef::<'_, Self> {
                id: MatrixGraphEdgeId { source, target },
                source,
                target,
                data,
            })
    }

    /// Mutable iterator over outgoing edges. That is, edges where `source == node`.
    ///
    /// Performance note: Due to restrictions in the API of
    /// [`MatrixGraph`](petgraph_old::matrix_graph::MatrixGraph) this is implemented by
    /// filtering [Self::edges_mut], which may be inefficient for large graphs.
    #[inline]
    fn outgoing_edges_mut(
        &mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.edges_mut().filter(move |edge| edge.source == node)
    }

    #[inline]
    fn incident_edges(&self, node: Self::NodeId) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.edges_directed(node, Direction::Incoming)
            .chain(self.edges_directed(node, Direction::Outgoing))
            .map(|(source, target, data)| EdgeRef::<'_, Self> {
                id: MatrixGraphEdgeId { source, target },
                source,
                target,
                data,
            })
    }

    /// Mutable iterator over incident edges of node. That is, all edges where node is either
    /// source or target.
    ///
    /// Performance note: Due to restrictions in the API of
    /// [`MatrixGraph`](petgraph_old::matrix_graph::MatrixGraph) this is implemented by
    /// filtering [Self::edges_mut], which may be inefficient for large graphs.
    #[inline]
    fn incident_edges_mut(
        &mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.edges_mut()
            .filter(move |edge| edge.source == node || edge.target == node)
    }

    // Adjacency
    #[inline]
    fn predecessors(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        self.neighbors_directed(node, Direction::Incoming)
            .map(|n| n)
    }

    #[inline]
    fn successors(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        self.neighbors_directed(node, Direction::Outgoing)
            .map(|n| n)
    }

    #[inline]
    fn adjacencies(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        self.edges_directed(node, Direction::Incoming)
            .chain(self.edges_directed(node, Direction::Outgoing))
            .map(|(_source, target, _data)| target)
    }

    // Edges between nodes

    /// Iterator over all edges between source and target. Note that this only considers edges
    /// where the provided source and target are actually the source and target of the edge.
    ///
    /// Performance note: Due to restrictions in the API of
    /// [`MatrixGraph`](petgraph_old::matrix_graph::MatrixGraph) this is implemented by
    /// filtering all outgoing edges of source.
    #[inline]
    fn edges_between(
        &self,
        source: Self::NodeId,
        target: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.edges_directed(source, Direction::Outgoing)
            .filter(move |(_s, t, _d)| *t == target)
            .map(move |(s, t, d)| EdgeRef::<'_, Self> {
                id: MatrixGraphEdgeId {
                    source: s,
                    target: t,
                },
                source: s,
                target: t,
                data: d,
            })
    }

    /// Mutable iterator over all edges between source and target. Note that this only considers
    /// edges where the provided source and target are actually the source and target of the edge.
    /// For an undirected equivalent, see [Self::edges_connecting_mut].
    ///
    /// Performance note: Due to restrictions in the API of
    /// [`MatrixGraph`](petgraph_old::MatrixGraph) this is implemented by filtering
    /// [Self::edges_mut], which may be inefficient for large graphs.
    #[inline]
    fn edges_between_mut(
        &mut self,
        source: Self::NodeId,
        target: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.edges_mut()
            .filter(move |edge| edge.source == source && edge.target == target)
    }

    /// Iterator over all edges connecting lhs and rhs, regardless of direction.
    ///
    /// Performance note: Due to restrictions in the API of
    /// [`MatrixGraph`](petgraph_old::matrix_graph::MatrixGraph) this is implemented by
    /// chaining two [Self::edges_between] calls, which may be inefficient for large graphs.
    #[inline]
    fn edges_connecting(
        &self,
        lhs: Self::NodeId,
        rhs: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.edges_between(lhs, rhs)
            .chain(self.edges_between(rhs, lhs))
    }

    /// Mutable iterator over all edges connecting lhs and rhs, regardless of direction.
    ///
    /// Performance note: Due to restrictions in the API of
    /// [`MatrixGraph`](petgraph_old::matrix_graph::MatrixGraph) this is implemented by filtering
    /// [Self::edges_mut], which may be inefficient for large graphs.
    #[inline]
    fn edges_connecting_mut(
        &mut self,
        lhs: Self::NodeId,
        rhs: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.edges_mut().filter(move |edge| {
            (edge.source == lhs && edge.target == rhs) || (edge.source == rhs && edge.target == lhs)
        })
    }

    // Existence checks
    #[inline]
    fn contains_node(&self, node: Self::NodeId) -> bool {
        self.has_node(node)
    }

    #[inline]
    fn contains_edge(&self, edge: Self::EdgeId) -> bool {
        self.has_edge(edge.source, edge.target)
    }

    #[inline]
    fn is_adjacent(&self, source: Self::NodeId, target: Self::NodeId) -> bool {
        self.has_edge(source, target)
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

impl<N, E, S: BuildHasher, Null: Nullable<Wrapped = E>, Ix: IndexType + Display> UndirectedGraph
    for MatrixGraph<N, E, S, Undirected, Null, Ix>
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
        MatrixGraph::node_count(self)
    }

    #[inline]
    fn edge_count(&self) -> usize {
        MatrixGraph::edge_count(self)
    }

    // Node iteration

    fn nodes(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        self.node_references()
            .map(|(id, data)| NodeRef::<'_, Self> { id, data })
    }

    fn nodes_mut<'a>(&'a mut self) -> impl Iterator<Item = NodeMut<'a, Self>> {
        self.all_nodes_mut()
            .map(|(id, data)| NodeMut::<'a, Self> { id, data })
    }

    #[inline]
    fn isolated_nodes(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        self.nodes().filter(|node| self.degree(node.id) == 0)
    }

    // Edge iteration

    fn edges(&self) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.all_edges()
            .map(|(source, target, data)| EdgeRef::<'_, Self> {
                id: MatrixGraphEdgeId { source, target },
                source,
                target,
                data,
            })
    }

    /// Mutable iterator over all edges.
    ///
    /// Performance note: Due to restrictions in the API of [`Graph`](petgraph_old::Graph) this is
    /// implemented by first collecting edges endpoints and ids into a Vec and then combining
    /// that with the mutable edge weights iterator. Therefore, this may be inefficient and/or
    /// use more memory than expected for large graphs.
    fn edges_mut(&mut self) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.all_edges_mut()
            .map(|(source, target, data)| EdgeMut::<'_, Self> {
                id: MatrixGraphEdgeId { source, target },
                source,
                target,
                data,
            })
    }

    // Lookup
    #[inline]
    fn node(&self, id: Self::NodeId) -> Option<NodeRef<'_, Self>> {
        if !self.has_node(id) {
            return None;
        }

        Some(NodeRef::<'_, Self> {
            id,
            data: self.node_weight(id),
        })
    }

    #[inline]
    fn node_mut(&mut self, id: Self::NodeId) -> Option<NodeMut<'_, Self>> {
        if !self.has_node(id) {
            return None;
        }

        Some(NodeMut::<'_, Self> {
            id,
            data: self.node_weight_mut(id),
        })
    }

    #[inline]
    fn edge(&self, id: Self::EdgeId) -> Option<EdgeRef<'_, Self>> {
        if !self.has_node(id.source) || !self.has_node(id.target) {
            return None;
        }

        if !self.has_edge(id.source, id.target) {
            return None;
        }

        Some(EdgeRef::<'_, Self> {
            id,
            source: id.source,
            target: id.target,
            data: self.edge_weight(id.source, id.target),
        })
    }

    #[inline]
    fn edge_mut(&mut self, id: Self::EdgeId) -> Option<EdgeMut<'_, Self>> {
        if !self.has_node(id.source) || !self.has_node(id.target) {
            return None;
        }

        if !self.has_edge(id.source, id.target) {
            return None;
        }

        Some(EdgeMut::<'_, Self> {
            id,
            source: id.source,
            target: id.target,
            data: self.edge_weight_mut(id.source, id.target),
        })
    }

    // Degree
    #[inline]
    fn degree(&self, node: Self::NodeId) -> usize {
        self.neighbors(node).count()
    }

    // Incidence
    #[inline]
    fn incident_edges(&self, node: Self::NodeId) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.edges(node)
            .map(|(source, target, data)| EdgeRef::<'_, Self> {
                id: MatrixGraphEdgeId { source, target },
                source,
                target,
                data,
            })
    }

    #[inline]
    fn incident_edges_mut(
        &mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.edges_mut()
            .filter(move |edge| edge.source == node || edge.target == node)
    }

    // Adjacency
    #[inline]
    fn adjacencies(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        self.neighbors(node).map(|n| n)
    }

    // Edges between nodes

    /// Iterator over all edges connecting lhs and rhs, regardless of direction.
    ///
    /// Performance note: Due to restrictions in the API of
    /// [`MatrixGraph`](petgraph_old::matrix_graph::MatrixGraph) this is implemented by
    /// filtering [Self::incident_edges], which may be inefficient for large graphs.
    #[inline]
    fn edges_connecting(
        &self,
        lhs: Self::NodeId,
        rhs: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.incident_edges(lhs).filter(move |edge| {
            (edge.source == lhs && edge.target == rhs) || (edge.source == rhs && edge.target == lhs)
        })
    }

    #[inline]
    fn edges_connecting_mut(
        &mut self,
        lhs: Self::NodeId,
        rhs: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.edges_mut().filter(move |edge| {
            (edge.source == lhs && edge.target == rhs) || (edge.source == rhs && edge.target == lhs)
        })
    }

    // Existence checks
    #[inline]
    fn contains_node(&self, id: Self::NodeId) -> bool {
        self.has_node(id)
    }

    #[inline]
    fn contains_edge(&self, id: Self::EdgeId) -> bool {
        self.has_edge(id.source, id.target)
    }

    #[inline]
    fn is_adjacent(&self, lhs: Self::NodeId, rhs: Self::NodeId) -> bool {
        self.has_edge(lhs, rhs)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.node_count() == 0
    }
}
