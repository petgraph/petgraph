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

pub use direction::Direction;

use crate::{node::Node, storage::GraphStorage};

type DetachedStorageEdge<S> = DetachedEdge<
    <S as GraphStorage>::EdgeId,
    <S as GraphStorage>::NodeId,
    <S as GraphStorage>::EdgeWeight,
>;

/// Active edge in the graph.
///
/// Edge that is part of the graph, it borrows the graph and can be used to access the source and
/// target nodes.
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

    source_id: &'a S::NodeId,
    target_id: &'a S::NodeId,

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
            .field("source_id", &self.source_id)
            .field("target_id", &self.target_id)
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

        source_id: &'a S::NodeId,
        target_id: &'a S::NodeId,
    ) -> Self {
        debug_assert!(storage.contains_node(source_id));
        debug_assert!(storage.contains_node(target_id));

        Self {
            storage,

            id,

            source_id,
            target_id,

            weight,
        }
    }

    #[must_use]
    pub const fn id(&self) -> &'a S::EdgeId {
        self.id
    }

    #[must_use]
    pub const fn source_id(&self) -> &'a S::NodeId {
        self.source_id
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
        self.storage.node(self.source_id).expect(
            "corrupted storage or violated contract upon creation of this edge; the source node \
             must be active in the same storage as this edge",
        )
    }

    #[must_use]
    pub const fn target_id(&self) -> &'a S::NodeId {
        self.target_id
    }

    #[must_use]
    pub fn target(&self) -> Node<'a, S> {
        self.storage.node(self.target_id).expect(
            "corrupted storage or violated contract upon creation of this edge; the target node \
             must be active in the same storage as this edge",
        )
    }

    #[must_use]
    pub const fn weight(&self) -> &'a S::EdgeWeight {
        self.weight
    }
}

impl<S> Edge<'_, S>
where
    S: GraphStorage,
    S::NodeId: Clone,
    S::EdgeId: Clone,
    S::EdgeWeight: Clone,
{
    #[must_use]
    pub fn detach(self) -> DetachedStorageEdge<S> {
        DetachedEdge::new(
            self.id.clone(),
            self.weight.clone(),
            self.source_id.clone(),
            self.target_id.clone(),
        )
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgeMut<'a, S>
where
    S: GraphStorage,
{
    id: &'a S::EdgeId,

    source_id: &'a S::NodeId,
    target_id: &'a S::NodeId,

    weight: &'a mut S::EdgeWeight,
}

impl<'a, S> EdgeMut<'a, S>
where
    S: GraphStorage,
{
    pub fn new(
        id: &'a S::EdgeId,
        weight: &'a mut S::EdgeWeight,

        source_id: &'a S::NodeId,
        target_id: &'a S::NodeId,
    ) -> Self {
        Self {
            id,

            source_id,
            target_id,

            weight,
        }
    }

    #[must_use]
    pub const fn id(&self) -> &'a S::EdgeId {
        self.id
    }

    #[must_use]
    pub const fn source_id(&self) -> &'a S::NodeId {
        self.source_id
    }

    #[must_use]
    pub const fn target_id(&self) -> &'a S::NodeId {
        self.target_id
    }

    #[must_use]
    pub fn weight(&self) -> &S::EdgeWeight {
        self.weight
    }

    pub fn weight_mut(&mut self) -> &mut S::EdgeWeight {
        self.weight
    }
}

impl<S> EdgeMut<'_, S>
where
    S: GraphStorage,
    S::NodeId: Clone,
    S::EdgeId: Clone,
    S::EdgeWeight: Clone,
{
    #[must_use]
    pub fn detach(self) -> DetachedStorageEdge<S> {
        DetachedEdge::new(
            self.id.clone(),
            self.weight.clone(),
            self.source_id.clone(),
            self.target_id.clone(),
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DetachedEdge<E, N, W> {
    pub id: E,

    pub source: N,
    pub target: N,

    pub weight: W,
}

impl<E, N, W> DetachedEdge<E, N, W> {
    pub const fn new(id: E, weight: W, source: N, target: N) -> Self {
        Self {
            id,
            source,
            target,
            weight,
        }
    }
}
