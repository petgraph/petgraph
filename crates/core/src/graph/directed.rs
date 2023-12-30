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
    pub fn directed_edges_between(
        &self,
        source: S::NodeId,
        target: S::NodeId,
    ) -> impl Iterator<Item = Edge<'_, S>> {
        self.storage.directed_edges_between(source, target)
    }

    /// Returns an iterator over all edges, with mutable weights, between the source and target
    /// node.
    ///
    /// The direction of the edges is always `source → target`.
    ///
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
    pub fn directed_edges_between_mut(
        &mut self,
        source: S::NodeId,
        target: S::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, S>> {
        self.storage.directed_edges_between_mut(source, target)
    }

    /// Returns an iterator over all neighbours of the given node in a specific direction.
    ///
    /// If the direction is `Outgoing`, the neighbours are all nodes that are reachable from the id
    /// are returned, if the direction is `Incoming`, all nodes that can reach the id are returned.
    ///
    /// If there is a self-loop between the node and itself, it will be returned regardless of the
    /// direction.
    ///
    /// This is an alias for [`Self::neighbours_directed`], due to spelling differences between
    /// American and British English.
    #[inline]
    pub fn neighbors_directed(
        &self,
        id: S::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = Node<'_, S>> {
        self.neighbours_directed(id, direction)
    }

    /// Returns an iterator over all neighbours of the given node in a specific direction.
    ///
    /// If the direction is `Outgoing`, the neighbours are all nodes that are reachable from the id
    /// are returned, if the direction is `Incoming`, all nodes that can reach the id are returned.
    ///
    /// If there is a self-loop between the node and itself, it will be returned regardless of the
    /// direction.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// use petgraph_core::edge::Direction;
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
    ///         .neighbours_directed(&b, Direction::Outgoing)
    ///         .map(|node| *node.id())
    ///         .collect::<HashSet<_>>(),
    ///     [a, c].into_iter().collect::<HashSet<_>>()
    /// );
    ///
    /// assert_eq!(
    ///     graph
    ///         .neighbours_directed(&b, Direction::Incoming)
    ///         .map(|node| *node.id())
    ///         .collect::<HashSet<_>>(),
    ///     [a].into_iter().collect::<HashSet<_>>()
    /// );
    /// ```
    pub fn neighbours_directed(
        &self,
        id: S::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = Node<'_, S>> {
        self.storage.node_directed_neighbours(id, direction)
    }

    /// Returns an iterator over all neighbours, with mutable weights, of the given node in a
    /// specific direction.
    ///
    /// If the direction is `Outgoing`, the neighbours are all nodes that are reachable from the id
    /// are returned, if the direction is `Incoming`, all nodes that can reach the id are returned.
    ///
    /// If there is a self-loop between the node and itself, it will be returned regardless of the
    /// direction.
    ///
    /// This is an alias for [`Self::neighbours_directed_mut`], due to spelling differences between
    /// American and British English.
    #[inline]
    pub fn neighbors_directed_mut(
        &mut self,
        id: S::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = NodeMut<'_, S>> {
        self.neighbours_directed_mut(id, direction)
    }

    /// Returns an iterator over all neighbours, with mutable weights, of the given node in a
    /// specific direction.
    ///
    /// If the direction is `Outgoing`, the neighbours are all nodes that are reachable from the id
    /// are returned, if the direction is `Incoming`, all nodes that can reach the id are returned.
    ///
    /// If there is a self-loop between the node and itself, it will be returned regardless of the
    /// direction.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// use petgraph_core::edge::Direction;
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
    /// for mut node in graph.neighbours_directed_mut(&b, Direction::Outgoing) {
    ///     *node.weight_mut() += 1;
    /// }
    ///
    /// assert_eq!(
    ///     graph
    ///         .nodes()
    ///         .map(|node| (*node.id(), *node.weight()))
    ///         .collect::<HashSet<_>>(),
    ///     [(a, 1), (b, 1), (c, 3)].into_iter().collect::<HashSet<_>>()
    /// );
    /// ```
    pub fn neighbours_directed_mut(
        &mut self,
        id: S::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = NodeMut<'_, S>> {
        self.storage.node_directed_neighbours_mut(id, direction)
    }

    /// Returns an iterator over all edges, where one of the endpoints is the given node.
    ///
    /// If the direction is `Outgoing`, the edges are all edges where the node is the source, if the
    /// direction is `Incoming`, all edges where the node is the target are returned.
    ///
    /// If there is a self-loop between the node and itself, it will be returned regardless of the
    /// direction.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// use petgraph_core::edge::Direction;
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
    ///         .connections_directed(&b, Direction::Outgoing)
    ///         .map(|edge| (*edge.id(), *edge.weight()))
    ///         .collect::<HashSet<_>>(),
    ///     [(ba, u8::MAX - 1), (bc, u8::MAX - 2)]
    ///         .into_iter()
    ///         .collect::<HashSet<_>>()
    /// );
    /// ```
    pub fn connections_directed(
        &self,
        id: S::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = Edge<'_, S>> {
        self.storage.node_directed_connections(id, direction)
    }

    /// Returns an iterator over all edges, with mutable weights, where one of the endpoints is the
    /// given node.
    ///
    /// If the direction is `Outgoing`, the edges are all edges where the node is the source, if the
    /// direction is `Incoming`, all edges where the node is the target are returned.
    ///
    /// If there is a self-loop between the node and itself, it will be returned regardless of the
    /// direction.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// use petgraph_core::edge::Direction;
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
    /// for mut edge in graph.connections_directed_mut(&b, Direction::Outgoing) {
    ///     *edge.weight_mut() -= 16;
    /// }
    ///
    /// assert_eq!(
    ///     graph
    ///         .edges()
    ///         .map(|edge| (*edge.id(), *edge.weight()))
    ///         .collect::<HashSet<_>>(),
    ///     [(ab, u8::MAX), (ba, u8::MAX - 17), (bc, u8::MAX - 18)]
    ///         .into_iter()
    ///         .collect::<HashSet<_>>()
    /// );
    /// ```
    pub fn connections_directed_mut(
        &mut self,
        id: S::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = EdgeMut<'_, S>> {
        self.storage.node_directed_connections_mut(id, direction)
    }

    // TODO: move into operators and then extension trait via `GraphOperators`
    // pub fn reverse(self) -> Result<Self, S::Error> {
    //     let (nodes, edges) = self.storage.into_parts();
    //
    //     let edges = edges.map(|mut edge| {
    //         let source = edge.u;
    //         let target = edge.v;
    //
    //         edge.u = target;
    //         edge.v = source;
    //
    //         edge
    //     });
    //
    //     Self::from_parts(nodes, edges)
    // }

    // These should go into extensions:
    // into_undirected, into_directed, from_edges, extend_with_edges
}
