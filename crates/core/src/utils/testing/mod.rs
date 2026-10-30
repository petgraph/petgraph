pub mod directed;
pub mod editable;
pub mod storable;
pub mod undirected;

pub(crate) use __common::*;

// WARNING: This module contains only internal testing helper functions and should not be considered
// part of the public API. It is subject to change at any time.
#[doc(hidden)]
pub mod __common {
    use crate::graph::EditableGraph;

    /// Constants for the number of nodes and edges in the test graph.
    pub const TEST_GRAPH_NODE_COUNT: usize = 5;
    pub const TEST_GRAPH_EDGE_COUNT: usize = 4;

    /// The `(source, target)` pairs of the edges in the test graph, given as indices into the node
    /// array returned by [`create_test_graph`].
    pub const TEST_GRAPH_EDGE_ENDPOINTS: [(usize, usize); TEST_GRAPH_EDGE_COUNT] =
        [(0, 1), (0, 2), (1, 3), (3, 2)];

    /// A helper function to create a simple graph for testing purposes.
    ///
    /// The graph looks as follows:
    /// ```text
    /// 0 --> 1
    /// |      |
    /// v      v
    /// 2 <----3     4
    /// ```
    /// The graphs directionality is determined by the graph type `G` passed to the function. The
    /// edges are added with the directionality shown in the diagram above.
    ///
    /// The function returns a tuple containing the constructed graph,
    /// a slice of the ids of added nodes, and a slice of the ids of added edges.
    ///
    /// For the number of nodes and edges in the graph and the edges see the constants
    /// [`TEST_GRAPH_NODE_COUNT`], [`TEST_GRAPH_EDGE_COUNT`] and [`TEST_GRAPH_EDGE_ENDPOINTS`].
    ///
    /// For ordering of added nodes and edges, see the implementation of the function.
    ///
    /// # Panics
    /// Panics if adding a node or an edge to the graph returns an error.
    pub fn create_test_graph<G: EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>>(
        graph_constructor: impl Fn() -> G,
    ) -> (
        G,
        [G::NodeId; TEST_GRAPH_NODE_COUNT],
        [G::EdgeId; TEST_GRAPH_EDGE_COUNT],
    ) {
        let mut graph = graph_constructor();

        let nodes = [
            graph.add_node(()).unwrap(),
            graph.add_node(()).unwrap(),
            graph.add_node(()).unwrap(),
            graph.add_node(()).unwrap(),
            graph.add_node(()).unwrap(),
        ];

        let edges: [G::EdgeId; TEST_GRAPH_EDGE_COUNT] = core::array::from_fn(|index| {
            let (source, target) = TEST_GRAPH_EDGE_ENDPOINTS[index];
            graph.add_edge(nodes[source], nodes[target], ()).unwrap()
        });

        assert_eq!(nodes.len(), TEST_GRAPH_NODE_COUNT);
        assert_eq!(edges.len(), TEST_GRAPH_EDGE_COUNT);

        (graph, nodes, edges)
    }

    /// Helper functions that returns the ids of the edges in the test graph whose `(source,
    /// target)` node indices satisfy `predicate`.
    ///
    /// `edges` has to be the array of edge ids returned by [`create_test_graph`]. The node indices
    /// passed to `predicate` refer to the array of node ids returned by it.
    pub fn expected_edges<E>(
        edges: [E; TEST_GRAPH_EDGE_COUNT],
        predicate: impl Fn(usize, usize) -> bool,
    ) -> impl Iterator<Item = E> {
        TEST_GRAPH_EDGE_ENDPOINTS
            .into_iter()
            .zip(edges)
            .filter(move |&((source, target), _)| predicate(source, target))
            .map(|(_, edge)| edge)
    }

    /// Helper function that returns the `(id, source, target)` triples of the edges in the test
    /// graph.
    ///
    /// `nodes` and `edges` have to be the arrays of ids returned by [`create_test_graph`].
    pub fn expected_edge_endpoints<N: Copy, E: Copy>(
        nodes: [N; TEST_GRAPH_NODE_COUNT],
        edges: [E; TEST_GRAPH_EDGE_COUNT],
    ) -> [(E, N, N); TEST_GRAPH_EDGE_COUNT] {
        core::array::from_fn(|index| {
            let (source, target) = TEST_GRAPH_EDGE_ENDPOINTS[index];
            (edges[index], nodes[source], nodes[target])
        })
    }

    /// Helper function to check if the edges returned by an iterator match the expected edges.
    ///
    /// The additional arguments are just for better error messages.
    pub fn check_if_edges_match<T>(
        expected_edges: impl IntoIterator<Item = T>,
        actual_edges: impl Iterator<Item = T>,
        method_name: &'static str,
        graph_type: &'static str,
        context: impl core::fmt::Display,
    ) where
        T: core::hash::Hash + Eq + core::fmt::Debug,
    {
        let mut expected_edges: hashbrown::HashSet<T, foldhash::fast::RandomState> =
            expected_edges.into_iter().collect();

        for edge in actual_edges {
            assert!(
                expected_edges.remove(&edge),
                "{graph_type}::{method_name}() contained unexpected edge id: {edge:?} for \
                 {context}",
            );
        }

        assert!(
            expected_edges.is_empty(),
            "{graph_type}::{method_name}() did not return all expected edges for {context}: \
             {expected_edges:?}",
        );
    }

    /// Helper function to check if the nodes returned by an iterator match the expected nodes.
    ///
    /// The additional arguments are just for better error messages.
    pub fn check_if_nodes_match<T>(
        expected_nodes: impl IntoIterator<Item = T>,
        actual_nodes: impl Iterator<Item = T>,
        method_name: &'static str,
        graph_type: &'static str,
        context: impl core::fmt::Debug,
    ) where
        T: core::hash::Hash + Eq + core::fmt::Debug,
    {
        let mut expected_nodes: hashbrown::HashSet<T, foldhash::fast::RandomState> =
            expected_nodes.into_iter().collect();

        for node in actual_nodes {
            assert!(
                expected_nodes.remove(&node),
                "{graph_type}::{method_name}() contained unexpected node id: {node:?} for \
                 {context:?}",
            );
        }

        assert!(
            expected_nodes.is_empty(),
            "{graph_type}::{method_name}() did not return all expected nodes for {context:?}: \
             {expected_nodes:?}",
        );
    }
}
