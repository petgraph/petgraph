//! # `petgraph-dino`: **D**irected and undirected **in**dex-**o**ptimized graphs.
//!
//! A general-purpose, powerful and efficient [`GraphStorage`] implementation that is designed to
//! handle graphs with parallel edges and self-loops.
//!
//! ## Overview
//!
//! Graphs are a fundamental data structure in various domains, including, but not limited to,
//! network analysis, computational biology, and social network analysis.
//! This library is centered around the [`DinoStorage`] type, an implementation of the
//! [`GraphStorage`] trait from [`petgraph-core`](petgraph_core),
//! which offers powerful capabilities to manage both directed and undirected graphs with parallel
//! edges and self-loops.
//! Indices into the graph are stable and are managed by the graph itself, meaning that they cannot
//! be freely chosen by the user.
//! Convenient aliases are provided for directed and undirected graphs, namely [`DinoGraph`],
//! [`DiDinoGraph`] for directed graphs, and [`UnDinoGraph`] for undirected graphs.
//!
//! [`DiDinoGraph`], and by extension [`DinoStorage`], are general purpose implementations,
//! designed to cater to a wide range of graph-related applications and use cases.
//!
//! ## Features
//!
//! - **General-purpose**: [`DinoStorage`] is designed to cater to a wide range of graph-related
//!   applications and use cases.
//! - **Parallel edges**: [`DinoStorage`] supports parallel edges, i.e., multiple edges between the
//!   same pair of nodes.
//! - **Self-loops**: [`DinoStorage`] supports self-loops, i.e., edges that connect a node to
//!   itself.
//! - **Managed indices**: Indices into the graph are stable and are managed by the graph itself, so
//!   they cannot be freely chosen by the user.
//! - **Directed and undirected graphs**: [`DinoStorage`] supports both directed and undirected
//!   graphs.
//! - **Generational Arena**: Edges and nodes are stored in an generational arena modelled after the
//!   excellent [`alot`] crate, which offers stable indices for a minimal overhead of two bytes per
//!   entry.
//! - **Compressed Bitmaps**: Using roaring bitmaps, `petgraph-dino` is able to efficiently store a
//!   lookup of relationships and is able to query them in constant time, while using a minimal
//!   amount of memory.
//!
//! ## Usage
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! petgraph-dino = "0.1.0"
//! ```
//!
//! ## Examples
//!
//! ### Creating a graph
//!
//! ```rust
//! use petgraph_core::{
//!     edge::marker::{Directed, Undirected},
//!     Graph,
//! };
//! use petgraph_dino::{DiDinoGraph, DinoGraph, DinoStorage, UnDinoGraph};
//!
//! // when inserting nodes and edges the weights can be inferred, in this case we're not doing that
//! // so need to specify the types explicitly.
//!
//! let mut digraph = DiDinoGraph::<(), ()>::new();
//! // or:
//! let mut digraph = DinoGraph::<(), (), Directed>::new();
//! // or:
//! let mut digraph = Graph::<DinoStorage<(), (), Directed>>::new();
//!
//! let mut ungraph = UnDinoGraph::<(), ()>::new();
//! // or:
//! let mut ungraph = DinoGraph::<(), (), Undirected>::new();
//! // or:
//! let mut ungraph = Graph::<DinoStorage<(), (), Undirected>>::new();
//! ```
//!
//! ### Inserting and removing nodes and edges
//!
//! ```
//! use petgraph_dino::DiDinoGraph;
//!
//! let mut graph = DiDinoGraph::new();
//!
//! let a = *graph.insert_node("A").id();
//! let b = *graph.insert_node("B").id();
//! let c = *graph.insert_node("C").id();
//!
//! let ab = *graph.insert_edge("A → B", &a, &b).id();
//! let bc = *graph.insert_edge("B → C", &b, &c).id();
//! let ca = *graph.insert_edge("C → A", &c, &a).id();
//!
//! assert_eq!(graph.num_nodes(), 3);
//! assert_eq!(graph.num_edges(), 3);
//!
//! assert_eq!(graph.remove_node(&a).unwrap().weight, "A");
//!
//! assert_eq!(graph.num_nodes(), 2);
//! assert_eq!(graph.num_edges(), 1);
//! ```
//!
//! ### Iterating over nodes and edges
// TODO
//!
#![feature(return_position_impl_trait_in_trait)]
#![warn(missing_docs)]
#![no_std]

