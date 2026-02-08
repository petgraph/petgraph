use super::{Cardinality, DensityHint, Graph};
use crate::{
    edge::{EdgeMut, EdgeRef},
    node::{NodeMut, NodeRef},
};

pub trait DirectedGraph: Graph {
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

    // Node Iteration

    fn nodes<'graph_ref>(&'graph_ref self) -> impl Iterator<Item = NodeRef<'graph_ref, Self>>
    where
        Self: 'graph_ref;
    fn nodes_mut<'graph_ref>(
        &'graph_ref mut self,
    ) -> impl Iterator<Item = NodeMut<'graph_ref, Self>>
    where
        Self: 'graph_ref;

    /// Nodes with degree 0 (no incident edges).
    #[inline]
    fn isolated_nodes<'graph_ref>(
        &'graph_ref self,
    ) -> impl Iterator<Item = NodeRef<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        self.nodes().filter(|node| self.degree(node.id) == 0)
    }

    // Edge iteration

    fn edges<'graph_ref>(&'graph_ref self) -> impl Iterator<Item = EdgeRef<'graph_ref, Self>>
    where
        Self: 'graph_ref;
    fn edges_mut<'graph_ref>(
        &'graph_ref mut self,
    ) -> impl Iterator<Item = EdgeMut<'graph_ref, Self>>
    where
        Self: 'graph_ref;

    // Lookup
    #[inline]
    fn node<'graph_ref>(&'graph_ref self, id: Self::NodeId) -> Option<NodeRef<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        self.nodes().find(|node| node.id == id)
    }

    #[inline]
    fn node_mut<'graph_ref>(
        &'graph_ref mut self,
        id: Self::NodeId,
    ) -> Option<NodeMut<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        self.nodes_mut().find(|node| node.id == id)
    }

    #[inline]
    fn edge<'graph_ref>(&'graph_ref self, id: Self::EdgeId) -> Option<EdgeRef<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        self.edges().find(|edge| edge.id == id)
    }

    #[inline]
    fn edge_mut<'graph_ref>(
        &'graph_ref mut self,
        id: Self::EdgeId,
    ) -> Option<EdgeMut<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
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
    fn incoming_edges<'graph_ref>(
        &'graph_ref self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        self.edges().filter(move |edge| edge.target == node)
    }

    #[inline]
    fn incoming_edges_mut<'graph_ref>(
        &'graph_ref mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        self.edges_mut().filter(move |edge| edge.target == node)
    }

    #[inline]
    fn outgoing_edges<'graph_ref>(
        &'graph_ref self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        self.edges().filter(move |edge| edge.source == node)
    }

    #[inline]
    fn outgoing_edges_mut<'graph_ref>(
        &'graph_ref mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        self.edges_mut().filter(move |edge| edge.source == node)
    }

    #[inline]
    fn incident_edges<'graph_ref>(
        &'graph_ref self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        self.edges()
            .filter(move |edge| edge.source == node || edge.target == node)
    }

    #[inline]
    fn incident_edges_mut<'graph_ref>(
        &'graph_ref mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
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
    fn edges_between<'graph_ref>(
        &'graph_ref self,
        source: Self::NodeId,
        target: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        self.edges()
            .filter(move |edge| edge.source == source && edge.target == target)
    }

    #[inline]
    fn edges_between_mut<'graph_ref>(
        &'graph_ref mut self,
        source: Self::NodeId,
        target: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        self.edges_mut()
            .filter(move |edge| edge.source == source && edge.target == target)
    }

    #[inline]
    fn edges_connecting<'graph_ref>(
        &'graph_ref self,
        lhs: Self::NodeId,
        rhs: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        self.edges().filter(move |edge| {
            (edge.source == lhs && edge.target == rhs) || (edge.source == rhs && edge.target == lhs)
        })
    }

    #[inline]
    fn edges_connecting_mut<'graph_ref>(
        &'graph_ref mut self,
        lhs: Self::NodeId,
        rhs: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
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
    fn sources<'graph_ref>(&'graph_ref self) -> impl Iterator<Item = NodeRef<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        self.nodes().filter(|node| self.in_degree(node.id) == 0)
    }

    /// Nodes with `out_degree = 0`.
    #[inline]
    fn sinks<'graph_ref>(&'graph_ref self) -> impl Iterator<Item = NodeRef<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        self.nodes().filter(|node| self.out_degree(node.id) == 0)
    }
}

