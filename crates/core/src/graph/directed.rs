use crate::{density_hint::DensityHint, edge::Edge as _, graph::Graph, node::Node as _};

pub trait DirectedGraph: Graph {
    fn density_hint(&self) -> DensityHint {
        DensityHint::Sparse
    }

    fn node_count(&self) -> usize {
        self.iter_nodes().count()
    }
    fn edge_count(&self) -> usize {
        self.iter_edges().count()
    }

    fn iter_nodes(&self) -> impl Iterator<Item = Self::NodeRef<'_>>;
    fn iter_nodes_mut(&mut self) -> impl Iterator<Item = Self::NodeMut<'_>>;

    fn iter_edges(&self) -> impl Iterator<Item = Self::EdgeRef<'_>>;
    fn iter_edges_mut(&mut self) -> impl Iterator<Item = Self::EdgeMut<'_>>;

    fn node(&self, id: Self::NodeId) -> Option<Self::NodeRef<'_>> {
        self.iter_nodes().find(|node| node.id() == id)
    }

    fn node_mut(&mut self, id: Self::NodeId) -> Option<Self::NodeMut<'_>> {
        self.iter_nodes_mut().find(|node| node.id() == id)
    }

    fn edge(&self, id: Self::EdgeId) -> Option<Self::EdgeRef<'_>> {
        self.iter_edges().find(|edge| edge.id() == id)
    }

    fn edge_mut(&mut self, id: Self::EdgeId) -> Option<Self::EdgeMut<'_>> {
        self.iter_edges_mut().find(|edge| edge.id() == id)
    }

    fn incoming_edges(&self, node: Self::NodeId) -> impl Iterator<Item = Self::EdgeRef<'_>> {
        self.iter_edges().filter(move |edge| edge.target() == node)
    }

    fn incoming_edges_mut(
        &mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeMut<'_>> {
        self.iter_edges_mut()
            .filter(move |edge| edge.target() == node)
    }

    fn predecessors(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        self.iter_edges()
            .filter(move |edge| edge.target() == node)
            .map(|edge| edge.source())
    }

    fn outgoing_edges(&self, node: Self::NodeId) -> impl Iterator<Item = Self::EdgeRef<'_>> {
        self.iter_edges().filter(move |edge| edge.source() == node)
    }

    fn outgoing_edges_mut(
        &mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeMut<'_>> {
        self.iter_edges_mut()
            .filter(move |edge| edge.source() == node)
    }

    fn successors(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        self.iter_edges()
            .filter(move |edge| edge.source() == node)
            .map(|edge| edge.target())
    }

    fn incident_edges(&self, node: Self::NodeId) -> impl Iterator<Item = Self::EdgeRef<'_>> {
        self.iter_edges()
            .filter(move |edge| edge.source() == node || edge.target() == node)
    }

    fn incident_edges_mut(
        &mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeMut<'_>> {
        self.iter_edges_mut()
            .filter(move |edge| edge.source() == node || edge.target() == node)
    }

    fn edges_between(
        &self,
        source: Self::NodeId,
        target: Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeRef<'_>> {
        self.iter_edges()
            .filter(move |edge| edge.source() == source && edge.target() == target)
    }

    fn edges_between_mut(
        &mut self,
        source: Self::NodeId,
        target: Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeMut<'_>> {
        self.iter_edges_mut()
            .filter(move |edge| edge.source() == source && edge.target() == target)
    }
}
