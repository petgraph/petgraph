use petgraph_core::{
    edge::{EdgeMut, EdgeRef},
    graph::{Cardinality, DensityHint, DirectedGraph},
    node::{NodeMut, NodeRef},
};

use crate::{DinoGraph, Directed};

impl<N, E> DirectedGraph for DinoGraph<N, E, Directed> {
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
        self.nodes.len()
    }

    #[inline]
    fn edge_count(&self) -> usize {
        self.edges.len()
    }

    // Node Iteration

    fn nodes(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        self.nodes.iter().map(|node| NodeRef::<Self> {
            id: node.id,
            data: &node.weight,
        })
    }

    fn nodes_mut(&mut self) -> impl Iterator<Item = NodeMut<'_, Self>> {
        self.nodes.iter_mut().map(|node| NodeMut::<Self> {
            id: node.id,
            data: &mut node.weight,
        })
    }

    #[inline]
    fn isolated_nodes(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        self.nodes
            .iter()
            .filter(|node| node.closures.is_isolated())
            .map(|node| NodeRef::<Self> {
                id: node.id,
                data: &node.weight,
            })
    }

    // Edge iteration
    fn edges(&self) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.edges.iter().map(|edge| EdgeRef::<Self> {
            id: edge.id,
            data: &edge.weight,
            source: edge.source,
            target: edge.target,
        })
    }

    fn edges_mut(&mut self) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.edges.iter_mut().map(|edge| EdgeMut::<Self> {
            id: edge.id,
            data: &mut edge.weight,
            source: edge.source,
            target: edge.target,
        })
    }

    // Lookup
    #[inline]
    fn node(&self, id: Self::NodeId) -> Option<NodeRef<'_, Self>> {
        self.nodes.get(id).map(|node| NodeRef::<Self> {
            id: node.id,
            data: &node.weight,
        })
    }

    #[inline]
    fn node_mut(&mut self, id: Self::NodeId) -> Option<NodeMut<'_, Self>> {
        self.nodes.get_mut(id).map(|node| NodeMut::<Self> {
            id: node.id,
            data: &mut node.weight,
        })
    }

    #[inline]
    fn edge(&self, id: Self::EdgeId) -> Option<EdgeRef<'_, Self>> {
        self.edges.get(id).map(|edge| EdgeRef::<Self> {
            id: edge.id,
            data: &edge.weight,
            source: edge.source,
            target: edge.target,
        })
    }

    #[inline]
    fn edge_mut(&mut self, id: Self::EdgeId) -> Option<EdgeMut<'_, Self>> {
        self.edges.get_mut(id).map(|edge| EdgeMut::<Self> {
            id: edge.id,
            data: &mut edge.weight,
            source: edge.source,
            target: edge.target,
        })
    }

    // Degree
    #[inline]
    fn in_degree(&self, node: Self::NodeId) -> usize {
        self.incoming_edges(node).count()
    }

    #[inline]
    fn out_degree(&self, node: Self::NodeId) -> usize {
        self.outgoing_edges(node).count()
    }

    #[inline]
    fn degree(&self, node: Self::NodeId) -> usize {
        self.in_degree(node) + self.out_degree(node)
    }

    // Edges by direction
    #[inline]
    fn incoming_edges(&self, node: Self::NodeId) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.nodes
            .get(node)
            .map(|n| n.closures.incoming_edges())
            .into_iter()
            .flatten()
            .filter_map(move |id| self.edge(id))
    }

    #[inline]
    fn incoming_edges_mut(
        &mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.nodes
            .get(node)
            .map(|n| n.closures.incoming_edges())
            .into_iter()
            .flatten()
            .filter_map(move |id| self.edge_mut(id))
    }

    #[inline]
    fn outgoing_edges(&self, node: Self::NodeId) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.nodes.get(node).map(|n| n.closures.outgoing_edges())
    }

    #[inline]
    fn outgoing_edges_mut(
        &mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.nodes.get(node).map(|n| {
            n.closures
                .outgoing_edges()
                .filter_map(move |id| self.edges.get_mut(id))
        })
    }

    #[inline]
    fn incident_edges(&self, node: Self::NodeId) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.nodes.get(node).map(|n| n.closures.incident_edges())
    }

    #[inline]
    fn incident_edges_mut(
        &mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.nodes.get(node).map(|n| {
            n.closures
                .incident_edges()
                .filter_map(move |id| self.edges.get_mut(id))
        })
    }

    // Adjacency
    #[inline]
    fn predecessors(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        self.nodes.get(node).map(|n| n.closures.incoming_nodes())
    }

    #[inline]
    fn successors(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        self.nodes.get(node).map(|n| n.closures.outgoing_nodes())
    }

    #[inline]
    fn adjacencies(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        self.nodes.get(node).map(|n| n.closures.neighbours())
    }

    // Edges between nodes
    #[inline]
    fn edges_between(
        &self,
        source: Self::NodeId,
        target: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.outgoing_edges(source)
            .filter(move |edge| edge.target == target)
    }

    #[inline]
    fn edges_between_mut(
        &mut self,
        source: Self::NodeId,
        target: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.outgoing_edges_mut(source)
            .filter(move |edge| edge.target == target)
    }

    #[inline]
    fn edges_connecting(
        &self,
        lhs: Self::NodeId,
        rhs: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.edges_between(lhs, rhs)
            .chain(self.edges_between(rhs, lhs))
    }

    #[inline]
    fn edges_connecting_mut(
        &mut self,
        lhs: Self::NodeId,
        rhs: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.edges_between_mut(lhs, rhs)
            .chain(self.edges_between_mut(rhs, lhs))
    }

    // Existence checks
    #[inline]
    fn contains_node(&self, node: Self::NodeId) -> bool {
        self.nodes.contains_key(node)
    }

    #[inline]
    fn contains_edge(&self, edge: Self::EdgeId) -> bool {
        self.edges.contains_key(edge)
    }

    #[inline]
    fn is_adjacent(&self, source: Self::NodeId, target: Self::NodeId) -> bool {
        self.nodes
            .get(source)
            .map(|node| node.closures.outgoing_nodes().any(|id| id == target))
            .unwrap_or(false)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.nodes.len() == 0
    }

    // Sources and sinks
    #[inline]
    fn sources(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        self.nodes
            .iter()
            .filter(|node| node.closures.incoming_nodes().next().is_none())
            .map(|node| NodeRef::new(node.id, &node.weight))
    }

    #[inline]
    fn sinks(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        self.nodes
            .iter()
            .filter(|node| node.closures.outgoing_nodes().next().is_none())
            .map(|node| NodeRef::new(node.id, &node.weight))
    }
}
