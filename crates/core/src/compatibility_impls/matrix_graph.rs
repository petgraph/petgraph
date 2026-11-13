use core::{
    fmt::{self, Debug, Display},
    hash::BuildHasher,
};

#[cfg(feature = "hashbrown")]
use hashbrown::{HashMap, HashSet};
#[cfg(feature = "hashbrown")]
use petgraph_old::visit::NodeIndexable;
use petgraph_old::{
    Directed, Direction, EdgeType, Undirected,
    csr::IndexType,
    matrix_graph::{MatrixError, MatrixGraph, NodeIndex, Nullable},
    visit::IntoNodeReferences,
};

use self::__private::CommonMatrixGraph;
#[cfg(feature = "hashbrown")]
use crate::graph::storable::{
    StorableGraph,
    containers::{DefaultHashMapHashbrown, DefaultVec},
};
use crate::{
    edge::{EdgeMut, EdgeRef},
    graph::{
        Cardinality, DensityHint, DirectedGraph, EditableGraph, Graph as NewGraph, UndirectedGraph,
    },
    id::Id,
    node::{NodeMut, NodeRef},
};

/// `EdgeId` type for the compatibility implementation of the [`Graph`](crate::graph::Graph) trait
/// for directed [`MatrixGraph`](petgraph_old::matrix_graph::MatrixGraph).
///
/// For the undirected version, see [`UnMatrixGraphEdgeId`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DirMatrixGraphEdgeId<Ix> {
    source: NodeIndex<Ix>,
    target: NodeIndex<Ix>,
    _private: (),
}

impl<Ix> DirMatrixGraphEdgeId<Ix> {
    pub const fn new(source: NodeIndex<Ix>, target: NodeIndex<Ix>) -> Self {
        Self {
            source,
            target,
            _private: (),
        }
    }

    pub const fn source(&self) -> &NodeIndex<Ix> {
        &self.source
    }

    pub const fn target(&self) -> &NodeIndex<Ix> {
        &self.target
    }
}

impl<Ix: Display> Display for DirMatrixGraphEdgeId<Ix> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Edge({}, {})", self.source, self.target)
    }
}

impl<Ix: Display + IndexType> Id for DirMatrixGraphEdgeId<Ix> {}

impl<N, E, S, Null: Nullable<Wrapped = E>, Ix: IndexType + Display> NewGraph
    for MatrixGraph<N, E, S, Directed, Null, Ix>
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
    type EdgeId = DirMatrixGraphEdgeId<Ix>;
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

