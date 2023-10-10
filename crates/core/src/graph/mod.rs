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
    /// use petgraph_core::{
    ///     edge::marker::{Directed, Undirected},
    ///     Graph, GraphStorage,
    /// };
    /// use petgraph_dino::DinosaurStorage;
    ///
    /// let storage = DinosaurStorage::<u8, u8, Directed>::with_capacity(None, None);
    /// let graph = Graph::new_in(storage);
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
    /// use petgraph_core::{
    ///     edge::marker::{Directed, Undirected},
    ///     Graph, GraphStorage,
    /// };
    /// use petgraph_dino::DinosaurStorage;
    ///
    /// let storage = DinosaurStorage::<u8, u8, Directed>::with_capacity(None, None);
    /// let graph = Graph::new_in(storage);
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
    /// use petgraph_core::{
    ///     edge::marker::{Directed, Undirected},
    ///     Graph, GraphStorage,
    /// };
    /// use petgraph_dino::DinosaurStorage;
    ///
    /// let storage = DinosaurStorage::<u8, u8, Directed>::with_capacity(None, None);
    /// let graph = Graph::new_in(storage);
    ///
    /// let storage = graph.into_storage();
    /// assert_eq!(storage.num_nodes(), 0);
    /// assert_eq!(storage.num_edges(), 0);
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
    /// let mut other = DiDinoGraph::new();
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
    ///
    /// let a = *graph.nodes().find(|node| *node.weight() == 0).unwrap().id();
    /// let b = *graph.nodes().find(|node| *node.weight() == 1).unwrap().id();
    /// let c = *graph.nodes().find(|node| *node.weight() == 2).unwrap().id();
    ///
    /// assert_eq!(
    ///     graph
    ///         .edges()
    ///         .map(|edge| (*edge.weight(), edge.endpoint_ids()))
    ///         .collect::<HashSet<_>>(),
    ///     [(u8::MAX, (&a, &b)), (u8::MAX - 1, (&b, &c))]
    ///         .into_iter()
    ///         .collect::<HashSet<_>>(),
    /// );
    /// ```
    ///
    /// # Errors
    ///
    /// If any of the nodes or edges are invalid, or any of the invariant checks of the underlying
    /// implementation fail, an error is returned.
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
    /// let mut graph = DiDinoGraph::new();
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
    ///
    /// # Errors
    ///
    /// If any of the nodes or edges are invalid, or any of the invariant checks of the new storage
    /// implementation fail, an error is returned.
    ///
    /// An example would be switching between a storage implementation that supports parallel edges
    /// to one that does not.
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
    /// let mut graph = DiDinoGraph::new();
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
    /// let mut graph = DiDinoGraph::new();
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
    /// let mut graph = DiDinoGraph::<_, u8>::new();
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
    /// let mut graph = DiDinoGraph::new();
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

    /// Returns the node with the given identifier, if it exists.
    ///
    /// The returned [`Node`] has a reference to the current graph, meaning that you're able to
    /// query for neighbours and [`Edge`]s on the returned value directly.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// # use petgraph_core::{GraphStorage, Node};
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::<_, u8>::new();
    /// let a = *graph.insert_node(0).id();
    /// # let b = graph.storage().next_node_id(NoValue::new());
    ///
    /// assert_eq!(
    ///     graph.node(&a).map(|node| (*node.id(), *node.weight())),
    ///     Some((a, 0))
    /// );
    /// assert_eq!(
    ///     graph.node(&b).map(|node| (*node.id(), *node.weight())),
    ///     None
    /// );
    /// ```
    pub fn node(&self, id: &S::NodeId) -> Option<Node<S>> {
        self.storage.node(id)
    }

    /// Returns the node, with a mutable weight, with the given identifier, if it exists.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// # use petgraph_core::{GraphStorage, Node};
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::<_, u8>::new();
    /// let a = *graph.insert_node(0).id();
    /// # let b = graph.storage().next_node_id(NoValue::new());
    ///
    /// if let Some(mut node) = graph.node_mut(&a) {
    ///     *node.weight_mut() = 1;
    /// }
    ///
    /// assert_eq!(
    ///     graph.node(&a).map(|node| (*node.id(), *node.weight())),
    ///     Some((a, 1))
    /// );
    ///
    /// assert!(graph.node_mut(&b).is_none());
    /// ```
    pub fn node_mut(&mut self, id: &S::NodeId) -> Option<NodeMut<S>> {
        self.storage.node_mut(id)
    }

    /// Returns `true` if the graph contains a node with the given identifier.
    ///
    /// This is generally faster than calling `self.node(id).is_some()`.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// # use petgraph_core::{GraphStorage, Node};
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::<_, u8>::new();
    /// let a = *graph.insert_node(0).id();
    /// # let b = graph.storage().next_node_id(NoValue::new());
    ///
    /// assert!(graph.contains_node(&a));
    /// assert!(!graph.contains_node(&b));
    /// ```
    pub fn contains_node(&self, id: &S::NodeId) -> bool {
        self.storage.contains_node(id)
    }

    /// Removes the node with the given identifier, if it exists.
    ///
    /// This will return the detached representation of the node, which can be used to reinsert the
    /// node into a new graph _or_ used to access any of the properties independently of the graph.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// # use petgraph_core::{DetachedNode, GraphStorage, Node};
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::<_, u8>::new();
    /// let a = *graph.insert_node(0).id();
    /// # let b = graph.storage().next_node_id(NoValue::new());
    ///
    /// assert_eq!(
    ///     graph.remove_node(&a),
    ///     Some(DetachedNode { id: a, weight: 0 })
    /// );
    /// assert_eq!(graph.remove_node(&a), None);
    /// assert_eq!(graph.remove_node(&b), None);
    /// ```
    pub fn remove_node(
        &mut self,
        id: &S::NodeId,
    ) -> Option<DetachedNode<S::NodeId, S::NodeWeight>> {
        self.storage.remove_node(id)
    }

    /// Returns the edge with the given identifier, if it exists.
    ///
    /// The returned [`Edge`] has a reference to the current graph, meaning that you're able to
    /// query the [`Node`] endpoints directly on the edge.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// # use petgraph_core::{DetachedNode, GraphStorage, Node};
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    /// let a = *graph.insert_node(0).id();
    /// let b = *graph.insert_node(1).id();
    ///
    /// let ab = *graph.insert_edge(u8::MAX, &a, &b).id();
    /// # let bc = graph.storage().next_edge_id(NoValue::new());
    ///
    /// assert_eq!(
    ///     graph
    ///         .edge(&ab)
    ///         .map(|edge| (*edge.id(), *edge.weight(), edge.endpoint_ids())),
    ///     Some((ab, u8::MAX, (&a, &b)))
    /// );
    /// assert!(graph.edge(&bc).is_none());
    /// ```
    pub fn edge(&self, id: &S::EdgeId) -> Option<Edge<S>> {
        self.storage.edge(id)
    }

    /// Returns the edge, with a mutable weight, with the given identifier, if it exists.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// # use petgraph_core::{DetachedNode, GraphStorage, Node};
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    /// let a = *graph.insert_node(0).id();
    /// let b = *graph.insert_node(1).id();
    ///
    /// let ab = *graph.insert_edge(u8::MAX, &a, &b).id();
    /// # let bc = graph.storage().next_edge_id(NoValue::new());
    ///
    /// if let Some(mut edge) = graph.edge_mut(&ab) {
    ///     *edge.weight_mut() = u8::MAX - 1;
    /// }
    ///
    /// assert_eq!(
    ///     graph
    ///         .edge(&ab)
    ///         .map(|edge| (*edge.id(), *edge.weight(), edge.endpoint_ids())),
    ///     Some((ab, u8::MAX - 1, (&a, &b)))
    /// );
    /// assert!(graph.edge_mut(&bc).is_none());
    /// ```
    pub fn edge_mut(&mut self, id: &S::EdgeId) -> Option<EdgeMut<S>> {
        self.storage.edge_mut(id)
    }

    /// Returns `true` if the graph contains an edge with the given identifier.
    ///
    /// This is generally faster than calling `self.edge(id).is_some()`.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// # use petgraph_core::{DetachedNode, GraphStorage, Node};
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    /// let a = *graph.insert_node(0).id();
    /// let b = *graph.insert_node(1).id();
    ///
    /// let ab = *graph.insert_edge(u8::MAX, &a, &b).id();
    /// # let bc = graph.storage().next_edge_id(NoValue::new());
    ///
    /// assert!(graph.contains_edge(&ab));
    /// assert!(!graph.contains_edge(&bc));
    /// ```
    pub fn contains_edge(&self, id: &S::EdgeId) -> bool {
        self.storage.contains_edge(id)
    }

    /// Removes the edge with the given identifier, if it exists.
    ///
    /// This will return the detached representation of the edge, which can be used to reinsert the
    /// edge into a new graph _or_ used to access any of the properties independently of the graph.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// # use petgraph_core::{DetachedEdge, DetachedNode, GraphStorage, Node};
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    /// let a = *graph.insert_node(0).id();
    /// let b = *graph.insert_node(1).id();
    ///
    /// let ab = *graph.insert_edge(u8::MAX, &a, &b).id();
    /// # let bc = graph.storage().next_edge_id(NoValue::new());
    ///
    /// assert_eq!(
    ///     graph.remove_edge(&ab),
    ///     Some(DetachedEdge {
    ///         id: ab,
    ///         weight: u8::MAX,
    ///         u: a,
    ///         v: b,
    ///     })
    /// );
    /// assert_eq!(graph.remove_edge(&ab), None);
    /// assert_eq!(graph.remove_edge(&bc), None);
    /// ```
    pub fn remove_edge(
        &mut self,
        id: &S::EdgeId,
    ) -> Option<DetachedEdge<S::EdgeId, S::NodeId, S::EdgeWeight>> {
        self.storage.remove_edge(id)
    }

    /// Returns the neighbours of the node with the given identifier.
    ///
    /// This is an alias for [`Self::neighbours`], as there's a spelling difference between the
    /// American and British English.
    #[inline]
    pub fn neighbors<'a: 'b, 'b>(
        &'a self,
        id: &'b S::NodeId,
    ) -> impl Iterator<Item = Node<'a, S>> + 'b {
        self.neighbours(id)
    }

    /// Returns the neighbours of the node with the given identifier.
    ///
    /// Returns an iterator over all nodes that are connected to the given node.
    /// In the case of a directed graph all edges (both incoming and outgoing) are taken into
    /// account.
    ///
    /// If the graph allows self-loops, and a self-loop exists, then the node will be returned
    /// as its own neighbour.
    ///
    /// The results won't be in any particular order.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    /// let a = *graph.insert_node(0).id();
    /// let b = *graph.insert_node(1).id();
    /// let c = *graph.insert_node(2).id();
    ///
    /// let ab = *graph.insert_edge(u8::MAX, &a, &b).id();
    /// let bc = *graph.insert_edge(u8::MAX - 1, &b, &c).id();
    /// let ca = *graph.insert_edge(u8::MAX - 2, &c, &a).id();
    /// let aa = *graph.insert_edge(u8::MAX - 3, &a, &a).id();
    ///
    /// assert_eq!(
    ///     graph
    ///         .neighbours(&a)
    ///         .map(|node| *node.id())
    ///         .collect::<HashSet<_>>(),
    ///     [a, b, c].into_iter().collect::<HashSet<_>>()
    /// );
    /// assert_eq!(
    ///     graph
    ///         .neighbours(&b)
    ///         .map(|node| *node.id())
    ///         .collect::<HashSet<_>>(),
    ///     [a, c].into_iter().collect::<HashSet<_>>()
    /// );
    /// ```
    pub fn neighbours<'a: 'b, 'b>(
        &'a self,
        id: &'b S::NodeId,
    ) -> impl Iterator<Item = Node<S>> + 'b {
        self.storage.node_neighbours(id)
    }

    /// Returns the neighbours of the node with the given identifier, with mutable weights.
    ///
    /// This is an alias for [`Self::neighbours_mut`], as there's a spelling difference between
    /// American and British English.
    #[inline]
    pub fn neighbors_mut<'a: 'b, 'b>(
        &'a mut self,
        id: &'b S::NodeId,
    ) -> impl Iterator<Item = NodeMut<'a, S>> + 'b {
        self.neighbours_mut(id)
    }

    /// Returns the neighbours of the node with the given identifier, with mutable weights.
    ///
    /// Returns an iterator over all nodes that are connected to the given node.
    /// In the case of a directed graph all edges (both incoming and outgoing) are taken into
    /// account.
    ///
    /// If the graph allows self-loops, and a self-loop exists, then the node will be returned as
    /// well.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    /// let a = *graph.insert_node(0).id();
    /// let b = *graph.insert_node(1).id();
    /// let c = *graph.insert_node(2).id();
    /// let d = *graph.insert_node(3).id();
    ///
    /// let ab = *graph.insert_edge(u8::MAX, &a, &b).id();
    /// let bc = *graph.insert_edge(u8::MAX - 1, &b, &c).id();
    /// let ca = *graph.insert_edge(u8::MAX - 2, &c, &a).id();
    /// let aa = *graph.insert_edge(u8::MAX - 3, &a, &a).id();
    ///
    /// for mut node in graph.neighbours_mut(&a) {
    ///     *node.weight_mut() += 16;
    /// }
    ///
    /// assert_eq!(
    ///     graph
    ///         .neighbours(&a)
    ///         .map(|node| (*node.id(), *node.weight()))
    ///         .collect::<HashSet<_>>(),
    ///     [(a, 16), (b, 17), (c, 18)]
    ///         .into_iter()
    ///         .collect::<HashSet<_>>()
    /// );
    /// assert_eq!(
    ///     graph.node(&d).map(|node| (*node.id(), *node.weight())),
    ///     Some((d, 3))
    /// );
    /// ```
    pub fn neighbours_mut<'a: 'b, 'b>(
        &'a mut self,
        id: &'b S::NodeId,
    ) -> impl Iterator<Item = NodeMut<'a, S>> + 'b {
        self.storage.node_neighbours_mut(id)
    }

    /// Returns the edges that are connected to the node with the given identifier.
    ///
    /// Returns an iterator over all edges that are connected to the given node.
    /// In the case of a directed graph this will return an iterator over all incoming and outgoing
    /// edges.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    /// let a = *graph.insert_node(0).id();
    /// let b = *graph.insert_node(1).id();
    /// let c = *graph.insert_node(2).id();
    /// let d = *graph.insert_node(3).id();
    ///
    /// let ab = *graph.insert_edge(u8::MAX, &a, &b).id();
    /// let bc = *graph.insert_edge(u8::MAX - 1, &b, &c).id();
    /// let ca = *graph.insert_edge(u8::MAX - 2, &c, &a).id();
    /// let aa = *graph.insert_edge(u8::MAX - 3, &a, &a).id();
    ///
    /// assert_eq!(
    ///     graph
    ///         .connections(&a)
    ///         .map(|edge| *edge.id())
    ///         .collect::<HashSet<_>>(),
    ///     [ab, ca, aa].into_iter().collect::<HashSet<_>>()
    /// );
    /// ```
    pub fn connections<'a: 'b, 'b>(
        &'a self,
        id: &'b S::NodeId,
    ) -> impl Iterator<Item = Edge<'a, S>> + 'b {
        self.storage.node_connections(id)
    }

    /// Returns the edges that are connected to the node with the given identifier, with mutable
    /// weights.
    ///
    /// Returns an iterator over all edges that are connected to the given node.
    /// In the case of a directed graph this will return an iterator over all incoming and outgoing
    /// edges.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    /// let a = *graph.insert_node(0).id();
    /// let b = *graph.insert_node(1).id();
    /// let c = *graph.insert_node(2).id();
    /// let d = *graph.insert_node(3).id();
    ///
    /// let ab = *graph.insert_edge(u8::MAX, &a, &b).id();
    /// let bc = *graph.insert_edge(u8::MAX - 1, &b, &c).id();
    /// let ca = *graph.insert_edge(u8::MAX - 2, &c, &a).id();
    /// let aa = *graph.insert_edge(u8::MAX - 3, &a, &a).id();
    ///
    /// for mut edge in graph.connections_mut(&a) {
    ///     *edge.weight_mut() -= 16;
    /// }
    ///
    /// assert_eq!(
    ///     graph
    ///         .connections(&a)
    ///         .map(|edge| (*edge.id(), *edge.weight()))
    ///         .collect::<HashSet<_>>(),
    ///     [(ab, u8::MAX - 16), (ca, u8::MAX - 18), (aa, u8::MAX - 19)]
    ///         .into_iter()
    ///         .collect::<HashSet<_>>()
    /// );
    /// assert_eq!(
    ///     graph.edge(&bc).map(|edge| (*edge.id(), *edge.weight())),
    ///     Some((bc, u8::MAX - 1))
    /// );
    /// ```
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
