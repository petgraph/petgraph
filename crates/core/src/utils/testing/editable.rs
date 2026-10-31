/// Generates a suite of tests for [`EditableGraph`][crate::graph::EditableGraph] implementations,
/// which are also [`DirectedGraph`][crate::graph::DirectedGraph]s. It runs on the
/// [`DEFAULT_TEST_GRAPH`][crate::utils::testing::__common::DEFAULT_TEST_GRAPH] and
/// [`ISOLATED_TEST_GRAPH`][crate::utils::testing::__common::ISOLATED_TEST_GRAPH].
///
/// For the expected invariants and the arguments of this macro, see
/// [`__test_editable_graph_inner`][crate::__test_editable_graph_inner]. To run the suite on other
/// test graphs, use
/// [`__test_directed_editable_graph_inner`][crate::__test_directed_editable_graph_inner].
///
/// For a reference usage, see the tests in [`crate::utils::test_graphs::directed`].
#[macro_export]
macro_rules! test_directed_editable_graph {
    ($graph_constructor:expr,) => {
        mod editable_graph_macro {
            mod default {
                #[allow(unused_imports)]
                use super::super::*;

                $crate::__test_directed_editable_graph_inner!(
                    $graph_constructor,
                    $crate::utils::testing::__common::DEFAULT_TEST_GRAPH,
                );
            }

            mod isolated {
                #[allow(unused_imports)]
                use super::super::*;

                $crate::__test_directed_editable_graph_inner!(
                    $graph_constructor,
                    $crate::utils::testing::__common::ISOLATED_TEST_GRAPH,
                );
            }
        }
    };
}

/// Generates a suite of tests for [`EditableGraph`][crate::graph::EditableGraph] implementations,
/// which are also [`DirectedGraph`][crate::graph::DirectedGraph]s and support self-loops. It runs
/// on the [`LOOP_TEST_GRAPH`][crate::utils::testing::__common::LOOP_TEST_GRAPH].
///
/// For the expected invariants and the arguments of this macro, see
/// [`__test_editable_graph_inner`][crate::__test_editable_graph_inner]. To run the suite on other
/// test graphs, use
/// [`__test_directed_editable_graph_inner`][crate::__test_directed_editable_graph_inner].
///
/// For a reference usage, see the tests in [`crate::utils::test_graphs::directed`].
#[macro_export]
macro_rules! test_directed_editable_graph_self_loops {
    ($graph_constructor:expr,) => {
        mod editable_graph_macro_self_loops {
            #[allow(unused_imports)]
            use super::*;

            $crate::__test_directed_editable_graph_inner!(
                $graph_constructor,
                $crate::utils::testing::__common::LOOP_TEST_GRAPH,
            );
        }
    };
}

/// Generates a suite of tests for [`EditableGraph`][crate::graph::EditableGraph] implementations,
/// which are also [`DirectedGraph`][crate::graph::DirectedGraph]s and support self-loops and
/// parallel edges. It runs on the
/// [`MULTI_TEST_GRAPH`][crate::utils::testing::__common::MULTI_TEST_GRAPH].
///
/// For the expected invariants and the arguments of this macro, see
/// [`__test_editable_graph_inner`][crate::__test_editable_graph_inner]. To run the suite on other
/// test graphs, use
/// [`__test_directed_editable_graph_inner`][crate::__test_directed_editable_graph_inner].
///
/// For a reference usage, see the tests in [`crate::utils::test_graphs::directed`].
#[macro_export]
macro_rules! test_directed_editable_graph_parallel_edges {
    ($graph_constructor:expr,) => {
        mod editable_graph_macro_parallel_edges {
            #[allow(unused_imports)]
            use super::*;

            $crate::__test_directed_editable_graph_inner!(
                $graph_constructor,
                $crate::utils::testing::__common::MULTI_TEST_GRAPH,
            );
        }
    };
}

/// Generates a suite of tests for [`EditableGraph`][crate::graph::EditableGraph] implementations,
/// which are also [`DirectedGraph`][crate::graph::DirectedGraph]s. It runs on the given test graph.
///
/// For the expected invariants and the arguments `$graph_constructor` and `$test_graph`, see
/// [`__test_editable_graph_inner`][crate::__test_editable_graph_inner].
///
/// Prefer using [`test_directed_editable_graph`][crate::test_directed_editable_graph] to run
/// the suite on the default test graphs.
#[doc(hidden)]
#[macro_export]
macro_rules! __test_directed_editable_graph_inner {
    ($graph_constructor:expr, $test_graph:expr,) => {
        $crate::__test_editable_graph_inner!(
            $graph_constructor,
            $test_graph,
            "DirectedGraph",
            $crate::graph::DirectedGraph::contains_node,
            $crate::graph::DirectedGraph::contains_edge,
            $crate::graph::DirectedGraph::node_count,
            $crate::graph::DirectedGraph::edge_count,
        );
    };
}

