mod compat;
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

/// A graph, which is generic over its storage.
///
/// This is the central point of interaction with `petgraph`, allowing for consistent access to a
/// graph, without worrying about the underlying storage implementations.
///
/// Limitations of the underlying storage implementation apply, meaning that some operations may be
/// more expensive than others, or certain capabilities such as parallel edges may not be available.
///
/// Each graph storage implementation has a section with more information about its capabilities.
///
/// For convenience each graph storage implementation has a type alias for the graph type, which
/// uses the implementation, such as [`DinoGraph`], [`DiDinoGraph`], and [`UnDinoGraph`] in
/// `petgraph-dino`.
///
/// Instead of relying on a single storage implementation, this design encourages (and recommends)
/// the use of `S` as a generic parameter for functions receiving a graph.
///
/// `petgraph` assumes that a directed graph is simply a specialization of an undirected graph where
/// edges have an additional property marking the directionality of the edge. This means that any
/// directed graph also necessarily implements the undirected graph interface, known simply as
/// [`GraphStorage`]. This is similar to other graph libraries such as `networkx` and `igraph`.
///
/// Endpoints of an edge are known as `u` and `v` in `petgraph`, where `u` and `v` are
/// interchangeable in an undirected graph, and `u` is the source and `v` is the target in a
/// directed graph.
///
/// # Storage Implementations
// TODO
///
/// # Example
///
/// ```
/// use petgraph_core::{
///     edge::marker::{Directed, Undirected},
///     Graph, GraphStorage,
/// };
/// use petgraph_dino::{DiDinoGraph, DinoGraph, DinosaurStorage, UnDinoGraph};
///
/// let digraph = Graph::<DinosaurStorage<u8, u8, Directed>>::new();
/// // ^ same as:
/// let digraph = DinoGraph::<u8, u8, Directed>::new();
/// // ^ same as:
/// let digraph = DiDinoGraph::<u8, u8>::new();
///
/// let ungraph = Graph::<DinosaurStorage<u8, u8, Undirected>>::new();
/// // ^ same as:
/// let ungraph = DinoGraph::<u8, u8, Undirected>::new();
/// // ^ same as:
/// let ungraph = UnDinoGraph::<u8, u8>::new();
///
/// fn sum_node_weights<S>(graph: &Graph<S>) -> u8
/// where
///     S: GraphStorage<NodeWeight = u8>,
/// {
///     graph.nodes().map(|node| *node.weight()).sum()
/// }
///
/// assert_eq!(sum_node_weights(&digraph), 0);
/// assert_eq!(sum_node_weights(&ungraph), 0);
/// ```
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
    /// Create a new graph on-top of the given storage.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{edge::marker::Undirected, Graph, GraphStorage};
    /// use petgraph_dino::DinosaurStorage;
    ///
    /// let storage = DinosaurStorage::with_capacity(None, None);
    /// let graph = Graph::<_>::new_in(storage);
    /// // ^ this is the same as `Graph::new()`
    /// # assert_eq!(graph.num_nodes(), 0);
    /// # assert_eq!(graph.num_edges(), 0);
    /// ```
    #[must_use]
    pub const fn new_in(storage: S) -> Self {
        Self { storage }
    }

    /// Creates a new graph with the default capacity.
    ///
    /// The default capacity is `None` for both nodes and edges.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{edge::marker::Undirected, Graph};
    /// use petgraph_dino::DinosaurStorage;
    ///
    /// let graph = Graph::<DinosaurStorage<u8, u8, Undirected>>::new();
    /// # assert_eq!(graph.num_nodes(), 0);
    /// # assert_eq!(graph.num_edges(), 0);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::new_in(S::with_capacity(None, None))
    }

    /// Creates a new graph with the given capacity.
    ///
    /// Helpful in cases where the number of nodes and edges is known in advance, and one wants to
    /// avoid reallocations.
    /// Be aware that some storage implementations may not support this and may not be able to
    /// provide the requested capacity in advance.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{edge::marker::Undirected, Graph};
    /// use petgraph_dino::DinosaurStorage;
    ///
    /// let graph = Graph::<DinosaurStorage<u8, u8, Undirected>>::with_capacity(Some(16), Some(16));
    /// # assert_eq!(graph.num_nodes(), 0);
    /// # assert_eq!(graph.num_edges(), 0);
    /// ```
    #[must_use]
    pub fn with_capacity(node_capacity: Option<usize>, edge_capacity: Option<usize>) -> Self {
        Self::new_in(S::with_capacity(node_capacity, edge_capacity))
    }

    /// Returns a reference to the underlying storage.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{edge::marker::Undirected, Graph, GraphStorage};
    /// use petgraph_dino::DinosaurStorage;
    ///
    /// let storage = DinosaurStorage::with_capacity(None, None);
    /// let graph = Graph::<_>::new_in(storage);
    ///
    /// assert_eq!(graph.storage().num_nodes(), 0);
    /// assert_eq!(graph.storage().num_edges(), 0);
    /// ```
    pub const fn storage(&self) -> &S {
        &self.storage
    }

    /// Returns the underlying storage.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{edge::marker::Undirected, Graph, GraphStorage};
    /// use petgraph_dino::DinosaurStorage;
    ///
    /// let storage = DinosaurStorage::with_capacity(None, None);
    /// let graph = Graph::<_>::new_in(storage);
    ///
    /// assert_eq!(graph.into_storage().num_nodes(), 0);
    /// assert_eq!(graph.into_storage().num_edges(), 0);
    /// ```
    pub fn into_storage(self) -> S {
        self.storage
    }

    /// Create a new graph from the given nodes and edges.
    ///
    /// Takes two iterators, one for nodes and one for edges, and returns a new graph with the given
    /// nodes and edges, if the iterator is well-formed.
    /// This is the reverse operation to [`Self::into_parts`], which converts a graph into its
    /// parts.
    ///
    /// The resulting graph may change the order of the nodes and edges and reassign their IDs.
    /// The only properties that stay consistent are the weights and the structural equivalence of
    /// the graph.
    /// For more information see [`GraphStorage::from_parts`].
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashSet;
    ///
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut other = DiDinoGraph::<u8, u8>::new();
    /// let a = *other.insert_node(0).id();
    /// let b = *other.insert_node(1).id();
    /// let c = *other.insert_node(2).id();
    ///
    /// other.insert_edge(u8::MAX, &a, &b);
    /// other.insert_edge(u8::MAX - 1, &b, &c);
    ///
    /// let other = other.into_parts();
    ///
    /// let graph = DiDinoGraph::from_parts(other.0, other.1).unwrap();
    /// assert_eq!(graph.num_nodes(), 3);
    /// assert_eq!(graph.num_edges(), 2);
    ///
    /// assert_eq!(
    ///     graph
    ///         .nodes()
    ///         .map(|node| *node.weight())
    ///         .collect::<HashSet<_>>(),
    ///     [0, 1, 2].into_iter().collect::<HashSet<_>>(),
    /// );
    /// assert_eq!(
    ///     graph
    ///         .edges()
    ///         .map(|edge| *edge.weight())
    ///         .collect::<HashSet<_>>(),
    ///     [u8::MAX, u8::MAX - 1].into_iter().collect::<HashSet<_>>(),
    /// );
    /// ```
    pub fn from_parts(
        nodes: impl IntoIterator<Item = DetachedNode<S::NodeId, S::NodeWeight>>,
        edges: impl IntoIterator<Item = DetachedEdge<S::EdgeId, S::NodeId, S::EdgeWeight>>,
    ) -> Result<Self, S::Error> {
        Ok(Self {
            storage: S::from_parts(nodes, edges)?,
        })
    }

    /// Converts the graph into its parts.
    ///
    /// This is the reverse operation to [`Self::from_parts`], which creates a graph from its parts.
    ///
    /// The iterables returned by this function are not guaranteed to be in any particular order,
    /// but must contain all nodes and edges.
    ///
    /// For more information see [`GraphStorage::into_parts`].
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashSet;
    ///
    /// use petgraph_core::{DetachedEdge, DetachedNode};
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::<u8, u8>::new();
    /// let a = *graph.insert_node(0).id();
    /// let b = *graph.insert_node(1).id();
    /// let c = *graph.insert_node(2).id();
    ///
    /// let ab = *graph.insert_edge(u8::MAX, &a, &b).id();
    /// let bc = *graph.insert_edge(u8::MAX - 1, &b, &c).id();
    ///
    /// let (nodes, edges) = graph.into_parts();
    /// let (nodes, edges) = (
    ///     nodes
    ///         .into_iter()
    ///         .map(|node| node.weight)
    ///         .collect::<HashSet<_>>(),
    ///     edges
    ///         .into_iter()
    ///         .map(|edge| edge.weight)
    ///         .collect::<HashSet<_>>(),
    /// );
    ///
    /// assert_eq!(nodes.len(), 3);
    /// assert_eq!(edges.len(), 2);
    ///
    /// assert_eq!(nodes, [0, 1, 2].into_iter().collect::<HashSet<_>>());
    /// assert_eq!(
    ///     edges,
    ///     [u8::MAX, u8::MAX - 1].into_iter().collect::<HashSet<_>>()
    /// );
    /// ```
    pub fn into_parts(
        self,
    ) -> (
        impl IntoIterator<Item = DetachedNode<S::NodeId, S::NodeWeight>>,
        impl IntoIterator<Item = DetachedEdge<S::EdgeId, S::NodeId, S::EdgeWeight>>,
    ) {
        self.storage.into_parts()
    }

    /// Converts the graph into a graph with a different storage implementation.
    ///
    /// Internally this simply calls [`GraphStorage::into_parts`] on `S` and then calls
    /// [`Graph::from_parts`] on `T`.
    // TODO: example
    pub fn convert<T>(self) -> Result<Graph<T>, T::Error>
    where
        T: GraphStorage<
                NodeId = S::NodeId,
                NodeWeight = S::NodeWeight,
                EdgeId = S::EdgeId,
                EdgeWeight = S::EdgeWeight,
            >,
    {
        let (nodes, edges) = self.storage.into_parts();

        Graph::from_parts(nodes, edges)
    }

    /// Returns the number of nodes in the graph.
    ///
    /// This is generally faster than iterating over all nodes and counting them via
    /// `self.nodes().count()`.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::<u8, u8>::new();
    /// let a = *graph.insert_node(0).id();
    /// let b = *graph.insert_node(1).id();
    /// let c = *graph.insert_node(2).id();
    ///
    /// let ab = *graph.insert_edge(u8::MAX, &a, &b).id();
    /// let bc = *graph.insert_edge(u8::MAX - 1, &b, &c).id();
    ///
    /// assert_eq!(graph.num_nodes(), 3);
    /// ```
    pub fn num_nodes(&self) -> usize {
        self.storage.num_nodes()
    }

    /// Returns the number of edges in the graph.
    ///
    /// This is generally faster than iterating over all edges and counting them via
    /// `self.edges().count()`.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::<u8, u8>::new();
    /// let a = *graph.insert_node(0).id();
    /// let b = *graph.insert_node(1).id();
    /// let c = *graph.insert_node(2).id();
    ///
    /// let ab = *graph.insert_edge(u8::MAX, &a, &b).id();
    /// let bc = *graph.insert_edge(u8::MAX - 1, &b, &c).id();
    ///
    /// assert_eq!(graph.num_edges(), 2);
    /// ```
    pub fn num_edges(&self) -> usize {
        self.storage.num_edges()
    }

    /// Returns `true` if the graph has no nodes or edges.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::<u8, u8>::new();
    /// assert!(graph.is_empty());
    ///
    /// let a = *graph.insert_node(0).id();
    /// assert!(!graph.is_empty());
    ///
    /// graph.remove_node(&a);
    /// assert!(graph.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.num_nodes() == 0 && self.num_edges() == 0
    }

    /// Clears the graph, removing all nodes and edges.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::<u8, u8>::new();
    /// let a = *graph.insert_node(0).id();
    /// let b = *graph.insert_node(1).id();
    /// let c = *graph.insert_node(2).id();
    ///
    /// let ab = *graph.insert_edge(u8::MAX, &a, &b).id();
    /// let bc = *graph.insert_edge(u8::MAX - 1, &b, &c).id();
    ///
    /// assert!(!graph.is_empty());
    ///
    /// graph.clear();
    ///
    /// assert!(graph.is_empty());
    /// ```
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
        u: &'b S::NodeId,
        v: &'b S::NodeId,
    ) -> impl Iterator<Item = Edge<'a, S>> + 'b {
        self.storage.edges_between(u, v)
    }

    pub fn edges_between_mut<'a: 'b, 'b>(
        &'a mut self,
        u: &'b S::NodeId,
        v: &'b S::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'a, S>> + 'b {
        self.storage.edges_between_mut(u, v)
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
