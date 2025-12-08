use super::Cardinality;
use crate::{density_hint::DensityHint, edge::Edge as _, graph::Graph, node::Node as _};

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

    fn nodes(&self) -> impl Iterator<Item = Self::NodeRef<'_>>;
    fn nodes_mut(&mut self) -> impl Iterator<Item = Self::NodeMut<'_>>;

    /// Nodes with degree 0 (no incident edges).
    fn isolated_nodes(&self) -> impl Iterator<Item = Self::NodeRef<'_>> {
        self.nodes().filter(|n| self.degree(n.id()) == 0)
    }

    // Edge iteration

    fn edges(&self) -> impl Iterator<Item = Self::EdgeRef<'_>>;
    fn edges_mut(&mut self) -> impl Iterator<Item = Self::EdgeMut<'_>>;

    // Lookup

    fn node(&self, id: Self::NodeId) -> Option<Self::NodeRef<'_>> {
        self.nodes().find(|n| n.id() == id)
    }

    fn node_mut(&mut self, id: Self::NodeId) -> Option<Self::NodeMut<'_>> {
        self.nodes_mut().find(|n| n.id() == id)
    }

    fn edge(&self, id: Self::EdgeId) -> Option<Self::EdgeRef<'_>> {
        self.edges().find(|e| e.id() == id)
    }

    fn edge_mut(&mut self, id: Self::EdgeId) -> Option<Self::EdgeMut<'_>> {
        self.edges_mut().find(|e| e.id() == id)
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

    fn incoming_edges(&self, node: Self::NodeId) -> impl Iterator<Item = Self::EdgeRef<'_>> {
        self.edges().filter(move |e| e.target() == node)
    }

    fn incoming_edges_mut(
        &mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeMut<'_>> {
        self.edges_mut().filter(move |e| e.target() == node)
    }

    fn outgoing_edges(&self, node: Self::NodeId) -> impl Iterator<Item = Self::EdgeRef<'_>> {
        self.edges().filter(move |e| e.source() == node)
    }

    fn outgoing_edges_mut(
        &mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeMut<'_>> {
        self.edges_mut().filter(move |e| e.source() == node)
    }

    fn incident_edges(&self, node: Self::NodeId) -> impl Iterator<Item = Self::EdgeRef<'_>> {
        self.edges()
            .filter(move |e| e.source() == node || e.target() == node)
    }

    fn incident_edges_mut(
        &mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeMut<'_>> {
        self.edges_mut()
            .filter(move |e| e.source() == node || e.target() == node)
    }

    // Adjacency

    fn predecessors(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        self.incoming_edges(node).map(|e| e.source())
    }

    fn successors(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        self.outgoing_edges(node).map(|e| e.target())
    }

    fn adjacencies(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        self.predecessors(node).chain(self.successors(node))
    }

    // Edges between nodes

    fn edges_between(
        &self,
        source: Self::NodeId,
        target: Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeRef<'_>> {
        self.edges()
            .filter(move |e| e.source() == source && e.target() == target)
    }

    fn edges_between_mut(
        &mut self,
        source: Self::NodeId,
        target: Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeMut<'_>> {
        self.edges_mut()
            .filter(move |e| e.source() == source && e.target() == target)
    }

    fn edges_connecting(
        &self,
        a: Self::NodeId,
        b: Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeRef<'_>> {
        self.edges().filter(move |e| {
            (e.source() == a && e.target() == b) || (e.source() == b && e.target() == a)
        })
    }

    fn edges_connecting_mut(
        &mut self,
        a: Self::NodeId,
        b: Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeMut<'_>> {
        self.edges_mut().filter(move |e| {
            (e.source() == a && e.target() == b) || (e.source() == b && e.target() == a)
        })
    }
}
