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
//! - [`SequentialGraphStorage`]: A trait for graph storage implementations that allow the mapping
//!   of their internal indices to a set of linear indices.
//!
//! [`GraphStorage`] proposes that [`DirectedGraphStorage`] is simply a specialization of an
//! undirected graph, meaning that the supertrait of [`DirectedGraphStorage`] is also
//! [`GraphStorage`] and that all implementations of [`DirectedGraphStorage`] must also support
//! undirected graph operations.
//!
//! # Implementation Notes
//!
//! * [`RetainableGraphStorage`] is subject to removal during the alpha period.
//! * [`SequentialGraphStorage`] is subject to removal or rename during the alpha period.
//! * [`AuxiliaryGraphStorage`] is subject to removal or rename during the alpha period.
//!
//! [`Graph`]: crate::graph::Graph

pub mod auxiliary;
mod r#mut;
mod parts;
mod prune;
mod reverse;

use core::error::Error;

use error_stack::Context;

pub use self::{
    auxiliary::AuxiliaryGraphStorage,
    parts::{GraphStorageFromParts, GraphStorageIntoParts},
    prune::GraphStoragePrune,
};
use crate::{
    edge::{Direction, Edge, EdgeId},
    node::{Node, NodeId},
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
pub trait GraphStorage {
    /// The weight of an edge.
    ///
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
    type Error: Error;

    /// The weight of a node.
    ///
    /// No constraints are enforced on this type (except that it needs to be `Sized`), but
    /// implementations _may_ choose to enforce additional constraints, or limit the type to a
    /// specific concrete type.
    /// For example a graph that is unable to store any node weights would choose to only support
    /// `()`.
    type NodeWeight;

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
    fn node(&self, id: NodeId) -> Option<Node<Self>>;

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
    fn contains_node(&self, id: NodeId) -> bool {
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
    fn edge(&self, id: EdgeId) -> Option<Edge<Self>>;

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
    fn contains_edge(&self, id: EdgeId) -> bool {
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
    fn edges_between(&self, u: NodeId, v: NodeId) -> impl Iterator<Item = Edge<'_, Self>> {
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
    fn node_connections(&self, id: NodeId) -> impl Iterator<Item = Edge<'_, Self>>;

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
    fn node_degree(&self, id: NodeId) -> usize {
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
    fn node_neighbours(&self, id: NodeId) -> impl Iterator<Item = Node<'_, Self>> {
        self.node_connections(id)
            .filter_map(move |edge: Edge<Self>| {
                let (u, v) = edge.endpoint_ids();

                // doing it this way allows us to also get ourselves as a neighbour if we have a
                // self-loop
                if u == id { self.node(v) } else { self.node(u) }
            })
    }

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
}

/// A trait for directed graph storage.
///
/// This trait is an extension of [`GraphStorage`] that provides methods for directed graphs.
/// The idea behind this is simple: a directed graph is just an undirected graph with additional
/// directionality.
/// This means that a directed graph can also implement all methods pertaining to an undirected
/// graph, by simply ignoring the directionality.
///
/// This has the benefit that functions stay consistent, and allow for directed, as well as
/// undirected exploration of directed graphs without additional effort, but might incur a
/// performance penalty for undirected exploration, depending on implementation.
///
/// You should never have to directly use this trait, but instead use [`Graph`] which is a thin
/// abstraction around [`GraphStorage`] and [`DirectedGraphStorage`].
///
/// # Example
///
/// ```
/// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
/// use petgraph_dino::DinoStorage;
///
/// let mut storage = DinoStorage::<(), Directed>::new();
/// # assert_eq!(storage.num_nodes(), 0);
/// # assert_eq!(storage.num_edges(), 0);
/// ```
///
/// [`Graph`]: crate::graph::Graph
pub trait DirectedGraphStorage: GraphStorage {
    /// Returns an iterator over all directed edges between the source and target node.
    ///
    /// This will return an iterator over all directed edges between the source and target node,
    /// where the direction is `source -> target`.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{
    ///     edge::marker::Directed,
    ///     storage::{DirectedGraphStorage, GraphStorage},
    /// };
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
    ///         .directed_edges_between(&a, &b)
    ///         .map(|edge| *edge.id())
    ///         .collect::<Vec<_>>(),
    ///     [ab]
    /// );
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// This method is implemented by calling [`Self::node_directed_connections`] and filtering the
    /// edges by their target.
    /// Most implementations should be able to provide a more efficient implementation.
    fn directed_edges_between(
        &self,
        source: NodeId,
        target: NodeId,
    ) -> impl Iterator<Item = Edge<'_, Self>> {
        self.node_directed_connections(source, Direction::Outgoing)
            .filter(move |edge| edge.target_id() == target)
    }

    /// Returns an iterator over all directed edges that are connected to the given node, by the
    /// given direction.
    ///
    /// This will return an iterator over all directed edges that are connected to the given node.
    /// The direction of the edges is determined by the `direction` parameter. If `direction` is
    /// [`Direction::Incoming`] only edges where the given node is the target will be returned, and
    /// if `direction` is [`Direction::Outgoing`] only edges where the given node is the source
    /// will be returned.
    ///
    /// There may be multiple edges between two nodes, if the implementation allows for parallel
    /// edges.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{
    ///     edge::{Direction, marker::Directed},
    ///     storage::{DirectedGraphStorage, GraphStorage},
    /// };
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
    ///         .node_directed_connections(&a, Direction::Outgoing)
    ///         .map(|mut edge| *edge.id())
    ///         .collect::<Vec<_>>(),
    ///     [ab]
    /// );
    ///
    /// assert_eq!(
    ///     storage
    ///         .node_directed_connections(&a, Direction::Incoming)
    ///         .map(|mut edge| *edge.id())
    ///         .collect::<Vec<_>>(),
    ///     [ca]
    /// );
    /// ```
    fn node_directed_connections(
        &self,
        id: NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = Edge<'_, Self>>;

    /// Returns the number of directed edges that are connected to the given node, by the given
    /// direction.
    ///
    /// This is also known as the either outdegree (if `direction` is [`Direction::Outgoing`]) 𝛿+(v)
    /// or indegree (if `direction` is [`Direction::Incoming`]) 𝛿-(v) of a node.
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{
    ///     edge::{Direction, marker::Directed},
    ///     storage::{DirectedGraphStorage, GraphStorage},
    /// };
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
    /// assert_eq!(storage.node_directed_degree(&a, Direction::Outgoing), 1);
    /// assert_eq!(storage.node_directed_degree(&a, Direction::Incoming), 1);
    /// ```
    fn node_directed_degree(&self, id: NodeId, direction: Direction) -> usize {
        self.node_directed_connections(id, direction).count()
    }

    /// Returns an iterator over all nodes that are connected to the given node, by the given
    /// direction.
    ///
    /// This will return an iterator over all nodes that are connected to the given node.
    /// The direction of the edges is determined by the `direction` parameter. If `direction` is
    /// [`Direction::Incoming`] only nodes where the given node of an edge is the target will be
    /// returned, and if it is [`Direction::Outgoing`] only nodes where the given node of an
    /// edge is the source will be returned.
    ///
    /// If the graph allows for parallel edges, the same node **MUST NOT** be returned multiple
    /// times, if an implementation allows for self-loops, the given node may also be returned.
    ///
    /// > **Note**: For more information of **MUST NOT** visit [RFC 2119](https://www.ietf.org/rfc/rfc2119.txt).
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{
    ///     edge::{Direction, marker::Directed},
    ///     storage::{DirectedGraphStorage, GraphStorage},
    /// };
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
    ///         .node_directed_neighbours(&a, Direction::Outgoing)
    ///         .map(|node| *node.id())
    ///         .collect::<Vec<_>>(),
    ///     [b]
    /// );
    ///
    /// assert_eq!(
    ///     storage
    ///         .node_directed_neighbours(&a, Direction::Incoming)
    ///         .map(|node| *node.id())
    ///         .collect::<Vec<_>>(),
    ///     [c]
    /// );
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// Implementations should try to provide a more efficient implementation than the default one,
    /// and must uphold the contract that the returned iterator does not contain duplicates.
    fn node_directed_neighbours(
        &self,
        id: NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = Node<'_, Self>> {
        self.node_directed_connections(id, direction)
            .map(move |edge| match direction {
                Direction::Outgoing => edge.target(),
                Direction::Incoming => edge.source(),
            })
    }
}
