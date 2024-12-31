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

use core::fmt::{Debug, Display, Formatter};

pub use self::{direction::Direction, marker::GraphDirectionality};
use crate::{
    DirectedGraphStorage,
    node::{Node, NodeId},
    storage::GraphStorage,
};

type DetachedStorageEdge<S> = DetachedEdge<<S as GraphStorage>::EdgeWeight>;

/// ID of an edge in a graph.
///
/// This is guaranteed to be unique within the graph, library authors and library consumers **must**
/// treat this as an opaque type akin to [`TypeId`].
///
/// The layout of the type is semver stable, but not part of the public API.
///
/// [`GraphStorage`] implementations may uphold additional invariants on the inner value and
/// code outside of the [`GraphStorage`] should **never** construct a [`EdgeId`] directly.
///
/// Accessing a [`GraphStorage`] implementation with a [`EdgeId`] not returned by an instance itself
/// is considered undefined behavior.
///
/// [`TypeId`]: core::any::TypeId
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgeId(usize);

impl Display for EdgeId {
    // we could also utilize a VTable here instead, that would allow for custom formatting
    // but that would be an additional pointer added to the type that must be carried around
    // that's about ~8 bytes on 64-bit systems
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "EdgeId({})", self.0)
    }
}

// TODO: find a better way to gate these functions
impl EdgeId {
    /// Creates a new [`EdgeId`].
    ///
    /// # Note
    ///
    /// Using this outside of the [`GraphStorage`] implementation is considered undefined behavior.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::edge::EdgeId;
    ///
    /// let id = EdgeId::new(0);
    /// ```
    // Hidden so that non-GraphStorage implementors are not tempted to use this.
    #[doc(hidden)]
    #[must_use]
    pub const fn new(id: usize) -> Self {
        Self(id)
    }

    /// Returns the inner value of the [`EdgeId`].
    ///
    /// # Note
    ///
    /// Using this outside of the [`GraphStorage`] implementation is considered undefined behavior.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::edge::EdgeId;
    ///
    /// let id = EdgeId::new(0);
    ///
    /// assert_eq!(id.into_inner(), 0);
    /// ```
    // Hidden so that non-GraphStorage implementors are not tempted to use this.
    #[doc(hidden)]
    #[must_use]
    pub const fn into_inner(self) -> usize {
        self.0
    }
}

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
pub struct Edge<'a, S: ?Sized>
where
    S: GraphStorage,
{
    storage: &'a S,

    id: EdgeId,

    u: NodeId,
    v: NodeId,

    weight: &'a S::EdgeWeight,
}

impl<S: ?Sized> Clone for Edge<'_, S>
where
    S: GraphStorage,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: ?Sized> Copy for Edge<'_, S> where S: GraphStorage {}

impl<S: ?Sized> Debug for Edge<'_, S>
where
    S: GraphStorage,
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

impl<'a, S: ?Sized> Edge<'a, S>
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

        id: EdgeId,
        weight: &'a S::EdgeWeight,

        u: NodeId,
        v: NodeId,
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
    pub const fn id(&self) -> EdgeId {
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
    pub const fn endpoint_ids(&self) -> (NodeId, NodeId) {
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

    /// Change the underlying storage of this edge.
    ///
    /// Should only be used when layering multiple [`GraphStorage`] implementations on top of each
    /// other.
    ///
    /// # Note
    ///
    /// This should not lead to any undefined behaviour, but might have unintended consequences if
    /// the storage does not recognize the inner id as valid.
    /// You should only use this if you know what you are doing.
    #[must_use]
    pub const fn change_storage_unchecked<T>(self, storage: &'a T) -> Edge<'a, T>
    where
        T: GraphStorage<EdgeWeight = S::EdgeWeight>,
    {
        Edge {
            storage,

            id: self.id,

            u: self.u,
            v: self.v,

            weight: self.weight,
        }
    }
}

impl<'a, S: ?Sized> Edge<'a, S>
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
    pub const fn source_id(&self) -> NodeId {
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
    pub const fn target_id(&self) -> NodeId {
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

impl<S: ?Sized> Edge<'_, S>
where
    S: GraphStorage,
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
        DetachedEdge::new(self.id, self.weight.clone(), self.u, self.v)
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
pub struct EdgeMut<'a, S: ?Sized>
where
    S: GraphStorage,
{
    id: EdgeId,

    weight: &'a mut S::EdgeWeight,

    u: NodeId,
    v: NodeId,
}

impl<'a, S: ?Sized> EdgeMut<'a, S>
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
    /// [`Graph::insert_edge`]: crate::graph::Graph::insert_edge
    pub fn new(id: EdgeId, weight: &'a mut S::EdgeWeight, u: NodeId, v: NodeId) -> Self {
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
    pub const fn id(&self) -> EdgeId {
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
    pub const fn endpoint_ids(&self) -> (NodeId, NodeId) {
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

    /// Change the underlying storage of this edge.
    ///
    /// Should only be used when layering multiple [`GraphStorage`] implementations on top of each
    /// other.
    ///
    /// # Note
    ///
    /// This should not lead to any undefined behaviour, but might have unintended consequences if
    /// the storage does not recognize the inner id as valid.
    /// You should only use this if you know what you are doing.
    #[must_use]
    pub fn change_storage_unchecked<T>(self) -> EdgeMut<'a, T>
    where
        T: GraphStorage<EdgeWeight = S::EdgeWeight>,
    {
        EdgeMut {
            id: self.id,

            weight: self.weight,

            u: self.u,
            v: self.v,
        }
    }
}

impl<'a, S: ?Sized> EdgeMut<'a, S>
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
    pub const fn source_id(&self) -> NodeId {
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
    pub const fn target_id(&self) -> NodeId {
        self.v
    }
}

impl<S: ?Sized> EdgeMut<'_, S>
where
    S: GraphStorage,
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
    ///
    /// [`Graph::from_parts`]: crate::graph::Graph::from_parts
    #[must_use]
    pub fn detach(self) -> DetachedStorageEdge<S> {
        DetachedEdge::new(self.id, self.weight.clone(), self.u, self.v)
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
pub struct DetachedEdge<W> {
    /// The unique id of the edge.
    pub id: EdgeId,

    /// The `u` endpoint of the `(u, v)` pair of endpoints.
    pub u: NodeId,
    /// The `v` endpoint of the `(u, v)` pair of endpoints.
    pub v: NodeId,

    /// The weight of the edge.
    pub weight: W,
}

impl<W> DetachedEdge<W> {
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
    pub const fn new(id: EdgeId, weight: W, u: NodeId, v: NodeId) -> Self {
        Self { id, u, v, weight }
    }
}
