mod directed;
mod linear;
mod resize;
mod retain;

use error_stack::{Context, Result};

pub use self::{
    directed::DirectedGraphStorage,
    linear::{LinearGraphStorage, LinearIndexLookup},
    resize::ResizableGraphStorage,
    retain::RetainableGraphStorage,
};
use crate::{
    edge::{DetachedEdge, Edge, EdgeMut},
    id::GraphId,
    node::{DetachedNode, Node, NodeMut},
};

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
    // TODO: example
    fn with_capacity(node_capacity: Option<usize>, edge_capacity: Option<usize>) -> Self;

    /// Create a new graph storage from the given nodes and edges.
    ///
    /// This takes a list of nodes and edges, and tries to create a graph from them. This is the
    /// reverse operation of [`Self::into_parts`], which converts the current graph storage into an
    /// iterable of nodes and edges.
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
    // TODO: example
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
    /// graph storage using [`Self::from_parts`] will result in a graph storage
    /// that is equal to the current graph storage.
    // TODO: example
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
    /// # Implementation Notes
    ///
    /// This is a default implementation, which uses [`Self::nodes`].`count()`, custom
    /// implementations, should if possible, override this.
    // TODO: example
    fn num_nodes(&self) -> usize {
        self.nodes().count()
    }

    /// Returns the number of edges in the graph.
    ///
    /// This is equivalent to [`Self::edges`].`count()`.
    ///
    /// # Implementation Notes
    ///
    /// This is a default implementation, which uses [`Self::edges`].`count()`, custom
    /// implementations, should if possible, override this.
    // TODO: example
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
    // TODO: example
    fn next_node_id(&self, attribute: <Self::NodeId as GraphId>::AttributeIndex) -> Self::NodeId;

    /// Inserts a new node into the graph.
    ///
    /// # Errors
    ///
    /// Returns an error if a node with the given identifier already exists, or if any of the
    /// invariants (depending on the implementation) are violated.
    // TODO: example
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
    /// # Implementation Notes
    ///
    /// All the same notes as [`Self::next_node_id`] apply here as well.
    // TODO: example
    fn next_edge_id(&self, attribute: <Self::EdgeId as GraphId>::AttributeIndex) -> Self::EdgeId;

    /// Inserts a new edge into the graph.
    ///
    /// # Errors
    ///
    /// Returns an error if parallel edges are not allowed, or any of the invariants (depending on
    /// the implementation) are violated.
    /// These invariants _may_ include that an edge between the source and target already exist, but
    /// some implementations may choose to allow parallel edges.
    // TODO: example
    fn insert_edge(
        &mut self,
        id: Self::EdgeId,
        weight: Self::EdgeWeight,

        source: &Self::NodeId,
        target: &Self::NodeId,
    ) -> Result<EdgeMut<Self>, Self::Error>;

    /// Removes the node with the given identifier from the graph.
    ///
    /// Returns the removed node if it existed.
    // TODO: example
    fn remove_node(
        &mut self,
        id: &Self::NodeId,
    ) -> Option<DetachedNode<Self::NodeId, Self::NodeWeight>>;

    /// Removes the edge with the given identifier from the graph.
    ///
    /// Returns the removed edge if it existed.
    // TODO: example
    fn remove_edge(
        &mut self,
        id: &Self::EdgeId,
    ) -> Option<DetachedEdge<Self::EdgeId, Self::NodeId, Self::EdgeWeight>>;

    fn clear(&mut self) -> Result<(), Self::Error>;

    fn node(&self, id: &Self::NodeId) -> Option<Node<Self>>;
    fn node_mut(&mut self, id: &Self::NodeId) -> Option<NodeMut<Self>>;

    fn contains_node(&self, id: &Self::NodeId) -> bool {
        self.node(id).is_some()
    }

    fn edge(&self, id: &Self::EdgeId) -> Option<Edge<Self>>;
    fn edge_mut(&mut self, id: &Self::EdgeId) -> Option<EdgeMut<Self>>;

    fn contains_edge(&self, id: &Self::EdgeId) -> bool {
        self.edge(id).is_some()
    }

    fn find_undirected_edges<'a: 'b, 'b>(
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

    // TODO: do we want to provide a `find_undirected_edges_mut`?

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
}
