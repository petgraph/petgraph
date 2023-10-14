//! # Edges
//!
//! This module contains the three edge types used by the graph, an edge is a connection between two
//! nodes, that may or may not be directed and has a weight.
//!
//! The three edge types are:
//! * [`Edge`]: An immutable edge, that borrows the graph.
//! * [`EdgeMut`]: A mutable edge, that borrows the graph.
//! * [`DetachedEdge`]: An edge that is not attached to the graph.
//!
//! In application code [`DetachedEdge`]s can easily be interchanged with [`Edge`]s and vice-versa,
//! as long as all components are [`Clone`]able.
//!
//! You should prefer to use [`Edge`]s and [`EdgeMut`]s over [`DetachedEdge`]s, as they are more
//! efficient, as these are simply (mutable) reference into the underlying graph (storage).
//!
//! [`EdgeMut`]s are only needed when you want to mutate the weight of an edge, otherwise you should
//! use [`Edge`]s.

mod compat;
mod direction;
pub mod marker;

use core::fmt::{Debug, Formatter};

pub use self::{direction::Direction, marker::GraphDirectionality};
use crate::{node::Node, storage::GraphStorage, DirectedGraphStorage};

type DetachedStorageEdge<S> = DetachedEdge<
    <S as GraphStorage>::EdgeId,
    <S as GraphStorage>::NodeId,
    <S as GraphStorage>::EdgeWeight,
>;

/// Active edge in the graph.
///
/// Edge that is part of the graph, it borrows the graph and can be used to access the endpoints.
///
/// Undirected graph implementations have no notion of a source and target, therefore endpoints can
/// only be accessed through [`Self::endpoint_ids`] and [`Self::endpoints`].
/// If the storage implementation is directed (implements [`DirectedGraphStorage`]), one can access
/// the source and target through [`Self::source_id`], [`Self::source`], [`Self::target_id`] and
/// [`Self::target`].
///
/// # Example
///
/// ```
/// use petgraph_dino::DiDinoGraph;
///
/// let mut graph = DiDinoGraph::new();
///
/// let a = *graph.insert_node("A").id();
/// let aa = *graph.insert_edge("A → A", &a, &a).id();
///
/// let edge = graph.edge(&aa).unwrap();
///
/// assert_eq!(edge.id(), &aa);
/// assert_eq!(edge.weight(), &"A → A");
/// ```
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Edge<'a, S>
where
    S: GraphStorage,
{
    storage: &'a S,

    id: &'a S::EdgeId,

    u: &'a S::NodeId,
    v: &'a S::NodeId,

    weight: &'a S::EdgeWeight,
}

impl<S> Clone for Edge<'_, S>
where
    S: GraphStorage,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> Copy for Edge<'_, S> where S: GraphStorage {}

impl<S> Debug for Edge<'_, S>
where
    S: GraphStorage,
    S::EdgeId: Debug,
    S::NodeId: Debug,
    S::EdgeWeight: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Edge")
            .field("id", &self.id)
            .field("u", &self.u)
            .field("v", &self.v)
            .field("weight", &self.weight)
            .finish()
    }
}