pub(crate) mod closure;
mod directed;
mod edge;
mod node;

mod retain;
pub(crate) mod slab;
#[cfg(test)]
mod tests;

extern crate alloc;

use core::fmt::{Debug, Display};

pub use edge::EdgeId;
use either::Either;
use error_stack::{Context, Report, Result};
pub use node::NodeId;
use petgraph_core::{
    edge::{
        marker::{Directed, GraphDirectionality, Undirected},
        DetachedEdge, EdgeMut,
    },
    id::GraphId,
    node::{DetachedNode, NodeMut},
    storage::GraphStorage,
    Graph,
};

use crate::{
    closure::Closures,
    edge::Edge,
    node::{Node, NodeClosures},
    slab::Slab,
};

/// Alias for a [`Graph`] that uses [`DinoStorage`] as its backing storage.
///
/// [`DinoGraph`] is a convenient alias for [`Graph`] that uses [`DinoStorage`] as its backing
/// storage.
///
/// It allows you to work with graphs supporting parallel edges and self-loops, and both directed
/// and undirected graphs.
///
/// # Type Parameters
///
/// - `N`: The type of the node weight.
/// - `E`: The type of the edge weight.
/// - `D`: The directionality of the graph, either [`Directed`] or [`Undirected`].
///
/// # Example
///
/// ```
/// use petgraph_core::edge::marker::Directed;
/// use petgraph_dino::DinoGraph;
///
/// #[derive(Debug)]
/// struct Node;
///
/// #[derive(Debug)]
/// struct Edge;
///
/// type Graph = DinoGraph<Node, Edge, Directed>;
///
/// let mut graph = Graph::new();
///
/// let a = *graph.insert_node(Node).id();
/// let b = *graph.insert_node(Node).id();
///
/// let ab = *graph.insert_edge(Edge, &a, &b).id();
/// ```
pub type DinoGraph<N, E, D> = Graph<DinoStorage<N, E, D>>;

/// Alias for a directed [`Graph`] that uses [`DinoStorage`] as its backing storage.
///
/// [`DiDinoGraph`] is a convenient alias for a directed [`Graph`] that uses [`DinoStorage`] as
/// its backing storage.
///
/// It allows you to work with graphs supporting parallel edges and self-loops and directed edges
/// only.
///
/// # Type Parameters
///
/// - `N`: The type of the node weight.
/// - `E`: The type of the edge weight.
///
/// # Example
///
/// ```
/// use petgraph_core::edge::marker::Directed;
/// use petgraph_dino::DiDinoGraph;
///
/// #[derive(Debug)]
/// struct Node;
///
/// #[derive(Debug)]
/// struct Edge;
///
/// type Graph = DiDinoGraph<Node, Edge>;
///
/// let mut graph = Graph::new();
///
/// let a = *graph.insert_node(Node).id();
/// let b = *graph.insert_node(Node).id();
///
/// let ab = *graph.insert_edge(Edge, &a, &b).id();
/// ```
pub type DiDinoGraph<N, E> = DinoGraph<N, E, Directed>;

/// Alias for an undirected [`Graph`] that uses [`DinoStorage`] as its backing storage.
///
/// [`UnDinoGraph`] is a convenient alias for an undirected [`Graph`] that uses [`DinoStorage`]
/// as its backing storage.
///
/// It allows you to work with graphs supporting parallel edges and self-loops and undirected edges
/// only.
///
/// # Type Parameters
///
/// - `N`: The type of the node weight.
/// - `E`: The type of the edge weight.
///
/// # Example
///
/// ```
/// use petgraph_core::edge::marker::Directed;
/// use petgraph_dino::UnDinoGraph;
///
/// #[derive(Debug)]
/// struct Node;
///
/// #[derive(Debug)]
/// struct Edge;
///
/// type Graph = UnDinoGraph<Node, Edge>;
///
/// let mut graph = Graph::new();
///
/// let a = *graph.insert_node(Node).id();
/// let b = *graph.insert_node(Node).id();
///
/// let ab = *graph.insert_edge(Edge, &a, &b).id();
/// ```
pub type UnDinoGraph<N, E> = DinoGraph<N, E, Undirected>;

