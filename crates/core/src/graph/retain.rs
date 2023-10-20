use crate::{edge::EdgeMut, graph::Graph, node::NodeMut, storage::RetainableGraphStorage};

impl<S> Graph<S>
where
    S: RetainableGraphStorage,
{
    /// Retains only the nodes and edges specified by the predicate.
    ///
    /// For every node and edge in the graph, the predicate is called with a mutable reference to
    /// the node or edge,
    /// if the predicate returns `false` then the node or edge is removed from the graph.
    ///
    /// The order in which the different functions are called is not guaranteed and may differ
    /// between implementations.
    ///
    /// Should a node be connected, all edges connected to that node will be removed as well, an
    /// implementation may decide to omit these when calling the `edges` closures, or call the
    /// closure for them as well, but ignore the result.
    ///
    /// # Example
    ///
    /// ```
    /// use std::{collections::HashSet, iter::once};
    ///
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node(1).id();
    /// let b = *graph.insert_node(2).id();
    /// let c = *graph.insert_node(3).id();
    ///
    /// let ab = *graph.insert_edge(4, &a, &b).id();
    /// let bc = *graph.insert_edge(5, &b, &c).id();
    /// let ca = *graph.insert_edge(6, &c, &a).id();
    ///
    /// graph.retain(|node| node.weight() % 2 == 1, |edge| edge.weight() % 2 == 0);
    ///
    /// assert_eq!(
    ///     graph
    ///         .nodes()
    ///         .map(|node| (*node.id(), *node.weight()))
    ///         .collect::<HashSet<_>>(),
    ///     [(a, 1), (c, 3)].into_iter().collect::<HashSet<_>>()
    /// );
    ///
    /// assert_eq!(
    ///     graph
    ///         .edges()
    ///         .map(|edge| (*edge.id(), *edge.weight()))
    ///         .collect::<HashSet<_>>(),
    ///     // as you can see here, although `ab` would've been retained, `b` was removed, so
    ///     // the edge was removed as well.
    ///     once((ca, 6)).collect::<HashSet<_>>()
    /// );
    /// ```
    pub fn retain(
        &mut self,
        nodes: impl FnMut(NodeMut<'_, S>) -> bool,
        edges: impl FnMut(EdgeMut<'_, S>) -> bool,
    ) {
        self.storage.retain(nodes, edges);
    }

    /// Retains only the nodes specified by the predicate.
    ///
    /// If you are going to retain edges as well, it is more efficient to use [`Self::retain`].
    ///
    /// For every node in the graph, the predicate is invoked, if the predicate
    /// evaluates to true the node is retained, otherwise the node and all connected edges are
    /// removed.
    ///
    /// # Example
    ///
    /// ```
    /// use std::{collections::HashSet, iter::once};
    ///
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node(1).id();
    /// let b = *graph.insert_node(2).id();
    /// let c = *graph.insert_node(3).id();
    ///
    /// let ab = *graph.insert_edge(4, &a, &b).id();
    /// let bc = *graph.insert_edge(5, &b, &c).id();
    /// let ca = *graph.insert_edge(6, &c, &a).id();
    ///
    /// graph.retain_nodes(|node| node.weight() % 2 == 1);
    ///
    /// assert_eq!(
    ///     graph
    ///         .nodes()
    ///         .map(|node| (*node.id(), *node.weight()))
    ///         .collect::<HashSet<_>>(),
    ///     [(a, 1), (c, 3)].into_iter().collect::<HashSet<_>>()
    /// );
    ///
    /// assert_eq!(
    ///     graph
    ///         .edges()
    ///         .map(|edge| (*edge.id(), *edge.weight()))
    ///         .collect::<HashSet<_>>(),
    ///     // `ab` and `bc` were removed, because the `b` node wasn't retained.
    ///     once((ca, 6)).collect::<HashSet<_>>()
    /// );
    /// ```
    pub fn retain_nodes(&mut self, f: impl FnMut(NodeMut<'_, S>) -> bool) {
        self.storage.retain_nodes(f);
    }

    /// Retains only the edges specified by the predicate.
    ///
    /// If you are going to retain nodes based on a predicate as well, it is more efficient to
    /// instead use [`Self::retain`].
    ///
    /// For every edge in the graph, the predicate is invoked, if the predicate
    /// evaluates to true the edge is retained, otherwise the edge is removed.
    ///
    /// # Example
    ///
    /// ```
    /// use std::{collections::HashSet, iter::once};
    ///
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node(1).id();
    /// let b = *graph.insert_node(2).id();
    /// let c = *graph.insert_node(3).id();
    ///
    /// let ab = *graph.insert_edge(4, &a, &b).id();
    /// let bc = *graph.insert_edge(5, &b, &c).id();
    /// let ca = *graph.insert_edge(6, &c, &a).id();
    ///
    /// graph.retain_edges(|edge| edge.weight() % 2 == 0);
    ///
    /// assert_eq!(
    ///     graph
    ///         .nodes()
    ///         .map(|node| (*node.id(), *node.weight()))
    ///         .collect::<HashSet<_>>(),
    ///     [(a, 1), (b, 2), (c, 3)].into_iter().collect::<HashSet<_>>()
    /// );
    ///
    /// assert_eq!(
    ///     graph
    ///         .edges()
    ///         .map(|edge| (*edge.id(), *edge.weight()))
    ///         .collect::<HashSet<_>>(),
    ///     [(ab, 4), (ca, 6)].into_iter().collect::<HashSet<_>>()
    /// );
    /// ```
    pub fn retain_edges(&mut self, f: impl FnMut(EdgeMut<'_, S>) -> bool) {
        self.storage.retain_edges(f);
    }
}
