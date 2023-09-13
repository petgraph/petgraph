mod directed;
mod insert;
mod resize;
mod retain;

use error_stack::Result;

use crate::{
    edge::{DetachedEdge, Edge, EdgeMut},
    node::{DetachedNode, Node, NodeMut},
    storage::GraphStorage,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Graph<S> {
    storage: S,
}

impl<S> Default for Graph<S>
where
    S: GraphStorage,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Graph<S>
where
    S: GraphStorage,
{
    #[must_use]
    pub fn new() -> Self {
        Self {
            storage: S::with_capacity(None, None),
        }
    }

    #[must_use]
    pub const fn new_in(storage: S) -> Self {
        Self { storage }
    }

    pub const fn storage(&self) -> &S {
        &self.storage
    }

    pub fn into_storage(self) -> S {
        self.storage
    }

    pub fn from_parts(
        nodes: impl IntoIterator<Item = DetachedNode<S::NodeId, S::NodeWeight>>,
        edges: impl IntoIterator<Item = DetachedEdge<S::EdgeId, S::NodeId, S::EdgeWeight>>,
    ) -> Result<Self, S::Error> {
        Ok(Self {
            storage: S::from_parts(nodes, edges)?,
        })
    }

    pub fn convert<S2>(self) -> Result<Graph<S2>, S2::Error>
    where
        S2: GraphStorage<
                NodeId = S::NodeId,
                NodeWeight = S::NodeWeight,
                EdgeId = S::EdgeId,
                EdgeWeight = S::EdgeWeight,
            >,
    {
        let (nodes, edges) = self.storage.into_parts();

        Graph::from_parts(nodes, edges)
    }

    #[must_use]
    pub fn with_capacity(node_capacity: Option<usize>, edge_capacity: Option<usize>) -> Self {
        Self {
            storage: S::with_capacity(node_capacity, edge_capacity),
        }
    }

    pub fn num_nodes(&self) -> usize {
        self.storage.num_nodes()
    }

    pub fn num_edges(&self) -> usize {
        self.storage.num_edges()
    }

    pub fn is_empty(&self) -> bool {
        self.num_nodes() == 0 && self.num_edges() == 0
    }

    pub fn clear(&mut self) {
        self.storage.clear();
    }

    pub fn node(&self, id: &S::NodeId) -> Option<Node<S>> {
        self.storage.node(id)
    }

    pub fn node_mut(&mut self, id: &S::NodeId) -> Option<NodeMut<S>> {
        self.storage.node_mut(id)
    }

    pub fn contains_node(&self, id: &S::NodeId) -> bool {
        self.storage.contains_node(id)
    }

    pub fn remove_node(
        &mut self,
        id: &S::NodeId,
    ) -> Option<DetachedNode<S::NodeId, S::NodeWeight>> {
        self.storage.remove_node(id)
    }

    pub fn edge(&self, id: &S::EdgeId) -> Option<Edge<S>> {
        self.storage.edge(id)
    }

    pub fn edge_mut(&mut self, id: &S::EdgeId) -> Option<EdgeMut<S>> {
        self.storage.edge_mut(id)
    }

    pub fn contains_edge(&self, id: &S::EdgeId) -> bool {
        self.storage.contains_edge(id)
    }

    pub fn remove_edge(
        &mut self,
        id: &S::EdgeId,
    ) -> Option<DetachedEdge<S::EdgeId, S::NodeId, S::EdgeWeight>> {
        self.storage.remove_edge(id)
    }

    #[inline]
    pub fn neighbors<'a: 'b, 'b>(
        &'a self,
        id: &'b S::NodeId,
    ) -> impl Iterator<Item = Node<'a, S>> + 'b {
        self.neighbours(id)
    }

    pub fn neighbours<'a: 'b, 'b>(
        &'a self,
        id: &'b S::NodeId,
    ) -> impl Iterator<Item = Node<S>> + 'b {
        self.storage.node_neighbours(id)
    }

    #[inline]
    pub fn neighbors_mut<'a: 'b, 'b>(
        &'a mut self,
        id: &'b S::NodeId,
    ) -> impl Iterator<Item = NodeMut<'a, S>> + 'b {
        self.neighbours_mut(id)
    }

    pub fn neighbours_mut<'a: 'b, 'b>(
        &'a mut self,
        id: &'b S::NodeId,
    ) -> impl Iterator<Item = NodeMut<'a, S>> + 'b {
        self.storage.node_neighbours_mut(id)
    }

    pub fn connections<'a: 'b, 'b>(
        &'a self,
        id: &'b S::NodeId,
    ) -> impl Iterator<Item = Edge<'a, S>> + 'b {
        self.storage.node_connections(id)
    }

    pub fn connections_mut<'a: 'b, 'b>(
        &'a mut self,
        id: &'b S::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'a, S>> + 'b {
        self.storage.node_connections_mut(id)
    }

    // TODO: `map`, `filter`, `filter_map`, `find`, `reverse`, `any`, `all`, etc.

    pub fn edges_between<'a: 'b, 'b>(
        &'a self,
        source: &'b S::NodeId,
        target: &'b S::NodeId,
    ) -> impl Iterator<Item = Edge<'a, S>> + 'b {
        self.storage.edges_between(source, target)
    }

    pub fn edges_between_mut<'a: 'b, 'b>(
        &'a mut self,
        source: &'b S::NodeId,
        target: &'b S::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'a, S>> + 'b {
        self.storage.edges_between_mut(source, target)
    }

    pub fn externals(&self) -> impl Iterator<Item = Node<S>> {
        self.storage.external_nodes()
    }

    pub fn externals_mut(&mut self) -> impl Iterator<Item = NodeMut<S>> {
        self.storage.external_nodes_mut()
    }

    pub fn nodes(&self) -> impl Iterator<Item = Node<S>> {
        self.storage.nodes()
    }

    pub fn nodes_mut(&mut self) -> impl Iterator<Item = NodeMut<S>> {
        self.storage.nodes_mut()
    }

    pub fn edges(&self) -> impl Iterator<Item = Edge<S>> {
        self.storage.edges()
    }

    pub fn edges_mut(&mut self) -> impl Iterator<Item = EdgeMut<S>> {
        self.storage.edges_mut()
    }
}
