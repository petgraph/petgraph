//! Graph storage implementations.
//!
//! This module contains the various graph storage implementations that are used by the [`Graph`]
//! type.
//!
//! # Overview
//!
//! The [`GraphStorage`] type is split into multiple sub-traits, while trying to keep the number of
//! sub-traits to a minimum.
//!
//! These include:
//! - [`GraphStorage`]: The core trait that all graph storage implementations must implement, maps
//!   to an undirected graph.
//! - [`DirectedGraphStorage`]: A trait for directed graph storage implementations.
//! - [`LinearGraphStorage`]: A trait for linear graph storage implementations. A linear graph
//!   storage is one where the node indices and edge indices can be mapped to a continuous integer
//!   using a [`LinearIndexLookup`].
//! - [`RetainableGraphStorage`]: A trait for retainable graph storage implementations.
//!
//! [`GraphStorage`] proposes that [`DirectedGraphStorage`] is simply a specialization of an
//! undirected graph, meaning that the supertrait of [`DirectedGraphStorage`] is also
//! [`GraphStorage`] and that all implementations of [`DirectedGraphStorage`] must also support
//! undirected graph operations.
//!
//! # Implementation Notes
//!
//! [`LinearGraphStorage`] and [`RetainableGraphStorage`] are subject to
//! removal during the alpha period.
mod directed;

mod retain;

use error_stack::{Context, Result};

pub use self::{directed::DirectedGraphStorage, retain::RetainableGraphStorage};
use crate::{
    edge::{DetachedEdge, Edge, EdgeMut},
    id::GraphId,
    node::{DetachedNode, Node, NodeMut},
};

/// A trait for graph storage implementations.
///
/// This trait is the core of the graph storage system, and is used to define the interface that all
/// graph storage implementations must implement.
///
/// A [`GraphStorage`] is never used alone, but instead is used in conjunction with a [`Graph`],
/// which is a thin abstraction layer over the [`GraphStorage`].
///
/// This allows us to have multiple different graph storage implementations, while still having a
/// convenient interface with a minimal amount of boilerplate and functions to implement.
///
/// You can instantiate a graph storage implementation directly, but it is recommended to use the
/// functions [`Graph::new`], [`Graph::new_in`], or [`Graph::with_capacity`] instead.
///
/// # Example
///
/// ```
/// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
/// use petgraph_dino::DinosaurStorage;
///
/// let mut storage = DinosaurStorage::<(), (), Directed>::new();
/// # assert_eq!(storage.num_nodes(), 0);
/// # assert_eq!(storage.num_edges(), 0);
/// ```
///
/// [`Graph`]: crate::graph::Graph
pub trait GraphStorage: Sized {
    /// The unique identifier for an edge.
    ///
    /// This is used to identify edges in the graph.
    /// The equivalent in a `HashMap` would be the key.
    /// In contrast to a `HashMap` (or similar data structures) the trait does not enforce any
    /// additional constraints,
    /// implementations are free to choose to limit identifiers to a certain subset if required, or
    /// choose a concrete type.
    ///
    /// Fundamentally, while [`Self::EdgeId`] must be of type [`GraphId`], the chosen
    /// [`Self::EdgeId`] of an implementation should either implement [`ManagedGraphId`] or
    /// [`ArbitraryGraphId`], which work as marker traits.
    ///
    /// [`ManagedGraphId`] indicates that the implementation manages the identifiers, subsequently,
    /// users are not allowed to create identifiers themselves.
    /// [`ArbitraryGraphId`] indicates that the implementation does not manage the identifiers, and
    /// users are allowed to create identifiers themselves.
    /// This is reflected in the API through the [`Attributes`] type, which needs to be supplied
    /// when creating edges, if the implementation does not manage the identifiers, [`Attributes`]
    /// will allow users to specify the identifier and weight of the edge, while if the
    /// implementation manages the identifiers, [`Attributes`] will only allow users to specify the
    /// weight of the edge.
    ///
    /// [`Attributes`]: crate::attributes::Attributes
    /// [`ManagedGraphId`]: crate::id::ManagedGraphId
    /// [`ArbitraryGraphId`]: crate::id::ArbitraryGraphId
    type EdgeId: GraphId;