impl<'a, S> Edge<'a, S>
where
    S: GraphStorage,
{
    /// Create a new edge.
    ///
    /// You should not need to use this directly, instead use [`Graph::edge`].
    ///
    /// This is only for implementors of [`GraphStorage`].
    ///
    /// In an undirected graph `u` and `v` are interchangeable, but in a directed graph (implements
    /// [`DirectedGraphStorage`]) `u` is the source and `v` is the target.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{
    ///     edge::{Direction, Edge},
    ///     node::Node,
    /// };
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// let aa = *graph.insert_edge("A → A", &a, &a).id();
    ///
    /// let edge = graph.edge(&aa).unwrap();
    /// let copy = Edge::new(
    ///     graph.storage(),
    ///     edge.id(),
    ///     edge.weight(),
    ///     edge.source_id(),
    ///     edge.target_id(),
    /// );
    /// // ^ exact same as `let copy = *edge;`
    /// ```
    ///
    /// # Contract
    ///
    /// The `id`, `source_id` and `target_id` must be valid in the given `storage`, and
    /// [`storage.node(source_id)`](GraphStorage::node) and
    /// [`storage.node(target_id)`](GraphStorage::node) must return `Some(_)`.
    /// The `weight` must be valid in the given `storage` for the specified `id`.
    ///
    /// The contract on `id` is not enforced, to avoid recursive calls to [`GraphStorage::edge`] and
    /// [`GraphStorage::contains_edge`].
    /// The contract on `source_id` and `target_id` is checked in debug builds.
    ///
    /// [`Graph::edge`]: crate::graph::Graph::edge
    pub fn new(
        storage: &'a S,

        id: &'a S::EdgeId,
        weight: &'a S::EdgeWeight,

        u: &'a S::NodeId,
        v: &'a S::NodeId,
    ) -> Self {
        debug_assert!(storage.contains_node(u));
        debug_assert!(storage.contains_node(v));

        Self {
            storage,

            id,

            u,
            v,

            weight,
        }
    }

    /// The unique id of the edge.
    ///
    /// This is guaranteed to be unique within the graph, the type returned depends on the
    /// implementation of [`GraphStorage`].
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{
    ///     edge::{Direction, Edge},
    ///     node::Node,
    /// };
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// let aa = *graph.insert_edge("A → A", &a, &a).id();
    ///
    /// let edge = graph.edge(&aa).unwrap();
    /// assert_eq!(edge.id(), &aa);
    /// ```
    #[must_use]
    pub const fn id(&self) -> &'a S::EdgeId {
        self.id
    }

    /// Get the ids of the endpoints of this edge.
    ///
    /// While [`Self::source_id`] and [`Self::target_id`] are only available for directed graphs,
    /// this method is available for both directed and undirected graphs.
    ///
    /// The order of the ids is not guaranteed for undirected graphs, but corresponds to `(source,
    /// target)` for directed graph, this should be considered an implementation detail and one
    /// **should not** rely on the order.
    ///
    /// You should use [`Self::source_id`] and [`Self::target_id`] directly if you need the source
    /// and target.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    ///
    /// let ab = *graph.insert_edge("A → B", &a, &b).id();
    ///
    /// let edge = graph.edge(&ab).unwrap();
    ///
    /// let (u, v) = edge.endpoint_ids(); // <- the order is not guaranteed for undirected graphs
    ///
    /// assert!((u, v) == (&a, &b) || (u, v) == (&b, &a));
    /// ```
    #[must_use]
    pub const fn endpoint_ids(&self) -> (&'a S::NodeId, &'a S::NodeId) {
        (self.u, self.v)
    }

    /// Get the endpoints of this edge.
    ///
    /// While [`Self::source`] and [`Self::target`] are only available for directed graphs, this
    /// method is available for both directed and undirected graphs
    ///
    /// The order of the endpoints is not guaranteed for undirected graphs, but corresponds to
    /// `(source, target)` for directed graph, this should be considered an implementation
    /// detail and one **should not** rely on the order.
    ///
    /// You should use [`Self::source`] and [`Self::target`] directly if you need the source and
    /// target.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    ///
    /// let ab = *graph.insert_edge("A → B", &a, &b).id();
    ///
    /// let edge = graph.edge(&ab).unwrap();
    ///
    /// let (u, v) = edge.endpoints(); // <- the order is not guaranteed for undirected graphs
    ///
    /// assert!((u.id(), v.id()) == (&a, &b) || (u.id(), v.id()) == (&b, &a));
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the endpoints are not active in the same storage as this edge.
    ///
    /// This error will only occur if the storage has been corrupted or if the contract on
    /// [`Edge::new`] is violated.
    #[must_use]
    pub fn endpoints(&self) -> (Node<'a, S>, Node<'a, S>) {
        (
            self.storage.node(self.u).expect(
                "corrupted storage or violated contract upon creation of this edge; the endpoint \
                 node must be active in the same storage as this edge",
            ),
            self.storage.node(self.v).expect(
                "corrupted storage or violated contract upon creation of this edge; the endpoint \
                 node must be active in the same storage as this edge",
            ),
        )
    }

    /// Get the weight of this edge.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{
    ///     edge::{Direction, Edge},
    ///     node::Node,
    /// };
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    /// let ab = *graph.insert_edge("A → B", &a, &b).id();
    ///
    /// let edge = graph.edge(&ab).unwrap();
    /// assert_eq!(edge.weight(), &"A → B");
    /// ```
    #[must_use]
    pub const fn weight(&self) -> &'a S::EdgeWeight {
        self.weight
    }
}