impl<G> DirectedGraph for &mut G
where
    G: DirectedGraph,
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
    fn nodes<'graph_ref>(&'graph_ref self) -> impl Iterator<Item = NodeRef<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        G::nodes(self)
    }

    #[inline]
    fn nodes_mut<'graph_ref>(
        &'graph_ref mut self,
    ) -> impl Iterator<Item = NodeMut<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        G::nodes_mut(self)
    }

    #[inline]
    fn isolated_nodes<'graph_ref>(
        &'graph_ref self,
    ) -> impl Iterator<Item = NodeRef<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        G::isolated_nodes(self)
    }

    #[inline]
    fn edges<'graph_ref>(&'graph_ref self) -> impl Iterator<Item = EdgeRef<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        G::edges(self)
    }

    #[inline]
    fn edges_mut<'graph_ref>(
        &'graph_ref mut self,
    ) -> impl Iterator<Item = EdgeMut<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        G::edges_mut(self)
    }

    #[inline]
    fn node<'graph_ref>(&'graph_ref self, id: Self::NodeId) -> Option<NodeRef<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        G::node(self, id)
    }

    #[inline]
    fn node_mut<'graph_ref>(
        &'graph_ref mut self,
        id: Self::NodeId,
    ) -> Option<NodeMut<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        G::node_mut(self, id)
    }

    #[inline]
    fn edge<'graph_ref>(&'graph_ref self, id: Self::EdgeId) -> Option<EdgeRef<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        G::edge(self, id)
    }

    #[inline]
    fn edge_mut<'graph_ref>(
        &'graph_ref mut self,
        id: Self::EdgeId,
    ) -> Option<EdgeMut<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        G::edge_mut(self, id)
    }

    #[inline]
    fn in_degree(&self, node: Self::NodeId) -> usize {
        G::in_degree(self, node)
    }

    #[inline]
    fn out_degree(&self, node: Self::NodeId) -> usize {
        G::out_degree(self, node)
    }

    #[inline]
    fn degree(&self, node: Self::NodeId) -> usize {
        G::degree(self, node)
    }

    #[inline]
    fn incoming_edges<'graph_ref>(
        &'graph_ref self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        G::incoming_edges(self, node)
    }

    #[inline]
    fn incoming_edges_mut<'graph_ref>(
        &'graph_ref mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        G::incoming_edges_mut(self, node)
    }

    #[inline]
    fn outgoing_edges<'graph_ref>(
        &'graph_ref self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        G::outgoing_edges(self, node)
    }

    #[inline]
    fn outgoing_edges_mut<'graph_ref>(
        &'graph_ref mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        G::outgoing_edges_mut(self, node)
    }

    #[inline]
    fn incident_edges<'graph_ref>(
        &'graph_ref self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        G::incident_edges(self, node)
    }

    #[inline]
    fn incident_edges_mut<'graph_ref>(
        &'graph_ref mut self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        G::incident_edges_mut(self, node)
    }

    #[inline]
    fn predecessors(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        G::predecessors(self, node)
    }

    #[inline]
    fn successors(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        G::successors(self, node)
    }

    #[inline]
    fn adjacencies(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        G::adjacencies(self, node)
    }

    #[inline]
    fn edges_between<'graph_ref>(
        &'graph_ref self,
        source: Self::NodeId,
        target: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        G::edges_between(self, source, target)
    }

    #[inline]
    fn edges_between_mut<'graph_ref>(
        &'graph_ref mut self,
        source: Self::NodeId,
        target: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        G::edges_between_mut(self, source, target)
    }

    #[inline]
    fn edges_connecting<'graph_ref>(
        &'graph_ref self,
        lhs: Self::NodeId,
        rhs: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        G::edges_connecting(self, lhs, rhs)
    }

    #[inline]
    fn edges_connecting_mut<'graph_ref>(
        &'graph_ref mut self,
        lhs: Self::NodeId,
        rhs: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        G::edges_connecting_mut(self, lhs, rhs)
    }

    #[inline]
    fn contains_node(&self, node: Self::NodeId) -> bool {
        G::contains_node(self, node)
    }

    #[inline]
    fn contains_edge(&self, edge: Self::EdgeId) -> bool {
        G::contains_edge(self, edge)
    }

    #[inline]
    fn is_adjacent(&self, source: Self::NodeId, target: Self::NodeId) -> bool {
        G::is_adjacent(self, source, target)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        G::is_empty(self)
    }

    #[inline]
    fn sources<'graph_ref>(&'graph_ref self) -> impl Iterator<Item = NodeRef<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        G::sources(self)
    }

    #[inline]
    fn sinks<'graph_ref>(&'graph_ref self) -> impl Iterator<Item = NodeRef<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        G::sinks(self)
    }
}