    /// The weight of an edge.
    ///
    /// This works in tandem with [`Self::EdgeId`], and is used to store the weight of an edge.
    /// The equivalent in a `HashMap` would be the value.
    /// No constraints are enforced on this type (except that it needs to be `Sized`), but
    /// implementations _may_ choose to enforce additional constraints, or limit the type to a
    /// specific concrete type.
    /// For example a graph that is unable to store any edge weights would choose to only support
    /// `()`.
    type EdgeWeight;

    /// The error type used by the graph.
    ///
    /// Some operations on a graph may fail, for example during insertion of a node or edge.
    /// This error (which needs to be an `error-stack` context) will be returned in a [`Report`] if
    /// any of these operations fail.
    ///
    /// # Implementation Notes
    ///
    /// Instead of being able to specify a different error type for each operation, we only allow
    /// for a single error type.
    /// This is very intentional, as error-stack allows us to layer errors on top of each other,
    /// meaning that a more specific error may be wrapped in this more general error type.
    ///
    /// [`Report`]: error_stack::Report
    type Error: Context;

    /// The unique identifier for a node.
    ///
    /// This is used to identify nodes in the graph.
    /// The equivalent in a `HashMap` would be the key.
    /// In contrast to a `HashMap` (or similar data structures) the trait does not enforce any
    /// additional constraints,
    /// implementations are free to choose to limit identifiers to a certain subset if required, or
    /// choose a concrete type.
    ///
    /// Fundamentally, while [`Self::NodeId`] must be of type [`GraphId`], the chosen
    /// [`Self::NodeId`] of an implementation should either implement [`ManagedGraphId`] or
    /// [`ArbitraryGraphId`], which work as marker traits.
    ///
    /// [`ManagedGraphId`] indicates that the implementation manages the identifiers, subsequently,
    /// users are not allowed to create identifiers themselves.
    /// [`ArbitraryGraphId`] indicates that the implementation does not manage the identifiers, and
    /// users are allowed to create identifiers themselves.
    /// This is reflected in the API through the [`Attributes`] type, which needs to be supplied
    /// when creating a node, if the implementation does not manage the identifiers, [`Attributes`]
    /// will allow users to specify the identifier and weight of the node, while if the
    /// implementation manages the identifiers, [`Attributes`] will only allow users to specify the
    /// weight of the node.
    ///
    /// [`Attributes`]: crate::attributes::Attributes
    /// [`ManagedGraphId`]: crate::id::ManagedGraphId
    /// [`ArbitraryGraphId`]: crate::id::ArbitraryGraphId
    type NodeId: GraphId;

    /// The weight of a node.
    ///
    /// This works in tandem with [`Self::NodeId`], and is used to store the weight of a node.
    /// The equivalent in a `HashMap` would be the value.
    /// No constraints are enforced on this type (except that it needs to be `Sized`), but
    /// implementations _may_ choose to enforce additional constraints, or limit the type to a
    /// specific concrete type.
    /// For example a graph that is unable to store any node weights would choose to only support
    /// `()`.
    type NodeWeight;

    /// Create a new graph storage with the given capacity.
    ///
    /// If the capacity is `None`, the implementation is free to choose a default capacity, or may
    /// not allocate any memory at all.
    ///
    /// The semantics are the same as [`Vec::with_capacity`] or [`Vec::new`], where a capacity of
    /// `None` should correspond to `Vec::new`, and a capacity of `Some(n)` should correspond to
    /// `Vec::with_capacity(n)`.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::{DiDinoGraph, DinosaurStorage};
    ///
    /// let storage = DinosaurStorage::<(), (), Directed>::new();
    /// # assert_eq!(storage.num_nodes(), 0);
    /// # assert_eq!(storage.num_edges(), 0);
    ///
    /// let storage = DiDinoGraph::<(), ()>::with_capacity(Some(10), Some(10));
    /// # assert_eq!(storage.num_nodes(), 0);
    /// # assert_eq!(storage.num_edges(), 0);
    /// ```
    fn with_capacity(node_capacity: Option<usize>, edge_capacity: Option<usize>) -> Self;