/// [`GraphStorage`] implementation that supports parallel edges and self-loops.
///
/// It uses roaring bitmaps to efficiently store a lookup of relationships and is able to query them
/// in constant time, while using a minimal amount of memory.
/// Nodes and edges are stored in a generational arena modelled after the excellent [`alot`] crate,
/// which offers stable indices for a minimal overhead of two bytes per entry.
///
/// # Type Parameters
///
/// - `N`: The type of the node weight.
/// - `E`: The type of the edge weight.
/// - `D`: The directionality of the graph, either [`Directed`] or [`Undirected`].
///
/// # Capabilities
///
/// > This is a template, which you should use to describe the capabilities of your graph storage
/// > implementation.
///
/// | Capability       | Note              |
/// |------------------|-------------------|
/// | Node Identifiers | Managed           |
/// | Edge Identifiers | Managed           |
/// | Node Weights     | ✓                 |
/// | Edge Weights     | ✓                 |
/// | Parallel Edges   | ✓                 |
/// | Self Loops       | ✓                 |
///
/// ## Space/Time Complexity
///
/// | Operation        | Time Complexity | Space Complexity |
/// |------------------|-----------------|------------------|
/// | Node By Id       | N/A             | N/A              |
/// | Edge By Id       | N/A             | N/A              |
/// | Edge Between     | N/A             | N/A              |
/// | Contains Node    | N/A             | N/A              |
/// | Contains Edge    | N/A             | N/A              |
/// | Insert Node      | N/A             | N/A              |
/// | Insert Edge      | N/A             | N/A              |
/// | Remove Node      | N/A             | N/A              |
/// | Remove Edge      | N/A             | N/A              |
/// | Node Count       | N/A             | N/A              |
/// | Edge Count       | N/A             | N/A              |
/// | Node Iter        | N/A             | N/A              |
/// | Edge Iter        | N/A             | N/A              |
/// | Node Neighbours  | N/A             | N/A              |
/// | Node Connections | N/A             | N/A              |
/// | External Nodes   | N/A             | N/A              |
///
/// ### Directed Graphs
///
/// | Operation                     | Time Complexity | Space Complexity |
/// |-------------------------------|-----------------|------------------|
/// | Edge Between                  | N/A             | N/A              |
/// | Directed Edge Neighbours      | N/A             | N/A              |
/// | Undirected Edge Neighbours    | N/A             | N/A              |
/// | Directed Edge Connections     | N/A             | N/A              |
/// | Undirected Edge Connections   | N/A             | N/A              |
///
///
/// # Example
///
/// ```
/// use petgraph_core::edge::marker::Directed;
/// use petgraph_dino::DinoStorage;
///
/// #[derive(Debug)]
/// struct Node;
///
/// #[derive(Debug)]
/// struct Edge;
///
/// type Graph = petgraph_core::Graph<DinoStorage<Node, Edge, Directed>>;
///
/// let mut graph = Graph::new();
///
/// let a = *graph.insert_node(Node).id();
/// let b = *graph.insert_node(Node).id();
///
/// let ab = *graph.insert_edge(Edge, &a, &b).id();
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct DinoStorage<N, E, D = Directed>
where
    D: GraphDirectionality,
{
    nodes: Slab<NodeId, Node<N>>,
    edges: Slab<EdgeId, Edge<E>>,

    closures: Closures,

    _marker: core::marker::PhantomData<fn() -> *const D>,
}

impl<N, E, D> DinoStorage<N, E, D>
where
    D: GraphDirectionality,
{
    /// Creates a new, empty [`DinoStorage`].
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::edge::marker::Directed;
    /// use petgraph_dino::DinoStorage;
    ///
    /// let storage = DinoStorage::<(), (), Directed>::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(None, None)
    }
}

