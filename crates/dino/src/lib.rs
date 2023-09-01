#![feature(return_position_impl_trait_in_trait)]
#![no_std]

mod closure;
mod common;
mod directed;
mod edge;
mod node;

extern crate alloc;

use core::{
    fmt::{Debug, Display},
    iter::empty,
};

pub use edge::EdgeId;
use either::Either;
use error_stack::{Context, Report, Result};
use hashbrown::{HashMap, HashSet};
pub use node::NodeId;
use petgraph_core::{
    edge::{DetachedEdge, EdgeMut},
    id::GraphId,
    node::{DetachedNode, NodeMut},
    storage::GraphStorage,
};

use crate::{closure::Closures, edge::Edge, node::Node};

pub struct Undirected;
pub struct Directed;

pub struct DinosaurStorage<N, E, D = Directed> {
    nodes: HashMap<NodeId, Node<N>>,
    edges: HashMap<EdgeId, Edge<E>>,

    closures: Closures,

    last_node_id: NodeId,
    last_edge_id: EdgeId,

    _marker: core::marker::PhantomData<fn() -> *const D>,
}

impl<N, E, D> DinosaurStorage<N, E, D> {
    fn new() -> Self {
        Self::with_capacity(None, None)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ExtinctionEvent {
    NodeNotFound,
    EdgeNotFound,
}

impl Display for ExtinctionEvent {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NodeNotFound => f.write_str("node not found"),
            Self::EdgeNotFound => f.write_str("edge not found"),
        }
    }
}

impl Context for ExtinctionEvent {}

// TODO: optional functions (+ directed) + linear
impl<N, E, D> GraphStorage for DinosaurStorage<N, E, D> {
    type EdgeId = EdgeId;
    type EdgeWeight = E;
    type Error = ExtinctionEvent;
    type NodeId = NodeId;
    type NodeWeight = N;

    fn with_capacity(node_capacity: Option<usize>, edge_capacity: Option<usize>) -> Self {
        Self {
            nodes: HashMap::with_capacity(node_capacity.unwrap_or(0)),
            edges: HashMap::with_capacity(edge_capacity.unwrap_or(0)),

            closures: Closures::new(),

            last_node_id: NodeId::new(0),
            last_edge_id: EdgeId::new(0),

            _marker: core::marker::PhantomData,
        }
    }

    fn from_parts(
        nodes: impl IntoIterator<Item = DetachedNode<Self::NodeId, Self::NodeWeight>>,
        edges: impl IntoIterator<Item = DetachedEdge<Self::EdgeId, Self::NodeId, Self::EdgeWeight>>,
    ) -> Result<Self, Self::Error> {
        let nodes: HashMap<_, _> = nodes
            .into_iter()
            .map(|node: DetachedNode<Self::NodeId, Self::NodeWeight>| {
                (node.id, Node::new(node.id, node.weight))
            })
            .collect();

        let edges: HashMap<_, _> = edges
            .into_iter()
            .map(
                |edge: DetachedEdge<Self::EdgeId, Self::NodeId, Self::EdgeWeight>| {
                    (
                        edge.id,
                        Edge::new(edge.id, edge.weight, edge.source, edge.target),
                    )
                },
            )
            .collect();

        let last_node_id = nodes.keys().max().copied().unwrap_or(NodeId::new(0));
        let last_edge_id = edges.keys().max().copied().unwrap_or(EdgeId::new(0));

        let mut closures = Closures::new();
        closures.refresh(&nodes, &edges);

        Ok(Self {
            nodes,
            edges,
            closures,

            last_node_id,
            last_edge_id,

            _marker: core::marker::PhantomData,
        })
    }

    fn into_parts(
        self,
    ) -> (
        impl Iterator<Item = DetachedNode<Self::NodeId, Self::NodeWeight>>,
        impl Iterator<Item = DetachedEdge<Self::EdgeId, Self::NodeId, Self::EdgeWeight>>,
    ) {
        let nodes = self.nodes.into_iter().map(|(_, node)| DetachedNode {
            id: node.id,
            weight: node.weight,
        });

        let edges = self.edges.into_iter().map(|(_, edge)| DetachedEdge {
            id: edge.id,
            source: edge.source,
            target: edge.target,
            weight: edge.weight,
        });

        (nodes, edges)
    }

    fn num_nodes(&self) -> usize {
        self.nodes.len()
    }

