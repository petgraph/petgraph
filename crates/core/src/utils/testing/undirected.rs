use crate::graph::EditableGraph;

pub const UNDIRECTED_TEST_GRAPH_NODE_COUNT: usize = 5;
pub const UNDIRECTED_TEST_GRAPH_EDGE_COUNT: usize = 4;

/// A helper function to create a simple undirected graph for testing purposes.
///
/// The graph looks as follows:
/// ```text
/// 0 --> 1
/// |      |
/// v      v
/// 2 <----3     4
/// ```
/// Note that the edges are technically undirected, but we denote here the order of the nodes when
/// adding the edges to the graph.
///
/// The function returns a tuple containing the constructed graph,
/// a slice of the ids of added nodes, and a slice of the ids of added edges.
///
/// For ordering of added nodes and edges, see the implementation of the function.
fn create_undirected_test_graph<G: EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>>(
    graph_constructor: impl Fn() -> G,
) -> (G, [G::NodeId; 5], [G::EdgeId; 4]) {
    let mut graph = graph_constructor();

    let node_zero = graph.add_node(()).unwrap();
    let node_one = graph.add_node(()).unwrap();
    let node_two = graph.add_node(()).unwrap();
    let node_three = graph.add_node(()).unwrap();
    let node_four = graph.add_node(()).unwrap();

    let nodes = [node_zero, node_one, node_two, node_three, node_four];

    let edge_zero = graph.add_edge(nodes[0], nodes[1], ()).unwrap();
    let edge_one = graph.add_edge(nodes[0], nodes[2], ()).unwrap();
    let edge_two = graph.add_edge(nodes[1], nodes[3], ()).unwrap();
    let edge_three = graph.add_edge(nodes[3], nodes[2], ()).unwrap();

    let edges = [edge_zero, edge_one, edge_two, edge_three];

    assert_eq!(nodes.len(), UNDIRECTED_TEST_GRAPH_NODE_COUNT);
    assert_eq!(edges.len(), UNDIRECTED_TEST_GRAPH_EDGE_COUNT);

    (graph, nodes, edges)
}

