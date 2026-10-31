/// Generates a suite of tests for [`UndirectedGraph`][crate::graph::UndirectedGraph]
/// implementations. It runs on the
/// [`DEFAULT_TEST_GRAPH`][crate::utils::testing::__common::DEFAULT_TEST_GRAPH] and
/// [`ISOLATED_TEST_GRAPH`][crate::utils::testing::__common::ISOLATED_TEST_GRAPH].
///
/// For the expected invariants and the arguments of this macro, see
/// [`__test_undirected_graph_inner`][crate::__test_undirected_graph_inner], which can also be used
/// to run the suite on other test graphs.
///
/// For a reference usage, see the tests in [`crate::utils::test_graphs::undirected`].
#[macro_export]
macro_rules! test_undirected_graph {
    ($graph_constructor:expr,) => {
        mod undirected_graph_macro {
            mod default {
                #[allow(unused_imports)]
                use super::super::*;

                $crate::__test_undirected_graph_inner!(
                    $graph_constructor,
                    $crate::utils::testing::__common::DEFAULT_TEST_GRAPH,
                );
            }

            mod isolated {
                #[allow(unused_imports)]
                use super::super::*;

                $crate::__test_undirected_graph_inner!(
                    $graph_constructor,
                    $crate::utils::testing::__common::ISOLATED_TEST_GRAPH,
                );
            }
        }
    };
}

/// Generates a suite of tests for [`UndirectedGraph`][crate::graph::UndirectedGraph]
/// implementations of graphs which support self-loops. It runs on the
/// [`LOOP_TEST_GRAPH`][crate::utils::testing::__common::LOOP_TEST_GRAPH].
///
/// For the expected invariants and the arguments of this macro, see
/// [`__test_undirected_graph_inner`][crate::__test_undirected_graph_inner], which can also be used
/// to run the suite on other test graphs.
///
/// For a reference usage, see the tests in [`crate::utils::test_graphs::undirected`].
#[macro_export]
macro_rules! test_undirected_graph_self_loops {
    ($graph_constructor:expr,) => {
        mod undirected_graph_macro_self_loops {
            #[allow(unused_imports)]
            use super::*;

            $crate::__test_undirected_graph_inner!(
                $graph_constructor,
                $crate::utils::testing::__common::LOOP_TEST_GRAPH,
            );
        }
    };
}

/// Generates a suite of tests for [`UndirectedGraph`][crate::graph::UndirectedGraph]
/// implementations of graphs which support self-loops and parallel edges. It runs on the
/// [`MULTI_TEST_GRAPH`][crate::utils::testing::__common::MULTI_TEST_GRAPH].
///
/// For the expected invariants and the arguments of this macro, see
/// [`__test_undirected_graph_inner`][crate::__test_undirected_graph_inner], which can also be used
/// to run the suite on other test graphs.
///
/// For a reference usage, see the tests in [`crate::utils::test_graphs::undirected`].
#[macro_export]
macro_rules! test_undirected_graph_parallel_edges {
    ($graph_constructor:expr,) => {
        mod undirected_graph_macro_parallel_edges {
            #[allow(unused_imports)]
            use super::*;

            $crate::__test_undirected_graph_inner!(
                $graph_constructor,
                $crate::utils::testing::__common::MULTI_TEST_GRAPH,
            );
        }
    };
}