impl<N, E, D> Default for DinoStorage<N, E, D>
where
    D: GraphDirectionality,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Error type for [`DinoStorage`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Error {
    /// The requested node was not found.
    NodeNotFound,
    /// The requested edge was not found.
    EdgeNotFound,
    /// The given node id does not match the id of the just inserted node.
    /// If this error occurs while using [`Graph`] this is most likely an internal error and
    /// should've never happened.
    /// Please report it.
    InconsistentNodeId,
    /// The given edge id does not match the id of the just inserted edge.
    ///
    /// If this error occurs while using [`Graph`] this is most likely an internal error and
    /// should've never happened.
    /// Please report it.
    InconsistentEdgeId,
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NodeNotFound => f.write_str("node not found"),
            Self::EdgeNotFound => f.write_str("edge not found"),
            Self::InconsistentNodeId => f.write_str(
                "The id of the inserted node is not the same as one returned by the insertion \
                 operation, if you retrieved the id from `next_node_id`, and in between the two \
                 functions calls you have not mutated the graph, then this is likely a bug in the \
                 library, please report it.",
            ),
            Self::InconsistentEdgeId => f.write_str(
                "The id of the inserted edge is not the same as one returned by the insertion \
                 operation, if you retrieved the id from `next_edge_id`, and in between the two \
                 functions calls you have not mutated the graph, then this is likely a bug in the \
                 library, please report it.",
            ),
        }
    }
}

impl Context for Error {}