/// Generates a suite of tests for [`EditableGraph`][crate::graph::EditableGraph] implementations,
/// which are also [`UndirectedGraph`][crate::graph::UndirectedGraph]s. It runs on the
/// [`DEFAULT_TEST_GRAPH`][crate::utils::testing::__common::DEFAULT_TEST_GRAPH] and
/// [`ISOLATED_TEST_GRAPH`][crate::utils::testing::__common::ISOLATED_TEST_GRAPH].
///
/// For the expected invariants and the arguments of this macro, see
/// [`__test_editable_graph_inner`][crate::__test_editable_graph_inner]. To run the suite on other
/// test graphs, use
/// [`__test_undirected_editable_graph_inner`][crate::__test_undirected_editable_graph_inner].
///
/// For a reference usage, see the tests in [`crate::utils::test_graphs::undirected`].
#[macro_export]
macro_rules! test_undirected_editable_graph {
    ($graph_constructor:expr,) => {
        mod editable_graph_macro {
            mod default {
                #[allow(unused_imports)]
                use super::super::*;

                $crate::__test_undirected_editable_graph_inner!(
                    $graph_constructor,
                    $crate::utils::testing::__common::DEFAULT_TEST_GRAPH,
                );
            }

            mod isolated {
                #[allow(unused_imports)]
                use super::super::*;

                $crate::__test_undirected_editable_graph_inner!(
                    $graph_constructor,
                    $crate::utils::testing::__common::ISOLATED_TEST_GRAPH,
                );
            }
        }
    };
}

/// Generates a suite of tests for [`EditableGraph`][crate::graph::EditableGraph] implementations,
/// which are also [`UndirectedGraph`][crate::graph::UndirectedGraph]s and support self-loops. It
/// runs on the [`LOOP_TEST_GRAPH`][crate::utils::testing::__common::LOOP_TEST_GRAPH].
///
/// For the expected invariants and the arguments of this macro, see
/// [`__test_editable_graph_inner`][crate::__test_editable_graph_inner]. To run the suite on other
/// test graphs, use
/// [`__test_undirected_editable_graph_inner`][crate::__test_undirected_editable_graph_inner].
///
/// For a reference usage, see the tests in [`crate::utils::test_graphs::undirected`].
#[macro_export]
macro_rules! test_undirected_editable_graph_self_loops {
    ($graph_constructor:expr,) => {
        mod editable_graph_macro_self_loops {
            #[allow(unused_imports)]
            use super::*;

            $crate::__test_undirected_editable_graph_inner!(
                $graph_constructor,
                $crate::utils::testing::__common::LOOP_TEST_GRAPH,
            );
        }
    };
}

/// Generates a suite of tests for [`EditableGraph`][crate::graph::EditableGraph] implementations,
/// which are also [`UndirectedGraph`][crate::graph::UndirectedGraph]s and support self-loops and
/// parallel edges. It runs on the
/// [`MULTI_TEST_GRAPH`][crate::utils::testing::__common::MULTI_TEST_GRAPH].
///
/// For the expected invariants and the arguments of this macro, see
/// [`__test_editable_graph_inner`][crate::__test_editable_graph_inner]. To run the suite on other
/// test graphs, use
/// [`__test_undirected_editable_graph_inner`][crate::__test_undirected_editable_graph_inner].
///
/// For a reference usage, see the tests in [`crate::utils::test_graphs::undirected`].
#[macro_export]
macro_rules! test_undirected_editable_graph_parallel_edges {
    ($graph_constructor:expr,) => {
        mod editable_graph_macro_parallel_edges {
            #[allow(unused_imports)]
            use super::*;

            $crate::__test_undirected_editable_graph_inner!(
                $graph_constructor,
                $crate::utils::testing::__common::MULTI_TEST_GRAPH,
            );
        }
    };
}