/// Generates a suite of tests for [`UndirectedGraph`][crate::graph::UndirectedGraph]
/// implementations.
///
/// One test is generated for each method in the [`UndirectedGraph`][crate::graph::UndirectedGraph]
/// trait. The following invariants are expected from the graph implementation:
/// - If the most recently added node or edge is removed, its ID will no longer be valid. I.e.,
///   calling methods with that ID should return `None` or otherwise indicate non-existence.
///
/// The arguments to this macro are as follows (`G` is used to denote the graph type being tested):
/// - `$graph_constructor`: `impl Fn() -> G`. An expression that constructs a new instance of the
///   graph type to be tested. The generated graph must be empty (e.g. `G::new()`).
///
/// For a reference usage, see the tests in [`crate::utils::test_graphs::undirected`].
#[macro_export]
macro_rules! test_undirected_graph {
    ($graph_constructor:expr,) => {
        mod undirected_graph_test_macro {
            #[allow(unused_imports)]
            use super::*;

            #[test]
            fn test_density_hint() {
                $crate::utils::testing::undirected::__private_testing::test_density_hint_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_cardinality() {
                $crate::utils::testing::undirected::__private_testing::test_cardinality_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_nodes() {
                $crate::utils::testing::undirected::__private_testing::test_nodes_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_nodes_mut() {
                $crate::utils::testing::undirected::__private_testing::test_nodes_mut_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_isolated_nodes() {
                $crate::utils::testing::undirected::__private_testing::test_isolated_nodes_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_edges() {
                $crate::utils::testing::undirected::__private_testing::test_edges_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_edges_mut() {
                $crate::utils::testing::undirected::__private_testing::test_edges_mut_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_node() {
                $crate::utils::testing::undirected::__private_testing::test_node_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_node_mut() {
                $crate::utils::testing::undirected::__private_testing::test_node_mut_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_edge() {
                $crate::utils::testing::undirected::__private_testing::test_edge_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_edge_mut() {
                $crate::utils::testing::undirected::__private_testing::test_edge_mut_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_degree() {
                $crate::utils::testing::undirected::__private_testing::test_degree_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_incident_edges() {
                $crate::utils::testing::undirected::__private_testing::test_incident_edges_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_incident_edges_mut() {
                $crate::utils::testing::undirected::__private_testing::test_incident_edges_mut_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_adjacencies() {
                $crate::utils::testing::undirected::__private_testing::test_adjacencies_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_edges_connecting() {
                $crate::utils::testing::undirected::__private_testing::test_edges_connecting_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_edges_connecting_mut() {
                $crate::utils::testing::undirected::__private_testing::test_edges_connecting_mut_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_contains_node() {
                $crate::utils::testing::undirected::__private_testing::test_contains_node_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_contains_edge() {
                $crate::utils::testing::undirected::__private_testing::test_contains_edge_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_is_adjacent() {
                $crate::utils::testing::undirected::__private_testing::test_is_adjacent_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_is_empty() {
                $crate::utils::testing::undirected::__private_testing::test_is_empty_inner(
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
        graph::{EditableGraph, UndirectedGraph},
        utils::testing::{
            check_if_edges_match, check_if_nodes_match,
            undirected::{
                UNDIRECTED_TEST_GRAPH_EDGE_COUNT, UNDIRECTED_TEST_GRAPH_NODE_COUNT,
                create_undirected_test_graph,
            },
        },
    };

    pub fn test_density_hint_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
    ) {
        let (graph, _, _) = create_undirected_test_graph(graph_constructor);

        // DensityHint is a hint, so this is intentionally only a smoke test.
        let _density_hint = UndirectedGraph::density_hint(&graph);
    }

    pub fn test_cardinality_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
    ) {
        let (mut graph, nodes, _) = create_undirected_test_graph(graph_constructor);

        assert_eq!(
            UndirectedGraph::node_count(&graph),
            UNDIRECTED_TEST_GRAPH_NODE_COUNT,
            "UndirectedGraph::node_count() did not match expected value"
        );
        assert_eq!(
            UndirectedGraph::edge_count(&graph),
            UNDIRECTED_TEST_GRAPH_EDGE_COUNT,
            "UndirectedGraph::edge_count() did not match expected value"
        );

        let cardinality = UndirectedGraph::cardinality(&graph);
        assert_eq!(
            cardinality.order, UNDIRECTED_TEST_GRAPH_NODE_COUNT,
            "UndirectedGraph::cardinality().order did not match expected value"
        );
        assert_eq!(
            cardinality.size, UNDIRECTED_TEST_GRAPH_EDGE_COUNT,
            "UndirectedGraph::cardinality().size did not match expected value"
        );

        graph.remove_node(nodes[0]).unwrap();

        assert_eq!(
            UndirectedGraph::node_count(&graph),
            UNDIRECTED_TEST_GRAPH_NODE_COUNT - 1,
            "UndirectedGraph::node_count() did not match expected value after removing node 0"
        );
        assert_eq!(
            UndirectedGraph::edge_count(&graph),
            UNDIRECTED_TEST_GRAPH_EDGE_COUNT - 2,
            "UndirectedGraph::edge_count() did not match expected value after removing node 0"
        );
    }

    pub fn test_nodes_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (graph, nodes, _) = create_undirected_test_graph(graph_constructor);

        assert_eq!(
            UndirectedGraph::nodes(&graph).count(),
            UNDIRECTED_TEST_GRAPH_NODE_COUNT,
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
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (mut graph, nodes, _) = create_undirected_test_graph(graph_constructor);

        assert_eq!(
            UndirectedGraph::nodes_mut(&mut graph).count(),
            UNDIRECTED_TEST_GRAPH_NODE_COUNT,
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
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (graph, nodes, _) = create_undirected_test_graph(graph_constructor);

        check_if_nodes_match(
            [nodes[4]],
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
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (graph, _, edges) = create_undirected_test_graph(graph_constructor);

        assert_eq!(
            UndirectedGraph::edges(&graph).count(),
            UNDIRECTED_TEST_GRAPH_EDGE_COUNT,
            "UndirectedGraph::edges().count() did not match expected value"
        );

        check_if_edges_match(
            edges.iter().copied(),
            UndirectedGraph::edges(&graph).map(|edge| edge.id),
            "edges",
            "UndirectedGraph",
            "graph",
        );
    }

    pub fn test_edges_mut_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (mut graph, _, edges) = create_undirected_test_graph(graph_constructor);

        assert_eq!(
            UndirectedGraph::edges_mut(&mut graph).count(),
            UNDIRECTED_TEST_GRAPH_EDGE_COUNT,
            "UndirectedGraph::edges_mut().count() did not match expected value"
        );

        check_if_edges_match(
            edges.iter().copied(),
            UndirectedGraph::edges_mut(&mut graph).map(|edge| edge.id),
            "edges_mut",
            "UndirectedGraph",
            "graph",
        );
    }

    pub fn test_node_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
    ) {
        let (mut graph, nodes, _) = create_undirected_test_graph(graph_constructor);

        for &node_id in &nodes {
            let node = UndirectedGraph::node(&graph, node_id)
                .expect("Expected node not found in test_node");
            assert_eq!(
                node.id, node_id,
                "UndirectedGraph::node() did not return expected node id"
            );
        }

        // We remove node 4 here, as some graph implementations might not have stable node ids,
        // but the newest node added is likely to be removable without another node taking its
        // id.
        graph.remove_node(nodes[4]).unwrap();

        assert!(
            UndirectedGraph::node(&graph, nodes[4]).is_none(),
            "UndirectedGraph::node() did not return None for removed node id"
        );
    }

    pub fn test_node_mut_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
    ) {
        let (mut graph, nodes, _) = create_undirected_test_graph(graph_constructor);

        for &node_id in &nodes {
            let node = UndirectedGraph::node_mut(&mut graph, node_id)
                .expect("Expected node not found in test_node_mut");
            assert_eq!(
                node.id, node_id,
                "UndirectedGraph::node_mut() did not return expected node id"
            );
        }

        // We remove node 4 here, as some graph implementations might not have stable node ids,
        // but the newest node added is likely to be removable without another node taking its
        // id.
        graph.remove_node(nodes[4]).unwrap();

        assert!(
            UndirectedGraph::node_mut(&mut graph, nodes[4]).is_none(),
            "UndirectedGraph::node_mut() did not return None for removed node id"
        );
    }

    pub fn test_edge_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
    ) {
        let (mut graph, _, edges) = create_undirected_test_graph(graph_constructor);

        for &edge_id in &edges {
            let edge = UndirectedGraph::edge(&graph, edge_id)
                .expect("Expected edge not found in test_edge");
            assert_eq!(
                edge.id, edge_id,
                "UndirectedGraph::edge() did not return expected edge id"
            );
        }

        // We remove edge 3 here, as some graph implementations might not have stable edge ids,
        // but the newest edge added is likely to be removable without another edge taking its
        // id.
        graph.remove_edge(edges[3]).unwrap();

        assert!(
            UndirectedGraph::edge(&graph, edges[3]).is_none(),
            "UndirectedGraph::edge() did not return None for removed edge id"
        );
    }

    pub fn test_edge_mut_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
    ) {
        let (mut graph, _, edges) = create_undirected_test_graph(graph_constructor);

        for &edge_id in &edges {
            let edge = UndirectedGraph::edge_mut(&mut graph, edge_id)
                .expect("Expected edge not found in test_edge_mut");
            assert_eq!(
                edge.id, edge_id,
                "UndirectedGraph::edge_mut() did not return expected edge id"
            );
        }

        // We remove edge 3 here, as some graph implementations might not have stable edge ids,
        // but the newest edge added is likely to be removable without another edge taking its
        // id.
        graph.remove_edge(edges[3]).unwrap();

        assert!(
            UndirectedGraph::edge_mut(&mut graph, edges[3]).is_none(),
            "UndirectedGraph::edge_mut() did not return None for removed edge id"
        );
    }

    pub fn test_degree_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
    ) {
        let (graph, nodes, _) = create_undirected_test_graph(graph_constructor);

        let expected_degrees = [
            (nodes[0], 0, 2),
            (nodes[1], 1, 2),
            (nodes[2], 2, 2),
            (nodes[3], 3, 2),
            (nodes[4], 4, 0),
        ];

        for &(node_id, node_number, expected_degree) in &expected_degrees {
            assert_eq!(
                UndirectedGraph::degree(&graph, node_id),
                expected_degree,
                "UndirectedGraph::degree() did not return expected value for node {node_number}"
            );
        }
    }

    pub fn test_incident_edges_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (graph, nodes, edges) = create_undirected_test_graph(graph_constructor);

        for (node_number, &node) in nodes.iter().enumerate() {
            let expected_edges = match node_number {
                0 => [Some(edges[0]), Some(edges[1])],
                1 => [Some(edges[0]), Some(edges[2])],
                2 => [Some(edges[1]), Some(edges[3])],
                3 => [Some(edges[2]), Some(edges[3])],
                4 => [None, None],
                _ => unreachable!(),
            };

            check_if_edges_match(
                core::iter::IntoIterator::into_iter(expected_edges).flatten(),
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
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (mut graph, nodes, edges) = create_undirected_test_graph(graph_constructor);

        for (node_number, &node) in nodes.iter().enumerate() {
            let expected_edges = match node_number {
                0 => [Some(edges[0]), Some(edges[1])],
                1 => [Some(edges[0]), Some(edges[2])],
                2 => [Some(edges[1]), Some(edges[3])],
                3 => [Some(edges[2]), Some(edges[3])],
                4 => [None, None],
                _ => unreachable!(),
            };

            check_if_edges_match(
                core::iter::IntoIterator::into_iter(expected_edges).flatten(),
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
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (graph, nodes, _) = create_undirected_test_graph(graph_constructor);

        for node_number in 0..nodes.len() {
            let expected_nodes = match node_number {
                0 | 3 => [Some(nodes[1]), Some(nodes[2])],
                1 | 2 => [Some(nodes[0]), Some(nodes[3])],
                4 => [None, None],
                _ => unreachable!(),
            };

            check_if_nodes_match(
                core::iter::IntoIterator::into_iter(expected_nodes).flatten(),
                UndirectedGraph::adjacencies(&graph, nodes[node_number]),
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
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (graph, nodes, edges) = create_undirected_test_graph(graph_constructor);

        let expected_edge_matrix = [
            [None, Some(edges[0]), Some(edges[1]), None, None],
            [Some(edges[0]), None, None, Some(edges[2]), None],
            [Some(edges[1]), None, None, Some(edges[3]), None],
            [None, Some(edges[2]), Some(edges[3]), None, None],
            [None, None, None, None, None],
        ];

        for lhs_number in 0..nodes.len() {
            for rhs_number in 0..nodes.len() {
                check_if_edges_match(
                    expected_edge_matrix[lhs_number][rhs_number],
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
    ) where
        G::NodeId: Hash + Eq,
        G::EdgeId: Hash + Eq,
    {
        let (mut graph, nodes, edges) = create_undirected_test_graph(graph_constructor);

        let expected_edge_matrix = [
            [None, Some(edges[0]), Some(edges[1]), None, None],
            [Some(edges[0]), None, None, Some(edges[2]), None],
            [Some(edges[1]), None, None, Some(edges[3]), None],
            [None, Some(edges[2]), Some(edges[3]), None, None],
            [None, None, None, None, None],
        ];

        for lhs_number in 0..nodes.len() {
            for rhs_number in 0..nodes.len() {
                check_if_edges_match(
                    expected_edge_matrix[lhs_number][rhs_number],
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
    ) {
        let (mut graph, nodes, _) = create_undirected_test_graph(graph_constructor);

        for &node in &nodes {
            assert!(
                UndirectedGraph::contains_node(&graph, node),
                "UndirectedGraph::contains_node() returned false for existing node: {node:?}"
            );
        }

        // We remove node 4 here, as some graph implementations might not have stable node ids,
        // but the newest node added is likely to be removable without another node taking its
        // id.
        graph.remove_node(nodes[4]).unwrap();

        assert!(
            !UndirectedGraph::contains_node(&graph, nodes[4]),
            "UndirectedGraph::contains_node() returned true for removed node: {:?}",
            nodes[4]
        );
    }

    pub fn test_contains_edge_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
    ) {
        let (mut graph, _, edges) = create_undirected_test_graph(graph_constructor);

        for &edge in &edges {
            assert!(
                UndirectedGraph::contains_edge(&graph, edge),
                "UndirectedGraph::contains_edge() returned false for existing edge: {edge:?}"
            );
        }

        // We remove edge 3 here, as some graph implementations might not have stable edge ids,
        // but the newest edge added is likely to be removable without another edge taking its
        // id.
        graph.remove_edge(edges[3]).unwrap();

        assert!(
            !UndirectedGraph::contains_edge(&graph, edges[3]),
            "UndirectedGraph::contains_edge() returned true for removed edge: {:?}",
            edges[3]
        );
    }

    pub fn test_is_adjacent_inner<
        G: UndirectedGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
    ) {
        let (graph, nodes, _) = create_undirected_test_graph(graph_constructor);

        let expected_adjacency_matrix = [
            [false, true, true, false, false],
            [true, false, false, true, false],
            [true, false, false, true, false],
            [false, true, true, false, false],
            [false, false, false, false, false],
        ];

        for lhs_number in 0..nodes.len() {
            for rhs_number in 0..nodes.len() {
                assert_eq!(
                    UndirectedGraph::is_adjacent(&graph, nodes[lhs_number], nodes[rhs_number],),
                    expected_adjacency_matrix[lhs_number][rhs_number],
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
    ) {
        let (graph, _, _) = create_undirected_test_graph(graph_constructor);

        assert!(
            !UndirectedGraph::is_empty(&graph),
            "UndirectedGraph::is_empty() returned true for a non-empty graph"
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