    fn num_edges(&self) -> usize {
        self.edges.len()
    }

    fn next_node_id(&self, _: <Self::NodeId as GraphId>::AttributeIndex) -> Self::NodeId {
        self.last_node_id.increment()
    }

    fn insert_node(
        &mut self,
        id: Self::NodeId,
        weight: Self::NodeWeight,
    ) -> Result<petgraph_core::node::Node<Self>, Self::Error> {
        self.nodes.insert(id, Node::new(id, weight));
        self.closures.update_node(id);

        // to not prolong the lifetime of the mutable borrow, we need to get the newest entry
        // instead of reusing the occupied reference.
        // The main problem here is actually that we need to return ourselves (`self`), which means
        // we cannot reuse.
        // TODO: in the future look if we don't want to return `NodeMut` instead, which would allow
        //  us to reuse the reference.
        let node = self
            .nodes
            .get(&id)
            .ok_or_else(|| Report::new(ExtinctionEvent::NodeNotFound))?;

        Ok(petgraph_core::node::Node::new(self, &node.id, &node.weight))
    }

    fn next_edge_id(&self, _: <Self::EdgeId as GraphId>::AttributeIndex) -> Self::EdgeId {
        self.last_edge_id.increment()
    }

    fn insert_edge(
        &mut self,
        id: Self::EdgeId,
        source: Self::NodeId,
        target: Self::NodeId,
        weight: Self::EdgeWeight,
    ) -> Result<petgraph_core::edge::Edge<Self>, Self::Error> {
        self.edges.insert(id, Edge::new(id, weight, source, target));
        // we now need to get the newest entry, problem here is that we cannot use the result from
        // `self.edges`, because then we would prolong the lifetime of the mutable borrow.

        let edge = self
            .edges
            .get(&id)
            .ok_or_else(|| Report::new(ExtinctionEvent::EdgeNotFound))?;

        self.closures.update_edge(edge);

        Ok(petgraph_core::edge::Edge::new(
            self,
            &edge.id,
            &edge.source,
            &edge.target,
            &edge.weight,
        ))
    }

    fn remove_node(
        &mut self,
        id: &Self::NodeId,
    ) -> Option<DetachedNode<Self::NodeId, Self::NodeWeight>> {
        let node = self.nodes.remove(id)?;
        self.closures.remove_node(*id);

        Some(DetachedNode::new(node.id, node.weight))
    }

    fn remove_edge(
        &mut self,
        id: &Self::EdgeId,
    ) -> Option<DetachedEdge<Self::EdgeId, Self::NodeId, Self::EdgeWeight>> {
        let edge = self.edges.remove(id)?;
        self.closures.remove_edge(&edge);

        Some(DetachedEdge::new(
            edge.id,
            edge.source,
            edge.target,
            edge.weight,
        ))
    }

    fn clear(&mut self) -> Result<(), Self::Error> {
        self.nodes.clear();
        self.edges.clear();
        self.closures.clear();

        Ok(())
    }

    fn node(&self, id: &Self::NodeId) -> Option<petgraph_core::node::Node<Self>> {
        self.nodes
            .get(id)
            .map(|node| petgraph_core::node::Node::new(self, &node.id, &node.weight))
    }

    fn node_mut(&mut self, id: &Self::NodeId) -> Option<NodeMut<Self>> {
        self.nodes
            .get_mut(id)
            .map(|node| NodeMut::new(&node.id, &mut node.weight))
    }

    fn contains_node(&self, id: &Self::NodeId) -> bool {
        self.nodes.contains_key(id)
    }

    fn edge(&self, id: &Self::EdgeId) -> Option<petgraph_core::edge::Edge<Self>> {
        self.edges.get(id).map(|edge| {
            petgraph_core::edge::Edge::new(self, &edge.id, &edge.source, &edge.target, &edge.weight)
        })
    }

    fn edge_mut(&mut self, id: &Self::EdgeId) -> Option<EdgeMut<Self>> {
        self.edges
            .get_mut(id)
            .map(|edge| EdgeMut::new(&edge.id, &edge.source, &edge.target, &mut edge.weight))
    }

    fn contains_edge(&self, id: &Self::EdgeId) -> bool {
        self.edges.contains_key(id)
    }