/// Generates a suite of tests for [`EditableGraph`][crate::graph::EditableGraph] implementations,
/// which are also [`UndirectedGraph`][crate::graph::UndirectedGraph]s. It runs on the given test
/// graph.
///
/// For the expected invariants and the arguments `$graph_constructor` and `$test_graph`, see
/// [`__test_editable_graph_inner`][crate::__test_editable_graph_inner].
///
/// Prefer using [`test_undirected_editable_graph`][crate::test_undirected_editable_graph] to run
/// the suite on the default test graphs.
#[doc(hidden)]
#[macro_export]
macro_rules! __test_undirected_editable_graph_inner {
    ($graph_constructor:expr, $test_graph:expr,) => {
        $crate::__test_editable_graph_inner!(
            $graph_constructor,
            $test_graph,
            "UndirectedGraph",
            $crate::graph::UndirectedGraph::contains_node,
            $crate::graph::UndirectedGraph::contains_edge,
            $crate::graph::UndirectedGraph::node_count,
            $crate::graph::UndirectedGraph::edge_count,
        );
    };
}

/// Generates a suite of tests for [`EditableGraph`][crate::graph::EditableGraph] implementations,
/// which runs on the given test graph.
///
/// At least one test is generated for each method in the
/// [`EditableGraph`][crate::graph::EditableGraph] trait. The expected results are derived from the
/// test graph, so the suite can be run on any test graph that the graph type is able to represent.
///
/// The following invariants are expected from the graph implementation:
/// - If the most recently added node or edge is removed, its ID will no longer be valid. I.e.,
///   calling methods with that ID should return `None`, an error, or otherwise indicate
///   non-existence.
/// - Removing the most recently added node or edge does not change the IDs of any other nodes or
///   edges.
///
/// The arguments to this macro are as follows (`G` is used to denote the graph type being tested):
/// - `$graph_constructor`: `impl Fn() -> G`. An expression that constructs a new instance of the
///   graph type to be tested. The generated graph must be empty (e.g. `G::new()`).
/// - `$test_graph`: [`TestGraph`][crate::utils::testing::__common::TestGraph]. The test graph,
///   which is built by adding its nodes and edges to the graphs returned by `$graph_constructor`.
/// - `$graph_type`: `&'static str`. The name of the trait providing the functions below, used in
///   error messages.
/// - `$contains_node`: `fn(&G, G::NodeId) -> bool`. Returns whether the node exists in the graph.
/// - `$contains_edge`: `fn(&G, G::EdgeId) -> bool`. Returns whether the edge exists in the graph.
/// - `$node_count`: `fn(&G) -> usize`. Returns the number of nodes in the graph.
/// - `$edge_count`: `fn(&G) -> usize`. Returns the number of edges in the graph.
///
/// Prefer using [`test_directed_editable_graph`][crate::test_directed_editable_graph] or
/// [`test_undirected_editable_graph`][crate::test_undirected_editable_graph] instead.
#[doc(hidden)]
#[macro_export]
macro_rules! __test_editable_graph_inner {
    (
        $graph_constructor:expr,
        $test_graph:expr,
        $graph_type:expr,
        $contains_node:expr,
        $contains_edge:expr,
        $node_count:expr,
        $edge_count:expr,
    ) => {
        #[test]
        fn test_add_node() {
            $crate::utils::testing::editable::__private_testing::test_add_node_inner(
                $graph_constructor,
                $test_graph,
                $graph_type,
                $contains_node,
                $node_count,
                $edge_count,
            );
        }

        #[test]
        fn test_add_edge() {
            $crate::utils::testing::editable::__private_testing::test_add_edge_inner(
                $graph_constructor,
                $test_graph,
                $graph_type,
                $contains_node,
                $contains_edge,
                $node_count,
                $edge_count,
            );
        }

        #[test]
        fn test_add_edge_with_removed_endpoint() {
            $crate::utils::testing::editable::__private_testing::test_add_edge_with_removed_endpoint_inner(
                $graph_constructor,
                $test_graph,
                $graph_type,
                $contains_edge,
                $edge_count,
            );
        }

        #[test]
        fn test_remove_node() {
            $crate::utils::testing::editable::__private_testing::test_remove_node_inner(
                $graph_constructor,
                $test_graph,
                $graph_type,
                $contains_node,
                $contains_edge,
                $node_count,
                $edge_count,
            );
        }

        #[test]
        fn test_remove_isolated_node() {
            $crate::utils::testing::editable::__private_testing::test_remove_isolated_node_inner(
                $graph_constructor,
                $test_graph,
                $graph_type,
                $contains_node,
                $contains_edge,
                $node_count,
                $edge_count,
            );
        }

        #[test]
        fn test_remove_all_nodes() {
            $crate::utils::testing::editable::__private_testing::test_remove_all_nodes_inner(
                $graph_constructor,
                $test_graph,
                $graph_type,
                $contains_node,
                $contains_edge,
                $node_count,
                $edge_count,
            );
        }

        #[test]
        fn test_remove_edge() {
            $crate::utils::testing::editable::__private_testing::test_remove_edge_inner(
                $graph_constructor,
                $test_graph,
                $graph_type,
                $contains_node,
                $contains_edge,
                $node_count,
                $edge_count,
            );
        }
    };
}

