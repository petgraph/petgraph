use core::{
    convert::Infallible,
    fmt::{self, Debug, Display},
    hash::BuildHasher,
};

#[cfg(feature = "hashbrown")]
use hashbrown::{HashMap, HashSet};
use petgraph_old::{
    Directed, Direction, Undirected,
    graphmap::{GraphMap, NodeTrait},
};

#[cfg(feature = "hashbrown")]
use crate::graph::storable::containers::DefaultHashMapHashbrown;
use crate::{
    edge::{EdgeMut, EdgeRef},
    graph::{
        Cardinality, DensityHint, DirectedGraph, EditableGraph, Graph as NewGraph, UndirectedGraph,
        storable::{DataContainer, StorableGraph},
    },
    id::Id,
    node::{NodeMut, NodeRef},
};

/// `EdgeId` type for the compatibility implementation of the [`Graph`](crate::graph::Graph) trait
/// for directed [`GraphMap`](petgraph_old::graphmap::GraphMap).
///
/// This is a wrapper around the source and target node ids of the edge, since `GraphMap` does not
/// have a unique edge id type.
///
/// For the undirected version, see [`UnGraphMapEdgeId`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DirGraphMapEdgeId<N> {
    pub source: N,
    pub target: N,
    _private: core::marker::PhantomData<N>,
}

impl<N> DirGraphMapEdgeId<N> {
    pub const fn new(source: N, target: N) -> Self {
        Self {
            source,
            target,
            _private: core::marker::PhantomData,
        }
    }
}

impl<N: Display> Display for DirGraphMapEdgeId<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Edge({}, {})", self.source, self.target)
    }
}

trait NodeTraitBounds: Display + core::fmt::Debug + Eq + Copy + NodeTrait + Id {}

impl<N: Display + core::fmt::Debug + Eq + Copy + NodeTrait + Id> NodeTraitBounds for N {}

impl<N: NodeTraitBounds> Id for DirGraphMapEdgeId<N> {}