impl<N, E, D> GraphStorage for DinoStorage<N, E, D>
where
    D: GraphDirectionality,
{
    type EdgeId = EdgeId;
    type EdgeWeight = E;
    type Error = Error;
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
        let mut nodes: Slab<_, _> = nodes
            .into_iter()
            .map(|node: DetachedNode<Self::NodeId, Self::NodeWeight>| {
                (node.id, Node::new(node.id, node.weight))
            })
            .collect();

        let edges: Slab<_, _> = edges
            .into_iter()
            .map(
                |edge: DetachedEdge<Self::EdgeId, Self::NodeId, Self::EdgeWeight>| {
                    (edge.id, Edge::new(edge.id, edge.weight, edge.u, edge.v))
                },
            )
            .collect();

        // TODO: test-case c:
        // TODO: this doesn't work if we remove a node
        // TODO: NodeId rename is not of concern for us though
        // TODO: what about nodes that are added or edges?
        //      We don't know their ID yet (need a way to get those -> PartialNode/Edge)

        let mut closures = Closures::new();
        closures.refresh(&mut nodes, &edges);

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
            u: edge.source,
            v: edge.target,
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

        if id != expected {
            // delete the node we just inserted
            // we don't need to update the closures, since we haven't added the node to them yet
            self.nodes.remove(id);

            return Err(Report::new(Error::InconsistentNodeId));
        }

        let node = self
            .nodes
            .get_mut(id)
            .ok_or_else(|| Report::new(Error::NodeNotFound))?;

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
        // TODO: option to disallow self-loops and parallel edges

        // undirected edges in the graph are stored in a canonical form, where the source node id is
        // always smaller than the target node id
        let (source, target) = if D::is_directed() {
            (*source, *target)
        } else if source > target {
            (*target, *source)
        } else {
            (*source, *target)
        };

        let expected = id;
        let id = self
            .edges
            .insert(Edge::new(expected, weight, source, target));

        if id != expected {
            // delete the edge we just inserted
            // we don't need to update the closures, since we haven't added the edge to them yet
            self.edges.remove(id);

            return Err(Report::new(Error::InconsistentEdgeId));
        }

        let edge = self
            .edges
            .get_mut(id)
            .ok_or_else(|| Report::new(Error::EdgeNotFound))?;
        // we do not need to set the node's id, since the assertion above guarantees that the id is
        // correct

        self.closures.create_edge(edge, &mut self.nodes);

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
        let node = self.nodes.remove(*id)?;

        for edge in node.edges() {
            if let Some(edge) = self.edges.remove(edge) {
                self.closures.remove_edge(&edge, &mut self.nodes);
            }
        }

        let (id, weight) = self.closures.remove_node(node, &mut self.nodes);

        Some(DetachedNode::new(id, weight))
    }

    fn remove_edge(
        &mut self,
        id: &Self::EdgeId,
    ) -> Option<DetachedEdge<Self::EdgeId, Self::NodeId, Self::EdgeWeight>> {
        let edge = self.edges.remove(*id)?;
        self.closures.remove_edge(&edge, &mut self.nodes);

        Some(DetachedEdge::new(
            edge.id,
            edge.weight,
            edge.source,
            edge.target,
        ))
    }

    fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
        self.closures.clear(&mut self.nodes);
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

    fn edges_between<'a: 'b, 'b>(
        &'a self,
        source: &'b Self::NodeId,
        target: &'b Self::NodeId,
    ) -> impl Iterator<Item = petgraph_core::edge::Edge<'a, Self>> + 'b {
        let edges = self
            .closures
            .edges()
            .undirected_endpoints_to_edges(*source, *target);

        edges.filter_map(move |edge| self.edge(&edge))
    }

    fn edges_between_mut<'a: 'b, 'b>(
        &'a mut self,
        source: &'b Self::NodeId,
        target: &'b Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'a, Self>> + 'b {
        let edges = self
            .closures
            .edges()
            .undirected_endpoints_to_edges(*source, *target);

        self.edges
            .filter_mut(edges)
            .map(move |edge| EdgeMut::new(&edge.id, &mut edge.weight, &edge.source, &edge.target))
    }

    fn node_connections<'a: 'b, 'b>(
        &'a self,
        id: &'b Self::NodeId,
    ) -> impl Iterator<Item = petgraph_core::edge::Edge<'a, Self>> + 'b {
        self.nodes
            .get(*id)
            .into_iter()
            .flat_map(move |node| node.edges())
            .filter_map(move |edge| self.edge(&edge))
    }

    fn node_connections_mut<'a: 'b, 'b>(
        &'a mut self,
        id: &'b Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'a, Self>> + 'b {
        let Self { nodes, edges, .. } = self;

        let allow = nodes
            .get(*id)
            .into_iter()
            .flat_map(move |node| node.edges());

        edges
            .filter_mut(allow)
            .map(move |edge| EdgeMut::new(&edge.id, &mut edge.weight, &edge.source, &edge.target))
    }

    fn node_neighbours<'a: 'b, 'b>(
        &'a self,
        id: &'b Self::NodeId,
    ) -> impl Iterator<Item = petgraph_core::node::Node<'a, Self>> + 'b {
        self.nodes
            .get(*id)
            .into_iter()
            .flat_map(move |node| node.neighbours())
            .filter_map(move |node| self.node(&node))
    }

    fn node_neighbours_mut<'a: 'b, 'b>(
        &'a mut self,
        id: &'b Self::NodeId,
    ) -> impl Iterator<Item = NodeMut<'a, Self>> + 'b {
        let Some(node) = self.nodes.get(*id) else {
            return Either::Right(core::iter::empty());
        };

        // SAFETY: we never access the closure argument mutably, only the weight.
        // Therefore it is safe for us to access both at the same time.
        let closure: &NodeClosures = unsafe { &*(&node.closures as *const _) };
        let neighbours = closure.neighbours();

        Either::Left(
            self.nodes
                .filter_mut(neighbours)
                .map(move |node| NodeMut::new(&node.id, &mut node.weight)),
        )
    }

    fn isolated_nodes(&self) -> impl Iterator<Item = petgraph_core::node::Node<Self>> {
        self.nodes
            .iter()
            .filter(|node| node.is_isolated())
            .map(move |node| petgraph_core::node::Node::new(self, &node.id, &node.weight))
    }

    fn isolated_nodes_mut(&mut self) -> impl Iterator<Item = NodeMut<Self>> {
        self.nodes
            .iter_mut()
            .filter(move |node| node.is_isolated())
            .map(move |node| NodeMut::new(&node.id, &mut node.weight))
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

    fn reserve_nodes(&mut self, additional: usize) {
        self.nodes.reserve(additional);
        self.closures.reserve(additional);
    }

    fn reserve_edges(&mut self, additional: usize) {
        self.edges.reserve(additional);
        self.closures.reserve(additional);
    }

    fn shrink_to_fit_nodes(&mut self) {
        self.nodes.shrink_to_fit();
        self.closures.shrink_to_fit();
    }

    fn shrink_to_fit_edges(&mut self) {
        self.edges.shrink_to_fit();
        self.closures.shrink_to_fit();
    }
}