    /// Create a new graph storage from the given nodes and edges.
    ///
    /// This takes a list of nodes and edges, and tries to create a graph from them. This is the
    /// reverse operation of [`Self::into_parts`], which converts the current graph storage into an
    /// iterable of nodes and edges.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinosaurStorage;
    ///
    /// let mut storage = DinosaurStorage::<u8, u8, Directed>::new();
    ///
    /// # let id = storage.next_node_id(NoValue::new());
    /// let a = *storage.insert_node(id, 1).unwrap().id();
    ///
    /// # let id = storage.next_node_id(NoValue::new());
    /// let b = *storage.insert_node(id, 2).unwrap().id();
    ///
    /// # let id = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(id, 3, &a, &b).unwrap();
    ///
    /// assert_eq!(storage.num_nodes(), 2);
    /// assert_eq!(storage.num_edges(), 1);
    ///
    /// let (nodes, edges) = storage.into_parts();
    ///
    /// let storage = DinosaurStorage::<_, _, Directed>::from_parts(nodes, edges).unwrap();
    ///
    /// assert_eq!(storage.num_nodes(), 2);
    /// assert_eq!(storage.num_edges(), 1);
    /// ```
    ///
    /// # Errors
    ///
    /// If any of the nodes or edges are invalid, or any of the invariant checks of the underlying
    /// implementation fail, an error is returned.
    ///
    /// The default implementation uses [`Self::insert_node`] and [`Self::insert_edge`] to insert
    /// the nodes and edges, which are fallible.
    /// The default implementation also works in a fail-slow manner, utilizing the `error-stack`
    /// feature of extending errors with others. This means that even if multiple errors occur, all
    /// of them will be returned, but has the potential downside of being slower in cases of
    /// failures.
    ///
    /// Implementations may choose to override this default implementation, but should try to also
    /// be fail-slow.
    fn from_parts(
        nodes: impl IntoIterator<Item = DetachedNode<Self::NodeId, Self::NodeWeight>>,
        edges: impl IntoIterator<Item = DetachedEdge<Self::EdgeId, Self::NodeId, Self::EdgeWeight>>,
    ) -> Result<Self, Self::Error> {
        let nodes = nodes.into_iter();
        let edges = edges.into_iter();

        let (_, nodes_max) = nodes.size_hint();
        let (_, edges_max) = edges.size_hint();

        let mut graph = Self::with_capacity(nodes_max, edges_max);

        // by default we try to fail slow, this way we can get as much data about potential errors
        // as possible.
        let mut result: Result<(), Self::Error> = Ok(());

        for node in nodes {
            if let Err(error) = graph.insert_node(node.id, node.weight) {
                match &mut result {
                    Err(errors) => errors.extend_one(error),
                    result => *result = Err(error),
                }
            }
        }

        result?;

        // we need to ensure that all nodes are inserted before we insert edges, otherwise we might
        // end up with invalid data (or redundant errors).
        let mut result: Result<(), Self::Error> = Ok(());

        for edge in edges {
            if let Err(error) = graph.insert_edge(edge.id, edge.weight, &edge.source, &edge.target)
            {
                match &mut result {
                    Err(errors) => errors.extend_one(error),
                    result => *result = Err(error),
                }
            }
        }

        result.map(|()| graph)
    }

    /// Convert the current graph storage into an iterable of nodes and edges.
    ///
    /// This is the reverse operation of [`Self::from_parts`], which takes an iterable of nodes and
    /// edges and tries to create a graph from them.
    ///
    /// The iterables returned by this function are not guaranteed to be in any particular order,
    /// but must contain all nodes and edges.
    ///
    /// It must always hold true that using the iterables returned by this function to create a new
    /// graph storage using [`Self::from_parts`] will result in a structurally identical graph.
    ///
    /// # Example
    ///
    /// ```
    /// use std::{collections::HashSet, iter::once};
    ///
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinosaurStorage;
    ///
    /// let mut storage = DinosaurStorage::<u8, u8, Directed>::new();
    ///
    /// # let id = storage.next_node_id(NoValue::new());
    /// let a = *storage.insert_node(id, 1).unwrap().id();
    ///
    /// # let id = storage.next_node_id(NoValue::new());
    /// let b = *storage.insert_node(id, 2).unwrap().id();
    ///
    /// # let id = storage.next_edge_id(NoValue::new());
    /// let ab = *storage.insert_edge(id, 3, &a, &b).unwrap().id();
    ///
    /// assert_eq!(storage.num_nodes(), 2);
    /// assert_eq!(storage.num_edges(), 1);
    ///
    /// let (nodes, edges) = storage.into_parts();
    ///
    /// let node_ids: HashSet<_> = nodes.map(|detached_node| detached_node.id).collect();
    /// let edge_ids: HashSet<_> = edges.map(|detached_edge| detached_edge.id).collect();
    ///
    /// assert_eq!(node_ids, [a, b].into_iter().collect());
    /// assert_eq!(edge_ids, once(ab).collect());
    /// ```
    fn into_parts(
        self,
    ) -> (
        impl Iterator<Item = DetachedNode<Self::NodeId, Self::NodeWeight>>,
        impl Iterator<Item = DetachedEdge<Self::EdgeId, Self::NodeId, Self::EdgeWeight>>,
    );