impl<'a, S> Edge<'a, S>
where
    S: DirectedGraphStorage,
{
    /// Get the source node id of this edge.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{
    ///     edge::{Direction, Edge},
    ///     node::Node,
    /// };
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    /// let ab = *graph.insert_edge("A → B", &a, &b).id();
    ///
    /// let edge = graph.edge(&ab).unwrap();
    /// assert_eq!(edge.source_id(), &a);
    /// ```
    #[must_use]
    pub const fn source_id(&self) -> &'a S::NodeId {
        self.u
    }

    /// Get the source node of this edge.
    ///
    /// This is a shortcut for [`self.storage.node(self.source_id())`](GraphStorage::node).
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{
    ///     edge::{Direction, Edge},
    ///     node::Node,
    /// };
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    /// let ab = *graph.insert_edge("A → B", &a, &b).id();
    ///
    /// let edge = graph.edge(&ab).unwrap();
    ///
    /// let source = edge.source();
    /// assert_eq!(source.id(), &a);
    /// ```
    ///
    ///
    /// # Panics
    ///
    /// Panics if the source node is not active in the same storage as this edge.
    ///
    /// This error will only occur if the storage has been corrupted or if the contract on
    /// [`Edge::new`] is violated.
    #[must_use]
    pub fn source(&self) -> Node<'a, S> {
        self.storage.node(self.u).expect(
            "corrupted storage or violated contract upon creation of this edge; the source node \
             must be active in the same storage as this edge",
        )
    }

    #[must_use]
    /// Get the target node id of this edge.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{
    ///     edge::{Direction, Edge},
    ///     node::Node,
    /// };
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    /// let ab = *graph.insert_edge("A → B", &a, &b).id();
    ///
    /// let edge = graph.edge(&ab).unwrap();
    /// assert_eq!(edge.target_id(), &b);
    /// ```
    pub const fn target_id(&self) -> &'a S::NodeId {
        self.v
    }

    /// Get the target node of this edge.
    ///
    /// This is a shortcut for [`self.storage.node(self.target_id())`](GraphStorage::node).
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{
    ///     edge::{Direction, Edge},
    ///     node::Node,
    /// };
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    /// let ab = *graph.insert_edge("A → B", &a, &b).id();
    ///
    /// let edge = graph.edge(&ab).unwrap();
    ///
    /// let target = edge.target();
    /// assert_eq!(target.id(), &b);
    /// ```
    ///
    ///
    /// # Panics
    ///
    /// Panics if the target node is not active in the same storage as this edge.
    ///
    /// This error will only occur if the storage has been corrupted or if the contract on
    /// [`Edge::new`] is violated.
    #[must_use]
    pub fn target(&self) -> Node<'a, S> {
        self.storage.node(self.v).expect(
            "corrupted storage or violated contract upon creation of this edge; the target node \
             must be active in the same storage as this edge",
        )
    }
}

impl<S> Edge<'_, S>
where
    S: GraphStorage,
    S::NodeId: Clone,
    S::EdgeId: Clone,
    S::EdgeWeight: Clone,
{
    /// Detach this edge from the graph.
    ///
    /// > **Note:** This will _not_ remove the node from the graph, it will only detach the this
    /// > instance of the node removing the lifetime dependency on the graph.
    ///
    /// This will return a [`DetachedEdge`], which can be reattached to the graph using
    /// [`Graph::from_parts`].
    ///
    /// This is especially useful in use-cases where you want direct (mutable access) to both the
    /// weight and id or do not want to bother with the graph's lifetime.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{
    ///     edge::{Direction, Edge},
    ///     node::Node,
    /// };
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    /// let ab = *graph.insert_edge("A → B", &a, &b).id();
    ///
    /// let edge = graph.edge(&ab).unwrap().detach();
    ///
    /// assert_eq!(edge.id, ab);
    /// assert_eq!(edge.weight, "A → B");
    /// ```
    ///
    /// [`Graph::from_parts`]: crate::graph::Graph::from_parts
    #[must_use]
    pub fn detach(self) -> DetachedStorageEdge<S> {
        DetachedEdge::new(
            self.id.clone(),
            self.weight.clone(),
            self.u.clone(),
            self.v.clone(),
        )
    }
}

