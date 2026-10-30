/// Generates a suite of tests for [`DirectedGraph`][crate::graph::DirectedGraph]
/// implementations.
///
/// At least one test is generated for each method in the
/// [`DirectedGraph`][crate::graph::DirectedGraph] trait. The following invariants are expected from
/// the graph implementation:
/// - If the most recently added node or edge is removed, its ID will no longer be valid. I.e.,
///   calling methods with that ID should return `None` or otherwise indicate non-existence.
///
/// The arguments to this macro are as follows (`G` is used to denote the graph type being tested):
/// - `$graph_constructor`: `impl Fn() -> G`. An expression that constructs a new instance of the
///   graph type to be tested. The generated graph must be empty (e.g. `G::new()`).
///
/// For a reference usage, see the tests in [`crate::utils::test_graphs::directed`].
#[macro_export]
macro_rules! test_directed_graph {
    ($graph_constructor:expr,) => {
        mod directed_graph_test_macro {
            #[allow(unused_imports)]
            use super::*;

            #[test]
            fn test_density_hint() {
                $crate::utils::testing::directed::__private_testing::test_density_hint_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_cardinality() {
                $crate::utils::testing::directed::__private_testing::test_cardinality_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_nodes() {
                $crate::utils::testing::directed::__private_testing::test_nodes_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_nodes_mut() {
                $crate::utils::testing::directed::__private_testing::test_nodes_mut_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_isolated_nodes() {
                $crate::utils::testing::directed::__private_testing::test_isolated_nodes_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_edges() {
                $crate::utils::testing::directed::__private_testing::test_edges_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_edges_mut() {
                $crate::utils::testing::directed::__private_testing::test_edges_mut_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_node() {
                $crate::utils::testing::directed::__private_testing::test_node_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_node_mut() {
                $crate::utils::testing::directed::__private_testing::test_node_mut_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_edge() {
                $crate::utils::testing::directed::__private_testing::test_edge_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_edge_mut() {
                $crate::utils::testing::directed::__private_testing::test_edge_mut_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_in_degree() {
                $crate::utils::testing::directed::__private_testing::test_in_degree_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_out_degree() {
                $crate::utils::testing::directed::__private_testing::test_out_degree_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_degree() {
                $crate::utils::testing::directed::__private_testing::test_degree_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_incoming_edges() {
                $crate::utils::testing::directed::__private_testing::test_incoming_edges_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_incoming_edges_mut() {
                $crate::utils::testing::directed::__private_testing::test_incoming_edges_mut_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_outgoing_edges() {
                $crate::utils::testing::directed::__private_testing::test_outgoing_edges_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_outgoing_edges_mut() {
                $crate::utils::testing::directed::__private_testing::test_outgoing_edges_mut_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_incident_edges() {
                $crate::utils::testing::directed::__private_testing::test_incident_edges_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_incident_edges_mut() {
                $crate::utils::testing::directed::__private_testing::test_incident_edges_mut_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_predecessors() {
                $crate::utils::testing::directed::__private_testing::test_predecessors_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_successors() {
                $crate::utils::testing::directed::__private_testing::test_successors_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_adjacencies() {
                $crate::utils::testing::directed::__private_testing::test_adjacencies_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_edges_between() {
                $crate::utils::testing::directed::__private_testing::test_edges_between_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_edges_between_mut() {
                $crate::utils::testing::directed::__private_testing::test_edges_between_mut_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_edges_connecting() {
                $crate::utils::testing::directed::__private_testing::test_edges_connecting_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_edges_connecting_mut() {
                $crate::utils::testing::directed::__private_testing::test_edges_connecting_mut_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_contains_node() {
                $crate::utils::testing::directed::__private_testing::test_contains_node_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_contains_edge() {
                $crate::utils::testing::directed::__private_testing::test_contains_edge_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_is_adjacent() {
                $crate::utils::testing::directed::__private_testing::test_is_adjacent_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_is_empty() {
                $crate::utils::testing::directed::__private_testing::test_is_empty_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_sources() {
                $crate::utils::testing::directed::__private_testing::test_sources_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_sinks() {
                $crate::utils::testing::directed::__private_testing::test_sinks_inner(
                    $graph_constructor,
                );
            }
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
        utils::testing::{
            TEST_GRAPH_EDGE_COUNT, TEST_GRAPH_EDGE_ENDPOINTS, TEST_GRAPH_NODE_COUNT,
            check_if_edges_match, check_if_nodes_match, create_test_graph, expected_edge_endpoints,
            expected_edges,
        },
    };

    pub fn test_cardinality_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
    ) {
        let (mut graph, nodes, edges) = create_test_graph(graph_constructor);
        assert_eq!(
            DirectedGraph::node_count(&graph),
            TEST_GRAPH_NODE_COUNT,
            "DirectedGraph::node_count() did not match expected value"
        );
        assert_eq!(
            DirectedGraph::edge_count(&graph),
            TEST_GRAPH_EDGE_COUNT,
            "DirectedGraph::edge_count() did not match expected value"
        );

        let cardinality = DirectedGraph::cardinality(&graph);
        assert_eq!(
            cardinality.order, TEST_GRAPH_NODE_COUNT,
            "DirectedGraph::cardinality().order did not match expected value"
        );
        assert_eq!(
            cardinality.size, TEST_GRAPH_EDGE_COUNT,
            "DirectedGraph::cardinality().size did not match expected value"
        );

        graph.remove_node(nodes[0]).unwrap();
        assert_eq!(
            DirectedGraph::node_count(&graph),
            TEST_GRAPH_NODE_COUNT - 1,
            "DirectedGraph::node_count() did not match expected value after removing node 0"
        );

        assert_eq!(
            DirectedGraph::edge_count(&graph),
            expected_edges(edges, |source, target| source != 0 && target != 0).count(),
            "DirectedGraph::edge_count() did not match expected value after removing node 0"
        );
    }

    pub fn test_density_hint_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
    ) {
        let (graph, _, _) = create_test_graph(graph_constructor);

        // DensityHint is a hint, so this is intentionally only a smoke test.
        let _density_hint = DirectedGraph::density_hint(&graph);
    }

    pub fn test_nodes_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
    ) {
        let (graph, nodes, _) = create_test_graph(graph_constructor);
        let nodes_count = DirectedGraph::nodes(&graph).count();
        assert_eq!(
            nodes_count, TEST_GRAPH_NODE_COUNT,
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
    ) {
        let (mut graph, nodes, _) = create_test_graph(graph_constructor);
        let nodes_count = DirectedGraph::nodes_mut(&mut graph).count();
        assert_eq!(
            nodes_count, TEST_GRAPH_NODE_COUNT,
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
    ) {
        let (graph, nodes, _) = create_test_graph(graph_constructor);
        let isolated_nodes_count = DirectedGraph::isolated_nodes(&graph).count();
        assert_eq!(
            isolated_nodes_count, 1,
            "DirectedGraph::isolated_nodes().count() did not match expected value"
        );
        let mut isolated_nodes_iter = DirectedGraph::isolated_nodes(&graph);
        let first_isolated_node = isolated_nodes_iter
            .next()
            .expect("Expected isolated node not found in test_isolated_nodes");
        assert_eq!(
            first_isolated_node.id, nodes[4],
            "DirectedGraph::isolated_nodes() did not return expected node id"
        );
        assert!(
            isolated_nodes_iter.next().is_none(),
            "DirectedGraph::isolated_nodes() returned more nodes than expected"
        );
    }

    pub fn test_edges_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (graph, nodes, edges) = create_test_graph(graph_constructor);
        let edges_count = DirectedGraph::edges(&graph).count();
        assert_eq!(
            edges_count, TEST_GRAPH_EDGE_COUNT,
            "DirectedGraph::edges().count() did not match expected value"
        );
        check_if_edges_match(
            expected_edge_endpoints(nodes, edges),
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
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (mut graph, nodes, edges) = create_test_graph(graph_constructor);
        let edges_count = DirectedGraph::edges_mut(&mut graph).count();
        assert_eq!(
            edges_count, TEST_GRAPH_EDGE_COUNT,
            "DirectedGraph::edges_mut().count() did not match expected value"
        );
        check_if_edges_match(
            expected_edge_endpoints(nodes, edges),
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
    ) {
        let (mut graph, nodes, _) = create_test_graph(graph_constructor);
        for &node_id in &nodes {
            let node =
                DirectedGraph::node(&graph, node_id).expect("Expected node not found in test_node");
            assert_eq!(
                node.id, node_id,
                "DirectedGraph::node() did not return expected node id"
            );
        }

        // We remove node 4 here, as some graph implementations might not have stable node ids,
        // but the newest node added is likely to be removable without another node taking its
        // id.
        graph.remove_node(nodes[4]).unwrap();
        assert!(
            DirectedGraph::node(&graph, nodes[4]).is_none(),
            "DirectedGraph::node() did not return None for removed node id"
        );
    }

    pub fn test_node_mut_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
    ) {
        let (mut graph, nodes, _) = create_test_graph(graph_constructor);
        for &node_id in &nodes {
            let node = DirectedGraph::node_mut(&mut graph, node_id)
                .expect("Expected node not found in test_node_mut");
            assert_eq!(
                node.id, node_id,
                "DirectedGraph::node_mut() did not return expected node id"
            );
        }

        // We remove node 4 here, as some graph implementations might not have stable node ids,
        // but the newest node added is likely to be removable without another node taking its
        // id.
        graph.remove_node(nodes[4]).unwrap();
        assert!(
            DirectedGraph::node_mut(&mut graph, nodes[4]).is_none(),
            "DirectedGraph::node_mut() did not return None for removed node id"
        );
    }

    pub fn test_edge_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
    ) {
        let (mut graph, _, edges) = create_test_graph(graph_constructor);
        for &edge_id in &edges {
            let edge =
                DirectedGraph::edge(&graph, edge_id).expect("Expected edge not found in test_edge");
            assert_eq!(
                edge.id, edge_id,
                "DirectedGraph::edge() did not return expected edge id"
            );
        }

        graph.remove_edge(edges[3]).unwrap();
        assert!(
            DirectedGraph::edge(&graph, edges[3]).is_none(),
            "DirectedGraph::edge() did not return None for removed edge id"
        );
    }

    pub fn test_edge_mut_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
    ) {
        let (mut graph, _, edges) = create_test_graph(graph_constructor);
        for &edge_id in &edges {
            let edge = DirectedGraph::edge_mut(&mut graph, edge_id)
                .expect("Expected edge not found in test_edge_mut");
            assert_eq!(
                edge.id, edge_id,
                "DirectedGraph::edge_mut() did not return expected edge id"
            );
        }

        graph.remove_edge(edges[3]).unwrap();
        assert!(
            DirectedGraph::edge_mut(&mut graph, edges[3]).is_none(),
            "DirectedGraph::edge_mut() did not return None for removed edge id"
        );
    }

    pub fn test_in_degree_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
    ) {
        let (graph, nodes, edges) = create_test_graph(graph_constructor);
        for (node_number, &node) in nodes.iter().enumerate() {
            assert_eq!(
                DirectedGraph::in_degree(&graph, node),
                expected_edges(edges, |_, target| target == node_number).count(),
                "DirectedGraph::in_degree() did not return expected value for node {node_number}"
            );
        }
    }

    pub fn test_out_degree_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
    ) {
        let (graph, nodes, edges) = create_test_graph(graph_constructor);
        for (node_number, &node) in nodes.iter().enumerate() {
            assert_eq!(
                DirectedGraph::out_degree(&graph, node),
                expected_edges(edges, |source, _| source == node_number).count(),
                "DirectedGraph::out_degree() did not return expected value for node {node_number}"
            );
        }
    }

    pub fn test_degree_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
    ) {
        let (graph, nodes, edges) = create_test_graph(graph_constructor);
        for (node_number, &node) in nodes.iter().enumerate() {
            // An edge counts once for each of its endpoints, so self-loops count twice.
            let expected_degree = expected_edges(edges, |source, _| source == node_number).count()
                + expected_edges(edges, |_, target| target == node_number).count();
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
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (graph, nodes, edges) = create_test_graph(graph_constructor);
        for (node_number, &node) in nodes.iter().enumerate() {
            check_if_edges_match(
                expected_edges(edges, |_, target| target == node_number),
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
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (mut graph, nodes, edges) = create_test_graph(graph_constructor);
        for (node_number, &node) in nodes.iter().enumerate() {
            check_if_edges_match(
                expected_edges(edges, |_, target| target == node_number),
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
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (graph, nodes, edges) = create_test_graph(graph_constructor);
        for (node_number, &node) in nodes.iter().enumerate() {
            check_if_edges_match(
                expected_edges(edges, |source, _| source == node_number),
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
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (mut graph, nodes, edges) = create_test_graph(graph_constructor);
        for (node_number, &node) in nodes.iter().enumerate() {
            check_if_edges_match(
                expected_edges(edges, |source, _| source == node_number),
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
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (graph, nodes, edges) = create_test_graph(graph_constructor);
        for (node_number, &node) in nodes.iter().enumerate() {
            check_if_edges_match(
                expected_edges(edges, |source, target| {
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
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (mut graph, nodes, edges) = create_test_graph(graph_constructor);
        for (node_number, &node) in nodes.iter().enumerate() {
            check_if_edges_match(
                expected_edges(edges, |source, target| {
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
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (graph, nodes, _) = create_test_graph(graph_constructor);
        assert!(
            DirectedGraph::predecessors(&graph, nodes[0])
                .next()
                .is_none(),
            "DirectedGraph::predecessors() did not return an empty iterator for node 0"
        );

        let expected_predecessors_one = [nodes[0]];
        check_if_nodes_match(
            expected_predecessors_one,
            DirectedGraph::predecessors(&graph, nodes[1]),
            "predecessors",
            "DirectedGraph",
            Some(1),
        );

        let expected_predecessors_two = [nodes[0], nodes[3]];
        check_if_nodes_match(
            expected_predecessors_two,
            DirectedGraph::predecessors(&graph, nodes[2]),
            "predecessors",
            "DirectedGraph",
            Some(2),
        );

        let expected_predecessors_three = [nodes[1]];
        check_if_nodes_match(
            expected_predecessors_three,
            DirectedGraph::predecessors(&graph, nodes[3]),
            "predecessors",
            "DirectedGraph",
            Some(3),
        );

        assert!(
            DirectedGraph::predecessors(&graph, nodes[4])
                .next()
                .is_none(),
            "DirectedGraph::predecessors() did not return an empty iterator for node 4"
        );
    }

    pub fn test_successors_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (graph, nodes, _) = create_test_graph(graph_constructor);
        let expected_successors_zero = [nodes[1], nodes[2]];
        check_if_nodes_match(
            expected_successors_zero,
            DirectedGraph::successors(&graph, nodes[0]),
            "successors",
            "DirectedGraph",
            Some(0),
        );

        let expected_successors_one = [nodes[3]];
        check_if_nodes_match(
            expected_successors_one,
            DirectedGraph::successors(&graph, nodes[1]),
            "successors",
            "DirectedGraph",
            Some(1),
        );

        assert!(
            DirectedGraph::successors(&graph, nodes[2]).next().is_none(),
            "DirectedGraph::successors() did not return an empty iterator for node 2"
        );

        let expected_successors_three = [nodes[2]];
        check_if_nodes_match(
            expected_successors_three,
            DirectedGraph::successors(&graph, nodes[3]),
            "successors",
            "DirectedGraph",
            Some(3),
        );

        assert!(
            DirectedGraph::successors(&graph, nodes[4]).next().is_none(),
            "DirectedGraph::successors() did not return an empty iterator for node 4"
        );
    }

    pub fn test_adjacencies_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (graph, nodes, _) = create_test_graph(graph_constructor);
        let expected_adjacencies_zero = [nodes[1], nodes[2]];
        check_if_nodes_match(
            expected_adjacencies_zero,
            DirectedGraph::adjacencies(&graph, nodes[0]),
            "adjacencies",
            "DirectedGraph",
            Some(0),
        );

        let expected_adjacencies_one = [nodes[0], nodes[3]];
        check_if_nodes_match(
            expected_adjacencies_one,
            DirectedGraph::adjacencies(&graph, nodes[1]),
            "adjacencies",
            "DirectedGraph",
            Some(1),
        );

        let expected_adjacencies_two = [nodes[0], nodes[3]];
        check_if_nodes_match(
            expected_adjacencies_two,
            DirectedGraph::adjacencies(&graph, nodes[2]),
            "adjacencies",
            "DirectedGraph",
            Some(2),
        );

        let expected_adjacencies_three = [nodes[1], nodes[2]];
        check_if_nodes_match(
            expected_adjacencies_three,
            DirectedGraph::adjacencies(&graph, nodes[3]),
            "adjacencies",
            "DirectedGraph",
            Some(3),
        );

        assert!(
            DirectedGraph::adjacencies(&graph, nodes[4])
                .next()
                .is_none(),
            "DirectedGraph::adjacencies() did not return an empty iterator for node 4"
        );
    }

    pub fn test_edges_between_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (graph, nodes, edges) = create_test_graph(graph_constructor);

        for lhs_number in 0..nodes.len() {
            for rhs_number in 0..nodes.len() {
                check_if_edges_match(
                    expected_edges(edges, |source, target| {
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
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (mut graph, nodes, edges) = create_test_graph(graph_constructor);

        for lhs_number in 0..nodes.len() {
            for rhs_number in 0..nodes.len() {
                check_if_edges_match(
                    expected_edges(edges, |source, target| {
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
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (graph, nodes, edges) = create_test_graph(graph_constructor);

        for lhs_number in 0..nodes.len() {
            for rhs_number in 0..nodes.len() {
                check_if_edges_match(
                    expected_edges(edges, |source, target| {
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
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (mut graph, nodes, edges) = create_test_graph(graph_constructor);

        for lhs_number in 0..nodes.len() {
            for rhs_number in 0..nodes.len() {
                check_if_edges_match(
                    expected_edges(edges, |source, target| {
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
    ) {
        let (mut graph, nodes, _) = create_test_graph(graph_constructor);

        for node in &nodes {
            assert!(
                DirectedGraph::contains_node(&graph, *node),
                "DirectedGraph::contains_node() returned false for existing node: {node:?}"
            );
        }

        // We remove node 4 here, as some graph implementations might not have stable node ids,
        // but the newest node added is likely to be removable without another node taking its
        // id.
        graph.remove_node(nodes[4]).unwrap();
        assert!(
            !DirectedGraph::contains_node(&graph, nodes[4]),
            "DirectedGraph::contains_node() returned true for removed node: {:?}",
            nodes[4]
        );
    }

    pub fn test_contains_edge_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
    ) {
        let (mut graph, _, edges) = create_test_graph(graph_constructor);

        for edge in &edges {
            assert!(
                DirectedGraph::contains_edge(&graph, *edge),
                "DirectedGraph::contains_edge() returned false for existing edge: {edge:?}",
            );
        }
        // We remove edge 3 here, as some graph implementations might not have stable edge ids,
        // but the newest edge added is likely to be removable without another edge taking its
        // id.
        graph.remove_edge(edges[3]).unwrap();
        assert!(
            !DirectedGraph::contains_edge(&graph, edges[3]),
            "DirectedGraph::contains_edge() returned true for removed edge: {:?}",
            edges[3]
        );
    }

    pub fn test_is_adjacent_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
    ) {
        let (graph, nodes, _) = create_test_graph(graph_constructor);

        let adjacent_node_pairs =
            TEST_GRAPH_EDGE_ENDPOINTS.map(|(source, target)| (nodes[source], nodes[target]));

        for source in &nodes {
            for target in &nodes {
                let expected_adjacency = adjacent_node_pairs.contains(&(*source, *target));
                assert_eq!(
                    DirectedGraph::is_adjacent(&graph, *source, *target),
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
    ) {
        let (graph, _, _) = create_test_graph(graph_constructor);

        assert!(
            !DirectedGraph::is_empty(&graph),
            "DirectedGraph::is_empty() returned true for a non-empty graph"
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
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (graph, nodes, _) = create_test_graph(graph_constructor);

        let expected_sources = [nodes[0], nodes[4]];
        check_if_nodes_match(
            expected_sources,
            DirectedGraph::sources(&graph).map(|n| n.id),
            "sources",
            "DirectedGraph",
            Option::<usize>::None,
        );
    }

    pub fn test_sinks_inner<
        G: DirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (graph, nodes, _) = create_test_graph(graph_constructor);

        let expected_sinks = [nodes[2], nodes[4]];
        check_if_nodes_match(
            expected_sinks,
            DirectedGraph::sinks(&graph).map(|n| n.id),
            "sinks",
            "DirectedGraph",
            Option::<usize>::None,
        );
    }
}