    /// Returns the number of nodes in the graph.
    ///
    /// This is equivalent to [`Self::nodes`].`count()`.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_dino::DinosaurStorage;
    ///
    /// let mut storage = DinosaurStorage::<(), (), Directed>::new();
    ///
    /// # let id = storage.next_node_id(NoValue::new());
    /// storage.insert_node(id, ()).unwrap();
    ///
    /// assert_eq!(storage.num_nodes(), 1);
    ///
    /// # let id = storage.next_node_id(NoValue::new());
    /// storage.insert_node(id, ()).unwrap();
    ///
    /// assert_eq!(storage.num_nodes(), 2);
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// This is a default implementation, which uses [`Self::nodes`].`count()`, custom
    /// implementations, should if possible, override this.
    fn num_nodes(&self) -> usize {
        self.nodes().count()
    }

    /// Returns the number of edges in the graph.
    ///
    /// This is equivalent to [`Self::edges`].`count()`.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_dino::DinosaurStorage;
    ///
    /// let mut storage = DinosaurStorage::<(), (), Directed>::new();
    ///
    /// # let id = storage.next_node_id(NoValue::new());
    /// # let a = *storage.insert_node(id, ()).unwrap().id();
    /// #
    /// # let id = storage.next_node_id(NoValue::new());
    /// # let b = *storage.insert_node(id, ()).unwrap().id();
    /// #
    /// # let id = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(id, (), &a, &b).unwrap();
    /// assert_eq!(storage.num_edges(), 1);
    /// #
    /// # let id = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(id, (), &a, &b).unwrap();
    /// #
    /// assert_eq!(storage.num_edges(), 2);
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// This is a default implementation, which uses [`Self::edges`].`count()`, custom
    /// implementations, should if possible, override this.
    fn num_edges(&self) -> usize {
        self.edges().count()
    }