impl<N: NodeTraitBounds, E, S: BuildHasher> NewGraph for GraphMap<N, E, Directed, S> {
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
    type EdgeId = DirGraphMapEdgeId<N>;
    type NodeData<'graph>
        = ()
    where
        Self: 'graph;
    type NodeDataMut<'graph>
        = ()
    where
        Self: 'graph;
    type NodeDataRef<'graph>
        = ()
    where
        Self: 'graph;
    type NodeId = N;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceOrTargetNotFound {
    SourceNodeDoesNotExist,
    TargetNodeDoesNotExist,
}

impl Display for SourceOrTargetNotFound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceOrTargetNotFound::SourceNodeDoesNotExist => {
                write!(f, "source node does not exist")
            }
            SourceOrTargetNotFound::TargetNodeDoesNotExist => {
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

impl<N, E, S> EditableGraph for GraphMap<N, E, Directed, S>
where
    N: NodeTraitBounds,
    S: BuildHasher,
{
    type AddEdgeError = SourceOrTargetNotFound;
    type AddNodeError = Infallible;
    type OwnedEdgeData = E;
    type OwnedNodeData = N;
    type RemoveEdgeError = NotFoundError<DirGraphMapEdgeId<N>>;
    type RemoveNodeError = NotFoundError<N>;

    fn add_node(&mut self, data: Self::OwnedNodeData) -> Result<Self::NodeId, Self::AddNodeError> {
        Ok(self.add_node(data))
    }

    fn add_edge(
        &mut self,
        source: Self::NodeId,
        target: Self::NodeId,
        data: Self::OwnedEdgeData,
    ) -> Result<Self::EdgeId, Self::AddEdgeError> {
        if !self.contains_node(source) {
            return Err(SourceOrTargetNotFound::SourceNodeDoesNotExist);
        }
        if !self.contains_node(target) {
            return Err(SourceOrTargetNotFound::TargetNodeDoesNotExist);
        }
        let _ = self.add_edge(source, target, data);
        Ok(DirGraphMapEdgeId::new(source, target))
    }

    fn remove_node(
        &mut self,
        id: Self::NodeId,
    ) -> Result<Self::OwnedNodeData, Self::RemoveNodeError> {
        if self.remove_node(id) {
            Ok(id)
        } else {
            Err(NotFoundError { id })
        }
    }

    fn remove_edge(
        &mut self,
        id: Self::EdgeId,
    ) -> Result<Self::OwnedEdgeData, Self::RemoveEdgeError> {
        self.remove_edge(id.source, id.target)
            .ok_or(NotFoundError { id })
    }
}

impl<N: NodeTraitBounds, E, S: BuildHasher> DirectedGraph for GraphMap<N, E, Directed, S> {
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
        Self::nodes(self).map(|id| NodeRef::<'_, Self> { id, data: () })
    }

    fn nodes_mut(&mut self) -> impl Iterator<Item = NodeMut<'_, Self>> {
        Self::nodes(self).map(|id| NodeMut::<'_, Self> { id, data: () })
    }

    /// Nodes with degree 0 (no incident edges).
    ///
    /// Due to restrictions in the API of [`GraphMap`](petgraph_old::graphmap::GraphMap)
    /// this is implemented by filtering [`Self::nodes`], which may be inefficient for large graphs.
    #[inline]
    fn isolated_nodes(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        self.nodes()
            .filter(|node| self.degree(*node) == 0)
            .map(|node| NodeRef::<'_, Self> { id: node, data: () })
    }

    // Edge iteration

    fn edges(&self) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.all_edges()
            .map(|(source, target, data)| EdgeRef::<'_, Self> {
                id: DirGraphMapEdgeId::new(source, target),
                source,
                target,
                data,
            })
    }

    fn edges_mut(&mut self) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.all_edges_mut()
            .map(|(source, target, data)| EdgeMut::<'_, Self> {
                id: DirGraphMapEdgeId::new(source, target),
                source,
                target,
                data,
            })
    }

    // Lookup
    #[inline]
    fn node(&self, id: Self::NodeId) -> Option<NodeRef<'_, Self>> {
        if !self.contains_node(id) {
            return None;
        }

        Some(NodeRef::<'_, Self> { id, data: () })
    }

    #[inline]
    fn node_mut(&mut self, id: Self::NodeId) -> Option<NodeMut<'_, Self>> {
        if !self.contains_node(id) {
            return None;
        }

        Some(NodeMut::<'_, Self> { id, data: () })
    }

    #[inline]
    fn edge(&self, id: Self::EdgeId) -> Option<EdgeRef<'_, Self>> {
        let edge_weight = self.edge_weight(id.source, id.target)?;

        Some(EdgeRef::<'_, Self> {
            id,
            source: id.source,
            target: id.target,
            data: edge_weight,
        })
    }

    #[inline]
    fn edge_mut(&mut self, id: Self::EdgeId) -> Option<EdgeMut<'_, Self>> {
        let edge_weight = self.edge_weight_mut(id.source, id.target)?;

        Some(EdgeMut::<'_, Self> {
            id,
            source: id.source,
            target: id.target,
            data: edge_weight,
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
                id: DirGraphMapEdgeId::new(source, target),
                source,
                target,
                data,
            })
    }

    /// Mutable iterator over incoming edges. That is, edges where `target == node`.
    ///
    /// Performance note: Due to restrictions in the API of
    /// [`GraphMap`](petgraph_old::graphmap::GraphMap) this is implemented by
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
                id: DirGraphMapEdgeId::new(source, target),
                source,
                target,
                data,
            })
    }

    /// Mutable iterator over outgoing edges. That is, edges where `source == node`.
    ///
    /// Performance note: Due to restrictions in the API of
    /// [`GraphMap`](petgraph_old::graphmap::GraphMap) this is implemented by
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
            .chain(self.edges_directed(node, Direction::Outgoing))
            .map(|(source, target, data)| EdgeRef::<'_, Self> {
                id: DirGraphMapEdgeId::new(source, target),
                source,
                target,
                data,
            })
    }

    /// Mutable iterator over incident edges of node. That is, all edges where node is either
    /// source or target.
    ///
    /// Performance note: Due to restrictions in the API of
    /// [`GraphMap`](petgraph_old::graphmap::GraphMap) this is implemented by
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
            .map(|(source, _target, _data)| source)
            .chain(
                self.edges_directed(node, Direction::Outgoing)
                    .map(|(_source, target, _data)| target),
            )
    }

    // Edges between nodes

    /// Iterator over all edges between source and target. Note that this only considers edges
    /// where the provided source and target are actually the source and target of the edge.
    ///
    /// Performance note: Due to restrictions in the API of
    /// [`GraphMap`](petgraph_old::graphmap::GraphMap) this is implemented by
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
                    id: DirGraphMapEdgeId::new(curr_source_idx, curr_target_idx),
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
    /// [`GraphMap`](petgraph_old::graphmap::GraphMap) this is implemented by filtering
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
    /// [`GraphMap`](petgraph_old::graphmap::GraphMap) this is implemented by
    /// chaining two [`Self::edges_between`] calls, which may be inefficient for large graphs.
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
    /// [`GraphMap`](petgraph_old::graphmap::GraphMap) this is implemented by filtering
    /// [`Self::edges_mut`], which may be inefficient for large graphs.
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
        Self::contains_node(self, node)
    }

    #[inline]
    fn contains_edge(&self, edge: Self::EdgeId) -> bool {
        Self::contains_edge(self, edge.source, edge.target)
    }

    #[inline]
    fn is_adjacent(&self, source: Self::NodeId, target: Self::NodeId) -> bool {
        Self::contains_edge(self, source, target)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.node_count() == 0
    }

    // Sources and sinks

    /// Nodes with `in_degree = 0`.
    #[inline]
    fn sources(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        self.nodes()
            .filter(|node| self.in_degree(*node) == 0)
            .map(|node| NodeRef::<'_, Self> { id: node, data: () })
    }

    /// Nodes with `out_degree = 0`.
    #[inline]
    fn sinks(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        self.nodes()
            .filter(|node| self.out_degree(*node) == 0)
            .map(|node| NodeRef::<'_, Self> { id: node, data: () })
    }
}

/// `EdgeId` type for the compatibility implementation of the [`Graph`](crate::graph::Graph) trait
/// for undirected [`GraphMap`](petgraph_old::graphmap::GraphMap).
///
/// This is a wrapper around the source and target node ids of the edge, since `GraphMap` does not
/// have a unique edge id type.
///
/// When creating an instance of this type, the source and target node
/// ids will be sorted so that the smaller node id is always the source and the larger node id is
/// always the target. This ensures that the same (undirected) edge is always represented by the
/// same `UnGraphMapEdgeId` instance, regardless of the order of the node ids when creating it.
///
/// For the directed version, see [`DirGraphMapEdgeId`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnGraphMapEdgeId<N> {
    pub source: N,
    pub target: N,
    _private: core::marker::PhantomData<N>,
}

