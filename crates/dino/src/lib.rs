#![feature(return_position_impl_trait_in_trait)]
#![no_std]

pub(crate) mod closure;
mod directed;
mod edge;
mod linear;
mod node;
mod resize;
mod retain;
pub(crate) mod slab;
#[cfg(test)]
mod tests;

extern crate alloc;

use core::{
    fmt::{Debug, Display},
    iter::empty,
};

use ::either::Either;
pub use edge::EdgeId;
use error_stack::{Context, Report, Result};
pub use node::NodeId;
use petgraph_core::{
    edge::{
        marker::{Directed, GraphDirection, Undirected},
        DetachedEdge, EdgeMut,
    },
    graph::Graph,
    id::GraphId,
    node::{DetachedNode, NodeMut},
    storage::GraphStorage,
};

use crate::{closure::Closures, edge::Edge, node::Node, slab::Slab};

pub type DinoGraph<N, E, D> = Graph<DinosaurStorage<N, E, D>>;
pub type DiDinoGraph<N, E> = DinoGraph<N, E, Directed>;
pub type UnDinoGraph<N, E> = DinoGraph<N, E, Undirected>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DinosaurStorage<N, E, D = Directed>
where
    D: GraphDirection,
{
    nodes: Slab<NodeId, Node<N>>,
    edges: Slab<EdgeId, Edge<E>>,

    closures: Closures,

    _marker: core::marker::PhantomData<fn() -> *const D>,
}

impl<N, E, D> DinosaurStorage<N, E, D>
where
    D: GraphDirection,
{
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(None, None)
    }
}