    /// Return the next node identifier for the given attribute.
    ///
    /// This is used to generate new node identifiers and should not be called by a user directly
    /// and is instead used by the [`Graph`] type to generate a new identifier that is then used
    /// during [`insert_node`].
    ///
    /// This function is only of interest for implementations that manage the identifiers of nodes
    /// (using the [`ManagedGraphId`] marker trait).
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{attributes::NoValue, edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinosaurStorage;
    ///
    /// let mut storage = DinosaurStorage::<(), (), Directed>::new();
    ///
    /// // `DinosaurStorage` uses `ManagedGraphId` for both node and edge identifiers,
    /// // so we must use `NoValue` here.
    /// let a = storage.next_node_id(NoValue::new());
    /// let b = storage.next_node_id(NoValue::new());
    ///
    /// assert_eq!(a, b);
    ///
    /// storage.insert_node(a, ()).unwrap();
    ///
    /// let c = storage.next_node_id(NoValue::new());
    ///
    /// assert_ne!(a, c);
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// When implementing this function, it is important to ensure that the returned identifier is
    /// stable, and unique and must ensure that the returned identifier is not already in use.
    ///
    /// Stable meaning that repeated calls to this function (without any other changes to the graph)
    /// should return the same identifier.
    ///
    /// The implementation of this function must also be fast, as it is called every time a new node
    /// is inserted and must be pure, meaning that it must not have any side-effects.
    ///
    /// If the [`Self::NodeId`] is a [`ManagedGraphId`], the implementation of this function must
    /// return [`Self::NodeId`] and must not take `attribute` into account (in fact it can't, as the
    /// value is always [`NoValue`]). Should the [`ArbitraryGraphId`] marker trait be implemented,
    /// this function should effectively be a no-op and given `attribute`.
    fn next_node_id(&self, attribute: <Self::NodeId as GraphId>::AttributeIndex) -> Self::NodeId;

    /// Inserts a new node into the graph.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{attributes::NoValue, edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinosaurStorage;
    ///
    /// let mut storage = DinosaurStorage::<u8, (), Directed>::new();
    ///
    /// // `DinosaurStorage` uses `ManagedGraphId` for both node and edge identifiers,
    /// // so we must use `NoValue` here.
    /// let id = storage.next_node_id(NoValue::new());
    /// storage.insert_node(id, 1).unwrap();
    /// #
    /// # assert_eq!(storage.node(&id).unwrap().weight(), &1);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if a node with the given identifier already exists, or if any of the
    /// invariants (depending on the implementation) are violated.
    fn insert_node(
        &mut self,
        id: Self::NodeId,

        weight: Self::NodeWeight,
    ) -> Result<NodeMut<Self>, Self::Error>;

    /// Return the next edge identifier for the given attribute.
    ///
    /// This is used to generate new edge identifiers and should not be called by a user directly
    /// and is instead used by the [`Graph`] type to generate a new identifier that is then used
    /// during [`insert_edge`].
    ///
    /// This function is only of interest for implementations that manage the identifiers of edges
    /// (using the [`ManagedGraphId`] marker trait).
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{attributes::NoValue, edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinosaurStorage;
    ///
    /// let mut storage = DinosaurStorage::<u8, (), Directed>::new();
    ///
    /// // `DinosaurStorage` uses `ManagedGraphId` for both node and edge identifiers,
    /// // so we must use `NoValue` here.
    /// let id = storage.next_node_id(NoValue::new());
    /// storage.insert_node(id, 1).unwrap();
    /// #
    /// # assert_eq!(storage.node(&id).unwrap().weight(), &1);
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// All the same notes as [`Self::next_node_id`] apply here as well.
    fn next_edge_id(&self, attribute: <Self::EdgeId as GraphId>::AttributeIndex) -> Self::EdgeId;

    /// Inserts a new edge into the graph.
    ///
    /// # Example
    ///
    ///```
    /// use petgraph_core::{attributes::NoValue, edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinosaurStorage;
    ///
    /// let mut storage = DinosaurStorage::<u8, u8, Directed>::new();
    /// #
    /// # let a = storage.next_node_id(NoValue::new());
    /// # storage.insert_node(a, 1).unwrap();
    /// #
    /// # let b = storage.next_node_id(NoValue::new());
    /// # storage.insert_node(b, 2).unwrap();
    ///
    /// // `DinosaurStorage` uses `ManagedGraphId` for both node and edge identifiers,
    /// // so we must use `NoValue` here.
    /// let id = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(id, 3, &a, &b).unwrap();
    /// #
    /// # assert_eq!(storage.edge(&id).unwrap().weight(), &3);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if parallel edges are not allowed, or any of the invariants (depending on
    /// the implementation) are violated.
    /// These invariants _may_ include that an edge between the source and target already exist, but
    /// some implementations may choose to allow parallel edges.
    fn insert_edge(
        &mut self,
        id: Self::EdgeId,
        weight: Self::EdgeWeight,

        source: &Self::NodeId,
        target: &Self::NodeId,
    ) -> Result<EdgeMut<Self>, Self::Error>;

    /// Removes the node with the given identifier from the graph.
    ///
    /// This will return [`None`] if the node does not exist, and will return the detached node if
    /// it does.
    ///
    /// Calling this function will also remove all edges that are connected to the node, but will
    /// not return those detached edges.
    ///
    /// To also return the detached edges that were removed, use a combination of
    /// [`Self::node_connections`], [`Self::remove_edge`] and [`Self::remove_node`] instead.
    ///
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{attributes::NoValue, edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinosaurStorage;
    ///
    /// let mut storage = DinosaurStorage::<u8, u8, Directed>::new();
    ///
    /// let a = storage.next_node_id(NoValue::new());
    /// storage.insert_node(a, 1).unwrap();
    ///
    /// let removed = storage.remove_node(&a).unwrap();
    /// assert_eq!(removed.id, a);
    /// assert_eq!(removed.weight, 1);
    /// #
    /// # assert_eq!(storage.num_nodes(), 0);
    /// ```
    ///
    /// ## Return Node with Edges
    ///
    /// ```
    /// use petgraph_core::{attributes::NoValue, edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinosaurStorage;
    ///
    /// let mut storage = DinosaurStorage::<u8, u8, Directed>::new();
    /// #
    /// # let a = storage.next_node_id(NoValue::new());
    /// storage.insert_node(a, 1).unwrap();
    /// #
    /// # let b = storage.next_node_id(NoValue::new());
    /// storage.insert_node(b, 2).unwrap();
    /// #
    /// # let ab = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(ab, 3, &a, &b).unwrap();
    ///
    /// let connections: Vec<_> = storage
    ///     .node_connections(&a)
    ///     .map(|edge| *edge.id())
    ///     .collect();
    /// #
    /// # assert_eq!(connections, [ab]);
    ///
    /// for edge in connections {
    ///     // do something with the detached edge
    ///     storage.remove_edge(&edge).unwrap();
    /// }
    ///
    /// storage.remove_node(&a).unwrap();
    /// ```
    fn remove_node(
        &mut self,
        id: &Self::NodeId,
    ) -> Option<DetachedNode<Self::NodeId, Self::NodeWeight>>;

    /// Removes the edge with the given identifier from the graph.
    ///
    /// This will return [`None`] if the edge does not exist, and will return the detached edge if
    /// it existed.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{attributes::NoValue, edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinosaurStorage;
    ///
    /// let mut storage = DinosaurStorage::<u8, u8, Directed>::new();
    /// #
    /// # let a = storage.next_node_id(NoValue::new());
    /// storage.insert_node(a, 1).unwrap();
    /// #
    /// # let b = storage.next_node_id(NoValue::new());
    /// storage.insert_node(b, 2).unwrap();
    /// #
    /// # let ab = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(ab, 3, &a, &b).unwrap();
    ///
    /// let removed = storage.remove_edge(&ab).unwrap();
    /// assert_eq!(removed.id, ab);
    /// assert_eq!(removed.weight, 3);
    /// #
    /// # assert_eq!(storage.num_edges(), 0);
    /// ```
    fn remove_edge(
        &mut self,
        id: &Self::EdgeId,
    ) -> Option<DetachedEdge<Self::EdgeId, Self::NodeId, Self::EdgeWeight>>;

    /// Clears the graph, removing all nodes and edges.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{attributes::NoValue, edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinosaurStorage;
    ///
    /// let mut storage = DinosaurStorage::<u8, u8, Directed>::new();
    /// #
    /// # let a = storage.next_node_id(NoValue::new());
    /// storage.insert_node(a, 1).unwrap();
    /// #
    /// # let b = storage.next_node_id(NoValue::new());
    /// storage.insert_node(b, 2).unwrap();
    /// #
    /// # let ab = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(ab, 3, &a, &b).unwrap();
    ///
    /// storage.clear();
    /// #
    /// # assert_eq!(storage.num_nodes(), 0);
    /// # assert_eq!(storage.num_edges(), 0);
    /// ```
    fn clear(&mut self);

    /// Returns the node with the given identifier.
    ///
    /// This will return [`None`] if the node does not exist, and will return the node if it does.
    ///
    /// The [`Node`] type returned by this function contains a reference to the current graph,
    /// meaning you are able to query for e.g. the neighours of the node.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{attributes::NoValue, edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinosaurStorage;
    ///
    /// let mut storage = DinosaurStorage::<u8, u8, Directed>::new();
    /// #
    /// # let a = storage.next_node_id(NoValue::new());
    /// storage.insert_node(a, 1).unwrap();
    ///
    /// let node = storage.node(&a).unwrap();
    ///
    /// assert_eq!(node.id(), &a);
    /// assert_eq!(node.weight(), &1);
    ///
    /// // This will access the underlying storage, and is equivalent to `storage.neighbours(&a)`.
    /// assert_eq!(node.neighbours().count(), 0);
    /// ```
    fn node(&self, id: &Self::NodeId) -> Option<Node<Self>>;

    /// Returns the node, with a mutable weight, with the given identifier.
    ///
    /// This will return [`None`] if the node does not exist, and will return the node if it does.
    ///
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{attributes::NoValue, edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinosaurStorage;
    ///
    /// let mut storage = DinosaurStorage::<u8, u8, Directed>::new();
    /// #
    /// # let a = storage.next_node_id(NoValue::new());
    /// storage.insert_node(a, 1).unwrap();
    ///
    /// let node = storage.node_mut(&a).unwrap();
    ///
    /// assert_eq!(node.id(), &a);
    /// assert_eq!(node.weight(), &mut 1);
    /// ```
    fn node_mut(&mut self, id: &Self::NodeId) -> Option<NodeMut<Self>>;

    /// Checks if the node with the given identifier exists.
    ///
    /// This is equivalent to [`Self::node`].`is_some()`.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{attributes::NoValue, edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinosaurStorage;
    ///
    /// let mut storage = DinosaurStorage::<u8, u8, Directed>::new();
    /// #
    /// # let a = storage.next_node_id(NoValue::new());
    /// storage.insert_node(a, 1).unwrap();
    /// # let b = storage.next_node_id(NoValue::new());
    ///
    /// assert!(storage.contains_node(&a));
    /// assert!(!storage.contains_node(&b));
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// The default implementation simply checks if [`Self::node`] returns [`Some`], but if
    /// possible, custom implementations that are able to do this more efficiently should override
    /// this.
    fn contains_node(&self, id: &Self::NodeId) -> bool {
        self.node(id).is_some()
    }

    /// Returns the edge with the given identifier, if it exists.
    ///
    /// This will return [`None`] if the edge does not exist, and will return the edge if it does.
    ///
    /// The [`Edge`] type returned by this function contains a reference to the current graph,
    /// meaning that continued exploration of the graph is possible.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{attributes::NoValue, edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinosaurStorage;
    ///
    /// let mut storage = DinosaurStorage::<u8, u8, Directed>::new();
    /// #
    /// # let a = storage.next_node_id(NoValue::new());
    /// storage.insert_node(a, 1).unwrap();
    /// # let b = storage.next_node_id(NoValue::new());
    /// storage.insert_node(b, 2).unwrap();
    ///
    /// # let ab = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(ab, 3, &a, &b).unwrap();
    ///
    /// let edge = storage.edge(&ab).unwrap();
    /// assert_eq!(edge.weight(), &3);
    ///
    /// assert_eq!(edge.source_id(), &a);
    /// // This will request the target from underlying storage,
    /// // meaning if one only wants to retrieve the id it is faster
    /// // to just use `edge.source_id()`.
    /// // Because `self.node()` returns an `Option`, so will `edge.source()`.
    /// assert_eq!(edge.source().unwrap().id(), &a);
    /// assert_eq!(edge.source().unwrap().weight(), &1);
    ///
    /// assert_eq!(edge.target_id(), &b);
    /// // This will request the target from underlying storage,
    /// // meaning if one only wants to retrieve the id it is faster
    /// // to just use `edge.target_id()`.
    /// // Because `self.node()` returns an `Option`, so will `edge.target()`.
    /// assert_eq!(edge.target().unwrap().id(), &b);
    /// assert_eq!(edge.target().unwrap().weight(), &2);
    /// ```
    fn edge(&self, id: &Self::EdgeId) -> Option<Edge<Self>>;

    /// Returns the edge, with a mutable weight, with the given identifier, if it exists.
    ///
    /// This will return [`None`] if the edge does not exist, and will return the edge if it does.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{attributes::NoValue, edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinosaurStorage;
    ///
    /// let mut storage = DinosaurStorage::<u8, u8, Directed>::new();
    /// #
    /// # let a = storage.next_node_id(NoValue::new());
    /// storage.insert_node(a, 1).unwrap();
    /// # let b = storage.next_node_id(NoValue::new());
    /// storage.insert_node(b, 2).unwrap();
    ///
    /// # let ab = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(ab, 3, &a, &b).unwrap();
    ///
    /// let mut edge = storage.edge_mut(&ab).unwrap();
    ///
    /// assert_eq!(edge.weight(), &mut 3);
    /// assert_eq!(edge.source_id(), &a);
    /// assert_eq!(edge.target_id(), &b);
    ///
    /// *edge.weight_mut() = 4;
    ///
    /// assert_eq!(storage.edge(&ab).unwrap().weight(), &4);
    /// ```
    fn edge_mut(&mut self, id: &Self::EdgeId) -> Option<EdgeMut<Self>>;

    /// Checks if the edge with the given identifier exists.
    ///
    /// This is equivalent to [`Self::edge`].`is_some()`.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{attributes::NoValue, edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinosaurStorage;
    ///
    /// let mut storage = DinosaurStorage::<u8, u8, Directed>::new();
    /// #
    /// # let a = storage.next_node_id(NoValue::new());
    /// storage.insert_node(a, 1).unwrap();
    /// # let b = storage.next_node_id(NoValue::new());
    /// storage.insert_node(b, 2).unwrap();
    ///
    /// # let ab = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(ab, 3, &a, &b).unwrap();
    /// # let ba = storage.next_edge_id(NoValue::new());
    ///
    /// assert!(storage.contains_edge(&ab));
    /// assert!(!storage.contains_edge(&ba));
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// The default implementation simply checks if [`Self::edge`] returns [`Some`], but if
    /// possible, custom implementations that are able to do this more efficiently should override
    /// this.
    fn contains_edge(&self, id: &Self::EdgeId) -> bool {
        self.edge(id).is_some()
    }

    fn edges_between<'a: 'b, 'b>(
        &'a self,
        source: &'b Self::NodeId,
        target: &'b Self::NodeId,
    ) -> impl Iterator<Item = Edge<'a, Self>> + 'b {
        // How does this work with a default implementation?
        let from_source = self
            .node_connections(source)
            .filter(move |edge| edge.target_id() == target);

        let from_target = self
            .node_connections(target)
            .filter(move |edge| edge.source_id() == source);

        from_source.chain(from_target)
    }

    // I'd love to provide a default implementation for this, but I just can't get it to work.
    fn edges_between_mut<'a: 'b, 'b>(
        &'a mut self,
        source: &'b Self::NodeId,
        target: &'b Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'a, Self>> + 'b;

    fn node_connections<'a: 'b, 'b>(
        &'a self,
        id: &'b Self::NodeId,
    ) -> impl Iterator<Item = Edge<'a, Self>> + 'b;

    fn node_connections_mut<'a: 'b, 'b>(
        &'a mut self,
        id: &'b Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'a, Self>> + 'b;

    fn node_neighbours<'a: 'b, 'b>(
        &'a self,
        id: &'b Self::NodeId,
    ) -> impl Iterator<Item = Node<'a, Self>> + 'b {
        self.node_connections(id)
            .filter_map(move |edge: Edge<Self>| {
                // doing it this way allows us to also get ourselves as a neighbour if we have a
                // self-loop
                if edge.source_id() == id {
                    edge.target()
                } else {
                    edge.source()
                }
            })
    }

    // I'd love to provide a default implementation for this, but I just can't get it to work.
    fn node_neighbours_mut<'a: 'b, 'b>(
        &'a mut self,
        id: &'b Self::NodeId,
    ) -> impl Iterator<Item = NodeMut<'a, Self>> + 'b;