impl<N, E, S, Null: Nullable<Wrapped = E>, Ix: IndexType + Display> CommonMatrixGraph
    for MatrixGraph<N, E, S, Directed, Null, Ix>
{
    #[inline]
    fn edge_id(source: NodeIndex<Ix>, target: NodeIndex<Ix>) -> Self::EdgeId {
        DirMatrixGraphEdgeId::new(source, target)
    }

    #[inline]
    fn edge_endpoints(id: Self::EdgeId) -> (NodeIndex<Ix>, NodeIndex<Ix>) {
        (id.source, id.target)
    }
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
        Self::node_count(self)
    }

    #[inline]
    fn edge_count(&self) -> usize {
        Self::edge_count(self)
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
    /// this is implemented by filtering [`Self::nodes`], which may be inefficient for large graphs.
    #[inline]
    fn isolated_nodes(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        self.nodes().filter(|node| self.degree(node.id) == 0)
    }

    // Edge iteration

    fn edges(&self) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.all_edges()
            .map(|(source, target, data)| EdgeRef::<'_, Self> {
                id: DirMatrixGraphEdgeId::new(source, target),
                source,
                target,
                data,
            })
    }

    fn edges_mut(&mut self) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.all_edges_mut()
            .map(|(source, target, data)| EdgeMut::<'_, Self> {
                id: DirMatrixGraphEdgeId::new(source, target),
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
            .map(|(target, source, data)| EdgeRef::<'_, Self> {
                id: DirMatrixGraphEdgeId::new(source, target),
                source,
                target,
                data,
            })
    }

    /// Mutable iterator over incoming edges. That is, edges where `target == node`.
    ///
    /// Performance note: Due to restrictions in the API of
    /// [`MatrixGraph`](petgraph_old::matrix_graph::MatrixGraph) this is implemented by
    /// filtering [`Self::edges_mut`], which may be inefficient for large graphs.
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
                id: DirMatrixGraphEdgeId::new(source, target),
                source,
                target,
                data,
            })
    }

    /// Mutable iterator over outgoing edges. That is, edges where `source == node`.
    ///
    /// Performance note: Due to restrictions in the API of
    /// [`MatrixGraph`](petgraph_old::matrix_graph::MatrixGraph) this is implemented by
    /// filtering [`Self::edges_mut`], which may be inefficient for large graphs.
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
            .map(|(target, source, data)| EdgeRef::<'_, Self> {
                id: DirMatrixGraphEdgeId::new(source, target),
                source,
                target,
                data,
            })
            .chain(self.edges_directed(node, Direction::Outgoing).filter_map(
                move |(source, target, data)| {
                    // Count self loops only once
                    if target != node {
                        Some(EdgeRef::<'_, Self> {
                            id: DirMatrixGraphEdgeId::new(source, target),
                            source,
                            target,
                            data,
                        })
                    } else {
                        None
                    }
                },
            ))
    }

    /// Mutable iterator over incident edges of node. That is, all edges where node is either
    /// source or target.
    ///
    /// Performance note: Due to restrictions in the API of
    /// [`MatrixGraph`](petgraph_old::matrix_graph::MatrixGraph) this is implemented by
    /// filtering [`Self::edges_mut`], which may be inefficient for large graphs.
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
    }

    #[inline]
    fn successors(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        self.neighbors_directed(node, Direction::Outgoing)
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
            .filter(move |(_s, curr_target_idx, _d)| *curr_target_idx == target)
            .map(
                move |(curr_source_idx, curr_target_idx, data)| EdgeRef::<'_, Self> {
                    id: DirMatrixGraphEdgeId::new(curr_source_idx, curr_target_idx),
                    source: curr_source_idx,
                    target: curr_target_idx,
                    data,
                },
            )
    }

    /// Mutable iterator over all edges between source and target. Note that this only considers
    /// edges where the provided source and target are actually the source and target of the edge.
    /// For an undirected equivalent, see [`Self::edges_connecting_mut`].
    ///
    /// Performance note: Due to restrictions in the API of
    /// [`MatrixGraph`](petgraph_old::matrix_graph::MatrixGraph) this is implemented by filtering
    /// [`Self::edges_mut`], which may be inefficient for large graphs.
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
    /// chaining two [`Self::edges_between`] calls, which may be inefficient for large graphs.
    ///
    /// Note that if lhs == rhs, then self-loops are returned only once.
    #[inline]
    fn edges_connecting(
        &self,
        lhs: Self::NodeId,
        rhs: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.edges_between(lhs, rhs).chain(
            self.edges_between(rhs, lhs)
                .filter(move |edge| edge.source != edge.target),
        )
    }

    /// Mutable iterator over all edges connecting lhs and rhs, regardless of direction.
    ///
    /// Performance note: Due to restrictions in the API of
    /// [`MatrixGraph`](petgraph_old::matrix_graph::MatrixGraph) this is implemented by filtering
    /// [`Self::edges_mut`], which may be inefficient for large graphs.
    ///
    /// Note that if lhs == rhs, then self-loops are returned only once.
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

/// `EdgeId` type for the compatibility implementation of the [`Graph`](crate::graph::Graph) trait
/// for undirected [`MatrixGraph`](petgraph_old::matrix_graph::MatrixGraph).
///
/// For the directed version, see [`DirMatrixGraphEdgeId`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnMatrixGraphEdgeId<Ix> {
    source: NodeIndex<Ix>,
    target: NodeIndex<Ix>,
    _private: (),
}

impl<Ix: IndexType> UnMatrixGraphEdgeId<Ix> {
    pub fn new(source: NodeIndex<Ix>, target: NodeIndex<Ix>) -> Self {
        if source.index() <= target.index() {
            Self {
                source,
                target,
                _private: (),
            }
        } else {
            Self {
                source: target,
                target: source,
                _private: (),
            }
        }
    }

    pub fn source(&self) -> &NodeIndex<Ix> {
        &self.source
    }

    pub fn target(&self) -> &NodeIndex<Ix> {
        &self.target
    }
}

impl<Ix: IndexType + Display> Display for UnMatrixGraphEdgeId<Ix> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Edge({}, {})", self.source, self.target)
    }
}