impl<N: Ord> UnGraphMapEdgeId<N> {
    pub fn new(source: N, target: N) -> Self {
        let (source, target) = if source <= target {
            (source, target)
        } else {
            (target, source)
        };
        Self {
            source,
            target,
            _private: core::marker::PhantomData,
        }
    }
}

impl<N: Display + Ord> Display for UnGraphMapEdgeId<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.source <= self.target {
            write!(f, "Edge({}, {})", self.source, self.target)
        } else {
            write!(f, "Edge({}, {})", self.target, self.source)
        }
    }
}

impl<N: NodeTraitBounds> Id for UnGraphMapEdgeId<N> {}

impl<N: NodeTraitBounds, E, S: BuildHasher> NewGraph for GraphMap<N, E, Undirected, S> {
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
    type EdgeId = UnGraphMapEdgeId<N>;
    type NodeData<'graph>
        = ()
    where
        Self: 'graph;
    type NodeDataMut<'graph>
        = ()
    where
        Self: 'graph;
    type NodeDataRef<'graph>
        = ()
    where
        Self: 'graph;
    type NodeId = N;
}

impl<N, E, S> EditableGraph for GraphMap<N, E, Undirected, S>
where
    N: NodeTraitBounds,
    S: BuildHasher,
{
    type AddEdgeError = SourceOrTargetNotFound;
    type AddNodeError = Infallible;
    type OwnedEdgeData = E;
    type OwnedNodeData = N;
    type RemoveEdgeError = NotFoundError<UnGraphMapEdgeId<N>>;
    type RemoveNodeError = NotFoundError<N>;

    fn add_node(&mut self, data: Self::OwnedNodeData) -> Result<Self::NodeId, Self::AddNodeError> {
        Ok(self.add_node(data))
    }

    fn add_edge(
        &mut self,
        source: Self::NodeId,
        target: Self::NodeId,
        data: Self::OwnedEdgeData,
    ) -> Result<Self::EdgeId, Self::AddEdgeError> {
        if !self.contains_node(source) {
            return Err(SourceOrTargetNotFound::SourceNodeDoesNotExist);
        }
        if !self.contains_node(target) {
            return Err(SourceOrTargetNotFound::TargetNodeDoesNotExist);
        }
        let _ = self.add_edge(source, target, data);
        Ok(UnGraphMapEdgeId::new(source, target))
    }

    fn remove_node(
        &mut self,
        id: Self::NodeId,
    ) -> Result<Self::OwnedNodeData, Self::RemoveNodeError> {
        if self.remove_node(id) {
            Ok(id)
        } else {
            Err(NotFoundError { id })
        }
    }

    fn remove_edge(
        &mut self,
        id: Self::EdgeId,
    ) -> Result<Self::OwnedEdgeData, Self::RemoveEdgeError> {
        self.remove_edge(id.source, id.target)
            .ok_or(NotFoundError { id })
    }
}

impl<N: NodeTraitBounds, E, S: BuildHasher> UndirectedGraph for GraphMap<N, E, Undirected, S> {
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
        Self::nodes(self).map(|id| NodeRef::<'_, Self> { id, data: () })
    }

    fn nodes_mut(&mut self) -> impl Iterator<Item = NodeMut<'_, Self>> {
        Self::nodes(self).map(|id| NodeMut::<'_, Self> { id, data: () })
    }

