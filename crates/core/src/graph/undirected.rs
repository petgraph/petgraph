use super::{Cardinality, DensityHint, Graph};
use crate::{edge::Edge as _, node::Node as _};

pub trait UndirectedGraph: Graph {
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

    // Node iteration

    fn nodes(&self) -> impl Iterator<Item = Self::NodeRef<'_>>;
    fn nodes_mut(&mut self) -> impl Iterator<Item = Self::NodeMut<'_>>;

    fn isolated_nodes(&self) -> impl Iterator<Item = Self::NodeRef<'_>> {
        self.nodes().filter(|n| self.degree(n.id()) == 0)
    }

    // Edge iteration

    fn edges(&self) -> impl Iterator<Item = Self::EdgeRef<'_>>;
    fn edges_mut(&mut self) -> impl Iterator<Item = Self::EdgeMut<'_>>;

    // Lookup

    fn node(&self, id: Self::NodeId) -> Option<Self::NodeRef<'_>> {
        self.nodes().find(|node| node.id() == id)
    }

    fn node_mut(&mut self, id: Self::NodeId) -> Option<Self::NodeMut<'_>> {
        self.nodes_mut().find(|node| node.id() == id)
    }

    fn edge(&self, id: Self::EdgeId) -> Option<Self::EdgeRef<'_>> {
        self.edges().find(|edge| edge.id() == id)
    }

    fn edge_mut(&mut self, id: Self::EdgeId) -> Option<Self::EdgeMut<'_>> {
        self.edges_mut().find(|edge| edge.id() == id)
    }

    // Degree

    fn degree(&self, node: Self::NodeId) -> usize {
        self.incident_edges(node).count()
    }

    // Incidence

    fn incident_edges(&self, node: Self::NodeId) -> impl Iterator<Item = Self::EdgeRef<'_>> {
        self.edges()
            .filter(move |edge| edge.source() == node || edge.target() == node)
    }

    fn incident_edges_mut(
        &mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeMut<'_>> {
        self.edges_mut()
            .filter(move |edge| edge.source() == node || edge.target() == node)
    }

    // Adjacency

    fn adjacencies(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        self.incident_edges(node).map(move |edge| {
            if edge.source() == node {
                edge.target()
            } else {
                edge.source()
            }
        })
    }

    // Edges between nodes

    fn edges_between(
        &self,
        lhs: Self::NodeId,
        rhs: Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeRef<'_>> {
        self.edges().filter(move |edge| {
            (edge.source() == lhs && edge.target() == rhs)
                || (edge.source() == rhs && edge.target() == lhs)
        })
    }

    fn edges_between_mut(
        &mut self,
        lhs: Self::NodeId,
        rhs: Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeMut<'_>> {
        self.edges_mut().filter(move |edge| {
            (edge.source() == lhs && edge.target() == rhs)
                || (edge.source() == rhs && edge.target() == lhs)
        })
    }

    // Existence checks

    fn contains_node(&self, id: Self::NodeId) -> bool {
        self.node(id).is_some()
    }

    fn contains_edge(&self, id: Self::EdgeId) -> bool {
        self.edge(id).is_some()
    }

    fn is_adjacent(&self, lhs: Self::NodeId, rhs: Self::NodeId) -> bool {
        self.edges_between(lhs, rhs).next().is_some()
    }

    fn is_empty(&self) -> bool {
        self.node_count() == 0
    }
}
