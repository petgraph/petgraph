use super::{Cardinality, DensityHint, Graph};
use crate::{
    edge::{EdgeMut, EdgeRef},
    node::{NodeMut, NodeRef},
};

pub trait UndirectedGraph: Graph {
    #[inline]
    fn density_hint(&self) -> DensityHint {
        DensityHint::Sparse
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
        self.nodes().count()
    }

    #[inline]
    fn edge_count(&self) -> usize {
        self.edges().count()
    }

    // Node iteration

    fn nodes(&self) -> impl Iterator<Item = NodeRef<'_, Self>>;
    fn nodes_mut(&mut self) -> impl Iterator<Item = NodeMut<'_, Self>>;

    #[inline]
    fn isolated_nodes(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        self.nodes().filter(|node| self.degree(node.id) == 0)
    }

    // Edge iteration

    fn edges(&self) -> impl Iterator<Item = EdgeRef<'_, Self>>;
    fn edges_mut(&mut self) -> impl Iterator<Item = EdgeMut<'_, Self>>;

    // Lookup
    #[inline]
    fn node(&self, id: Self::NodeId) -> Option<NodeRef<'_, Self>> {
        self.nodes().find(|node| node.id == id)
    }

    #[inline]
    fn node_mut(&mut self, id: Self::NodeId) -> Option<NodeMut<'_, Self>> {
        self.nodes_mut().find(|node| node.id == id)
    }

    #[inline]
    fn edge(&self, id: Self::EdgeId) -> Option<EdgeRef<'_, Self>> {
        self.edges().find(|edge| edge.id == id)
    }

    #[inline]
    fn edge_mut(&mut self, id: Self::EdgeId) -> Option<EdgeMut<'_, Self>> {
        self.edges_mut().find(|edge| edge.id == id)
    }

    // Degree
    #[inline]
    fn degree(&self, node: Self::NodeId) -> usize {
        self.incident_edges(node).count()
    }

    // Incidence
    #[inline]
    fn incident_edges(&self, node: Self::NodeId) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.edges()
            .filter(move |edge| edge.source == node || edge.target == node)
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
        self.incident_edges(node).map(move |edge| {
            if edge.source == node {
                edge.target
            } else {
                edge.source
            }
        })
    }

    // Edges between nodes
    #[inline]
    fn edges_connecting(
        &self,
        lhs: Self::NodeId,
        rhs: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.edges().filter(move |edge| {
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
        self.node(id).is_some()
    }

    #[inline]
    fn contains_edge(&self, id: Self::EdgeId) -> bool {
        self.edge(id).is_some()
    }

    #[inline]
    fn is_adjacent(&self, lhs: Self::NodeId, rhs: Self::NodeId) -> bool {
        self.edges_connecting(lhs, rhs).next().is_some()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.node_count() == 0
    }
}

impl<G> UndirectedGraph for &mut G
where
    G: UndirectedGraph,
{
    #[inline]
    fn density_hint(&self) -> DensityHint {
        G::density_hint(self)
    }

    #[inline]
    fn cardinality(&self) -> Cardinality {
        G::cardinality(self)
    }

    #[inline]
    fn node_count(&self) -> usize {
        G::node_count(self)
    }

    #[inline]
    fn edge_count(&self) -> usize {
        G::edge_count(self)
    }

    #[inline]
    fn nodes(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        G::nodes(self)
    }

    #[inline]
    fn nodes_mut(&mut self) -> impl Iterator<Item = NodeMut<'_, Self>> {
        G::nodes_mut(self)
    }

    #[inline]
    fn isolated_nodes(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        G::isolated_nodes(self)
    }

    #[inline]
    fn edges(&self) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        G::edges(self)
    }

    #[inline]
    fn edges_mut(&mut self) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        G::edges_mut(self)
    }

    #[inline]
    fn node(&self, id: Self::NodeId) -> Option<NodeRef<'_, Self>> {
        G::node(self, id)
    }

    #[inline]
    fn node_mut(&mut self, id: Self::NodeId) -> Option<NodeMut<'_, Self>> {
        G::node_mut(self, id)
    }

    #[inline]
    fn edge(&self, id: Self::EdgeId) -> Option<EdgeRef<'_, Self>> {
        G::edge(self, id)
    }

    #[inline]
    fn edge_mut(&mut self, id: Self::EdgeId) -> Option<EdgeMut<'_, Self>> {
        G::edge_mut(self, id)
    }

    #[inline]
    fn degree(&self, node: Self::NodeId) -> usize {
        G::degree(self, node)
    }

    #[inline]
    fn incident_edges(&self, node: Self::NodeId) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        G::incident_edges(self, node)
    }

    #[inline]
    fn incident_edges_mut(
        &mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        G::incident_edges_mut(self, node)
    }

    #[inline]
    fn adjacencies(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        G::adjacencies(self, node)
    }

    #[inline]
    fn edges_connecting(
        &self,
        lhs: Self::NodeId,
        rhs: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        G::edges_connecting(self, lhs, rhs)
    }

    #[inline]
    fn edges_connecting_mut(
        &mut self,
        lhs: Self::NodeId,
        rhs: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        G::edges_connecting_mut(self, lhs, rhs)
    }

    #[inline]
    fn contains_node(&self, id: Self::NodeId) -> bool {
        G::contains_node(self, id)
    }

    #[inline]
    fn contains_edge(&self, id: Self::EdgeId) -> bool {
        G::contains_edge(self, id)
    }

    #[inline]
    fn is_adjacent(&self, lhs: Self::NodeId, rhs: Self::NodeId) -> bool {
        G::is_adjacent(self, lhs, rhs)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        G::is_empty(self)
    }
}