    fn find_undirected_edges<'a: 'b, 'b>(
        &'a self,
        source: &'b Self::NodeId,
        target: &'b Self::NodeId,
    ) -> impl Iterator<Item = petgraph_core::edge::Edge<'a, Self>> + 'b {
        let source_to_target = self
            .closures
            .edges
            .endpoints_to_edges()
            .get(&(*source, *target));
        let target_to_source = self
            .closures
            .edges
            .endpoints_to_edges()
            .get(&(*target, *source));

        source_to_target
            .into_iter()
            .flatten()
            .chain(target_to_source.into_iter().flatten())
            .filter_map(move |edge| self.edge(edge))
    }

    fn node_connections<'a: 'b, 'b>(
        &'a self,
        id: &'b Self::NodeId,
    ) -> impl Iterator<Item = petgraph_core::edge::Edge<'a, Self>> + 'b {
        self.closures
            .nodes
            .get(*id)
            .into_iter()
            .flat_map(closure::NodeClosure::edges)
            .filter_map(|edge| self.edge(edge))
    }

    fn node_connections_mut<'a: 'b, 'b>(
        &'a mut self,
        id: &'b Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'a, Self>> + 'b {
        let Self {
            closures, edges, ..
        } = self;

        let Some(closure) = closures.nodes.get(*id) else {
            return Either::Left(empty());
        };

        let available = closure.edges();

        if available.is_empty() {
            return Either::Left(empty());
        }

        Either::Right(
            edges
                .values_mut()
                .filter(move |edge| available.contains(&edge.id))
                .map(move |edge| {
                    EdgeMut::new(&edge.id, &edge.source, &edge.target, &mut edge.weight)
                }),
        )
    }

    fn node_neighbours<'a: 'b, 'b>(
        &'a self,
        id: &'b Self::NodeId,
    ) -> impl Iterator<Item = petgraph_core::node::Node<'a, Self>> + 'b {
        self.closures
            .nodes
            .get(*id)
            .into_iter()
            .flat_map(closure::NodeClosure::neighbours)
            .filter_map(move |node| self.node(node))
    }

    fn node_neighbours_mut<'a: 'b, 'b>(
        &'a mut self,
        id: &'b Self::NodeId,
    ) -> impl Iterator<Item = NodeMut<'a, Self>> + 'b {
        let Self {
            closures, nodes, ..
        } = self;

        let Some(closure) = closures.nodes.get(*id) else {
            return Either::Left(empty());
        };

        let available = closure.neighbours();

        if available.is_empty() {
            return Either::Left(empty());
        }

        Either::Right(
            nodes
                .values_mut()
                .filter(move |node| available.contains(&node.id))
                .map(move |node| NodeMut::new(&node.id, &mut node.weight)),
        )
    }

    fn external_nodes(&self) -> impl Iterator<Item = petgraph_core::node::Node<Self>> {
        self.closures
            .nodes
            .externals()
            .iter()
            .filter_map(move |node| self.node(&node))
    }

    fn external_nodes_mut(&mut self) -> impl Iterator<Item = NodeMut<Self>> {
        let Self {
            nodes, closures, ..
        } = self;

        let externals = closures.nodes.externals();

        if externals.is_empty() {
            return Either::Left(empty());
        }

        Either::Right(nodes.iter_mut().filter_map(move |(id, node)| {
            let is_external = externals.contains(id);

            is_external.then(|| NodeMut::new(id, &mut node.weight))
        }))
    }

    fn nodes(&self) -> impl Iterator<Item = petgraph_core::node::Node<Self>> {
        self.nodes
            .iter()
            .map(move |(id, node)| petgraph_core::node::Node::new(self, id, &node.weight))
    }

    fn nodes_mut(&mut self) -> impl Iterator<Item = NodeMut<Self>> {
        self.nodes
            .iter_mut()
            .map(move |(id, node)| NodeMut::new(id, &mut node.weight))
    }

    fn edges(&self) -> impl Iterator<Item = petgraph_core::edge::Edge<Self>> {
        self.edges.iter().map(move |(id, edge)| {
            petgraph_core::edge::Edge::new(self, id, &edge.source, &edge.target, &edge.weight)
        })
    }

    fn edges_mut(&mut self) -> impl Iterator<Item = EdgeMut<Self>> {
        self.edges
            .iter_mut()
            .map(move |(id, edge)| EdgeMut::new(id, &edge.source, &edge.target, &mut edge.weight))
    }
}
