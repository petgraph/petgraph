use super::{DirectedGraphStorage, GraphStorage};
use crate::{
    DetachedEdge, DetachedNode, EdgeMut, NodeMut,
    edge::{Direction, EdgeId},
    node::NodeId,
};

pub trait GraphStorageMut: GraphStorage {
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
    fn insert_node(&mut self, weight: Self::NodeWeight) -> Result<NodeMut<Self>, Self::Error> {
        self.insert_node_with(|_| weight)
    }

    fn insert_node_with(
        &mut self,
        weight: impl FnOnce(NodeId) -> Self::NodeWeight,
    ) -> Result<NodeMut<Self>, Self::Error>;

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
        source: NodeId,
        target: NodeId,

        weight: Self::EdgeWeight,
    ) -> Result<EdgeMut<Self>, Self::Error> {
        self.insert_edge_with(source, target, |_| weight)
    }

    fn insert_edge_with(
        &mut self,
        source: NodeId,
        target: NodeId,

        weight: impl FnOnce(EdgeId) -> Self::EdgeWeight,
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
    fn remove_node(&mut self, id: NodeId) -> Option<DetachedNode<Self::NodeWeight>>;

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
    fn remove_edge(&mut self, id: EdgeId) -> Option<DetachedEdge<Self::EdgeWeight>>;

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
    fn node_mut(&mut self, id: NodeId) -> Option<NodeMut<Self>>;

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
    fn edge_mut(&mut self, id: EdgeId) -> Option<EdgeMut<Self>>;

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
        u: NodeId,
        v: NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>>;

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
    fn node_connections_mut(&mut self, id: NodeId) -> impl Iterator<Item = EdgeMut<'_, Self>>;

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
    fn node_neighbours_mut(&mut self, id: NodeId) -> impl Iterator<Item = NodeMut<'_, Self>>;

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

pub trait DirectedGraphStorageMut: DirectedGraphStorage {
    /// Returns an iterator over all directed edges between the source and target node with mutable
    /// weights.
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
    /// for mut edge in storage.directed_edges_between_mut(&a, &b) {
    ///     *edge.weight_mut() += 1;
    /// }
    ///
    /// assert_eq!(
    ///     storage
    ///         .edges_between(&a, &b)
    ///         .map(|mut edge| *edge.weight())
    ///         .collect::<Vec<_>>(),
    ///     [4, 4]
    /// );
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// This method is implemented by calling [`Self::node_directed_connections_mut`] and filtering
    /// the edges by their target.
    /// Most implementations should be able to provide a more efficient implementation.
    fn directed_edges_between_mut(
        &mut self,
        source: NodeId,
        target: NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.node_directed_connections_mut(source, Direction::Outgoing)
            .filter(move |edge| edge.target_id() == target)
    }

    /// Returns an iterator over all directed edges that are connected to the given node, by the
    /// given direction with mutable weights.
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
    /// for mut edge in storage.node_directed_connections_mut(&a, Direction::Outgoing) {
    ///     *edge.weight_mut() += 1;
    /// }
    ///
    /// assert_eq!(
    ///     storage
    ///         .node_connections(&a)
    ///         .map(|mut edge| *edge.weight())
    ///         .collect::<Vec<_>>(),
    ///     [5, 5]
    /// );
    /// ```
    fn node_directed_connections_mut(
        &mut self,
        id: NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>>;

    /// Returns an iterator over all nodes that are connected to the given node, by the given
    /// direction with mutable weights.
    ///
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
    /// for mut node in storage.node_directed_neighbours_mut(&a, Direction::Outgoing) {
    ///     *node.weight_mut() += 1;
    /// }
    ///
    /// assert_eq!(
    ///     storage
    ///         .node_neighbours(&a)
    ///         .map(|node| *node.weight())
    ///         .collect::<Vec<_>>(),
    ///     [3, 3]
    /// );
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// No default implementation is provided, as a mutable iterator based on
    /// [`Self::node_directed_connections_mut`] could potentially lead to a double mutable borrow.
    // I'd love to provide a default implementation for this, but I just can't get it to work.
    fn node_directed_neighbours_mut(
        &mut self,
        id: NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = NodeMut<'_, Self>>;
}