    /// Nodes with degree 0 (no incident edges).
    ///
    /// Due to restrictions in the API of [`GraphMap`](petgraph_old::graphmap::GraphMap)
    /// this is implemented by filtering [`Self::nodes`], which may be inefficient for large graphs.
    #[inline]
    fn isolated_nodes(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        self.nodes()
            .filter(|node| self.degree(*node) == 0)
            .map(|node| NodeRef::<'_, Self> { id: node, data: () })
    }

    // Edge iteration

    fn edges(&self) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.all_edges()
            .map(|(source, target, data)| EdgeRef::<'_, Self> {
                id: UnGraphMapEdgeId::new(source, target),
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
                id: UnGraphMapEdgeId::new(source, target),
                source,
                target,
                data,
            })
    }

    // Lookup
    #[inline]
    fn node(&self, id: Self::NodeId) -> Option<NodeRef<'_, Self>> {
        if !self.contains_node(id) {
            return None;
        }

        Some(NodeRef::<'_, Self> { id, data: () })
    }

    #[inline]
    fn node_mut(&mut self, id: Self::NodeId) -> Option<NodeMut<'_, Self>> {
        if !self.contains_node(id) {
            return None;
        }

        Some(NodeMut::<'_, Self> { id, data: () })
    }

    #[inline]
    fn edge(&self, id: Self::EdgeId) -> Option<EdgeRef<'_, Self>> {
        let edge_weight = self.edge_weight(id.source, id.target)?;

        Some(EdgeRef::<'_, Self> {
            id,
            source: id.source,
            target: id.target,
            data: edge_weight,
        })
    }

    #[inline]
    fn edge_mut(&mut self, id: Self::EdgeId) -> Option<EdgeMut<'_, Self>> {
        let edge_weight = self.edge_weight_mut(id.source, id.target)?;

        Some(EdgeMut::<'_, Self> {
            id,
            source: id.source,
            target: id.target,
            data: edge_weight,
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
                id: UnGraphMapEdgeId::new(source, target),
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
    /// [`GraphMap`](petgraph_old::graphmap::GraphMap) this is implemented by
    /// filtering [`Self::incident_edges`], which may be inefficient for large graphs.
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
    fn contains_node(&self, node: Self::NodeId) -> bool {
        Self::contains_node(self, node)
    }

    #[inline]
    fn contains_edge(&self, edge: Self::EdgeId) -> bool {
        Self::contains_edge(self, edge.source, edge.target)
    }

    #[inline]
    fn is_adjacent(&self, source: Self::NodeId, target: Self::NodeId) -> bool {
        Self::contains_edge(self, source, target)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.node_count() == 0
    }
}

pub struct GraphMapEdgeDataContainer<N, E, S>
where
    N: NodeTraitBounds,
    S: BuildHasher,
{
    pub map: HashMap<(N, N), E, S>,
}

impl<N, E, S> DataContainer< for GraphMapEdgeDataContainer<N, E, S>
where
    N: NodeTraitBounds,
    E: Clone,
    S: BuildHasher,
{
    fn get(&self, id: Id) -> &Data {
        self.map
            .get(&(id.source, id.target))
            .expect("Edge data not found")
    }

    fn get_mut(&mut self, id: Id) -> &mut Data {
        todo!()
    }

    fn insert(&mut self, id: Id, data: Data) {
        todo!()
    }

    fn remove(&mut self, id: Id) {
        todo!()
    }

    fn clear(&mut self) {
        todo!()
    }
}

#[cfg(feature = "hashbrown")]
impl<N: NodeTraitBounds, E, Dir, S: BuildHasher> StorableGraph for GraphMap<N, E, Dir, S>
where
    GraphMap<N, E, Dir, S>: NewGraph<NodeId = N>,
{
    type EdgeDataContainer<Data: Clone> =
        GraphMapEdgeDataContainer<(Self::NodeId, Self::NodeId), Data, S>;
    type EdgeVisitContainer = HashSet<(Self::NodeId, Self::NodeId), S>;
    type NodeDataContainer<Data: Clone> = DefaultHashMapHashbrown<Self::NodeId, Data, S>;
    type NodeVisitContainer = HashSet<Self::NodeId, S>;

    fn node_visit_container(&self) -> Self::NodeVisitContainer {
        todo!()
    }

    fn node_data_container<Data: Clone>(
        &self,
        default_value: Data,
    ) -> Self::NodeDataContainer<Data> {
        todo!()
    }

    fn edge_visit_container(&self) -> Self::EdgeVisitContainer {
        todo!()
    }

    fn edge_data_container<Data: Clone>(
        &self,
        default_value: Data,
    ) -> Self::EdgeDataContainer<Data> {
        todo!()
    }
}