/// Generates a suite of tests for [`UndirectedGraph`][crate::graph::UndirectedGraph]
/// implementations, which runs on the given test graph.
///
/// At least one test is generated for each method in the
/// [`UndirectedGraph`][crate::graph::UndirectedGraph] trait. The expected results are derived from
/// the test graph, so the suite can be run on any test graph that the graph type is able to
/// represent. The following invariants are expected from the graph implementation:
/// - If the most recently added node or edge is removed, its ID will no longer be valid. I.e.,
///   calling methods with that ID should return `None` or otherwise indicate non-existence.
///
/// The arguments to this macro are as follows (`G` is used to denote the graph type being tested):
/// - `$graph_constructor`: `impl Fn() -> G`. An expression that constructs a new instance of the
///   graph type to be tested. The generated graph must be empty (e.g. `G::new()`).
/// - `$test_graph`: [`TestGraph`][crate::utils::testing::__common::TestGraph]. The test graph,
///   which is built by adding its nodes and edges to the graphs returned by `$graph_constructor`.
///
/// Prefer using [`test_undirected_graph`][crate::test_undirected_graph] to run the suite on the
/// default test graph.
#[doc(hidden)]
#[macro_export]
macro_rules! __test_undirected_graph_inner {
    ($graph_constructor:expr, $test_graph:expr,) => {
        #[test]
        fn test_density_hint() {
            $crate::utils::testing::undirected::__private_testing::test_density_hint_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_cardinality() {
            $crate::utils::testing::undirected::__private_testing::test_cardinality_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_nodes() {
            $crate::utils::testing::undirected::__private_testing::test_nodes_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_nodes_mut() {
            $crate::utils::testing::undirected::__private_testing::test_nodes_mut_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_isolated_nodes() {
            $crate::utils::testing::undirected::__private_testing::test_isolated_nodes_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_edges() {
            $crate::utils::testing::undirected::__private_testing::test_edges_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_edges_mut() {
            $crate::utils::testing::undirected::__private_testing::test_edges_mut_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_node() {
            $crate::utils::testing::undirected::__private_testing::test_node_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_node_mut() {
            $crate::utils::testing::undirected::__private_testing::test_node_mut_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_edge() {
            $crate::utils::testing::undirected::__private_testing::test_edge_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_edge_mut() {
            $crate::utils::testing::undirected::__private_testing::test_edge_mut_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_degree() {
            $crate::utils::testing::undirected::__private_testing::test_degree_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_incident_edges() {
            $crate::utils::testing::undirected::__private_testing::test_incident_edges_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_incident_edges_mut() {
            $crate::utils::testing::undirected::__private_testing::test_incident_edges_mut_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_adjacencies() {
            $crate::utils::testing::undirected::__private_testing::test_adjacencies_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_edges_connecting() {
            $crate::utils::testing::undirected::__private_testing::test_edges_connecting_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_edges_connecting_mut() {
            $crate::utils::testing::undirected::__private_testing::test_edges_connecting_mut_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_contains_node() {
            $crate::utils::testing::undirected::__private_testing::test_contains_node_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_contains_edge() {
            $crate::utils::testing::undirected::__private_testing::test_contains_edge_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_is_adjacent() {
            $crate::utils::testing::undirected::__private_testing::test_is_adjacent_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_is_empty() {
            $crate::utils::testing::undirected::__private_testing::test_is_empty_inner(
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
        graph::{EditableGraph, UndirectedGraph},
        id::Id,
        utils::testing::{TestGraph, check_if_edges_match, check_if_nodes_match},
    };

    /// Returns the given `(id, source, target)` triple of an edge with its endpoints swapped, if
    /// the swapped triple is contained in `expected_endpoints`. This is needed, as undirected
    /// graphs may return the endpoints of an edge in either order.
    fn normalize_endpoints<N: Id, E: Id>(
        expected_endpoints: &[(E, N, N)],
        (id, source, target): (E, N, N),
    ) -> (E, N, N) {
        if expected_endpoints.contains(&(id, target, source)) {
            (id, target, source)
        } else {
            (id, source, target)
        }
    }

    pub fn test_density_hint_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) {
        let (graph, _, _) = test_graph.build(graph_constructor);

        // DensityHint is a hint, so this is intentionally only a smoke test.
        let _density_hint = UndirectedGraph::density_hint(&graph);
    }

    pub fn test_cardinality_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) {
        let (mut graph, nodes, edges) = test_graph.build(graph_constructor);

        assert_eq!(
            UndirectedGraph::node_count(&graph),
            test_graph.node_count,
            "UndirectedGraph::node_count() did not match expected value"
        );
        assert_eq!(
            UndirectedGraph::edge_count(&graph),
            test_graph.edge_count(),
            "UndirectedGraph::edge_count() did not match expected value"
        );

        let cardinality = UndirectedGraph::cardinality(&graph);
        assert_eq!(
            cardinality.order, test_graph.node_count,
            "UndirectedGraph::cardinality().order did not match expected value"
        );
        assert_eq!(
            cardinality.size,
            test_graph.edge_count(),
            "UndirectedGraph::cardinality().size did not match expected value"
        );

        let Some(&first_node) = nodes.first() else {
            return;
        };

        graph.remove_node(first_node).unwrap();

        assert_eq!(
            UndirectedGraph::node_count(&graph),
            test_graph.node_count - 1,
            "UndirectedGraph::node_count() did not match expected value after removing node 0"
        );

        assert_eq!(
            UndirectedGraph::edge_count(&graph),
            test_graph
                .expected_edges(&edges, |source, target| source != 0 && target != 0)
                .count(),
            "UndirectedGraph::edge_count() did not match expected value after removing node 0"
        );
    }

    pub fn test_nodes_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (graph, nodes, _) = test_graph.build(graph_constructor);

        assert_eq!(
            UndirectedGraph::nodes(&graph).count(),
            test_graph.node_count,
            "UndirectedGraph::nodes().count() did not match expected value"
        );

        check_if_nodes_match(
            nodes.iter().copied(),
            UndirectedGraph::nodes(&graph).map(|node| node.id),
            "nodes",
            "UndirectedGraph",
            "graph",
        );
    }

    pub fn test_nodes_mut_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (mut graph, nodes, _) = test_graph.build(graph_constructor);

        assert_eq!(
            UndirectedGraph::nodes_mut(&mut graph).count(),
            test_graph.node_count,
            "UndirectedGraph::nodes_mut().count() did not match expected value"
        );

        check_if_nodes_match(
            nodes.iter().copied(),
            UndirectedGraph::nodes_mut(&mut graph).map(|node| node.id),
            "nodes_mut",
            "UndirectedGraph",
            "graph",
        );
    }

    pub fn test_isolated_nodes_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
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
            UndirectedGraph::isolated_nodes(&graph).map(|node| node.id),
            "isolated_nodes",
            "UndirectedGraph",
            "graph",
        );
    }

    pub fn test_edges_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (graph, nodes, edges) = test_graph.build(graph_constructor);

        assert_eq!(
            UndirectedGraph::edges(&graph).count(),
            test_graph.edge_count(),
            "UndirectedGraph::edges().count() did not match expected value"
        );

        let expected_endpoints = test_graph.expected_edge_endpoints(&nodes, &edges);
        check_if_edges_match(
            expected_endpoints.iter().copied(),
            UndirectedGraph::edges(&graph).map(|edge| {
                normalize_endpoints(&expected_endpoints, (edge.id, edge.source, edge.target))
            }),
            "edges",
            "UndirectedGraph",
            "graph",
        );
    }

    pub fn test_edges_mut_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (mut graph, nodes, edges) = test_graph.build(graph_constructor);

        assert_eq!(
            UndirectedGraph::edges_mut(&mut graph).count(),
            test_graph.edge_count(),
            "UndirectedGraph::edges_mut().count() did not match expected value"
        );

        let expected_endpoints = test_graph.expected_edge_endpoints(&nodes, &edges);
        check_if_edges_match(
            expected_endpoints.iter().copied(),
            UndirectedGraph::edges_mut(&mut graph).map(|edge| {
                normalize_endpoints(&expected_endpoints, (edge.id, edge.source, edge.target))
            }),
            "edges_mut",
            "UndirectedGraph",
            "graph",
        );
    }

    pub fn test_node_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) {
        let (mut graph, nodes, _) = test_graph.build(graph_constructor);

        for &node_id in &nodes {
            let node = UndirectedGraph::node(&graph, node_id)
                .expect("Expected node not found in test_node");
            assert_eq!(
                node.id, node_id,
                "UndirectedGraph::node() did not return expected node id"
            );
        }

        let Some(&last_node) = nodes.last() else {
            return;
        };

        graph.remove_node(last_node).unwrap();

        assert!(
            UndirectedGraph::node(&graph, last_node).is_none(),
            "UndirectedGraph::node() did not return None for removed node id"
        );
    }

    pub fn test_node_mut_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) {
        let (mut graph, nodes, _) = test_graph.build(graph_constructor);

        for &node_id in &nodes {
            let node = UndirectedGraph::node_mut(&mut graph, node_id)
                .expect("Expected node not found in test_node_mut");
            assert_eq!(
                node.id, node_id,
                "UndirectedGraph::node_mut() did not return expected node id"
            );
        }

        let Some(&last_node) = nodes.last() else {
            return;
        };

        graph.remove_node(last_node).unwrap();

        assert!(
            UndirectedGraph::node_mut(&mut graph, last_node).is_none(),
            "UndirectedGraph::node_mut() did not return None for removed node id"
        );
    }

    pub fn test_edge_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) {
        let (mut graph, _, edges) = test_graph.build(graph_constructor);

        for &edge_id in &edges {
            let edge = UndirectedGraph::edge(&graph, edge_id)
                .expect("Expected edge not found in test_edge");
            assert_eq!(
                edge.id, edge_id,
                "UndirectedGraph::edge() did not return expected edge id"
            );
        }

        let Some(&last_edge) = edges.last() else {
            return;
        };

        graph.remove_edge(last_edge).unwrap();

        assert!(
            UndirectedGraph::edge(&graph, last_edge).is_none(),
            "UndirectedGraph::edge() did not return None for removed edge id"
        );
    }

    pub fn test_edge_mut_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) {
        let (mut graph, _, edges) = test_graph.build(graph_constructor);

        for &edge_id in &edges {
            let edge = UndirectedGraph::edge_mut(&mut graph, edge_id)
                .expect("Expected edge not found in test_edge_mut");
            assert_eq!(
                edge.id, edge_id,
                "UndirectedGraph::edge_mut() did not return expected edge id"
            );
        }

        let Some(&last_edge) = edges.last() else {
            return;
        };

        graph.remove_edge(last_edge).unwrap();

        assert!(
            UndirectedGraph::edge_mut(&mut graph, last_edge).is_none(),
            "UndirectedGraph::edge_mut() did not return None for removed edge id"
        );
    }

    pub fn test_degree_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
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
                UndirectedGraph::degree(&graph, node),
                expected_degree,
                "UndirectedGraph::degree() did not return expected value for node {node_number}"
            );
        }
    }

    pub fn test_incident_edges_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
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
                UndirectedGraph::incident_edges(&graph, node).map(|edge| edge.id),
                "incident_edges",
                "UndirectedGraph",
                format_args!("node {node_number}"),
            );
        }
    }

    pub fn test_incident_edges_mut_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
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
                UndirectedGraph::incident_edges_mut(&mut graph, node).map(|edge| edge.id),
                "incident_edges_mut",
                "UndirectedGraph",
                format_args!("node {node_number}"),
            );
        }
    }

    pub fn test_adjacencies_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
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
                    if source == node_number {
                        Some(target)
                    } else if target == node_number {
                        Some(source)
                    } else {
                        None
                    }
                }),
                UndirectedGraph::adjacencies(&graph, node),
                "adjacencies",
                "UndirectedGraph",
                format_args!("node {node_number}"),
            );
        }
    }

    pub fn test_edges_connecting_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
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
                    UndirectedGraph::edges_connecting(&graph, nodes[lhs_number], nodes[rhs_number])
                        .map(|edge| edge.id),
                    "edges_connecting",
                    "UndirectedGraph",
                    format_args!("node pair ({lhs_number}, {rhs_number})"),
                );
            }
        }
    }

    pub fn test_edges_connecting_mut_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
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
                    UndirectedGraph::edges_connecting_mut(
                        &mut graph,
                        nodes[lhs_number],
                        nodes[rhs_number],
                    )
                    .map(|edge| edge.id),
                    "edges_connecting_mut",
                    "UndirectedGraph",
                    format_args!("node pair ({lhs_number}, {rhs_number})"),
                );
            }
        }
    }

    pub fn test_contains_node_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) {
        let (mut graph, nodes, _) = test_graph.build(graph_constructor);

        for &node in &nodes {
            assert!(
                UndirectedGraph::contains_node(&graph, node),
                "UndirectedGraph::contains_node() returned false for existing node: {node:?}"
            );
        }

        let Some(&last_node) = nodes.last() else {
            return;
        };

        graph.remove_node(last_node).unwrap();

        assert!(
            !UndirectedGraph::contains_node(&graph, last_node),
            "UndirectedGraph::contains_node() returned true for removed node: {last_node:?}"
        );
    }

    pub fn test_contains_edge_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) {
        let (mut graph, _, edges) = test_graph.build(graph_constructor);

        for &edge in &edges {
            assert!(
                UndirectedGraph::contains_edge(&graph, edge),
                "UndirectedGraph::contains_edge() returned false for existing edge: {edge:?}"
            );
        }

        let Some(&last_edge) = edges.last() else {
            return;
        };

        graph.remove_edge(last_edge).unwrap();

        assert!(
            !UndirectedGraph::contains_edge(&graph, last_edge),
            "UndirectedGraph::contains_edge() returned true for removed edge: {last_edge:?}"
        );
    }

    pub fn test_is_adjacent_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) {
        let (graph, nodes, _) = test_graph.build(graph_constructor);

        for lhs_number in 0..nodes.len() {
            for rhs_number in 0..nodes.len() {
                let expected_adjacency = test_graph
                    .edge_endpoints
                    .contains(&(lhs_number, rhs_number))
                    || test_graph
                        .edge_endpoints
                        .contains(&(rhs_number, lhs_number));
                assert_eq!(
                    UndirectedGraph::is_adjacent(&graph, nodes[lhs_number], nodes[rhs_number],),
                    expected_adjacency,
                    "UndirectedGraph::is_adjacent() returned incorrect result for nodes {:?} and \
                     {:?}",
                    nodes[lhs_number],
                    nodes[rhs_number]
                );
            }
        }
    }

    pub fn test_is_empty_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) {
        let (graph, _, _) = test_graph.build(graph_constructor);

        assert_eq!(
            UndirectedGraph::is_empty(&graph),
            test_graph.node_count == 0,
            "UndirectedGraph::is_empty() did not match expected value for the test graph"
        );

        let mut new_graph = graph_constructor();

        assert!(
            UndirectedGraph::is_empty(&new_graph),
            "UndirectedGraph::is_empty() returned false for an empty graph"
        );

        let node_one = new_graph.add_node(()).unwrap();

        assert!(
            !UndirectedGraph::is_empty(&new_graph),
            "UndirectedGraph::is_empty() returned true for a graph with one node"
        );

        new_graph.remove_node(node_one).unwrap();

        assert!(
            UndirectedGraph::is_empty(&new_graph),
            "UndirectedGraph::is_empty() returned false for an empty graph after removing the \
             only node"
        );
    }
}
