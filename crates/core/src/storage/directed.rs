use crate::{
    edge::{Direction, Edge, EdgeMut},
    node::{Node, NodeMut},
    storage::GraphStorage,
};

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
    fn directed_edges_between<'a: 'b, 'b>(
        &'a self,
        source: &'b Self::NodeId,
        target: &'b Self::NodeId,
    ) -> impl Iterator<Item = Edge<'a, Self>> + 'b {
        self.node_directed_connections(source, Direction::Outgoing)
            .filter(move |edge| edge.target_id() == target)
    }

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
    fn directed_edges_between_mut<'a: 'b, 'b>(
        &'a mut self,
        source: &'b Self::NodeId,
        target: &'b Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'a, Self>> + 'b {
        self.node_directed_connections_mut(source, Direction::Outgoing)
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
    ///     edge::{marker::Directed, Direction},
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
    fn node_directed_connections<'a: 'b, 'b>(
        &'a self,
        id: &'b Self::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = Edge<'a, Self>> + 'b;

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
    ///     edge::{marker::Directed, Direction},
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
    fn node_directed_connections_mut<'a: 'b, 'b>(
        &'a mut self,
        id: &'b Self::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = EdgeMut<'a, Self>> + 'b;

    /// Returns the number of directed edges that are connected to the given node, by the given
    /// direction.
    ///
    /// This is also known as the either outdegree (if `direction` is [`Direction::Outgoing`]) 𝛿+(v)
    /// or indegree (if `direction` is [`Direction::Incoming`]) 𝛿-(v) of a node.
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{
    ///     edge::{marker::Directed, Direction},
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
    fn node_directed_degree(&self, id: &Self::NodeId, direction: Direction) -> usize {
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
    ///     edge::{marker::Directed, Direction},
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
    fn node_directed_neighbours<'a: 'b, 'b>(
        &'a self,
        id: &'b Self::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = Node<'a, Self>> + 'b {
        self.node_directed_connections(id, direction)
            .map(move |edge| match direction {
                Direction::Outgoing => edge.target(),
                Direction::Incoming => edge.source(),
            })
    }

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
    ///     edge::{marker::Directed, Direction},
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
    fn node_directed_neighbours_mut<'a: 'b, 'b>(
        &'a mut self,
        id: &'b Self::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = NodeMut<'a, Self>> + 'b;
}
