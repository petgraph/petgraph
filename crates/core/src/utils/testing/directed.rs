/// Generates a suite of tests for [`DirectedGraph`][crate::graph::DirectedGraph]
/// implementations. It runs on the
/// [`DEFAULT_TEST_GRAPH`][crate::utils::testing::__common::DEFAULT_TEST_GRAPH] and
/// [`ISOLATED_TEST_GRAPH`][crate::utils::testing::__common::ISOLATED_TEST_GRAPH].
///
/// For the expected invariants and the arguments of this macro, see
/// [`__test_directed_graph_inner`][crate::__test_directed_graph_inner], which can also be used to
/// run the suite on other test graphs.
///
/// For a reference usage, see the tests in [`crate::utils::test_graphs::directed`].
#[macro_export]
macro_rules! test_directed_graph {
    ($graph_constructor:expr,) => {
        mod directed_graph_macro {
            mod default {
                #[allow(unused_imports)]
                use super::super::*;

                $crate::__test_directed_graph_inner!(
                    $graph_constructor,
                    $crate::utils::testing::__common::DEFAULT_TEST_GRAPH,
                );
            }

            mod isolated {
                #[allow(unused_imports)]
                use super::super::*;

                $crate::__test_directed_graph_inner!(
                    $graph_constructor,
                    $crate::utils::testing::__common::ISOLATED_TEST_GRAPH,
                );
            }
        }
    };
}

/// Generates a suite of tests for [`DirectedGraph`][crate::graph::DirectedGraph]
/// implementations of graphs which support self-loops. It runs on the
/// [`LOOP_TEST_GRAPH`][crate::utils::testing::__common::LOOP_TEST_GRAPH].
///
/// For the expected invariants and the arguments of this macro, see
/// [`__test_directed_graph_inner`][crate::__test_directed_graph_inner], which can also be used to
/// run the suite on other test graphs.
///
/// For a reference usage, see the tests in [`crate::utils::test_graphs::directed`].
#[macro_export]
macro_rules! test_directed_graph_self_loops {
    ($graph_constructor:expr,) => {
        mod directed_graph_macro_self_loops {
            #[allow(unused_imports)]
            use super::*;

            $crate::__test_directed_graph_inner!(
                $graph_constructor,
                $crate::utils::testing::__common::LOOP_TEST_GRAPH,
            );
        }
    };
}

/// Generates a suite of tests for [`DirectedGraph`][crate::graph::DirectedGraph]
/// implementations of graphs which support self-loops and parallel edges. It runs on the
/// [`MULTI_TEST_GRAPH`][crate::utils::testing::__common::MULTI_TEST_GRAPH].
///
/// For the expected invariants and the arguments of this macro, see
/// [`__test_directed_graph_inner`][crate::__test_directed_graph_inner], which can also be used to
/// run the suite on other test graphs.
///
/// For a reference usage, see the tests in [`crate::utils::test_graphs::directed`].
#[macro_export]
macro_rules! test_directed_graph_parallel_edges {
    ($graph_constructor:expr,) => {
        mod directed_graph_macro_parallel_edges {
            #[allow(unused_imports)]
            use super::*;

            $crate::__test_directed_graph_inner!(
                $graph_constructor,
                $crate::utils::testing::__common::MULTI_TEST_GRAPH,
            );
        }
    };
}