impl<Ix: IndexType + Display> Id for UnMatrixGraphEdgeId<Ix> {}

impl<N, E, S, Null: Nullable<Wrapped = E>, Ix: IndexType + Display> NewGraph
    for MatrixGraph<N, E, S, Undirected, Null, Ix>
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
    type EdgeId = UnMatrixGraphEdgeId<Ix>;
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

impl<N, E, S, Null: Nullable<Wrapped = E>, Ix: IndexType + Display> CommonMatrixGraph
    for MatrixGraph<N, E, S, Undirected, Null, Ix>
{
    #[inline]
    fn edge_id(source: NodeIndex<Ix>, target: NodeIndex<Ix>) -> Self::EdgeId {
        UnMatrixGraphEdgeId::new(source, target)
    }

    #[inline]
    fn edge_endpoints(id: Self::EdgeId) -> (NodeIndex<Ix>, NodeIndex<Ix>) {
        (id.source, id.target)
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
        Self::node_count(self)
    }

    #[inline]
    fn edge_count(&self) -> usize {
        Self::edge_count(self)
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
                id: UnMatrixGraphEdgeId::new(source, target),
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
                id: UnMatrixGraphEdgeId::new(source, target),
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
        // Count self loops twice
        + self.contains_edge(UnMatrixGraphEdgeId::new(node, node)) as usize
    }

    // Incidence
    #[inline]
    fn incident_edges(&self, node: Self::NodeId) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.edges(node)
            .map(|(source, target, data)| EdgeRef::<'_, Self> {
                id: UnMatrixGraphEdgeId::new(source, target),
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
        self.neighbors(node)
    }

    // Edges between nodes

    /// Iterator over all edges connecting lhs and rhs, regardless of direction.
    ///
    /// Performance note: Due to restrictions in the API of
    /// [`MatrixGraph`](petgraph_old::matrix_graph::MatrixGraph) this is implemented by
    /// filtering [`Self::incident_edges`], which may be inefficient for large graphs.
    ///
    /// Note that if lhs == rhs, then self-loops are returned only once.
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceOrTargetNotFound {
    SourceNodeDoesNotExist,
    TargetNodeDoesNotExist,
}

impl Display for SourceOrTargetNotFound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceNodeDoesNotExist => {
                write!(f, "source node does not exist")
            }
            Self::TargetNodeDoesNotExist => {
                write!(f, "target node does not exist")
            }
        }
    }
}

impl core::error::Error for SourceOrTargetNotFound {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotFoundError<T> {
    pub id: T,
}

impl<T> Display for NotFoundError<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "id {} does not exist", self.id)
    }
}

impl<T: Display + Debug> core::error::Error for NotFoundError<T> {}

impl<N, E, S, Dir, Null, Ix> EditableGraph for MatrixGraph<N, E, S, Dir, Null, Ix>
where
    S: BuildHasher,
    Dir: EdgeType,
    Null: Nullable<Wrapped = E>,
    Ix: IndexType + Display,
    Self: CommonMatrixGraph<NodeId = NodeIndex<Ix>>,
{
    type AddEdgeError = SourceOrTargetNotFound;
    type AddNodeError = MatrixError;
    type OwnedEdgeData = E;
    type OwnedNodeData = N;
    type RemoveEdgeError = NotFoundError<Self::EdgeId>;
    type RemoveNodeError = NotFoundError<NodeIndex<Ix>>;

    fn add_node(&mut self, data: Self::OwnedNodeData) -> Result<Self::NodeId, Self::AddNodeError> {
        self.try_add_node(data)
    }

    fn add_edge(
        &mut self,
        source: Self::NodeId,
        target: Self::NodeId,
        data: Self::OwnedEdgeData,
    ) -> Result<Self::EdgeId, Self::AddEdgeError> {
        if !self.has_node(source) {
            return Err(SourceOrTargetNotFound::SourceNodeDoesNotExist);
        }
        if !self.has_node(target) {
            return Err(SourceOrTargetNotFound::TargetNodeDoesNotExist);
        }
        let _ = self.update_edge(source, target, data);
        Ok(Self::edge_id(source, target))
    }

    fn remove_node(
        &mut self,
        id: Self::NodeId,
    ) -> Result<Self::OwnedNodeData, Self::RemoveNodeError> {
        if !self.has_node(id) {
            return Err(NotFoundError { id });
        }
        Ok(self.remove_node(id))
    }

    fn remove_edge(
        &mut self,
        id: Self::EdgeId,
    ) -> Result<Self::OwnedEdgeData, Self::RemoveEdgeError> {
        let (source, target) = Self::edge_endpoints(id);
        self.try_remove_edge(source, target)
            .ok_or(NotFoundError { id })
    }
}

