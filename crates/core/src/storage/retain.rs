use crate::{edge::EdgeMut, node::NodeMut, storage::GraphStorage};

/// Graph storage, which can retain nodes and edges based on a predicate.
pub trait RetainableGraphStorage: GraphStorage {
    /// Retains all nodes and edges for which the predicate returns `true`.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{
    ///     edge::marker::Directed,
    ///     storage::{GraphStorage, RetainableGraphStorage},
    /// };
    /// use petgraph_dino::DinosaurStorage;
    ///
    /// let mut storage = DinosaurStorage::<_, _, Directed>::new();
    ///
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
    /// # let bc = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(bc, 5, &b, &c).unwrap();
    /// # let ca = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(ca, 6, &c, &a).unwrap();
    ///
    /// storage.retain(|node| node.weight() % 2 == 1, |edge| edge.weight() % 2 == 0);
    ///
    /// assert_eq!(
    ///     storage.nodes().map(|node| *node.id()).collect::<Vec<_>>(),
    ///     vec![a, c]
    /// );
    /// assert_eq!(
    ///     storage.edges().map(|edge| *edge.id()).collect::<Vec<_>>(),
    ///     vec![ca]
    /// );
    /// ```
    fn retain(
        &mut self,
        mut nodes: impl FnMut(NodeMut<'_, Self>) -> bool,
        mut edges: impl FnMut(EdgeMut<'_, Self>) -> bool,
    ) {
        self.retain_nodes(&mut nodes);
        self.retain_edges(&mut edges);
    }

    /// Retains all nodes for which the predicate returns `true`.
    ///
    /// If you are going to retain edges as well, it is more efficient to use [`Self::retain`].
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{
    ///     edge::marker::Directed,
    ///     storage::{GraphStorage, RetainableGraphStorage},
    /// };
    /// use petgraph_dino::DinosaurStorage;
    ///
    /// let mut storage = DinosaurStorage::<_, _, Directed>::new();
    ///
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
    /// # let bc = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(bc, 5, &b, &c).unwrap();
    /// # let ca = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(ca, 6, &c, &a).unwrap();
    ///
    /// storage.retain_nodes(|node| node.weight() % 2 == 1);
    ///
    /// assert_eq!(
    ///     storage.nodes().map(|node| *node.id()).collect::<Vec<_>>(),
    ///     vec![a, c]
    /// );
    /// assert_eq!(
    ///     storage.edges().map(|edge| *edge.id()).collect::<Vec<_>>(),
    ///     vec![ca]
    /// );
    /// ```
    fn retain_nodes(&mut self, f: impl FnMut(NodeMut<'_, Self>) -> bool);

    /// Retains all edges for which the predicate returns `true`.
    ///
    /// If you are going to retain nodes as well, it is more efficient to use [`Self::retain`].
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{
    ///     edge::marker::Directed,
    ///     storage::{GraphStorage, RetainableGraphStorage},
    /// };
    /// use petgraph_dino::DinosaurStorage;
    ///
    /// let mut storage = DinosaurStorage::<_, _, Directed>::new();
    ///
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
    /// # let bc = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(bc, 5, &b, &c).unwrap();
    /// # let ca = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(ca, 6, &c, &a).unwrap();
    ///
    /// storage.retain_edges(|edge| edge.weight() % 2 == 0);
    ///
    /// assert_eq!(
    ///     storage.nodes().map(|node| *node.id()).collect::<Vec<_>>(),
    ///     vec![a, b, c]
    /// );
    /// assert_eq!(
    ///     storage.edges().map(|edge| *edge.id()).collect::<Vec<_>>(),
    ///     vec![ab, ca]
    /// );
    /// ```
    fn retain_edges(&mut self, f: impl FnMut(EdgeMut<'_, Self>) -> bool);
}
