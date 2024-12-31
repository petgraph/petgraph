use super::r#mut::GraphMut;
use crate::{edge::EdgeMut, graph::Graph, node::NodeMut};

// TODO: this shouldn't be a trait? I think
/// Graph storage, which can retain nodes and edges based on a predicate.
pub trait GraphRetain: GraphMut {
    /// Retains all nodes and edges for which the predicate returns `true`.
    ///
    /// There are no guarantees about the order in which the predicate is called.
    /// This also means that calls to the two different closures could be interspersed with each
    /// other.
    ///
    /// Edges connected to a node which wasn't retained, all edges connected to it will be returned
    /// as well.
    /// The `edges` function may still (but doesn't need to be!) be called on those edges, even
    /// though they will be removed anyway, ignoring their return value.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{
    ///     edge::marker::Directed,
    ///     storage::{GraphStorage, RetainableGraphStorage},
    /// };
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<_, _, Directed>::new();
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
    /// For every node the predicate is called, if the predicate returns `true`, the node is
    /// retained, otherwise the node and all connected edges will be removed.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{
    ///     edge::marker::Directed,
    ///     storage::{GraphStorage, RetainableGraphStorage},
    /// };
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<_, _, Directed>::new();
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
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<_, _, Directed>::new();
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
