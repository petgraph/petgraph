use super::{Cardinality, DensityHint, Graph};
use crate::{
    edge::{EdgeMut, EdgeRef},
    node::{NodeMut, NodeRef},
};

pub trait DirectedGraph: Graph {
    fn density_hint(&self) -> DensityHint {
        DensityHint::Sparse
    }

    // Cardinality
    fn cardinality(&self) -> Cardinality {
        Cardinality {
            order: self.node_count(),
            size: self.edge_count(),
        }
    }

    fn node_count(&self) -> usize {
        self.nodes().count()
    }
    fn edge_count(&self) -> usize {
        self.edges().count()
    }

    // Node Iteration

    fn nodes(&self) -> impl Iterator<Item = NodeRef<'_, Self>>;
    fn nodes_mut(&mut self) -> impl Iterator<Item = NodeMut<'_, Self>>;

    /// Nodes with degree 0 (no incident edges).
    fn isolated_nodes(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        self.nodes().filter(|node| self.degree(node.id) == 0)
    }

    // Edge iteration

    fn edges(&self) -> impl Iterator<Item = EdgeRef<'_, Self>>;
    fn edges_mut(&mut self) -> impl Iterator<Item = EdgeMut<'_, Self>>;

    // Lookup

    fn node(&self, id: Self::NodeId) -> Option<NodeRef<'_, Self>> {
        self.nodes().find(|node| node.id == id)
    }

    fn node_mut(&mut self, id: Self::NodeId) -> Option<NodeMut<'_, Self>> {
        self.nodes_mut().find(|node| node.id == id)
    }

    fn edge(&self, id: Self::EdgeId) -> Option<EdgeRef<'_, Self>> {
        self.edges().find(|edge| edge.id == id)
    }

    fn edge_mut(&mut self, id: Self::EdgeId) -> Option<EdgeMut<'_, Self>> {
        self.edges_mut().find(|edge| edge.id == id)
    }

    // Degree

    fn in_degree(&self, node: Self::NodeId) -> usize {
        self.incoming_edges(node).count()
    }

    fn out_degree(&self, node: Self::NodeId) -> usize {
        self.outgoing_edges(node).count()
    }

    fn degree(&self, node: Self::NodeId) -> usize {
        self.in_degree(node) + self.out_degree(node)
    }

    // Edges by direction

    fn incoming_edges(&self, node: Self::NodeId) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.edges().filter(move |edge| edge.target == node)
    }

    fn incoming_edges_mut(
        &mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.edges_mut().filter(move |edge| edge.target == node)
    }

    fn outgoing_edges(&self, node: Self::NodeId) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.edges().filter(move |edge| edge.source == node)
    }

    fn outgoing_edges_mut(
        &mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.edges_mut().filter(move |edge| edge.source == node)
    }

    fn incident_edges(&self, node: Self::NodeId) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.edges()
            .filter(move |edge| edge.source == node || edge.target == node)
    }

    fn incident_edges_mut(
        &mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.edges_mut()
            .filter(move |edge| edge.source == node || edge.target == node)
    }

    // Adjacency

    fn predecessors(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        self.incoming_edges(node).map(|edge| edge.source)
    }

    fn successors(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        self.outgoing_edges(node).map(|edge| edge.target)
    }

    fn adjacencies(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        self.predecessors(node).chain(self.successors(node))
    }

    // Edges between nodes

    fn edges_between(
        &self,
        source: Self::NodeId,
        target: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.edges()
            .filter(move |edge| edge.source == source && edge.target == target)
    }

    fn edges_between_mut(
        &mut self,
        source: Self::NodeId,
        target: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.edges_mut()
            .filter(move |edge| edge.source == source && edge.target == target)
    }

    fn edges_connecting(
        &self,
        lhs: Self::NodeId,
        rhs: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.edges().filter(move |edge| {
            (edge.source == lhs && edge.target == rhs) || (edge.source == rhs && edge.target == lhs)
        })
    }

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

    fn contains_node(&self, node: Self::NodeId) -> bool {
        self.node(node).is_some()
    }

    fn contains_edge(&self, edge: Self::EdgeId) -> bool {
        self.edge(edge).is_some()
    }

    fn is_adjacent(&self, source: Self::NodeId, target: Self::NodeId) -> bool {
        self.edges_between(source, target).next().is_some()
    }

    fn is_empty(&self) -> bool {
        self.node_count() == 0
    }

    // Sources and sinks

    /// Nodes with `in_degree = 0`.
    fn sources(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        self.nodes().filter(|node| self.in_degree(node.id) == 0)
    }

    /// Nodes with `out_degree = 0`.
    fn sinks(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        self.nodes().filter(|node| self.out_degree(node.id) == 0)
    }
}
