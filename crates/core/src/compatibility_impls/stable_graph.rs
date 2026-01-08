use alloc::vec::Vec;
use core::fmt::Display;

use petgraph_old::{
    Directed, Direction, Undirected,
    csr::IndexType,
    graph::{EdgeIndex, Graph as OldGraph, NodeIndex},
    prelude::StableGraph,
    visit::{EdgeRef as _, IntoNodeReferences},
};

use crate::{
    edge::{EdgeMut, EdgeRef},
    graph::{Cardinality, DensityHint, DirectedGraph, Graph as NewGraph, UndirectedGraph},
    node::{NodeMut, NodeRef},
};

impl<N, E, Ty, Ix: IndexType + Display> NewGraph for StableGraph<N, E, Ty, Ix> {
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
    type EdgeId = EdgeIndex<Ix>;
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

impl<N, E, Ix: IndexType + Display> DirectedGraph for StableGraph<N, E, Directed, Ix> {
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
        StableGraph::node_count(self)
    }

    #[inline]
    fn edge_count(&self) -> usize {
        StableGraph::edge_count(self)
    }

    // Node Iteration

    fn nodes(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        self.node_references()
            .map(|(id, data)| NodeRef::<'_, Self> { id, data })
    }

    fn nodes_mut<'a>(&'a mut self) -> impl Iterator<Item = NodeMut<'a, Self>> {
        // This returns the correct NodeIndexes because node_weights_mut() guarantees the values
        // to be in order and Graph uses gap-less NodeIndexes from 0 to n-1 where n is the number of
        // nodes.
        self.node_weights_mut()
            .enumerate()
            .map(|(idx, data)| NodeMut::<'_, Self> {
                id: NodeIndex::new(idx),
                data,
            })
    }

    /// Nodes with degree 0 (no incident edges).
    #[inline]
    fn isolated_nodes(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        self.nodes().filter(|node| self.degree(node.id) == 0)
    }

    // Edge iteration

    fn edges(&self) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.edge_indices()
            .zip(self.edge_weights())
            .map(|(edge_id, data)| {
                let (source, target) = self.edge_endpoints(edge_id).unwrap();
                EdgeRef::<'_, Self> {
                    id: edge_id,
                    source,
                    target,
                    data,
                }
            })
    }

    /// Mutable iterator over all edges.
    ///
    /// Performance note: Due to restrictions in the API of [`Graph`](petgraph_old::Graph) this is
    /// implemented by first collecting edges endpoints and ids into a Vec and then combining
    /// that with the mutable edge weights iterator. Therefore, this may be inefficient and/or
    /// use more memory than expected for large graphs.
    fn edges_mut(&mut self) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        // This returns the EdgeWeights corresponding to the correct EdgeIndexes because StableGraph
        // internally stores them in order.
        let edge_info = self
            .edge_indices()
            .map(|edge_id| {
                let (source, target) = self.edge_endpoints(edge_id).unwrap();
                (edge_id, source, target)
            })
            .collect::<Vec<_>>();
        self.edge_weights_mut().zip(edge_info.into_iter()).map(
            |(data, (edge_id, source, target))| EdgeMut::<'_, Self> {
                id: edge_id,
                source,
                target,
                data,
            },
        )
    }

    // Lookup
    #[inline]
    fn node(&self, id: Self::NodeId) -> Option<NodeRef<'_, Self>> {
        self.node_weight(id)
            .map(|data| NodeRef::<'_, Self> { id, data })
    }

    #[inline]
    fn node_mut(&mut self, id: Self::NodeId) -> Option<NodeMut<'_, Self>> {
        self.node_weight_mut(id)
            .map(|data| NodeMut::<'_, Self> { id, data })
    }

    #[inline]
    fn edge(&self, id: Self::EdgeId) -> Option<EdgeRef<'_, Self>> {
        self.edge_weight(id).map(|data| {
            let (source, target) = self.edge_endpoints(id).unwrap();
            EdgeRef::<'_, Self> {
                id,
                source,
                target,
                data,
            }
        })
    }

    #[inline]
    fn edge_mut(&mut self, id: Self::EdgeId) -> Option<EdgeMut<'_, Self>> {
        let (source, target) = self.edge_endpoints(id).unwrap();
        self.edge_weight_mut(id).map(|data| EdgeMut::<'_, Self> {
            id,
            source,
            target,
            data,
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
        self.neighbors_undirected(node).count()
    }

    // Edges by direction
    #[inline]
    fn incoming_edges(&self, node: Self::NodeId) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.edges_directed(node, Direction::Incoming)
            .map(|e| EdgeRef::<'_, Self> {
                id: e.id(),
                source: e.source(),
                target: e.target(),
                data: e.weight(),
            })
    }

    /// Mutable iterator over incoming edges. That is, edges where `target == node`.
    ///
    /// Performance note: Due to restrictions in the API of [`Graph`](petgraph_old::Graph) this is
    /// implemented by filtering [Self::edges_mut], which may be inefficient for large graphs.
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
            .map(|e| EdgeRef::<'_, Self> {
                id: e.id(),
                source: e.source(),
                target: e.target(),
                data: e.weight(),
            })
    }

    /// Mutable iterator over outgoing edges. That is, edges where `source == node`.
    ///
    /// Performance note: Due to restrictions in the API of [`Graph`](petgraph_old::Graph) this is
    /// implemented by filtering [Self::edges_mut], which may be inefficient for large graphs.
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
            .map(|e| EdgeRef::<'_, Self> {
                id: e.id(),
                source: e.source(),
                target: e.target(),
                data: e.weight(),
            })
    }

    /// Mutable iterator over incident edges of node. That is, all edges where node is either
    /// source or target.
    ///
    /// Performance note: Due to restrictions in the API of [`Graph`](petgraph_old::Graph) this is
    /// implemented by filtering [Self::edges_mut], which may be inefficient for large graphs.
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
        self.neighbors_undirected(node).map(|n| n)
    }

    // Edges between nodes
    #[inline]
    fn edges_between(
        &self,
        source: Self::NodeId,
        target: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.edges_connecting(source, target)
            .map(|edge| EdgeRef::<'_, Self> {
                id: edge.id(),
                source: edge.source(),
                target: edge.target(),
                data: edge.weight(),
            })
    }

    /// Mutable iterator over all edges between source and target. Note that this only considers
    /// edges where the provided source and target are actually the source and target of the edge.
    /// For an undirected equivalent, see [Self::edges_connecting_mut].
    ///
    /// Performance note: Due to restrictions in the API of [`Graph`](petgraph_old::Graph) this is
    /// implemented by filtering [Self::edges_mut], which may be inefficient for large graphs.
    #[inline]
    fn edges_between_mut(
        &mut self,
        source: Self::NodeId,
        target: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.edges_mut()
            .filter(move |edge| edge.source == source && edge.target == target)
    }

    #[inline]
    fn edges_connecting(
        &self,
        lhs: Self::NodeId,
        rhs: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.edges_connecting(lhs, rhs)
            .chain(self.edges_connecting(rhs, lhs))
            .map(|edge| EdgeRef::<'_, Self> {
                id: edge.id(),
                source: edge.source(),
                target: edge.target(),
                data: edge.weight(),
            })
    }

    /// Mutable iterator over all edges connecting lhs and rhs, regardless of direction.
    ///
    /// Performance note: Due to restrictions in the API of [`Graph`](petgraph_old::Graph) this is
    /// implemented by filtering [Self::edges_mut], which may be inefficient for large graphs.
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
        self.contains_node(node)
    }

    #[inline]
    fn contains_edge(&self, edge: Self::EdgeId) -> bool {
        self.edge_weight(edge).is_some()
    }

    #[inline]
    fn is_adjacent(&self, source: Self::NodeId, target: Self::NodeId) -> bool {
        self.contains_edge(source, target)
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

impl<N, E, Ix: IndexType + Display> UndirectedGraph for StableGraph<N, E, Undirected, Ix> {
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
        self.node_count()
    }

    #[inline]
    fn edge_count(&self) -> usize {
        self.edge_count()
    }

    // Node iteration

    fn nodes(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        self.node_references()
            .map(|(id, data)| NodeRef::<'_, Self> { id, data })
    }

    fn nodes_mut(&mut self) -> impl Iterator<Item = NodeMut<'_, Self>> {
        // This returns the correct NodeIndexes because node_weights_mut() guarantees the values
        // to be in order and Graph uses gap-less NodeIndexes from 0 to n-1 where n is the number of
        // nodes.
        self.node_weights_mut()
            .enumerate()
            .map(|(idx, data)| NodeMut::<'_, Self> {
                id: NodeIndex::new(idx),
                data,
            })
    }

    #[inline]
    fn isolated_nodes(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        self.nodes().filter(|node| self.degree(node.id) == 0)
    }

    // Edge iteration

    fn edges(&self) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.edge_indices()
            .zip(self.edge_weights())
            .map(|(edge_id, data)| {
                let (source, target) = self.edge_endpoints(edge_id).unwrap();
                EdgeRef::<'_, Self> {
                    id: edge_id,
                    source,
                    target,
                    data,
                }
            })
    }

    fn edges_mut(&mut self) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        // This returns the EdgeWeights corresponding to the correct EdgeIndexes because StableGraph
        // internally stores them in order.
        let edge_info = self
            .edge_indices()
            .map(|edge_id| {
                let (source, target) = self.edge_endpoints(edge_id).unwrap();
                (edge_id, source, target)
            })
            .collect::<Vec<_>>();
        self.edge_weights_mut().zip(edge_info.into_iter()).map(
            |(data, (edge_id, source, target))| EdgeMut::<'_, Self> {
                id: edge_id,
                source,
                target,
                data,
            },
        )
    }

    // Lookup
    #[inline]
    fn node(&self, id: Self::NodeId) -> Option<NodeRef<'_, Self>> {
        self.node_weight(id)
            .map(|data| NodeRef::<'_, Self> { id, data })
    }

    #[inline]
    fn node_mut(&mut self, id: Self::NodeId) -> Option<NodeMut<'_, Self>> {
        self.node_weight_mut(id)
            .map(|data| NodeMut::<'_, Self> { id, data })
    }

    #[inline]
    fn edge(&self, id: Self::EdgeId) -> Option<EdgeRef<'_, Self>> {
        let (source, target) = self.edge_endpoints(id).unwrap();
        self.edge_weight(id).map(|data| EdgeRef::<'_, Self> {
            id,
            source,
            target,
            data,
        })
    }

    #[inline]
    fn edge_mut(&mut self, id: Self::EdgeId) -> Option<EdgeMut<'_, Self>> {
        let (source, target) = self.edge_endpoints(id).unwrap();
        self.edge_weight_mut(id).map(|data| EdgeMut::<'_, Self> {
            id,
            source,
            target,
            data,
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
        self.edges_directed(node, Direction::Incoming)
            .chain(self.edges_directed(node, Direction::Outgoing))
            .map(|e| EdgeRef::<'_, Self> {
                id: e.id(),
                source: e.source(),
                target: e.target(),
                data: e.weight(),
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
    #[inline]
    fn edges_connecting(
        &self,
        lhs: Self::NodeId,
        rhs: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.edges_connecting(lhs, rhs)
            .map(|edge| EdgeRef::<'_, Self> {
                id: edge.id(),
                source: edge.source(),
                target: edge.target(),
                data: edge.weight(),
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
        self.contains_node(id)
    }

    #[inline]
    fn contains_edge(&self, id: Self::EdgeId) -> bool {
        self.edge_weight(id).is_some()
    }

    #[inline]
    fn is_adjacent(&self, lhs: Self::NodeId, rhs: Self::NodeId) -> bool {
        self.contains_edge(lhs, rhs)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.node_count() == 0
    }
}