#[cfg(feature = "hashbrown")]
impl<N, E, S, Null, Ix> StorableGraph for MatrixGraph<N, E, S, Directed, Null, Ix>
where
    S: BuildHasher + Default,
    Null: Nullable<Wrapped = E>,
    Ix: IndexType + Display,
{
    // Change this to a more efficient container once the port_matrix_graph PR lands
    type EdgeDataContainer<Data: Clone> = DefaultHashMapHashbrown<Self::EdgeId, Data, S>;
    type EdgeVisitContainer = HashSet<Self::EdgeId, S>;
    type NodeDataContainer<Data: Clone> = DefaultVec<Data>;
    type NodeVisitContainer = HashSet<NodeIndex<Ix>, S>;

    fn node_visit_container(&self) -> Self::NodeVisitContainer {
        HashSet::default()
    }

    fn node_data_container<Data: Clone>(
        &self,
        default_value: Data,
    ) -> Self::NodeDataContainer<Data> {
        DefaultVec::new_with_capacity(default_value, self.node_bound())
    }

    fn edge_visit_container(&self) -> Self::EdgeVisitContainer {
        HashSet::default()
    }

    fn edge_data_container<Data: Clone>(
        &self,
        default_value: Data,
    ) -> Self::EdgeDataContainer<Data> {
        DefaultHashMapHashbrown {
            map: HashMap::default(),
            default: default_value,
        }
    }
}

// WARNING: This module contains only internal helpers and should not be considered part of the
// public API. It is subject to change at any time.
#[doc(hidden)]
mod __private {
    use core::hash::Hash;

    use crate::graph::Graph;

    /// Functionality shared by directed and undirected
    /// [`MatrixGraph`](petgraph_old::matrix_graph::MatrixGraph)s.
    pub trait CommonMatrixGraph: Graph<EdgeId: Hash + Eq> {
        /// Returns the `EdgeId` of the edge from `source` to `target`.
        fn edge_id(source: Self::NodeId, target: Self::NodeId) -> Self::EdgeId;

        /// Returns the `(source, target)` node ids of the edge with the given `EdgeId`.
        fn edge_endpoints(id: Self::EdgeId) -> (Self::NodeId, Self::NodeId);
    }
}

#[cfg(test)]
mod test {
    use petgraph_old::matrix_graph::MatrixGraph;

    use super::*;
    use crate::{
        test_directed_editable_graph, test_directed_editable_graph_self_loops, test_directed_graph,
        test_directed_graph_self_loops, test_undirected_editable_graph,
        test_undirected_editable_graph_self_loops, test_undirected_graph,
        test_undirected_graph_self_loops,
    };

    mod directed {
        use super::*;

        test_directed_graph!(MatrixGraph::<(), (), foldhash::fast::RandomState, Directed>::new,);
        test_directed_graph_self_loops!(
            MatrixGraph::<(), (), foldhash::fast::RandomState, Directed>::new,
        );

        test_directed_editable_graph!(
            MatrixGraph::<(), (), foldhash::fast::RandomState, Directed>::new,
        );
        test_directed_editable_graph_self_loops!(
            MatrixGraph::<(), (), foldhash::fast::RandomState, Directed>::new,
        );
    }

    mod undirected {
        use super::*;

        test_undirected_graph!(
            MatrixGraph::<(), (), foldhash::fast::RandomState, Undirected>::new_undirected,
        );
        test_undirected_graph_self_loops!(
            MatrixGraph::<(), (), foldhash::fast::RandomState, Undirected>::new_undirected,
        );
        test_undirected_editable_graph!(
            MatrixGraph::<(), (), foldhash::fast::RandomState, Undirected>::new_undirected,
        );
        test_undirected_editable_graph_self_loops!(
            MatrixGraph::<(), (), foldhash::fast::RandomState, Undirected>::new_undirected,
        );
    }
}