/// Active (mutable) edge in the graph.
///
/// Edge that is part of the graph, it borrows the graph and can be used to mutably access the
/// weight of an edge. The source and node are not available mutably, as changing those might
/// violate some internal constraints of the [`GraphStorage`] they are part of.
///
/// To prevent multiple borrows of the same edge, while still allowing for multiple borrows into the
/// same storage, this type does not carry a reference to the storage itself.
///
/// # Example
///
/// ```
/// use petgraph_dino::DiDinoGraph;
///
/// let mut graph = DiDinoGraph::new();
///
/// let a = *graph.insert_node("A").id();
/// let b = *graph.insert_node("B").id();
/// let mut ab = graph.insert_edge("A → B", &a, &b);
///
/// assert_eq!(ab.weight(), &"A → B");
/// assert_eq!(ab.weight_mut(), &mut "A → B");
/// ```
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgeMut<'a, S>
where
    S: GraphStorage,
{
    id: &'a S::EdgeId,

    weight: &'a mut S::EdgeWeight,

    u: &'a S::NodeId,
    v: &'a S::NodeId,
}

impl<'a, S> EdgeMut<'a, S>
where
    S: GraphStorage,
{
    /// Create a new edge.
    ///
    /// You should not need to use this directly, instead use [`Graph::edge_mut`] or
    /// [`Graph::insert_edge`].
    ///
    /// This is only for implementors of [`GraphStorage`].
    ///
    /// In an undirected graph `u` and `v` are interchangeable, but in a directed graph (implements
    /// [`DirectedGraphStorage`]) `u` is the source and `v` is the target.
    ///
    /// # Contract
    ///
    /// The `id`, `source_id` and `target_id` must be valid in the given `storage`, and the `weight`
    /// must be valid in the given `storage` for the specified `id`.
    ///
    /// [`Graph::edge_mut`]: crate::graph::Graph::edge_mut
    /// [`Graph::insert_node`]: crate::graph::Graph::insert_node
    pub fn new(
        id: &'a S::EdgeId,
        weight: &'a mut S::EdgeWeight,

        u: &'a S::NodeId,
        v: &'a S::NodeId,
    ) -> Self {
        Self { id, weight, u, v }
    }

    /// The unique id of the edge.
    ///
    /// This is guaranteed to be unique within the graph, the type returned depends on the
    /// implementation of [`GraphStorage`].
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    /// let ab = *graph.insert_edge("A → B", &a, &b).id();
    ///
    /// let edge = graph.edge_mut(&ab).unwrap();
    ///
    /// assert_eq!(edge.id(), &ab);
    /// ```
    #[must_use]
    pub const fn id(&self) -> &'a S::EdgeId {
        self.id
    }

    /// The ids of the endpoints of the edge.
    ///
    /// While [`Self::source_id`] and [`Self::target_id`] are only available for directed graphs,
    /// this method is available for both directed and undirected graphs.
    ///
    /// The order of the ids is not guaranteed for undirected graphs, but corresponds to `(source,
    /// target)` for directed graph, this should be considered an implementation detail and one
    /// **should not** rely on the order.
    ///
    /// You should use [`Self::source_id`] and [`Self::target_id`] directly if you need the source
    /// and target.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    ///
    /// let ab = graph.insert_edge("A → B", &a, &b);
    ///
    /// let (u, v) = ab.endpoint_ids(); // <- the order is not guaranteed for undirected graphs
    ///
    /// assert!((u, v) == (&a, &b) || (u, v) == (&b, &a));
    /// ```
    #[must_use]
    pub const fn endpoint_ids(&self) -> (&'a S::NodeId, &'a S::NodeId) {
        (self.u, self.v)
    }

    /// The weight of the edge.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    /// let ab = *graph.insert_edge("A → B", &a, &b).id();
    ///
    /// let edge = graph.edge_mut(&ab).unwrap();
    ///
    /// assert_eq!(edge.weight(), &"A → B");
    /// ```
    #[must_use]
    pub fn weight(&self) -> &S::EdgeWeight {
        self.weight
    }

    /// The (mutable) weight of the edge.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    /// let ab = *graph.insert_edge("A → B", &a, &b).id();
    ///
    /// let mut edge = graph.edge_mut(&ab).unwrap();
    ///
    /// assert_eq!(edge.weight_mut(), &mut "A → B");
    /// ```
    pub fn weight_mut(&mut self) -> &mut S::EdgeWeight {
        self.weight
    }
}