// WARNING: This module contains only internal testing helper functions and should not be considered
// part of the public API. It is subject to change at any time.
#[doc(hidden)]
pub mod __private_testing {
    use alloc::vec::Vec;

    use crate::{graph::EditableGraph, utils::testing::TestGraph};

    /// Asserts that all given nodes are contained in the graph.
    fn assert_contains_nodes<G: EditableGraph>(
        graph: &G,
        nodes: &[G::NodeId],
        graph_type: &'static str,
        contains_node: fn(&G, G::NodeId) -> bool,
        context: &'static str,
    ) {
        for &node in nodes {
            assert!(
                contains_node(graph, node),
                "{graph_type}::contains_node() returned false for existing node {node:?} {context}"
            );
        }
    }

    /// Asserts that all given edges are contained in the graph.
    fn assert_contains_edges<G: EditableGraph>(
        graph: &G,
        edges: &[G::EdgeId],
        graph_type: &'static str,
        contains_edge: fn(&G, G::EdgeId) -> bool,
        context: &'static str,
    ) {
        for &edge in edges {
            assert!(
                contains_edge(graph, edge),
                "{graph_type}::contains_edge() returned false for existing edge {edge:?} {context}"
            );
        }
    }

    pub fn test_add_node_inner<G: EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>>(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
        graph_type: &'static str,
        contains_node: fn(&G, G::NodeId) -> bool,
        node_count: fn(&G) -> usize,
        edge_count: fn(&G) -> usize,
    ) {
        let mut graph = graph_constructor();
        assert_eq!(
            node_count(&graph),
            0,
            "{graph_type}::node_count() did not return 0 for a newly constructed graph"
        );

        let mut nodes = Vec::with_capacity(test_graph.node_count);
        for index in 0..test_graph.node_count {
            let node = graph
                .add_node(())
                .expect("EditableGraph::add_node() returned an error");

            assert!(
                contains_node(&graph, node),
                "{graph_type}::contains_node() returned false for newly added node {node:?}"
            );
            assert_eq!(
                node_count(&graph),
                index + 1,
                "{graph_type}::node_count() did not match expected value after adding node {index}"
            );
            for &previous_node in &nodes {
                assert_ne!(
                    previous_node, node,
                    "EditableGraph::add_node() returned an already existing node id"
                );
            }

            nodes.push(node);
        }

        assert_contains_nodes(
            &graph,
            &nodes,
            graph_type,
            contains_node,
            "after adding all nodes",
        );
        assert_eq!(
            edge_count(&graph),
            0,
            "{graph_type}::edge_count() did not return 0 after only adding nodes"
        );
    }

    pub fn test_add_edge_inner<G: EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>>(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
        graph_type: &'static str,
        contains_node: fn(&G, G::NodeId) -> bool,
        contains_edge: fn(&G, G::EdgeId) -> bool,
        node_count: fn(&G) -> usize,
        edge_count: fn(&G) -> usize,
    ) {
        let mut graph = graph_constructor();
        let nodes: Vec<G::NodeId> = (0..test_graph.node_count)
            .map(|_| graph.add_node(()).unwrap())
            .collect();

        let mut edges = Vec::with_capacity(test_graph.edge_count());
        for (index, &(source, target)) in test_graph.edge_endpoints.iter().enumerate() {
            let edge = graph
                .add_edge(nodes[source], nodes[target], ())
                .expect("EditableGraph::add_edge() returned an error for existing nodes");

            assert!(
                contains_edge(&graph, edge),
                "{graph_type}::contains_edge() returned false for newly added edge {edge:?}"
            );
            assert_eq!(
                edge_count(&graph),
                index + 1,
                "{graph_type}::edge_count() did not match expected value after adding edge {index}"
            );
            for &previous_edge in &edges {
                assert_ne!(
                    previous_edge, edge,
                    "EditableGraph::add_edge() returned an already existing edge id"
                );
            }

            edges.push(edge);
        }

        assert_contains_edges(
            &graph,
            &edges,
            graph_type,
            contains_edge,
            "after adding all edges",
        );
        assert_contains_nodes(
            &graph,
            &nodes,
            graph_type,
            contains_node,
            "after adding all edges",
        );
        assert_eq!(
            node_count(&graph),
            test_graph.node_count,
            "{graph_type}::node_count() changed after adding edges"
        );
    }