/// Generates a suite of tests for [`DirectedGraph`][crate::graph::DirectedGraph]
/// implementations, which runs on the given test graph.
///
/// At least one test is generated for each method in the
/// [`DirectedGraph`][crate::graph::DirectedGraph] trait. The expected results are derived from the
/// test graph, so the suite can be run on any test graph that the graph type is able to represent.
/// The following invariants are expected from the graph implementation:
/// - If the most recently added node or edge is removed, its ID will no longer be valid. I.e.,
///   calling methods with that ID should return `None` or otherwise indicate non-existence.
///
/// The arguments to this macro are as follows (`G` is used to denote the graph type being tested):
/// - `$graph_constructor`: `impl Fn() -> G`. An expression that constructs a new instance of the
///   graph type to be tested. The generated graph must be empty (e.g. `G::new()`).
/// - `$test_graph`: [`TestGraph`][crate::utils::testing::__common::TestGraph]. The test graph,
///   which is built by adding its nodes and edges to the graphs returned by `$graph_constructor`.
///
/// Prefer using [`test_directed_graph`][crate::test_directed_graph] to run the suite on the
/// default test graph.
#[doc(hidden)]
#[macro_export]
macro_rules! __test_directed_graph_inner {
    ($graph_constructor:expr, $test_graph:expr,) => {
        #[test]
        fn test_density_hint() {
            $crate::utils::testing::directed::__private_testing::test_density_hint_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_cardinality() {
            $crate::utils::testing::directed::__private_testing::test_cardinality_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_nodes() {
            $crate::utils::testing::directed::__private_testing::test_nodes_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_nodes_mut() {
            $crate::utils::testing::directed::__private_testing::test_nodes_mut_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_isolated_nodes() {
            $crate::utils::testing::directed::__private_testing::test_isolated_nodes_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_edges() {
            $crate::utils::testing::directed::__private_testing::test_edges_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_edges_mut() {
            $crate::utils::testing::directed::__private_testing::test_edges_mut_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_node() {
            $crate::utils::testing::directed::__private_testing::test_node_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_node_mut() {
            $crate::utils::testing::directed::__private_testing::test_node_mut_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_edge() {
            $crate::utils::testing::directed::__private_testing::test_edge_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_edge_mut() {
            $crate::utils::testing::directed::__private_testing::test_edge_mut_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_in_degree() {
            $crate::utils::testing::directed::__private_testing::test_in_degree_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_out_degree() {
            $crate::utils::testing::directed::__private_testing::test_out_degree_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_degree() {
            $crate::utils::testing::directed::__private_testing::test_degree_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_incoming_edges() {
            $crate::utils::testing::directed::__private_testing::test_incoming_edges_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_incoming_edges_mut() {
            $crate::utils::testing::directed::__private_testing::test_incoming_edges_mut_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_outgoing_edges() {
            $crate::utils::testing::directed::__private_testing::test_outgoing_edges_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_outgoing_edges_mut() {
            $crate::utils::testing::directed::__private_testing::test_outgoing_edges_mut_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_incident_edges() {
            $crate::utils::testing::directed::__private_testing::test_incident_edges_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_incident_edges_mut() {
            $crate::utils::testing::directed::__private_testing::test_incident_edges_mut_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_predecessors() {
            $crate::utils::testing::directed::__private_testing::test_predecessors_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_successors() {
            $crate::utils::testing::directed::__private_testing::test_successors_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_adjacencies() {
            $crate::utils::testing::directed::__private_testing::test_adjacencies_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_edges_between() {
            $crate::utils::testing::directed::__private_testing::test_edges_between_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_edges_between_mut() {
            $crate::utils::testing::directed::__private_testing::test_edges_between_mut_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_edges_connecting() {
            $crate::utils::testing::directed::__private_testing::test_edges_connecting_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_edges_connecting_mut() {
            $crate::utils::testing::directed::__private_testing::test_edges_connecting_mut_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_contains_node() {
            $crate::utils::testing::directed::__private_testing::test_contains_node_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_contains_edge() {
            $crate::utils::testing::directed::__private_testing::test_contains_edge_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_is_adjacent() {
            $crate::utils::testing::directed::__private_testing::test_is_adjacent_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_is_empty() {
            $crate::utils::testing::directed::__private_testing::test_is_empty_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_sources() {
            $crate::utils::testing::directed::__private_testing::test_sources_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_sinks() {
            $crate::utils::testing::directed::__private_testing::test_sinks_inner(
                $graph_constructor,
                $test_graph,
            );
        }
    };
}

// WARNING: This module contains only internal testing helper functions and should not be considered
// part of the public API. It is subject to change at any time.
#[doc(hidden)]
pub mod __private_testing {
    use core::hash::Hash;

    use crate::{
        graph::{DirectedGraph, EditableGraph},
        utils::testing::{TestGraph, check_if_edges_match, check_if_nodes_match},
    };

    pub fn test_cardinality_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) {
        let (mut graph, nodes, edges) = test_graph.build(graph_constructor);
        assert_eq!(
            DirectedGraph::node_count(&graph),
            test_graph.node_count,
            "DirectedGraph::node_count() did not match expected value"
        );
        assert_eq!(
            DirectedGraph::edge_count(&graph),
            test_graph.edge_count(),
            "DirectedGraph::edge_count() did not match expected value"
        );

        let cardinality = DirectedGraph::cardinality(&graph);
        assert_eq!(
            cardinality.order, test_graph.node_count,
            "DirectedGraph::cardinality().order did not match expected value"
        );
        assert_eq!(
            cardinality.size,
            test_graph.edge_count(),
            "DirectedGraph::cardinality().size did not match expected value"
        );

        let Some(&first_node) = nodes.first() else {
            return;
        };

        graph.remove_node(first_node).unwrap();
        assert_eq!(
            DirectedGraph::node_count(&graph),
            test_graph.node_count - 1,
            "DirectedGraph::node_count() did not match expected value after removing node 0"
        );

        assert_eq!(
            DirectedGraph::edge_count(&graph),
            test_graph
                .expected_edges(&edges, |source, target| source != 0 && target != 0)
                .count(),
            "DirectedGraph::edge_count() did not match expected value after removing node 0"
        );
    }

    pub fn test_density_hint_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) {
        let (graph, _, _) = test_graph.build(graph_constructor);

        // DensityHint is a hint, so this is intentionally only a smoke test.
        let _density_hint = DirectedGraph::density_hint(&graph);
    }

    pub fn test_nodes_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) {
        let (graph, nodes, _) = test_graph.build(graph_constructor);
        let nodes_count = DirectedGraph::nodes(&graph).count();
        assert_eq!(
            nodes_count, test_graph.node_count,
            "DirectedGraph::nodes().count() did not match expected value"
        );
        for node in DirectedGraph::nodes(&graph) {
            assert!(
                nodes.contains(&node.id),
                "DirectedGraph::nodes() contained unexpected node id: {:?}",
                node.id
            );
        }
    }

    pub fn test_nodes_mut_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) {
        let (mut graph, nodes, _) = test_graph.build(graph_constructor);
        let nodes_count = DirectedGraph::nodes_mut(&mut graph).count();
        assert_eq!(
            nodes_count, test_graph.node_count,
            "DirectedGraph::nodes_mut().count() did not match expected value"
        );
        for node in DirectedGraph::nodes_mut(&mut graph) {
            assert!(
                nodes.contains(&node.id),
                "DirectedGraph::nodes_mut() contained unexpected node id: {:?}",
                node.id
            );
        }
    }

    pub fn test_isolated_nodes_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (graph, nodes, _) = test_graph.build(graph_constructor);
        check_if_nodes_match(
            test_graph.expected_nodes(&nodes, |node| {
                !test_graph.has_edge(|source, target| source == node || target == node)
            }),
            DirectedGraph::isolated_nodes(&graph).map(|node| node.id),
            "isolated_nodes",
            "DirectedGraph",
            "graph",
        );
    }

    pub fn test_edges_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (graph, nodes, edges) = test_graph.build(graph_constructor);
        let edges_count = DirectedGraph::edges(&graph).count();
        assert_eq!(
            edges_count,
            test_graph.edge_count(),
            "DirectedGraph::edges().count() did not match expected value"
        );
        check_if_edges_match(
            test_graph.expected_edge_endpoints(&nodes, &edges),
            DirectedGraph::edges(&graph).map(|edge| (edge.id, edge.source, edge.target)),
            "edges",
            "DirectedGraph",
            "graph",
        );
    }

    pub fn test_edges_mut_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (mut graph, nodes, edges) = test_graph.build(graph_constructor);
        let edges_count = DirectedGraph::edges_mut(&mut graph).count();
        assert_eq!(
            edges_count,
            test_graph.edge_count(),
            "DirectedGraph::edges_mut().count() did not match expected value"
        );
        check_if_edges_match(
            test_graph.expected_edge_endpoints(&nodes, &edges),
            DirectedGraph::edges_mut(&mut graph).map(|edge| (edge.id, edge.source, edge.target)),
            "edges_mut",
            "DirectedGraph",
            "graph",
        );
    }

    pub fn test_node_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) {
        let (mut graph, nodes, _) = test_graph.build(graph_constructor);
        for &node_id in &nodes {
            let node =
                DirectedGraph::node(&graph, node_id).expect("Expected node not found in test_node");
            assert_eq!(
                node.id, node_id,
                "DirectedGraph::node() did not return expected node id"
            );
        }

        let Some(&last_node) = nodes.last() else {
            return;
        };

        graph.remove_node(last_node).unwrap();
        assert!(
            DirectedGraph::node(&graph, last_node).is_none(),
            "DirectedGraph::node() did not return None for removed node id"
        );
    }

    pub fn test_node_mut_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) {
        let (mut graph, nodes, _) = test_graph.build(graph_constructor);
        for &node_id in &nodes {
            let node = DirectedGraph::node_mut(&mut graph, node_id)
                .expect("Expected node not found in test_node_mut");
            assert_eq!(
                node.id, node_id,
                "DirectedGraph::node_mut() did not return expected node id"
            );
        }

        let Some(&last_node) = nodes.last() else {
            return;
        };

        graph.remove_node(last_node).unwrap();
        assert!(
            DirectedGraph::node_mut(&mut graph, last_node).is_none(),
            "DirectedGraph::node_mut() did not return None for removed node id"
        );
    }

    pub fn test_edge_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) {
        let (mut graph, _, edges) = test_graph.build(graph_constructor);
        for &edge_id in &edges {
            let edge =
                DirectedGraph::edge(&graph, edge_id).expect("Expected edge not found in test_edge");
            assert_eq!(
                edge.id, edge_id,
                "DirectedGraph::edge() did not return expected edge id"
            );
        }

        let Some(&last_edge) = edges.last() else {
            return;
        };

        graph.remove_edge(last_edge).unwrap();
        assert!(
            DirectedGraph::edge(&graph, last_edge).is_none(),
            "DirectedGraph::edge() did not return None for removed edge id"
        );
    }

    pub fn test_edge_mut_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) {
        let (mut graph, _, edges) = test_graph.build(graph_constructor);
        for &edge_id in &edges {
            let edge = DirectedGraph::edge_mut(&mut graph, edge_id)
                .expect("Expected edge not found in test_edge_mut");
            assert_eq!(
                edge.id, edge_id,
                "DirectedGraph::edge_mut() did not return expected edge id"
            );
        }

        let Some(&last_edge) = edges.last() else {
            return;
        };

        graph.remove_edge(last_edge).unwrap();
        assert!(
            DirectedGraph::edge_mut(&mut graph, last_edge).is_none(),
            "DirectedGraph::edge_mut() did not return None for removed edge id"
        );
    }

    pub fn test_in_degree_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) {
        let (graph, nodes, edges) = test_graph.build(graph_constructor);
        for (node_number, &node) in nodes.iter().enumerate() {
            assert_eq!(
                DirectedGraph::in_degree(&graph, node),
                test_graph
                    .expected_edges(&edges, |_, target| target == node_number)
                    .count(),
                "DirectedGraph::in_degree() did not return expected value for node {node_number}"
            );
        }
    }

    pub fn test_out_degree_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) {
        let (graph, nodes, edges) = test_graph.build(graph_constructor);
        for (node_number, &node) in nodes.iter().enumerate() {
            assert_eq!(
                DirectedGraph::out_degree(&graph, node),
                test_graph
                    .expected_edges(&edges, |source, _| source == node_number)
                    .count(),
                "DirectedGraph::out_degree() did not return expected value for node {node_number}"
            );
        }
    }

    pub fn test_degree_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) {
        let (graph, nodes, edges) = test_graph.build(graph_constructor);
        for (node_number, &node) in nodes.iter().enumerate() {
            // An edge counts once for each of its endpoints, so self-loops count twice.
            let expected_degree = test_graph
                .expected_edges(&edges, |source, _| source == node_number)
                .count()
                + test_graph
                    .expected_edges(&edges, |_, target| target == node_number)
                    .count();
            assert_eq!(
                DirectedGraph::degree(&graph, node),
                expected_degree,
                "DirectedGraph::degree() did not return expected value for node {node_number}"
            );
        }
    }

    pub fn test_incoming_edges_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (graph, nodes, edges) = test_graph.build(graph_constructor);
        for (node_number, &node) in nodes.iter().enumerate() {
            check_if_edges_match(
                test_graph.expected_edges(&edges, |_, target| target == node_number),
                DirectedGraph::incoming_edges(&graph, node).map(|edge| edge.id),
                "incoming_edges",
                "DirectedGraph",
                format_args!("node {node_number}"),
            );
        }
    }

    pub fn test_incoming_edges_mut_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (mut graph, nodes, edges) = test_graph.build(graph_constructor);
        for (node_number, &node) in nodes.iter().enumerate() {
            check_if_edges_match(
                test_graph.expected_edges(&edges, |_, target| target == node_number),
                DirectedGraph::incoming_edges_mut(&mut graph, node).map(|edge| edge.id),
                "incoming_edges_mut",
                "DirectedGraph",
                format_args!("node {node_number}"),
            );
        }
    }

    pub fn test_outgoing_edges_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (graph, nodes, edges) = test_graph.build(graph_constructor);
        for (node_number, &node) in nodes.iter().enumerate() {
            check_if_edges_match(
                test_graph.expected_edges(&edges, |source, _| source == node_number),
                DirectedGraph::outgoing_edges(&graph, node).map(|edge| edge.id),
                "outgoing_edges",
                "DirectedGraph",
                format_args!("node {node_number}"),
            );
        }
    }

    pub fn test_outgoing_edges_mut_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (mut graph, nodes, edges) = test_graph.build(graph_constructor);
        for (node_number, &node) in nodes.iter().enumerate() {
            check_if_edges_match(
                test_graph.expected_edges(&edges, |source, _| source == node_number),
                DirectedGraph::outgoing_edges_mut(&mut graph, node).map(|edge| edge.id),
                "outgoing_edges_mut",
                "DirectedGraph",
                format_args!("node {node_number}"),
            );
        }
    }

    pub fn test_incident_edges_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (graph, nodes, edges) = test_graph.build(graph_constructor);
        for (node_number, &node) in nodes.iter().enumerate() {
            check_if_edges_match(
                test_graph.expected_edges(&edges, |source, target| {
                    source == node_number || target == node_number
                }),
                DirectedGraph::incident_edges(&graph, node).map(|edge| edge.id),
                "incident_edges",
                "DirectedGraph",
                format_args!("node {node_number}"),
            );
        }
    }

    pub fn test_incident_edges_mut_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (mut graph, nodes, edges) = test_graph.build(graph_constructor);
        for (node_number, &node) in nodes.iter().enumerate() {
            check_if_edges_match(
                test_graph.expected_edges(&edges, |source, target| {
                    source == node_number || target == node_number
                }),
                DirectedGraph::incident_edges_mut(&mut graph, node).map(|edge| edge.id),
                "incident_edges_mut",
                "DirectedGraph",
                format_args!("node {node_number}"),
            );
        }
    }

    pub fn test_predecessors_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (graph, nodes, _) = test_graph.build(graph_constructor);
        for (node_number, &node) in nodes.iter().enumerate() {
            check_if_nodes_match(
                test_graph.expected_neighbors(&nodes, |source, target| {
                    (target == node_number).then_some(source)
                }),
                DirectedGraph::predecessors(&graph, node),
                "predecessors",
                "DirectedGraph",
                format_args!("node {node_number}"),
            );
        }
    }

    pub fn test_successors_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (graph, nodes, _) = test_graph.build(graph_constructor);
        for (node_number, &node) in nodes.iter().enumerate() {
            check_if_nodes_match(
                test_graph.expected_neighbors(&nodes, |source, target| {
                    (source == node_number).then_some(target)
                }),
                DirectedGraph::successors(&graph, node),
                "successors",
                "DirectedGraph",
                format_args!("node {node_number}"),
            );
        }
    }

    pub fn test_adjacencies_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (graph, nodes, _) = test_graph.build(graph_constructor);
        for (node_number, &node) in nodes.iter().enumerate() {
            let expected_predecessors = test_graph.expected_neighbors(&nodes, |source, target| {
                (target == node_number).then_some(source)
            });
            let expected_successors = test_graph.expected_neighbors(&nodes, |source, target| {
                (source == node_number).then_some(target)
            });
            check_if_nodes_match(
                expected_predecessors.chain(expected_successors),
                DirectedGraph::adjacencies(&graph, node),
                "adjacencies",
                "DirectedGraph",
                format_args!("node {node_number}"),
            );
        }
    }

    pub fn test_edges_between_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (graph, nodes, edges) = test_graph.build(graph_constructor);

        for lhs_number in 0..nodes.len() {
            for rhs_number in 0..nodes.len() {
                check_if_edges_match(
                    test_graph.expected_edges(&edges, |source, target| {
                        (source, target) == (lhs_number, rhs_number)
                    }),
                    DirectedGraph::edges_between(&graph, nodes[lhs_number], nodes[rhs_number])
                        .map(|edge| edge.id),
                    "edges_between",
                    "DirectedGraph",
                    format_args!("node pair ({lhs_number}, {rhs_number})"),
                );
            }
        }
    }

    pub fn test_edges_between_mut_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (mut graph, nodes, edges) = test_graph.build(graph_constructor);

        for lhs_number in 0..nodes.len() {
            for rhs_number in 0..nodes.len() {
                check_if_edges_match(
                    test_graph.expected_edges(&edges, |source, target| {
                        (source, target) == (lhs_number, rhs_number)
                    }),
                    DirectedGraph::edges_between_mut(
                        &mut graph,
                        nodes[lhs_number],
                        nodes[rhs_number],
                    )
                    .map(|edge| edge.id),
                    "edges_between_mut",
                    "DirectedGraph",
                    format_args!("node pair ({lhs_number}, {rhs_number})"),
                );
            }
        }
    }

    pub fn test_edges_connecting_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (graph, nodes, edges) = test_graph.build(graph_constructor);

        for lhs_number in 0..nodes.len() {
            for rhs_number in 0..nodes.len() {
                check_if_edges_match(
                    test_graph.expected_edges(&edges, |source, target| {
                        (source, target) == (lhs_number, rhs_number)
                            || (target, source) == (lhs_number, rhs_number)
                    }),
                    DirectedGraph::edges_connecting(&graph, nodes[lhs_number], nodes[rhs_number])
                        .map(|edge| edge.id),
                    "edges_connecting",
                    "DirectedGraph",
                    format_args!("node pair ({lhs_number}, {rhs_number})"),
                );
            }
        }
    }

    pub fn test_edges_connecting_mut_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (mut graph, nodes, edges) = test_graph.build(graph_constructor);

        for lhs_number in 0..nodes.len() {
            for rhs_number in 0..nodes.len() {
                check_if_edges_match(
                    test_graph.expected_edges(&edges, |source, target| {
                        (source, target) == (lhs_number, rhs_number)
                            || (target, source) == (lhs_number, rhs_number)
                    }),
                    DirectedGraph::edges_connecting_mut(
                        &mut graph,
                        nodes[lhs_number],
                        nodes[rhs_number],
                    )
                    .map(|edge| edge.id),
                    "edges_connecting_mut",
                    "DirectedGraph",
                    format_args!("node pair ({lhs_number}, {rhs_number})"),
                );
            }
        }
    }

    pub fn test_contains_node_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) {
        let (mut graph, nodes, _) = test_graph.build(graph_constructor);

        for node in &nodes {
            assert!(
                DirectedGraph::contains_node(&graph, *node),
                "DirectedGraph::contains_node() returned false for existing node: {node:?}"
            );
        }

        let Some(&last_node) = nodes.last() else {
            return;
        };

        graph.remove_node(last_node).unwrap();
        assert!(
            !DirectedGraph::contains_node(&graph, last_node),
            "DirectedGraph::contains_node() returned true for removed node: {last_node:?}"
        );
    }

    pub fn test_contains_edge_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) {
        let (mut graph, _, edges) = test_graph.build(graph_constructor);

        for edge in &edges {
            assert!(
                DirectedGraph::contains_edge(&graph, *edge),
                "DirectedGraph::contains_edge() returned false for existing edge: {edge:?}",
            );
        }

        let Some(&last_edge) = edges.last() else {
            return;
        };

        graph.remove_edge(last_edge).unwrap();
        assert!(
            !DirectedGraph::contains_edge(&graph, last_edge),
            "DirectedGraph::contains_edge() returned true for removed edge: {last_edge:?}"
        );
    }

    pub fn test_is_adjacent_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) {
        let (graph, nodes, _) = test_graph.build(graph_constructor);

        for (source_number, &source) in nodes.iter().enumerate() {
            for (target_number, &target) in nodes.iter().enumerate() {
                let expected_adjacency = test_graph
                    .edge_endpoints
                    .contains(&(source_number, target_number));
                assert_eq!(
                    DirectedGraph::is_adjacent(&graph, source, target),
                    expected_adjacency,
                    "DirectedGraph::is_adjacent() returned incorrect result for nodes {source:?} \
                     and {target:?}",
                );
            }
        }
    }

    pub fn test_is_empty_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) {
        let (graph, _, _) = test_graph.build(graph_constructor);

        assert_eq!(
            DirectedGraph::is_empty(&graph),
            test_graph.node_count == 0,
            "DirectedGraph::is_empty() did not match expected value for the test graph"
        );

        let mut new_graph = graph_constructor();
        assert!(
            DirectedGraph::is_empty(&new_graph),
            "DirectedGraph::is_empty() returned false for an empty graph"
        );

        let node_one = new_graph.add_node(()).unwrap();
        assert!(
            !DirectedGraph::is_empty(&new_graph),
            "DirectedGraph::is_empty() returned true for a graph with one node"
        );

        new_graph.remove_node(node_one).unwrap();
        assert!(
            DirectedGraph::is_empty(&new_graph),
            "DirectedGraph::is_empty() returned false for an empty graph after removing the only \
             node"
        );
    }

    pub fn test_sources_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (graph, nodes, _) = test_graph.build(graph_constructor);

        check_if_nodes_match(
            test_graph.expected_nodes(&nodes, |node| {
                !test_graph.has_edge(|_, target| target == node)
            }),
            DirectedGraph::sources(&graph).map(|node| node.id),
            "sources",
            "DirectedGraph",
            "graph",
        );
    }

    pub fn test_sinks_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (graph, nodes, _) = test_graph.build(graph_constructor);

        check_if_nodes_match(
            test_graph.expected_nodes(&nodes, |node| {
                !test_graph.has_edge(|source, _| source == node)
            }),
            DirectedGraph::sinks(&graph).map(|node| node.id),
            "sinks",
            "DirectedGraph",
            "graph",
        );
    }
}