    fn external_nodes(&self) -> impl Iterator<Item = Node<Self>> {
        self.nodes()
            .filter(|node| self.node_neighbours(node.id()).next().is_none())
    }

    // I'd love to provide a default implementation for this, but I just can't get it to work.
    fn external_nodes_mut(&mut self) -> impl Iterator<Item = NodeMut<Self>>;

    fn nodes(&self) -> impl Iterator<Item = Node<Self>>;

    fn nodes_mut(&mut self) -> impl Iterator<Item = NodeMut<Self>>;

    fn edges(&self) -> impl Iterator<Item = Edge<Self>>;

    fn edges_mut(&mut self) -> impl Iterator<Item = EdgeMut<Self>>;

    fn reserve(&mut self, additional_nodes: usize, additional_edges: usize) {
        self.reserve_nodes(additional_nodes);
        self.reserve_edges(additional_edges);
    }

    #[allow(unused_variables)]
    fn reserve_nodes(&mut self, additional: usize) {}
    #[allow(unused_variables)]
    fn reserve_edges(&mut self, additional: usize) {}

    fn reserve_exact(&mut self, additional_nodes: usize, additional_edges: usize) {
        self.reserve_exact_nodes(additional_nodes);
        self.reserve_exact_edges(additional_edges);
    }

    fn reserve_exact_nodes(&mut self, additional: usize) {
        self.reserve_nodes(additional);
    }
    fn reserve_exact_edges(&mut self, additional: usize) {
        self.reserve_edges(additional);
    }

    fn shrink_to_fit(&mut self) {
        self.shrink_to_fit_nodes();
        self.shrink_to_fit_edges();
    }

    fn shrink_to_fit_nodes(&mut self) {}
    fn shrink_to_fit_edges(&mut self) {}
}
