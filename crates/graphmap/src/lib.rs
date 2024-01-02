#![no_std]

mod auxiliary;
mod directed;
mod hash;
mod retain;
mod reverse;
mod sequential;

extern crate alloc;

use core::hash::Hash;

use error_stack::{Report, ResultExt};
use hashbrown::{hash_map::DefaultHashBuilder, HashMap};
use petgraph_core::{
    edge::EdgeId, node::NodeId, DetachedEdge, DetachedNode, Edge, EdgeMut, GraphDirectionality,
    GraphStorage, Node, NodeMut,
};
use petgraph_dino::DinoStorage;

use crate::hash::ValueHash;

pub struct Entry<K, V> {
    key: K,
    value: V,
}

pub enum MapError {
    UnderlyingStorage,
    NodeExists,
    EdgeExists,
}

type Backend<NK, NV, EK, EV, D> = DinoStorage<Entry<NK, NV>, Entry<EK, EV>, D>;

// TODO: better name
// TODO: reduce generics
pub struct EntryStorage<NK, NV, EK, EV, D>
where
    D: GraphDirectionality,
    NK: Hash,
    EK: Hash,
{
    inner: Backend<NK, NV, EK, EV, D>,

    nodes: HashMap<ValueHash<NK>, NodeId>,
    edges: HashMap<ValueHash<EK>, EdgeId>,

    hasher: DefaultHashBuilder,
}