impl<N, E, D> Default for DinosaurStorage<N, E, D>
where
    D: GraphDirection,
{
    fn default() -> Self {
        Self::new()
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

impl<N, E, D> GraphStorage for DinosaurStorage<N, E, D>
where
    D: GraphDirection,
{
    type EdgeId = EdgeId;
    type EdgeWeight = E;
    type Error = ExtinctionEvent;
    type NodeId = NodeId;
    type NodeWeight = N;

    fn with_capacity(node_capacity: Option<usize>, edge_capacity: Option<usize>) -> Self {
        Self {
            nodes: Slab::with_capacity(node_capacity),
            edges: Slab::with_capacity(edge_capacity),

            closures: Closures::new(),

            _marker: core::marker::PhantomData,
        }
    }

    fn from_parts(
        nodes: impl IntoIterator<Item = DetachedNode<Self::NodeId, Self::NodeWeight>>,
        edges: impl IntoIterator<Item = DetachedEdge<Self::EdgeId, Self::NodeId, Self::EdgeWeight>>,
    ) -> Result<Self, Self::Error> {
        let nodes: Slab<_, _> = nodes
            .into_iter()
            .map(|node: DetachedNode<Self::NodeId, Self::NodeWeight>| {
                (node.id, Node::new(node.id, node.weight))
            })
            .collect();

        let edges: Slab<_, _> = edges
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

        let mut closures = Closures::new();
        closures.refresh(&nodes, &edges);

        Ok(Self {
            nodes,
            edges,
            closures,

            _marker: core::marker::PhantomData,
        })
    }

    fn into_parts(
        self,
    ) -> (
        impl Iterator<Item = DetachedNode<Self::NodeId, Self::NodeWeight>>,
        impl Iterator<Item = DetachedEdge<Self::EdgeId, Self::NodeId, Self::EdgeWeight>>,
    ) {
        let nodes = self.nodes.into_iter().map(|node| DetachedNode {
            id: node.id,
            weight: node.weight,
        });

        let edges = self.edges.into_iter().map(|edge| DetachedEdge {
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
        self.nodes.next_key()
    }

    fn insert_node(
        &mut self,
        id: Self::NodeId,
        weight: Self::NodeWeight,
    ) -> Result<NodeMut<Self>, Self::Error> {
        let expected = id;
        let id = self.nodes.insert(Node::new(expected, weight));

        // TODO: we might want to make this `debug_assert_eq` or a warning.
        assert_eq!(
            id, expected,
            "The id of the inserted node is not the same as one returned by the insertion \
             operation, if you retrieved the id from `next_node_id`, and in between the two \
             functions calls you have not mutated the graph, then this is likely a bug in the \
             library, please report it."
        );

        self.closures.update_node(id);

        let node = self
            .nodes
            .get_mut(id)
            .ok_or_else(|| Report::new(ExtinctionEvent::NodeNotFound))?;
        // we do not need to set the node's id, since the assertion above guarantees that the id is
        // correct

        Ok(NodeMut::new(&node.id, &mut node.weight))
    }

    fn next_edge_id(&self, _: <Self::EdgeId as GraphId>::AttributeIndex) -> Self::EdgeId {
        self.edges.next_key()
    }

    fn insert_edge(
        &mut self,
        id: Self::EdgeId,
        weight: Self::EdgeWeight,

        source: &Self::NodeId,
        target: &Self::NodeId,
    ) -> Result<EdgeMut<Self>, Self::Error> {
        let expected = id;
        let id = self
            .edges
            .insert(Edge::new(expected, weight, *source, *target));

        // TODO: we might want to make this `debug_assert_eq` or a warning.
        assert_eq!(
            id, expected,
            "The id of the inserted edge is not the same as one returned by the insertion \
             operation, if you retrieved the id from `next_edge_id`, and in between the two \
             functions calls you have not mutated the graph, then this is likely a bug in the \
             library, please report it."
        );

        let edge = self
            .edges
            .get_mut(id)
            .ok_or_else(|| Report::new(ExtinctionEvent::EdgeNotFound))?;
        // we do not need to set the node's id, since the assertion above guarantees that the id is
        // correct

        self.closures.update_edge(edge);

        Ok(EdgeMut::new(
            &edge.id,
            &mut edge.weight,
            &edge.source,
            &edge.target,
        ))
    }

    fn remove_node(
        &mut self,
        id: &Self::NodeId,
    ) -> Option<DetachedNode<Self::NodeId, Self::NodeWeight>> {
        if let Some(node) = self.closures.nodes.get(*id) {
            for edge in node.edges() {
                self.edges.remove(*edge);
            }
        }

        let node = self.nodes.remove(*id)?;
        self.closures.remove_node(*id);

        Some(DetachedNode::new(node.id, node.weight))
    }

    fn remove_edge(
        &mut self,
        id: &Self::EdgeId,
    ) -> Option<DetachedEdge<Self::EdgeId, Self::NodeId, Self::EdgeWeight>> {
        let edge = self.edges.remove(*id)?;
        self.closures.remove_edge(&edge);

        Some(DetachedEdge::new(
            edge.id,
            edge.weight,
            edge.source,
            edge.target,
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
            .get(*id)
            .map(|node| petgraph_core::node::Node::new(self, &node.id, &node.weight))
    }

    fn node_mut(&mut self, id: &Self::NodeId) -> Option<NodeMut<Self>> {
        self.nodes
            .get_mut(*id)
            .map(|node| NodeMut::new(&node.id, &mut node.weight))
    }

    fn contains_node(&self, id: &Self::NodeId) -> bool {
        self.nodes.contains_key(*id)
    }

    fn edge(&self, id: &Self::EdgeId) -> Option<petgraph_core::edge::Edge<Self>> {
        self.edges.get(*id).map(|edge| {
            petgraph_core::edge::Edge::new(self, &edge.id, &edge.weight, &edge.source, &edge.target)
        })
    }

    fn edge_mut(&mut self, id: &Self::EdgeId) -> Option<EdgeMut<Self>> {
        self.edges
            .get_mut(*id)
            .map(|edge| EdgeMut::new(&edge.id, &mut edge.weight, &edge.source, &edge.target))
    }

    fn contains_edge(&self, id: &Self::EdgeId) -> bool {
        self.edges.contains_key(*id)
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
                .iter_mut()
                .filter(move |edge| available.contains(&edge.id))
                .map(move |edge| {
                    EdgeMut::new(&edge.id, &mut edge.weight, &edge.source, &edge.target)
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
                .iter_mut()
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

        Either::Right(nodes.entries_mut().filter_map(move |(id, node)| {
            let is_external = externals.contains(&id);

            is_external.then(|| NodeMut::new(&node.id, &mut node.weight))
        }))
    }

    fn nodes(&self) -> impl Iterator<Item = petgraph_core::node::Node<Self>> {
        self.nodes
            .iter()
            .map(move |node| petgraph_core::node::Node::new(self, &node.id, &node.weight))
    }

    fn nodes_mut(&mut self) -> impl Iterator<Item = NodeMut<Self>> {
        self.nodes
            .iter_mut()
            .map(move |node| NodeMut::new(&node.id, &mut node.weight))
    }

    fn edges(&self) -> impl Iterator<Item = petgraph_core::edge::Edge<Self>> {
        self.edges.iter().map(move |edge| {
            petgraph_core::edge::Edge::new(self, &edge.id, &edge.weight, &edge.source, &edge.target)
        })
    }

    fn edges_mut(&mut self) -> impl Iterator<Item = EdgeMut<Self>> {
        self.edges
            .iter_mut()
            .map(move |edge| EdgeMut::new(&edge.id, &mut edge.weight, &edge.source, &edge.target))
    }
}
