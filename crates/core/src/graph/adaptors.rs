use crate::{
    edge::{EdgeMut, EdgeRef},
    graph::{Cardinality, DensityHint, DirectedGraph, Graph, UndirectedGraph, delegate_impl},
    node::{NodeMut, NodeRef},
};

pub struct DirectedAdapter<'graph_ref, G: UndirectedGraph> {
    pub graph: &'graph_ref mut G,
}

impl<'graph_ref, G: UndirectedGraph> Graph for DirectedAdapter<'graph_ref, G> {
    delegate_impl!(G);
}

impl<'graph_ref, G: UndirectedGraph> DirectedGraph for DirectedAdapter<'graph_ref, G> {
    #[inline]
    fn density_hint(&self) -> DensityHint {
        self.graph.density_hint()
    }

    #[inline]
    fn cardinality(&self) -> Cardinality {
        Cardinality {
            order: self.graph.cardinality().order,
            size: self.graph.cardinality().size * 2,
        }
    }

    #[inline]
    fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    #[inline]
    fn edge_count(&self) -> usize {
        self.graph.edge_count() * 2
    }

    fn nodes(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        self.graph.nodes()
    }

    fn nodes_mut(&mut self) -> impl Iterator<Item = NodeMut<'_, Self>> {
        self.graph.nodes_mut()
    }

    #[inline]
    fn isolated_nodes(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        self.graph.isolated_nodes()
    }

    fn edges(&self) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.graph
            .edges()
            .map(|edge| EdgeRef::<'_, G> {
                id: edge.id,
                source: edge.target,
                target: edge.source,
                data: edge.data,
            })
            .chain(self.graph.edges())
    }

    fn edges_mut(&mut self) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.graph.edges_mut()
    }

    #[inline]
    fn node(&self, id: Self::NodeId) -> Option<NodeRef<'_, Self>> {
        self.graph.node(id)
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
        self.edges().filter(move |edge| edge.target == node)
    }

    #[inline]
    fn incoming_edges_mut(
        &mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.edges_mut().filter(move |edge| edge.target == node)
    }

    #[inline]
    fn outgoing_edges(&self, node: Self::NodeId) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.edges().filter(move |edge| edge.source == node)
    }

    #[inline]
    fn outgoing_edges_mut(
        &mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.edges_mut().filter(move |edge| edge.source == node)
    }

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
    fn predecessors(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        self.incoming_edges(node).map(|edge| edge.source)
    }

    #[inline]
    fn successors(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        self.outgoing_edges(node).map(|edge| edge.target)
    }

    #[inline]
    fn adjacencies(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        self.predecessors(node).chain(self.successors(node))
    }

    // Edges between nodes
    #[inline]
    fn edges_between(
        &self,
        source: Self::NodeId,
        target: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.edges()
            .filter(move |edge| edge.source == source && edge.target == target)
    }

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
    fn contains_node(&self, node: Self::NodeId) -> bool {
        self.node(node).is_some()
    }

    #[inline]
    fn contains_edge(&self, edge: Self::EdgeId) -> bool {
        self.edge(edge).is_some()
    }

    #[inline]
    fn is_adjacent(&self, source: Self::NodeId, target: Self::NodeId) -> bool {
        self.edges_between(source, target).next().is_some()
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

pub struct UndirectedAdapter<'graph_ref, G: DirectedGraph> {
    pub graph: &'graph_ref mut G,
}

impl<'graph_ref, G: DirectedGraph> Graph for UndirectedAdapter<'graph_ref, G> {
    delegate_impl!(G);
}

impl<'graph_ref, G: DirectedGraph> UndirectedGraph for UndirectedAdapter<'graph_ref, G> {
    #[inline]
    fn density_hint(&self) -> DensityHint {
        self.graph.density_hint()
    }

    #[inline]
    fn cardinality(&self) -> Cardinality {
        self.graph.cardinality()
    }

    #[inline]
    fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    #[inline]
    fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    #[inline]
    fn nodes(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        self.graph.nodes()
    }

    #[inline]
    fn nodes_mut(&mut self) -> impl Iterator<Item = NodeMut<'_, Self>> {
        self.graph.nodes_mut()
    }

    #[inline]
    fn isolated_nodes(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        self.graph.isolated_nodes()
    }

    #[inline]
    fn edges(&self) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.graph.edges()
    }

    #[inline]
    fn edges_mut(&mut self) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.graph.edges_mut()
    }

    #[inline]
    fn node(&self, id: Self::NodeId) -> Option<NodeRef<'_, Self>> {
        self.graph.node(id)
    }

    #[inline]
    fn node_mut(&mut self, id: Self::NodeId) -> Option<NodeMut<'_, Self>> {
        self.graph.node_mut(id)
    }

    #[inline]
    fn edge(&self, id: Self::EdgeId) -> Option<EdgeRef<'_, Self>> {
        self.graph.edge(id)
    }

    #[inline]
    fn edge_mut(&mut self, id: Self::EdgeId) -> Option<EdgeMut<'_, Self>> {
        self.graph.edge_mut(id)
    }

    #[inline]
    fn degree(&self, node: Self::NodeId) -> usize {
        self.graph.degree(node)
    }

    #[inline]
    fn incident_edges(&self, node: Self::NodeId) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.graph.incident_edges(node)
    }

    #[inline]
    fn incident_edges_mut(
        &mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.graph.incident_edges_mut(node)
    }

    #[inline]
    fn adjacencies(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        self.graph.adjacencies(node)
    }

    #[inline]
    fn edges_connecting(
        &self,
        source: Self::NodeId,
        target: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.graph.edges_connecting(source, target)
    }

    #[inline]
    fn edges_connecting_mut(
        &mut self,
        source: Self::NodeId,
        target: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.graph.edges_connecting_mut(source, target)
    }

    #[inline]
    fn contains_node(&self, id: Self::NodeId) -> bool {
        self.graph.contains_node(id)
    }

    #[inline]
    fn contains_edge(&self, id: Self::EdgeId) -> bool {
        self.graph.contains_edge(id)
    }

    #[inline]
    fn is_adjacent(&self, source: Self::NodeId, target: Self::NodeId) -> bool {
        self.graph.is_adjacent(source, target)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.graph.is_empty()
    }
}
