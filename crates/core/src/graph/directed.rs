use error_stack::Result;

use crate::{
    edge::{Direction, Edge, EdgeMut},
    graph::Graph,
    node::{Node, NodeMut},
    storage::DirectedGraphStorage,
};

impl<S> Graph<S>
where
    S: DirectedGraphStorage,
{
    /// Returns an iterator over all edges between the source and target node.
    ///
    /// The direction of the edges is always `source → target`.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    /// let a = *graph.insert_node(0).id();
    /// let b = *graph.insert_node(1).id();
    /// let c = *graph.insert_node(2).id();
    ///
    /// let ab = *graph.insert_edge(u8::MAX, &a, &b).id();
    /// let ba = *graph.insert_edge(u8::MAX - 1, &b, &a).id();
    /// let bc = *graph.insert_edge(u8::MAX - 2, &b, &c).id();
    ///
    /// assert_eq!(
    ///     graph
    ///         .directed_edges_between(&a, &b)
    ///         .map(|edge| (*edge.id(), *edge.weight()))
    ///         .collect::<HashSet<_>>(),
    ///     [(ab, u8::MAX)].into_iter().collect::<HashSet<_>>()
    /// );
    ///
    /// assert_eq!(
    ///     graph
    ///         .directed_edges_between(&b, &a)
    ///         .map(|edge| (*edge.id(), *edge.weight()))
    ///         .collect::<HashSet<_>>(),
    ///     [(ba, u8::MAX - 1)].into_iter().collect::<HashSet<_>>()
    /// );
    /// ```
    pub fn directed_edges_between<'a: 'b, 'b>(
        &'a self,
        source: &'b S::NodeId,
        target: &'b S::NodeId,
    ) -> impl Iterator<Item = Edge<'a, S>> + 'b {
        self.storage.directed_edges_between(source, target)
    }

    /// Returns an iterator over all edges, with mutable weights, between the source and target
    /// node.
    ///
    /// The direction of the edges is always `source → target`.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    /// let a = *graph.insert_node(0).id();
    /// let b = *graph.insert_node(1).id();
    /// let c = *graph.insert_node(2).id();
    ///
    /// let ab = *graph.insert_edge(u8::MAX, &a, &b).id();
    /// let ba = *graph.insert_edge(u8::MAX - 1, &b, &a).id();
    /// let bc = *graph.insert_edge(u8::MAX - 2, &b, &c).id();
    ///
    /// for mut edge in graph.directed_edges_between_mut(&a, &b) {
    ///     *edge.weight_mut() -= 16;
    /// }
    ///
    /// assert_eq!(
    ///     graph
    ///         .edges()
    ///         .map(|node| (*node.id(), *node.weight()))
    ///         .collect::<HashSet<_>>(),
    ///     [(ab, u8::MAX - 16), (ba, u8::MAX - 1), (bc, u8::MAX - 2)]
    ///         .into_iter()
    ///         .collect::<HashSet<_>>()
    /// );
    /// ```
    pub fn directed_edges_between_mut<'a: 'b, 'b>(
        &'a mut self,
        source: &'b S::NodeId,
        target: &'b S::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'a, S>> + 'b {
        self.storage.directed_edges_between_mut(source, target)
    }

    #[inline]
    pub fn neighbors_directed<'a: 'b, 'b>(
        &'a self,
        id: &'b S::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = Node<'a, S>> + 'b {
        self.neighbours_directed(id, direction)
    }

    pub fn neighbours_directed<'a: 'b, 'b>(
        &'a self,
        id: &'b S::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = Node<'a, S>> + 'b {
        self.storage.node_directed_neighbours(id, direction)
    }

    #[inline]
    pub fn neighbors_directed_mut<'a: 'b, 'b>(
        &'a mut self,
        id: &'b S::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = NodeMut<'a, S>> + 'b {
        self.neighbours_directed_mut(id, direction)
    }

    pub fn neighbours_directed_mut<'a: 'b, 'b>(
        &'a mut self,
        id: &'b S::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = NodeMut<'a, S>> + 'b {
        self.storage.node_directed_neighbours_mut(id, direction)
    }

    pub fn connections_directed<'a: 'b, 'b>(
        &'a self,
        id: &'b S::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = Edge<'a, S>> + 'b {
        self.storage.node_directed_connections(id, direction)
    }

    pub fn connections_directed_mut<'a: 'b, 'b>(
        &'a mut self,
        id: &'b S::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = EdgeMut<'a, S>> + 'b {
        self.storage.node_directed_connections_mut(id, direction)
    }

    // TODO: move into operators and then extension trait via `GraphOperators`
    pub fn reverse(self) -> Result<Self, S::Error> {
        let (nodes, edges) = self.storage.into_parts();

        let edges = edges.map(|mut edge| {
            let source = edge.u;
            let target = edge.v;

            edge.u = target;
            edge.v = source;

            edge
        });

        Self::from_parts(nodes, edges)
    }

    // These should go into extensions:
    // into_undirected, into_directed, from_edges, extend_with_edges
}