impl<'a, S> EdgeMut<'a, S>
where
    S: DirectedGraphStorage,
{
    /// The source node id of the edge.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    /// let ab = *graph.insert_edge("A → B", &a, &b).id();
    ///
    /// let edge = graph.edge_mut(&ab).unwrap();
    ///
    /// assert_eq!(edge.source_id(), &a);
    /// ```
    #[must_use]
    pub const fn source_id(&self) -> &'a S::NodeId {
        self.u
    }

    /// The target node id of the edge.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    /// let ab = *graph.insert_edge("A → B", &a, &b).id();
    ///
    /// let edge = graph.edge_mut(&ab).unwrap();
    ///
    /// assert_eq!(edge.target_id(), &b);
    /// ```
    #[must_use]
    pub const fn target_id(&self) -> &'a S::NodeId {
        self.v
    }
}

impl<S> EdgeMut<'_, S>
where
    S: GraphStorage,
    S::NodeId: Clone,
    S::EdgeId: Clone,
    S::EdgeWeight: Clone,
{
    /// Detaches the edge from the graph.
    ///
    /// > **Note:** This will _not_ remove the node from the graph, it will only detach the this
    /// > instance of the node removing the lifetime dependency on the graph.
    ///
    ///
    /// This will return an [`DetachedEdge`], which can be reattached to the graph using
    /// [`Graph::from_parts`].
    ///
    /// This is especially useful in use-cases where you want direct (mutable access) to both the
    /// weight and id or do not want to bother with the graph's lifetime.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    /// let ab = *graph.insert_edge("A → B", &a, &b).id();
    ///
    /// let mut edge = graph.edge_mut(&ab).unwrap().detach();
    ///
    /// assert_eq!(edge.id, ab);
    /// assert_eq!(edge.weight, "A → B");
    /// ```
    #[must_use]
    pub fn detach(self) -> DetachedStorageEdge<S> {
        DetachedEdge::new(
            self.id.clone(),
            self.weight.clone(),
            self.u.clone(),
            self.v.clone(),
        )
    }
}

/// Detached edge from a graph.
///
/// This edge is no longer considered to be, but can be reattached to the graph using
/// [`Graph::from_parts`].
///
/// Especially useful in cases of decomposition ([`Graph::into_parts`]) or if one does not want to
/// deal with the graph's lifetime or needs a node to outlive the graph.
///
/// # Example
///
/// ```
/// use petgraph_dino::DiDinoGraph;
///
/// let mut graph = DiDinoGraph::new();
///
/// let a = *graph.insert_node("A").id();
/// let b = *graph.insert_node("B").id();
/// let ab = *graph.insert_edge("A → B", &a, &b).id();
///
/// let mut edge = graph.edge_mut(&ab).unwrap().detach();
///
/// assert_eq!(edge.id, ab);
/// assert_eq!(edge.weight, "A → B");
/// ```
///
/// [`Graph::into_parts`]: crate::graph::Graph::into_parts
/// [`Graph::from_parts`]: crate::graph::Graph::from_parts
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DetachedEdge<E, N, W> {
    pub id: E,

    pub u: N,
    pub v: N,

    pub weight: W,
}

impl<E, N, W> DetachedEdge<E, N, W> {
    /// Create a new detached edge.
    ///
    /// In an undirected graph `u` and `v` are interchangeable, but in a directed graph (implements
    /// [`DirectedGraphStorage`]) `u` is the source and `v` is the target.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::edge::DetachedEdge;
    ///
    /// let edge = DetachedEdge::new(0, "A → B", 1, 2);
    /// ```
    pub const fn new(id: E, weight: W, u: N, v: N) -> Self {
        Self { id, u, v, weight }
    }
}
