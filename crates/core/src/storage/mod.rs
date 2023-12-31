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
//! - [`RetainableGraphStorage`]: A trait for retainable graph storage implementations.
//! - [`AuxiliaryGraphStorage`]: A trait to access storage for arbitrary additional data.
//!
//! [`GraphStorage`] proposes that [`DirectedGraphStorage`] is simply a specialization of an
//! undirected graph, meaning that the supertrait of [`DirectedGraphStorage`] is also
//! [`GraphStorage`] and that all implementations of [`DirectedGraphStorage`] must also support
//! undirected graph operations.
//!
//! # Implementation Notes
//!
//! [`RetainableGraphStorage`] is subject to removal during the alpha period.
//!
//! [`Graph`]: crate::graph::Graph
mod directed;

pub mod auxiliary;
mod retain;

use error_stack::{Context, Result};

pub use self::{
    auxiliary::AuxiliaryGraphStorage, directed::DirectedGraphStorage,
    retain::RetainableGraphStorage,
};
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
/// # Capabilities
///
/// > This is a template, which you should use to describe the capabilities of your graph storage
/// > implementation.
///
/// | Capability       | Note              |
/// |------------------|-------------------|
/// | Node Identifiers | Arbitrary/Managed |
/// | Edge Identifiers | Arbitrary/Managed |
/// | Node Weights     | ✓/✗               |
/// | Edge Weights     | ✓/✗               |
/// | Parallel Edges   | ✓/✗               |
/// | Self Loops       | ✓/✗               |
///
/// ## Space/Time Complexity
///
/// > Optional, but recommended.
///
/// | Operation        | Time Complexity | Space Complexity |
/// |------------------|-----------------|------------------|
/// | Node By Id       |                 |                  |
/// | Edge By Id       |                 |                  |
/// | Edge Between     |                 |                  |
/// | Contains Node    |                 |                  |
/// | Contains Edge    |                 |                  |
/// | Insert Node      |                 |                  |
/// | Insert Edge      |                 |                  |
/// | Remove Node      |                 |                  |
/// | Remove Edge      |                 |                  |
/// | Node Count       |                 |                  |
/// | Edge Count       |                 |                  |
/// | Node Iter        |                 |                  |
/// | Edge Iter        |                 |                  |
/// | Node Neighbours  |                 |                  |
/// | Node Connections |                 |                  |
/// | External Nodes   |                 |                  |
///
/// ### Directed Graphs
///
/// > Optional, but recommended.
///
/// | Operation                     | Time Complexity | Space Complexity |
/// |-------------------------------|-----------------|------------------|
/// | Edge Between                  |                 |                  |
/// | Directed Edge Neighbours      |                 |                  |
/// | Undirected Edge Neighbours    |                 |                  |
/// | Directed Edge Connections     |                 |                  |
/// | Undirected Edge Connections   |                 |                  |
///
/// # Example
///
/// ```
/// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
/// use petgraph_dino::DinoStorage;
///
/// let mut storage = DinoStorage::<(), (), Directed>::new();
/// # assert_eq!(storage.num_nodes(), 0);
/// # assert_eq!(storage.num_edges(), 0);
/// ```
///
/// [`Graph`]: crate::graph::Graph
/// [`Graph::new`]: crate::graph::Graph::new
/// [`Graph::new_in`]: crate::graph::Graph::new_in
/// [`Graph::with_capacity`]: crate::graph::Graph::with_capacity
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
    /// use petgraph_dino::{DiDinoGraph, DinoStorage};
    ///
    /// let storage = DinoStorage::<(), (), Directed>::new();
    /// # assert_eq!(storage.num_nodes(), 0);
    /// # assert_eq!(storage.num_edges(), 0);
    ///
    /// // we need to explicitly state the type of the node and edge weights, as we do insert
    /// // any nodes/edges and therefore cannot infer them.
    /// let storage = DiDinoGraph::<(), ()>::with_capacity(Some(10), Some(10));
    /// # assert_eq!(storage.num_nodes(), 0);
    /// # assert_eq!(storage.num_edges(), 0);
    /// ```
    fn with_capacity(node_capacity: Option<usize>, edge_capacity: Option<usize>) -> Self;

    /// Create a new graph storage from the given nodes and edges.
    ///
    /// This takes an iterator of nodes and edges, and tries to create a graph from them.
    /// This is the reverse operation of [`Self::into_parts`], which converts the current graph
    /// storage into an iterable of nodes and edges.
    ///
    /// The ordering of the nodes and edges in the resulting graph storage is not preserved, if that
    /// is the case in a storage implementation it should be considered an implementation
    /// detail and not be relied upon.
    /// The same applies to identifiers of nodes and edges, which may be changed during
    /// construction.
    /// The only properties that can be relied upon is that all nodes and edges will be present,
    /// their weights will be the same, and that the graph will be structurally identical.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
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
    /// let storage = DinoStorage::<_, _, Directed>::from_parts(nodes, edges).unwrap();
    ///
    /// assert_eq!(storage.num_nodes(), 2);
    /// assert_eq!(storage.num_edges(), 1);
    /// ```
    ///
    /// # Errors
    ///
    /// If any of the nodes or edges are invalid, or any of the constraint checks of the underlying
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
            if let Err(error) = graph.insert_edge(edge.id, edge.weight, edge.u, edge.v) {
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
    /// The ids of said nodes and edges may also be changed during this operation, but the weights
    /// of the nodes and edges must be the same.
    ///
    /// It must always hold true that using the iterables returned by this function to create a new
    /// graph storage using [`Self::from_parts`] will result in a structurally identical graph and
    /// that calling [`Self::from_parts`] on the same implementation that invoked
    /// [`Self::into_parts`] must not error out.
    ///
    /// # Example
    ///
    /// ```
    /// use std::{collections::HashSet, iter::once};
    ///
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
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
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<(), (), Directed>::new();
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
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<(), (), Directed>::new();
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
    /// during [`Graph::insert_node`].
    ///
    /// This function is only of interest for implementations that manage the identifiers of nodes
    /// (using the [`ManagedGraphId`] marker trait).
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{attributes::NoValue, edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<(), (), Directed>::new();
    ///
    /// // `DinoStorage` uses `ManagedGraphId` for both node and edge identifiers,
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
    ///
    /// [`Graph`]: crate::graph::Graph
    /// [`Graph::insert_node`]: crate::graph::Graph::insert_node
    /// [`ManagedGraphId`]: crate::id::ManagedGraphId
    /// [`ArbitraryGraphId`]: crate::id::ArbitraryGraphId
    /// [`NoValue`]: crate::attributes::NoValue
    fn next_node_id(&self, attribute: <Self::NodeId as GraphId>::AttributeIndex) -> Self::NodeId;

    /// Inserts a new node into the graph.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{attributes::NoValue, edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, (), Directed>::new();
    ///
    /// // `DinoStorage` uses `ManagedGraphId` for both node and edge identifiers,
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
    /// constraints (depending on the implementation) are violated.
    fn insert_node(
        &mut self,
        id: Self::NodeId,

        weight: Self::NodeWeight,
    ) -> Result<NodeMut<Self>, Self::Error>;

    /// Return the next edge identifier for the given attribute.
    ///
    /// This is used to generate new edge identifiers and should not be called by a user directly
    /// and is instead used by the [`Graph`] type to generate a new identifier that is then used
    /// during [`Graph::insert_edge`].
    ///
    /// This function is only of interest for implementations that manage the identifiers of edges
    /// (using the [`ManagedGraphId`] marker trait).
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{attributes::NoValue, edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, (), Directed>::new();
    ///
    /// // `DinoStorage` uses `ManagedGraphId` for both node and edge identifiers,
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
    ///
    /// [`Graph`]: crate::graph::Graph
    /// [`Graph::insert_edge`]: crate::graph::Graph::insert_edge
    /// [`ManagedGraphId`]: crate::id::ManagedGraphId
    fn next_edge_id(&self, attribute: <Self::EdgeId as GraphId>::AttributeIndex) -> Self::EdgeId;

    /// Inserts a new edge into the graph.
    ///
    /// If the storage implementation is undirected `u` and `v` are interchangeable, but if the
    /// storage implementation is directed `u` is the source and `v` is the target.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{attributes::NoValue, edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
    /// #
    /// # let a = storage.next_node_id(NoValue::new());
    /// # storage.insert_node(a, 1).unwrap();
    /// #
    /// # let b = storage.next_node_id(NoValue::new());
    /// # storage.insert_node(b, 2).unwrap();
    ///
    /// // `DinoStorage` uses `ManagedGraphId` for both node and edge identifiers,
    /// // so we must use `NoValue` here.
    /// let id = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(id, 3, &a, &b).unwrap();
    /// #
    /// # assert_eq!(storage.edge(&id).unwrap().weight(), &3);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if parallel edges are not allowed, or any of the constraints (depending on
    /// the implementation) are violated.
    /// These constraints _may_ include that an edge between the source and target already exist,
    /// but some implementations may choose to allow parallel edges.
    fn insert_edge(
        &mut self,
        id: Self::EdgeId,
        weight: Self::EdgeWeight,

        u: Self::NodeId,
        v: Self::NodeId,
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
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
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
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
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
        id: Self::NodeId,
    ) -> Option<DetachedNode<Self::NodeId, Self::NodeWeight>>;

    /// Removes the edge with the given identifier from the graph.
    ///
    /// This will return [`None`] if the edge does not exist, and will return the detached edge if
    /// it existed.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
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
        id: Self::EdgeId,
    ) -> Option<DetachedEdge<Self::EdgeId, Self::NodeId, Self::EdgeWeight>>;

    /// Clears the graph, removing all nodes and edges.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
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
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
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
    fn node(&self, id: Self::NodeId) -> Option<Node<Self>>;

    /// Returns the node, with a mutable weight, with the given identifier.
    ///
    /// This will return [`None`] if the node does not exist, and will return the node if it does.
    ///
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
    /// #
    /// # let a = storage.next_node_id(NoValue::new());
    /// storage.insert_node(a, 1).unwrap();
    ///
    /// let node = storage.node_mut(&a).unwrap();
    ///
    /// assert_eq!(node.id(), &a);
    /// assert_eq!(node.weight(), &mut 1);
    /// ```
    fn node_mut(&mut self, id: Self::NodeId) -> Option<NodeMut<Self>>;

    /// Checks if the node with the given identifier exists.
    ///
    /// This is equivalent to [`Self::node`].`is_some()`.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
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
    fn contains_node(&self, id: Self::NodeId) -> bool {
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
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
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
    /// assert_eq!(edge.source().id(), &a);
    /// assert_eq!(edge.source().weight(), &1);
    ///
    /// assert_eq!(edge.target_id(), &b);
    /// // This will request the target from underlying storage,
    /// // meaning if one only wants to retrieve the id it is faster
    /// // to just use `edge.target_id()`.
    /// // Because `self.node()` returns an `Option`, so will `edge.target()`.
    /// assert_eq!(edge.target().id(), &b);
    /// assert_eq!(edge.target().weight(), &2);
    /// ```
    fn edge(&self, id: Self::EdgeId) -> Option<Edge<Self>>;

    /// Returns the edge, with a mutable weight, with the given identifier, if it exists.
    ///
    /// This will return [`None`] if the edge does not exist, and will return the edge if it does.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
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
    fn edge_mut(&mut self, id: Self::EdgeId) -> Option<EdgeMut<Self>>;

    /// Checks if the edge with the given identifier exists.
    ///
    /// This is equivalent to [`Self::edge`].`is_some()`.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
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
    fn contains_edge(&self, id: Self::EdgeId) -> bool {
        self.edge(id).is_some()
    }

    /// Returns an iterator over all edges between the two given nodes.
    ///
    /// This will return an iterator over all edges between the two given nodes. The output will
    /// include all undirected edges between the two nodes, this means that for a directed graph
    /// both the forward and reverse edges will be included.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
    /// #
    /// # let a = storage.next_node_id(NoValue::new());
    /// storage.insert_node(a, 1).unwrap();
    /// # let b = storage.next_node_id(NoValue::new());
    /// storage.insert_node(b, 2).unwrap();
    ///
    /// # let ab = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(ab, 3, &a, &b).unwrap();
    /// # let ba = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(ba, 4, &b, &a).unwrap();
    ///
    /// assert_eq!(
    ///     storage
    ///         .edges_between(&a, &b)
    ///         .map(|edge| *edge.id())
    ///         .collect::<Vec<_>>(),
    ///     [ab, ba]
    /// );
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// The default implementation simply calls [`Self::node_connections`] on both nodes and then
    /// chains those, after filtering for the respective end. Most implementations should be able to
    /// provide a more efficient implementation.
    fn edges_between(
        &self,
        u: Self::NodeId,
        v: Self::NodeId,
    ) -> impl Iterator<Item = Edge<'_, Self>> {
        // How does this work with a default implementation?
        let from_source = self.node_connections(u).filter(move |edge| {
            let (edge_u, edge_v) = edge.endpoint_ids();

            let other = if edge_u == u { edge_v } else { edge_u };
            other == v
        });

        let from_target = self.node_connections(v).filter(move |edge| {
            let (edge_u, edge_v) = edge.endpoint_ids();

            // we exclude self-loops here as they are already included in `from_source`
            let other = if edge_u == v { edge_v } else { edge_u };
            other == u && edge_u != edge_v
        });

        from_source.chain(from_target)
    }

    /// Returns an iterator over all edges between the two given nodes, with mutable weights.
    ///
    /// This will return an iterator over all edges between the two given nodes. The output will
    /// include all undirected edges between the two nodes, this means that for a directed graph
    /// both the forward and reverse edges will be included.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
    /// #
    /// # let a = storage.next_node_id(NoValue::new());
    /// storage.insert_node(a, 1).unwrap();
    /// # let b = storage.next_node_id(NoValue::new());
    /// storage.insert_node(b, 2).unwrap();
    ///
    /// # let ab = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(ab, 3, &a, &b).unwrap();
    /// # let ba = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(ba, 4, &b, &a).unwrap();
    ///
    /// for mut edge in storage.edges_between_mut(&a, &b) {
    ///     *edge.weight_mut() += 1;
    /// }
    ///
    /// assert_eq!(
    ///     storage
    ///         .edges_between(&a, &b)
    ///         .map(|mut edge| *edge.weight())
    ///         .collect::<Vec<_>>(),
    ///     [4, 5]
    /// );
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// Due to the fact that this function returns mutable references to the edges, it is not
    /// possible to easily provide a default implementation for this function, as a call to
    /// [`Self::node_connections_mut`] for `source` and `target` would lead to a double mutable
    /// borrow.
    // I'd love to provide a default implementation for this, but I just can't get it to work.
    fn edges_between_mut(
        &mut self,
        u: Self::NodeId,
        v: Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>>;

    /// Returns an iterator over all edges that are connected to the given node.
    ///
    /// This will return an iterator over all edges that are connected to the given node.
    /// This includes all edges, meaning that for a directed graph both the incoming and outgoing
    /// edges will be included.
    ///
    /// There may be multiple edges between two nodes, if the implementation allows for parallel
    /// edges.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
    /// #
    /// # let a = storage.next_node_id(NoValue::new());
    /// storage.insert_node(a, 1).unwrap();
    /// # let b = storage.next_node_id(NoValue::new());
    /// storage.insert_node(b, 2).unwrap();
    /// # let c = storage.next_node_id(NoValue::new());
    /// storage.insert_node(c, 3).unwrap();
    ///
    /// # let ab = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(ab, 4, &a, &b).unwrap();
    /// # let ca = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(ca, 5, &c, &a).unwrap();
    ///
    /// assert_eq!(
    ///     storage
    ///         .node_connections(&a)
    ///         .map(|mut edge| *edge.id())
    ///         .collect::<Vec<_>>(),
    ///     [ab, ca]
    /// );
    /// ```
    fn node_connections(&self, id: Self::NodeId) -> impl Iterator<Item = Edge<'_, Self>>;

    /// Returns an iterator over all edges that are connected to the given node, with mutable
    /// weights.
    ///
    /// This will return an iterator over all edges that are connected to the given node.
    /// This includes all edges, meaning that for a directed graph both the incoming and outgoing
    /// edges will be included.
    ///
    /// There may be multiple edges between two nodes, if the implementation allows for parallel
    /// edges.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
    /// #
    /// # let a = storage.next_node_id(NoValue::new());
    /// storage.insert_node(a, 1).unwrap();
    /// # let b = storage.next_node_id(NoValue::new());
    /// storage.insert_node(b, 2).unwrap();
    /// # let c = storage.next_node_id(NoValue::new());
    /// storage.insert_node(c, 3).unwrap();
    ///
    /// # let ab = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(ab, 4, &a, &b).unwrap();
    /// # let ca = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(ca, 5, &c, &a).unwrap();
    ///
    /// for mut edge in storage.node_connections_mut(&a) {
    ///     *edge.weight_mut() += 1;
    /// }
    ///
    /// assert_eq!(
    ///     storage
    ///         .node_connections(&a)
    ///         .map(|mut edge| *edge.weight())
    ///         .collect::<Vec<_>>(),
    ///     [5, 6]
    /// );
    /// ```
    fn node_connections_mut(&mut self, id: Self::NodeId)
    -> impl Iterator<Item = EdgeMut<'_, Self>>;

    /// Returns the number of edges that are connected to the given node.
    ///
    /// This is also commonly known as the degree or valency of the node.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
    /// #
    /// # let a = storage.next_node_id(NoValue::new());
    /// storage.insert_node(a, 1).unwrap();
    /// # let b = storage.next_node_id(NoValue::new());
    /// storage.insert_node(b, 2).unwrap();
    /// # let c = storage.next_node_id(NoValue::new());
    /// storage.insert_node(c, 3).unwrap();
    ///
    /// # let ab = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(ab, 4, &a, &b).unwrap();
    /// # let ca = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(ca, 5, &c, &a).unwrap();
    ///
    /// assert_eq!(storage.node_degree(&a), 2);
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// The default implementation simply calls [`Self::node_connections`] and counts the number of
    /// edges.
    /// This is unlikely to be the most efficient implementation, so custom implementations should
    /// override this.
    fn node_degree(&self, id: Self::NodeId) -> usize {
        self.node_connections(id).count()
    }

    /// Returns an iterator over all nodes that are connected to the given node.
    ///
    /// This will return an iterator over all nodes that are connected to the given node.
    /// This includes all nodes, meaning that for a directed graph both the incoming and outgoing
    /// edges are taken into account.
    ///
    /// If the graph allows for parallel edges, the same node **SHOULD NOT** be returned multiple
    /// times, if an implementation allows for self-loops, the given node may also be returned.
    ///
    /// The results won't be in any particular order, any order should be considered an
    /// implementation detail of the storage implementation.
    ///
    /// > **Note**: For more information of **SHOULD NOT** visit [RFC 2119](https://www.ietf.org/rfc/rfc2119.txt).
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
    /// #
    /// # let a = storage.next_node_id(NoValue::new());
    /// storage.insert_node(a, 1).unwrap();
    /// # let b = storage.next_node_id(NoValue::new());
    /// storage.insert_node(b, 2).unwrap();
    /// # let c = storage.next_node_id(NoValue::new());
    /// storage.insert_node(c, 3).unwrap();
    ///
    /// # let ab = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(ab, 4, &a, &b).unwrap();
    /// # let ca = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(ca, 5, &c, &a).unwrap();
    ///
    /// assert_eq!(
    ///     storage
    ///         .node_neighbours(&a)
    ///         .map(|node| *node.id())
    ///         .collect::<Vec<_>>(),
    ///     [b, c]
    /// );
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// Implementations should try to provide a more efficient implementation than the default one,
    /// and must uphold the contract that the returned iterator does not contain duplicates.
    ///
    /// Due to it's allocation free nature, the default implementation currently returns an iterator
    /// that may contain duplicates and is based on [`Self::node_connections`].
    ///
    /// Changing the requirement from **SHOULD NOT** to **MUST NOT** may occur in the future, and is
    /// to be considered a breaking change.
    fn node_neighbours(&self, id: Self::NodeId) -> impl Iterator<Item = Node<'_, Self>> {
        self.node_connections(id)
            .filter_map(move |edge: Edge<Self>| {
                let (u, v) = edge.endpoint_ids();

                // doing it this way allows us to also get ourselves as a neighbour if we have a
                // self-loop
                if u == id { self.node(v) } else { self.node(u) }
            })
    }

    /// Returns an iterator over all nodes that are connected to the given node, with mutable
    /// weights.
    ///
    /// This will return an iterator over all nodes that are connected to the given node.
    /// This includes all nodes, meaning that for a directed graph both the incoming and outgoing
    /// edges are taken into account.
    ///
    /// If the graph allows for parallel edges, the same node **MUST NOT** be returned multiple
    /// times.
    ///
    /// > **Note**: For more information of **MUST NOT** visit [RFC 2119](https://www.ietf.org/rfc/rfc2119.txt).
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
    /// #
    /// # let a = storage.next_node_id(NoValue::new());
    /// storage.insert_node(a, 1).unwrap();
    /// # let b = storage.next_node_id(NoValue::new());
    /// storage.insert_node(b, 2).unwrap();
    /// # let c = storage.next_node_id(NoValue::new());
    /// storage.insert_node(c, 3).unwrap();
    ///
    /// # let ab = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(ab, 4, &a, &b).unwrap();
    /// # let ca = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(ca, 5, &c, &a).unwrap();
    ///
    /// for mut node in storage.node_neighbours_mut(&a) {
    ///     *node.weight_mut() += 1;
    /// }
    ///
    /// assert_eq!(
    ///     storage
    ///         .node_neighbours(&a)
    ///         .map(|node| *node.weight())
    ///         .collect::<Vec<_>>(),
    ///     [3, 4]
    /// );
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// No default implementation is provided, as a mutable iterator based on
    /// [`Self::node_connections_mut`] could potentially lead to a double mutable borrow.
    fn node_neighbours_mut(&mut self, id: Self::NodeId) -> impl Iterator<Item = NodeMut<'_, Self>>;

    /// Returns an iterator over all nodes that do not have any edges connected to them.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
    /// #
    /// # let a = storage.next_node_id(NoValue::new());
    /// storage.insert_node(a, 1).unwrap();
    /// # let b = storage.next_node_id(NoValue::new());
    /// storage.insert_node(b, 2).unwrap();
    /// # let c = storage.next_node_id(NoValue::new());
    /// storage.insert_node(c, 3).unwrap();
    ///
    /// # let ab = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(ab, 4, &a, &b).unwrap();
    ///
    /// let isolated_nodes: Vec<_> = storage.isolated_nodes().map(|node| *node.id()).collect();
    ///
    /// assert_eq!(isolated_nodes, [c]);
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// The default implementation is based on [`Self::nodes`], and filters out all nodes that have
    /// no edges via [`Self::node_neighbours`].
    /// This is quite inefficient, and implementations should try to provide a more efficient
    /// implementation if possible.
    fn isolated_nodes(&self) -> impl Iterator<Item = Node<Self>> {
        self.nodes().filter(|node| self.node_degree(node.id()) == 0)
    }

    /// Returns an iterator over all nodes that do not have any edges connected to them, with
    /// mutable weights.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
    /// #
    /// # let a = storage.next_node_id(NoValue::new());
    /// storage.insert_node(a, 1).unwrap();
    /// # let b = storage.next_node_id(NoValue::new());
    /// storage.insert_node(b, 2).unwrap();
    /// # let c = storage.next_node_id(NoValue::new());
    /// storage.insert_node(c, 3).unwrap();
    ///
    /// # let ab = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(ab, 4, &a, &b).unwrap();
    ///
    /// for mut external in storage.isolated_nodes_mut() {
    ///     *external.weight_mut() += 1;
    /// }
    ///
    /// assert_eq!(storage.node(&c).unwrap().weight(), &4);
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// No default implementation is provided, as a mutable iterator based on [`Self::nodes_mut`]
    /// and [`Self::node_neighbours`] would lead to a mutable borrow of the storage implementation
    /// followed by a shared borrow, which is not allowed.
    fn isolated_nodes_mut(&mut self) -> impl Iterator<Item = NodeMut<Self>>;

    /// Returns an iterator over all nodes in the graph.
    ///
    /// This **MUST** return all nodes in the graph, and **MUST NOT** return the same node multiple
    /// times.
    ///
    /// > **Note**: For more information of **MUST** and **MUST NOT** visit [RFC 2119](https://www.ietf.org/rfc/rfc2119.txt).
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
    /// #
    /// # let a = storage.next_node_id(NoValue::new());
    /// storage.insert_node(a, 1).unwrap();
    /// # let b = storage.next_node_id(NoValue::new());
    /// storage.insert_node(b, 2).unwrap();
    /// # let c = storage.next_node_id(NoValue::new());
    /// storage.insert_node(c, 3).unwrap();
    ///
    /// assert_eq!(
    ///     storage.nodes().map(|node| *node.id()).collect::<Vec<_>>(),
    ///     [a, b, c]
    /// );
    /// ```
    fn nodes(&self) -> impl Iterator<Item = Node<Self>>;

    /// Returns an iterator over all nodes in the graph, with mutable weights.
    ///
    /// This **MUST** return all nodes in the graph, and **MUST NOT** return the same node multiple
    /// times.
    ///
    /// > **Note**: For more information of **MUST** and **MUST NOT** visit [RFC 2119](https://www.ietf.org/rfc/rfc2119.txt).
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
    /// #
    /// # let a = storage.next_node_id(NoValue::new());
    /// storage.insert_node(a, 1).unwrap();
    /// # let b = storage.next_node_id(NoValue::new());
    /// storage.insert_node(b, 2).unwrap();
    /// # let c = storage.next_node_id(NoValue::new());
    /// storage.insert_node(c, 3).unwrap();
    ///
    /// for mut node in storage.nodes_mut() {
    ///     *node.weight_mut() += 1;
    /// }
    ///
    /// assert_eq!(
    ///     storage
    ///         .nodes()
    ///         .map(|node| *node.weight())
    ///         .collect::<Vec<_>>(),
    ///     [2, 3, 4]
    /// );
    /// ```
    fn nodes_mut(&mut self) -> impl Iterator<Item = NodeMut<Self>>;

    /// Returns an iterator over all edges in the graph.
    ///
    /// This **MUST** return all edges in the graph, and **MUST NOT** return the same edge multiple
    /// times.
    ///
    /// > **Note**: For more information of **MUST** and **MUST NOT** visit [RFC 2119](https://www.ietf.org/rfc/rfc2119.txt).
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
    /// #
    /// # let a = storage.next_node_id(NoValue::new());
    /// storage.insert_node(a, 1).unwrap();
    /// # let b = storage.next_node_id(NoValue::new());
    /// storage.insert_node(b, 2).unwrap();
    /// # let c = storage.next_node_id(NoValue::new());
    /// storage.insert_node(c, 3).unwrap();
    ///
    /// # let ab = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(ab, 4, &a, &b).unwrap();
    /// # let ca = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(ca, 5, &c, &a).unwrap();
    ///
    /// assert_eq!(
    ///     storage.edges().map(|edge| *edge.id()).collect::<Vec<_>>(),
    ///     [ab, ca]
    /// );
    /// ```
    fn edges(&self) -> impl Iterator<Item = Edge<Self>>;

    /// Returns an iterator over all edges in the graph, with mutable weights.
    ///
    /// This **MUST** return all edges in the graph, and **MUST NOT** return the same edge multiple
    /// times.
    ///
    /// > **Note**: For more information of **MUST** and **MUST NOT** visit [RFC 2119](https://www.ietf.org/rfc/rfc2119.txt).
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
    /// #
    /// # let a = storage.next_node_id(NoValue::new());
    /// storage.insert_node(a, 1).unwrap();
    /// # let b = storage.next_node_id(NoValue::new());
    /// storage.insert_node(b, 2).unwrap();
    /// # let c = storage.next_node_id(NoValue::new());
    /// storage.insert_node(c, 3).unwrap();
    ///
    /// # let ab = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(ab, 4, &a, &b).unwrap();
    /// # let ca = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(ca, 5, &c, &a).unwrap();
    ///
    /// for mut edge in storage.edges_mut() {
    ///     *edge.weight_mut() += 1;
    /// }
    ///
    /// assert_eq!(
    ///     storage
    ///         .edges()
    ///         .map(|edge| *edge.weight())
    ///         .collect::<Vec<_>>(),
    ///     [5, 6]
    /// );
    /// ```
    fn edges_mut(&mut self) -> impl Iterator<Item = EdgeMut<Self>>;

    /// Reserves capacity for at least `additional_nodes` nodes and `additional_edges` edges.
    ///
    /// This will reserve capacity for at least `additional_nodes` nodes and `additional_edges`, but
    /// may reserve more.
    /// This function **MUST NOT** change the number of nodes or edges in the graph.
    /// A storage implementation **MAY** decide to not reserve any additional capacity.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
    ///
    /// storage.reserve(16, 16);
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// The default implementation calls [`Self::reserve_nodes`] and [`Self::reserve_edges`].
    fn reserve(&mut self, additional_nodes: usize, additional_edges: usize) {
        self.reserve_nodes(additional_nodes);
        self.reserve_edges(additional_edges);
    }

    /// Reserves capacity for at least `additional_nodes` nodes.
    ///
    /// This will reserve capacity for at least `additional_nodes` nodes, but may reserve more.
    /// This function **MUST NOT** change the number of nodes in the graph.
    /// A storage implementation **MAY** decide to not reserve any additional capacity.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
    ///
    /// storage.reserve_nodes(16);
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// The default implementation does not reserve any additional capacity, and is simply empty.
    /// Implementations that wish to support resizing should override this.
    #[allow(unused_variables)]
    fn reserve_nodes(&mut self, additional: usize) {}

    /// Reserves capacity for at least `additional_edges` edges.
    ///
    /// This will reserve capacity for at least `additional_edges` edges, but may reserve more.
    /// This function **MUST NOT** change the number of edges in the graph.
    /// A storage implementation **MAY** decide to not reserve any additional capacity.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
    ///
    /// storage.reserve_edges(16);
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// The default implementation does not reserve any additional capacity, and is simply empty.
    /// Implementations that wish to support resizing should override this.
    #[allow(unused_variables)]
    fn reserve_edges(&mut self, additional: usize) {}

    /// Reserves the minimum capacity for exactly `additional_nodes` nodes and `additional_edges`
    ///
    /// This will reserve the minimum capacity for exactly `additional_nodes` nodes and
    /// `additional_edges` edges.
    ///
    /// This function **MUST NOT** change the number of nodes or edges in the graph.
    /// A storage implementation **MAY** decide to not reserve any additional capacity.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
    ///
    /// storage.reserve_exact(16, 16);
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// The default implementation calls [`Self::reserve_exact_nodes`] and
    /// [`Self::reserve_exact_edges`].
    fn reserve_exact(&mut self, additional_nodes: usize, additional_edges: usize) {
        self.reserve_exact_nodes(additional_nodes);
        self.reserve_exact_edges(additional_edges);
    }

    /// Reserves the minimum capacity for exactly `additional_nodes` nodes.
    ///
    /// This will reserve the minimum capacity for exactly `additional_nodes` nodes.
    ///
    /// This function **MUST NOT** change the number of nodes in the graph.
    /// A storage implementation **MAY** decide to not reserve any additional capacity.
    ///
    /// The implementation may try to not deliberately over-allocate, but this is not required,
    /// should frequent insertions be expected prefer [`Self::reserve_nodes`].
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
    ///
    /// storage.reserve_exact_nodes(16);
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// The default implementation forwards to [`Self::reserve_nodes`].
    fn reserve_exact_nodes(&mut self, additional: usize) {
        self.reserve_nodes(additional);
    }

    /// Reserves the minimum capacity for exactly `additional_edges` edges.
    ///
    /// This will reserve the minimum capacity for exactly `additional_edges` edges.
    ///
    /// This function **MUST NOT** change the number of edges in the graph.
    /// A storage implementation **MAY** decide to not reserve any additional capacity.
    ///
    /// The implementation may try to not deliberately over-allocate, but this is not required,
    /// should frequent insertions be expected prefer [`Self::reserve_edges`].
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
    ///
    /// storage.reserve_exact_edges(16);
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// The default implementation forwards to [`Self::reserve_edges`].
    fn reserve_exact_edges(&mut self, additional: usize) {
        self.reserve_edges(additional);
    }

    /// Shrinks the capacity of the storage as much as possible.
    ///
    /// This will shrink the capacity of the storage as much as possible.
    ///
    /// This function **MUST NOT** change the number of nodes or edges in the graph.
    /// A storage implementation **MAY** decide to not shrink the capacity at all.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
    ///
    /// storage.reserve(16, 16);
    /// storage.shrink_to_fit();
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// The default implementation calls [`Self::shrink_to_fit_nodes`] and
    /// [`Self::shrink_to_fit_edges`].
    fn shrink_to_fit(&mut self) {
        self.shrink_to_fit_nodes();
        self.shrink_to_fit_edges();
    }

    /// Shrinks the capacity of the node storage as much as possible.
    ///
    /// This will shrink the capacity of the node storage as much as possible.
    ///
    /// This function **MUST NOT** change the number of nodes in the graph.
    /// A storage implementation **MAY** decide to not shrink the capacity at all.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
    ///
    /// storage.reserve_nodes(16);
    /// storage.shrink_to_fit_nodes();
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// The default implementation does not shrink the capacity at all and is effectively a no-op.
    /// Implementations that wish to support resizing should override this.
    fn shrink_to_fit_nodes(&mut self) {}

    /// Shrinks the capacity of the edge storage as much as possible.
    ///
    /// This will shrink the capacity of the edge storage as much as possible.
    ///
    /// This function **MUST NOT** change the number of edges in the graph.
    /// A storage implementation **MAY** decide to not shrink the capacity at all.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
    ///
    /// storage.reserve_edges(16);
    /// storage.shrink_to_fit_edges();
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// The default implementation does not shrink the capacity at all and is effectively a no-op.
    /// Implementations that wish to support resizing should override this.
    fn shrink_to_fit_edges(&mut self) {}
}