    pub fn test_add_edge_with_removed_endpoint_inner<
        G: EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
        graph_type: &'static str,
        contains_edge: fn(&G, G::EdgeId) -> bool,
        edge_count: fn(&G) -> usize,
    ) {
        let (mut graph, nodes, edges) = test_graph.build(graph_constructor);

        // The newest node is removed, as some graph implementations might not have stable node
        // ids, but the newest node added is likely to be removable without another node taking
        // its id.
        let removed_node = graph.add_node(()).unwrap();
        graph.remove_node(removed_node).unwrap();

        let Some(&first_node) = nodes.first() else {
            return;
        };

        assert!(
            graph.add_edge(first_node, removed_node, ()).is_err(),
            "EditableGraph::add_edge() did not return an error for a removed target node"
        );
        assert!(
            graph.add_edge(removed_node, first_node, ()).is_err(),
            "EditableGraph::add_edge() did not return an error for a removed source node"
        );

        assert_eq!(
            edge_count(&graph),
            test_graph.edge_count(),
            "{graph_type}::edge_count() changed after failing to add edges"
        );
        assert_contains_edges(
            &graph,
            &edges,
            graph_type,
            contains_edge,
            "after failing to add edges",
        );
    }

    pub fn test_remove_node_inner<G: EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>>(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
        graph_type: &'static str,
        contains_node: fn(&G, G::NodeId) -> bool,
        contains_edge: fn(&G, G::EdgeId) -> bool,
        node_count: fn(&G) -> usize,
        edge_count: fn(&G) -> usize,
    ) {
        let (mut graph, nodes, edges) = test_graph.build(graph_constructor);
        let [first_node, .., last_node] = nodes[..] else {
            return;
        };

        // The newest node and edges are removed, as some graph implementations might not have
        // stable ids, but the newest node and edges added are likely to be removable without
        // another node or edge taking their ids.
        let removed_node = graph.add_node(()).unwrap();
        let incoming_edge = graph.add_edge(last_node, removed_node, ()).unwrap();
        let outgoing_edge = graph.add_edge(removed_node, first_node, ()).unwrap();

        graph
            .remove_node(removed_node)
            .expect("EditableGraph::remove_node() returned an error for an existing node");

        assert!(
            !contains_node(&graph, removed_node),
            "{graph_type}::contains_node() returned true for removed node {removed_node:?}"
        );
        assert!(
            !contains_edge(&graph, incoming_edge),
            "{graph_type}::contains_edge() returned true for edge {incoming_edge:?} incident to \
             removed node"
        );
        assert!(
            !contains_edge(&graph, outgoing_edge),
            "{graph_type}::contains_edge() returned true for edge {outgoing_edge:?} incident to \
             removed node"
        );
        assert_eq!(
            node_count(&graph),
            test_graph.node_count,
            "{graph_type}::node_count() did not match expected value after removing a node"
        );
        assert_eq!(
            edge_count(&graph),
            test_graph.edge_count(),
            "{graph_type}::edge_count() did not match expected value after removing a node with \
             incident edges"
        );
        assert_contains_nodes(
            &graph,
            &nodes,
            graph_type,
            contains_node,
            "after removing another node",
        );
        assert_contains_edges(
            &graph,
            &edges,
            graph_type,
            contains_edge,
            "after removing a non-incident node",
        );

        assert!(
            graph.remove_node(removed_node).is_err(),
            "EditableGraph::remove_node() did not return an error for an already removed node"
        );

        // Node ids might not be stable here, so only the counts are checked.
        let (mut graph, nodes, edges) = test_graph.build(graph_constructor);
        graph.remove_node(nodes[0]).unwrap();
        assert_eq!(
            node_count(&graph),
            test_graph.node_count - 1,
            "{graph_type}::node_count() did not match expected value after removing node 0"
        );

        assert_eq!(
            edge_count(&graph),
            test_graph
                .expected_edges(&edges, |source, target| source != 0 && target != 0)
                .count(),
            "{graph_type}::edge_count() did not match expected value after removing node 0"
        );
    }

    pub fn test_remove_isolated_node_inner<
        G: EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
        graph_type: &'static str,
        contains_node: fn(&G, G::NodeId) -> bool,
        contains_edge: fn(&G, G::EdgeId) -> bool,
        node_count: fn(&G) -> usize,
        edge_count: fn(&G) -> usize,
    ) {
        let (mut graph, nodes, edges) = test_graph.build(graph_constructor);

        let isolated_node = graph.add_node(()).unwrap();
        graph
            .remove_node(isolated_node)
            .expect("EditableGraph::remove_node() returned an error for an existing node");

        assert!(
            !contains_node(&graph, isolated_node),
            "{graph_type}::contains_node() returned true for removed node {isolated_node:?}"
        );
        assert_eq!(
            node_count(&graph),
            test_graph.node_count,
            "{graph_type}::node_count() did not match expected value after removing an isolated \
             node"
        );
        assert_eq!(
            edge_count(&graph),
            test_graph.edge_count(),
            "{graph_type}::edge_count() changed after removing an isolated node"
        );
        assert_contains_nodes(
            &graph,
            &nodes,
            graph_type,
            contains_node,
            "after removing an isolated node",
        );
        assert_contains_edges(
            &graph,
            &edges,
            graph_type,
            contains_edge,
            "after removing an isolated node",
        );
    }

    pub fn test_remove_all_nodes_inner<G: EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>>(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
        graph_type: &'static str,
        contains_node: fn(&G, G::NodeId) -> bool,
        contains_edge: fn(&G, G::EdgeId) -> bool,
        node_count: fn(&G) -> usize,
        edge_count: fn(&G) -> usize,
    ) {
        let (mut graph, nodes, edges) = test_graph.build(graph_constructor);

        // Nodes are removed newest first, as some graph implementations might not have stable
        // node ids.
        for (index, &node) in nodes.iter().enumerate().rev() {
            graph
                .remove_node(node)
                .expect("EditableGraph::remove_node() returned an error for an existing node");
            assert!(
                !contains_node(&graph, node),
                "{graph_type}::contains_node() returned true for removed node {node:?}"
            );
            assert_eq!(
                node_count(&graph),
                index,
                "{graph_type}::node_count() did not match expected value after removing node \
                 {index}"
            );
        }

        assert_eq!(
            edge_count(&graph),
            0,
            "{graph_type}::edge_count() did not return 0 after removing all nodes"
        );
        for &edge in &edges {
            assert!(
                !contains_edge(&graph, edge),
                "{graph_type}::contains_edge() returned true for edge {edge:?} after removing all \
                 nodes"
            );
        }
    }

    pub fn test_remove_edge_inner<G: EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>>(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
        graph_type: &'static str,
        contains_node: fn(&G, G::NodeId) -> bool,
        contains_edge: fn(&G, G::EdgeId) -> bool,
        node_count: fn(&G) -> usize,
        edge_count: fn(&G) -> usize,
    ) {
        let (mut graph, nodes, edges) = test_graph.build(graph_constructor);
        let Some((&last_edge, remaining_edges)) = edges.split_last() else {
            return;
        };

        graph
            .remove_edge(last_edge)
            .expect("EditableGraph::remove_edge() returned an error for an existing edge");

        assert!(
            !contains_edge(&graph, last_edge),
            "{graph_type}::contains_edge() returned true for removed edge {last_edge:?}"
        );
        assert_eq!(
            edge_count(&graph),
            test_graph.edge_count() - 1,
            "{graph_type}::edge_count() did not match expected value after removing the last edge"
        );
        assert_eq!(
            node_count(&graph),
            test_graph.node_count,
            "{graph_type}::node_count() changed after removing the last edge"
        );
        assert_contains_edges(
            &graph,
            remaining_edges,
            graph_type,
            contains_edge,
            "after removing the last edge",
        );
        assert_contains_nodes(
            &graph,
            &nodes,
            graph_type,
            contains_node,
            "after removing the last edge",
        );

        assert!(
            graph.remove_edge(last_edge).is_err(),
            "EditableGraph::remove_edge() did not return an error for an already removed edge"
        );

        let (source, target) = test_graph.edge_endpoints[remaining_edges.len()];
        let new_edge = graph
            .add_edge(nodes[source], nodes[target], ())
            .expect("EditableGraph::add_edge() returned an error after removing the same edge");
        assert!(
            contains_edge(&graph, new_edge),
            "{graph_type}::contains_edge() returned false for re-added edge {new_edge:?}"
        );
        assert_eq!(
            edge_count(&graph),
            test_graph.edge_count(),
            "{graph_type}::edge_count() did not match expected value after re-adding the last edge"
        );
    }
}
