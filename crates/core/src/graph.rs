use error_stack::Result;

use crate::{
    attributes::Attributes,
    edge::{DetachedEdge, Direction, Edge, EdgeMut},
    index::{ArbitraryGraphIndex, GraphIndex},
    node::{DetachedNode, Node, NodeMut},
    storage::{DirectedGraphStorage, GraphStorage, RetainGraphStorage},
};

pub struct Graph<S> {
    storage: S,
}

impl<S> Graph<S>
where
    S: GraphStorage,
{
    pub fn new() -> Self {
        Self {
            storage: S::with_capacity(None, None),
        }
    }

    pub fn from_parts(
        nodes: impl IntoIterator<Item = DetachedNode<S::NodeIndex, S::NodeWeight>>,
        edges: impl IntoIterator<Item = DetachedEdge<S::EdgeIndex, S::NodeIndex, S::EdgeWeight>>,
    ) -> Result<Self, S::Error> {
        Ok(Self {
            storage: S::from_parts(nodes, edges)?,
        })
    }

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

    pub fn node(&self, id: &S::NodeIndex) -> Option<Node<S>> {
        self.storage.node(id)
    }

    pub fn node_mut(&mut self, id: &S::NodeIndex) -> Option<NodeMut<S>> {
        self.storage.node_mut(id)
    }

    pub fn remove_node(
        &mut self,
        id: S::NodeIndex,
    ) -> Option<DetachedNode<S::NodeIndex, S::NodeWeight>> {
        self.storage.remove_node(id)
    }

    pub fn edge(&self, id: &S::EdgeIndex) -> Option<Edge<S>> {
        self.storage.edge(id)
    }

    pub fn edge_mut(&mut self, id: &S::EdgeIndex) -> Option<EdgeMut<S>> {
        self.storage.edge_mut(id)
    }

    pub fn remove_edge(
        &mut self,
        id: S::EdgeIndex,
    ) -> Option<DetachedEdge<S::EdgeIndex, S::NodeIndex, S::EdgeWeight>> {
        self.storage.remove_edge(id)
    }

    #[inline(always)]
    pub fn neighbors(&self, id: &S::NodeIndex) -> impl Iterator<Item = Node<S>> {
        self.neighbours(id)
    }

    pub fn neighbours(&self, id: &S::NodeIndex) -> impl Iterator<Item = Node<S>> {
        self.storage.node_neighbours(id)
    }

    #[inline(always)]
    pub fn neighbors_mut(&mut self, id: &S::NodeIndex) -> impl Iterator<Item = NodeMut<S>> {
        self.neighbours_mut(id)
    }

    pub fn neighbours_mut(&mut self, id: &S::NodeIndex) -> impl Iterator<Item = NodeMut<S>> {
        self.storage.node_neighbours_mut(id)
    }

    pub fn connections(&self, id: &S::NodeIndex) -> impl Iterator<Item = Edge<S>> {
        self.storage.node_connections(id)
    }

    pub fn connections_mut(&mut self, id: &S::NodeIndex) -> impl Iterator<Item = EdgeMut<S>> {
        self.storage.node_connections_mut(id)
    }

    // TODO: `map`, `filter`, `filter_map`, `find`, `reverse`, `any`, `all`, etc.

    pub fn find_undirected_edges(
        &self,
        source: &S::NodeIndex,
        target: &S::NodeIndex,
    ) -> impl Iterator<Item = Edge<S>> {
        self.storage.find_undirected_edges(source, target)
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

impl<S> Graph<S>
where
    S: DirectedGraphStorage,
{
    #[inline(always)]
    pub fn neighbors_directed(
        &self,
        id: &S::NodeIndex,
        direction: Direction,
    ) -> impl Iterator<Item = S::NodeIndex> {
        self.neighbours_directed(id, direction)
    }

    pub fn neighbours_directed(
        &self,
        id: &S::NodeIndex,
        direction: Direction,
    ) -> impl Iterator<Item = S::NodeIndex> {
        self.storage.node_directed_neighbours(id, direction)
    }

    pub fn neighbors_directed_mut(
        &mut self,
        id: &S::NodeIndex,
        direction: Direction,
    ) -> impl Iterator<Item = S::NodeIndex> {
        self.neighbours_directed_mut(id, direction)
    }

    #[inline(always)]
    pub fn neighbours_directed_mut(
        &mut self,
        id: &S::NodeIndex,
        direction: Direction,
    ) -> impl Iterator<Item = S::NodeIndex> {
        self.storage.node_directed_neighbours_mut(id, direction)
    }

    pub fn connections_directed(
        &self,
        id: &S::NodeIndex,
        direction: Direction,
    ) -> impl Iterator<Item = Edge<S>> {
        self.storage.node_directed_connections(id, direction)
    }

    pub fn connections_directed_mut(
        &mut self,
        id: &S::NodeIndex,
        direction: Direction,
    ) -> impl Iterator<Item = EdgeMut<S>> {
        self.storage.node_directed_connections_mut(id, direction)
    }

    pub fn find_directed_edges(
        &self,
        source: &S::NodeIndex,
        target: &S::NodeIndex,
    ) -> impl Iterator<Item = Edge<S>> + '_ {
        self.storage.find_directed_edges(source, target)
    }

    // TODO: should this be part of `GraphIterator`?
    pub fn reverse(self) -> Result<Self, S::Error> {
        let (nodes, edges) = self.storage.into_parts();

        let edges = edges.map(|edge| {
            let source = edge.source;
            let target = edge.target;

            edge.source = target;
            edge.target = source;

            edge
        });

        Self::from_parts(nodes, edges)
    }

    // These should go into extensions:
    // into_undirected, into_directed, from_edges, extend_with_edges
}

impl<S> Graph<S>
where
    S: GraphStorage,
{
    pub fn insert_node(
        &mut self,
        attributes: Attributes<<S::NodeIndex as GraphIndex>::InsertValue, S::NodeWeight>,
    ) -> Result<Node<S>, S::Error> {
        let Attributes { id, weight } = attributes;

        let id = <S::NodeIndex as GraphIndex>::convert(&self.storage, id);

        self.storage.insert_node(id, weight)
    }
}

impl<S> Graph<S>
where
    S: GraphStorage,
    S::NodeIndex: ArbitraryGraphIndex,
{
    pub fn upsert_node(
        &mut self,
        id: S::NodeIndex,
        weight: S::NodeWeight,
    ) -> Result<Node<S>, S::Error> {
        if let Some(mut node) = self.storage.node_mut(&id) {
            *node.weight_mut() = weight;
            Ok(node.as_ref())
        } else {
            self.storage.insert_node(id, weight)
        }
    }
}

impl<S> Graph<S>
where
    S: GraphStorage,
{
    pub fn insert_edge(
        &mut self,
        attributes: Attributes<<S::EdgeIndex as GraphIndex>::InsertValue, S::EdgeWeight>,
        source: S::NodeIndex,
        target: S::NodeIndex,
    ) -> Result<Edge<S>, S::Error> {
        let Attributes { id, weight } = attributes;

        let id = <S::EdgeIndex as GraphIndex>::convert(&self.storage, id);

        self.storage.insert_edge(id, source, target, weight)
    }
}

impl<S> Graph<S>
where
    S: GraphStorage,
    S::EdgeIndex: ArbitraryGraphIndex<Storage = S>,
{
    pub fn upsert_edge(
        &mut self,
        id: S::EdgeIndex,
        source: S::NodeIndex,
        target: S::NodeIndex,
        weight: S::EdgeWeight,
    ) -> Result<Edge<S>, S::Error> {
        if let Some(mut edge) = self.storage.edge_mut(&id) {
            *edge.weight_mut() = weight;
            Ok(edge.as_ref())
        } else {
            self.storage.insert_edge(id, source, target, weight)
        }
    }
}

impl<S> Graph<S>
where
    S: RetainGraphStorage,
{
    pub fn retain(
        &mut self,
        nodes: impl FnMut(NodeMut<'_, S>) -> bool,
        edges: impl FnMut(EdgeMut<'_, S>) -> bool,
    ) {
        self.storage.retain(nodes, edges);
    }

    pub fn retain_nodes(&mut self, f: impl FnMut(NodeMut<'_, S>) -> bool) {
        self.storage.retain_nodes(f);
    }

    pub fn retain_edges(&mut self, f: impl FnMut(EdgeMut<'_, S>) -> bool) {
        self.storage.retain_edges(f);
    }
}