impl<NK, NV, EK, EV, D> GraphStorage for EntryStorage<NK, NV, EK, EV, D>
where
    D: GraphDirectionality,
    NK: Hash,
    EK: Hash,
{
    type EdgeWeight = Entry<EK, EV>;
    type Error = MapError;
    type NodeWeight = Entry<NK, EV>;

    fn with_capacity(node_capacity: Option<usize>, edge_capacity: Option<usize>) -> Self {
        Self {
            inner: DinoStorage::with_capacity(node_capacity, edge_capacity),
            nodes: HashMap::with_capacity(node_capacity.unwrap_or(0)),
            edges: HashMap::with_capacity(edge_capacity.unwrap_or(0)),
            hasher: DefaultHashBuilder::default(),
        }
    }

    fn from_parts(
        nodes: impl IntoIterator<Item = DetachedNode<Self::NodeWeight>>,
        edges: impl IntoIterator<Item = DetachedEdge<Self::EdgeWeight>>,
    ) -> error_stack::Result<Self, Self::Error> {
        todo!()
    }

    fn into_parts(
        self,
    ) -> (
        impl Iterator<Item = DetachedNode<Self::NodeWeight>>,
        impl Iterator<Item = DetachedEdge<Self::EdgeWeight>>,
    ) {
        self.inner.into_parts()
    }

    fn num_nodes(&self) -> usize {
        self.inner.num_nodes()
    }

    fn num_edges(&self) -> usize {
        self.inner.num_edges()
    }

    fn next_node_id(&self) -> NodeId {
        self.inner.next_node_id()
    }

    fn insert_node(
        &mut self,
        id: NodeId,
        weight: Self::NodeWeight,
    ) -> error_stack::Result<NodeMut<Self>, Self::Error> {
        let hash = ValueHash::new(&self.hasher, &weight.key);

        if self.nodes.contains_key(&hash) {
            return Err(Report::new(MapError::NodeExists));
        }

        let node = self
            .inner
            .insert_node(id, weight)
            .change_context(MapError::UnderlyingStorage)?;
        self.nodes.insert(hash, id);

        // SAFETY: Any node in the inner storage is guaranteed to be valid for this graph storage
        Ok(unsafe { node.change_storage_unchecked() })
    }

    fn next_edge_id(&self) -> EdgeId {
        self.inner.next_edge_id()
    }

    fn insert_edge(
        &mut self,
        id: EdgeId,
        weight: Self::EdgeWeight,
        u: NodeId,
        v: NodeId,
    ) -> error_stack::Result<EdgeMut<Self>, Self::Error> {
        let hash = ValueHash::new(&self.hasher, &weight.key);

        if self.edges.contains_key(&hash) {
            return Err(Report::new(MapError::EdgeExists));
        }

        let edge = self
            .inner
            .insert_edge(id, weight, u, v)
            .change_context(MapError::UnderlyingStorage)?;
        self.edges.insert(hash, id);

        // SAFETY: Any edge in the inner storage is guaranteed to be valid for this graph storage
        Ok(unsafe { edge.change_storage_unchecked() })
    }

    fn remove_node(&mut self, id: NodeId) -> Option<DetachedNode<Self::NodeWeight>> {
        let node = self.inner.remove_node(id)?;
        let hash = ValueHash::new(&self.hasher, &node.weight.key);
        self.nodes.remove(&hash);

        Some(node)
    }

    fn remove_edge(&mut self, id: EdgeId) -> Option<DetachedEdge<Self::EdgeWeight>> {
        let edge = self.inner.remove_edge(id)?;
        let hash = ValueHash::new(&self.hasher, &edge.weight.key);
        self.edges.remove(&hash);

        Some(edge)
    }

    fn clear(&mut self) {
        self.inner.clear();
        self.nodes.clear();
        self.edges.clear();
    }

    fn node(&self, id: NodeId) -> Option<Node<Self>> {
        let node = self.inner.node(id)?;
        // SAFETY: Any node in the inner storage is guaranteed to be valid for this graph storage
        Some(unsafe { node.change_storage_unchecked() })
    }

    fn node_mut(&mut self, id: NodeId) -> Option<NodeMut<Self>> {
        let node = self.inner.node_mut(id)?;
        // SAFETY: Any node in the inner storage is guaranteed to be valid for this graph storage
        Some(unsafe { node.change_storage_unchecked() })
    }

    fn contains_node(&self, id: NodeId) -> bool {
        self.inner.contains_node(id)
    }

    fn edge(&self, id: EdgeId) -> Option<Edge<Self>> {
        let edge = self.inner.edge(id)?;

        // SAFETY: Any edge in the inner storage is guaranteed to be valid for this graph storage
        Some(unsafe { edge.change_storage_unchecked() })
    }

    fn edge_mut(&mut self, id: EdgeId) -> Option<EdgeMut<Self>> {
        let edge = self.inner.edge_mut(id)?;

        // SAFETY: Any edge in the inner storage is guaranteed to be valid for this graph storage
        Some(unsafe { edge.change_storage_unchecked() })
    }

    fn contains_edge(&self, id: EdgeId) -> bool {
        self.inner.contains_edge(id)
    }

    fn edges_between(&self, u: NodeId, v: NodeId) -> impl Iterator<Item = Edge<'_, Self>> {
        self.inner.edges_between(u, v).map(|edge| {
            // SAFETY: Any edge in the inner storage is guaranteed to be valid for this graph
            // storage
            unsafe { edge.change_storage_unchecked() }
        })
    }

    fn edges_between_mut(
        &mut self,
        u: NodeId,
        v: NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.inner.edges_between_mut(u, v).map(|edge| {
            // SAFETY: Any edge in the inner storage is guaranteed to be valid for this graph
            // storage
            unsafe { edge.change_storage_unchecked() }
        })
    }

    fn node_connections(&self, id: NodeId) -> impl Iterator<Item = Edge<'_, Self>> {
        self.inner.node_connections(id).map(|edge| {
            // SAFETY: Any edge in the inner storage is guaranteed to be valid for this graph
            // storage
            unsafe { edge.change_storage_unchecked() }
        })
    }

    fn node_connections_mut(&mut self, id: NodeId) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.inner.node_connections_mut(id).map(|edge| {
            // SAFETY: Any edge in the inner storage is guaranteed to be valid for this graph
            // storage
            unsafe { edge.change_storage_unchecked() }
        })
    }

    fn node_degree(&self, id: NodeId) -> usize {
        self.inner.node_degree(id)
    }

    fn node_neighbours(&self, id: NodeId) -> impl Iterator<Item = Node<'_, Self>> {
        self.inner.node_neighbours(id).map(|node| {
            // SAFETY: Any node in the inner storage is guaranteed to be valid for this graph
            // storage
            unsafe { node.change_storage_unchecked() }
        })
    }

    fn node_neighbours_mut(&mut self, id: NodeId) -> impl Iterator<Item = NodeMut<'_, Self>> {
        self.inner.node_neighbours_mut(id).map(|node| {
            // SAFETY: Any node in the inner storage is guaranteed to be valid for this graph
            // storage
            unsafe { node.change_storage_unchecked() }
        })
    }

    fn isolated_nodes(&self) -> impl Iterator<Item = Node<Self>> {
        self.inner.isolated_nodes().map(|node| {
            // SAFETY: Any node in the inner storage is guaranteed to be valid for this graph
            // storage
            unsafe { node.change_storage_unchecked() }
        })
    }

    fn isolated_nodes_mut(&mut self) -> impl Iterator<Item = NodeMut<Self>> {
        self.inner.isolated_nodes_mut().map(|node| {
            // SAFETY: Any node in the inner storage is guaranteed to be valid for this graph
            // storage
            unsafe { node.change_storage_unchecked() }
        })
    }

    fn nodes(&self) -> impl Iterator<Item = Node<Self>> {
        self.inner.nodes().map(|node| {
            // SAFETY: Any node in the inner storage is guaranteed to be valid for this graph
            // storage
            unsafe { node.change_storage_unchecked() }
        })
    }

    fn nodes_mut(&mut self) -> impl Iterator<Item = NodeMut<Self>> {
        self.inner.nodes_mut().map(|node| {
            // SAFETY: Any node in the inner storage is guaranteed to be valid for this graph
            // storage
            unsafe { node.change_storage_unchecked() }
        })
    }

    fn edges(&self) -> impl Iterator<Item = Edge<Self>> {
        self.inner.edges().map(|edge| {
            // SAFETY: Any edge in the inner storage is guaranteed to be valid for this graph
            // storage
            unsafe { edge.change_storage_unchecked() }
        })
    }

    fn edges_mut(&mut self) -> impl Iterator<Item = EdgeMut<Self>> {
        self.inner.edges_mut().map(|edge| {
            // SAFETY: Any edge in the inner storage is guaranteed to be valid for this graph
            // storage
            unsafe { edge.change_storage_unchecked() }
        })
    }

    fn reserve(&mut self, additional_nodes: usize, additional_edges: usize) {
        self.inner.reserve(additional_nodes, additional_edges);
        self.nodes.reserve(additional_nodes);
        self.edges.reserve(additional_edges);
    }

    fn reserve_nodes(&mut self, additional: usize) {
        self.inner.reserve_nodes(additional);
        self.nodes.reserve(additional);
    }

    fn reserve_edges(&mut self, additional: usize) {
        self.inner.reserve_edges(additional);
        self.edges.reserve(additional);
    }

    fn reserve_exact(&mut self, additional_nodes: usize, additional_edges: usize) {
        self.inner.reserve_exact(additional_nodes, additional_edges);

        self.nodes.reserve(additional_nodes);
        self.edges.reserve(additional_edges);
    }

    fn reserve_exact_nodes(&mut self, additional: usize) {
        self.inner.reserve_exact_nodes(additional);
        self.nodes.reserve(additional);
    }

    fn reserve_exact_edges(&mut self, additional: usize) {
        self.inner.reserve_exact_edges(additional);
        self.edges.reserve(additional);
    }

    fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit();
        self.nodes.shrink_to_fit();
        self.edges.shrink_to_fit();
    }

    fn shrink_to_fit_nodes(&mut self) {
        self.inner.shrink_to_fit_nodes();
        self.nodes.shrink_to_fit();
    }

    fn shrink_to_fit_edges(&mut self) {
        self.inner.shrink_to_fit_edges();
        self.edges.shrink_to_fit();
    }
}
