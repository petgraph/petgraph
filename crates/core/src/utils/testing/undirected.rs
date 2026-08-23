pub const UNDIRECTED_TEST_GRAPH_NODE_COUNT: usize = 5;
pub const UNDIRECTED_TEST_GRAPH_EDGE_COUNT: usize = 4;

/// A macro to create a simple undirected graph for testing purposes.
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
/// The macro returns a tuple containing the constructed graph,
/// a vector of the indices of added nodes, and a vector of the indices of added edges.
///
/// For ordering of added nodes and edges, see the implementation of the macro.
#[macro_export]
macro_rules! create_undirected_test_graph {
    ($graph_constructor:expr, $add_node:expr, $add_edge:expr) => {{
        let mut graph = $graph_constructor();

        let node_zero = $add_node(&mut graph, ());
        let node_one = $add_node(&mut graph, ());
        let node_two = $add_node(&mut graph, ());
        let node_three = $add_node(&mut graph, ());
        let node_four = $add_node(&mut graph, ());

        let nodes = [node_zero, node_one, node_two, node_three, node_four];

        let edge_zero = $add_edge(&mut graph, nodes[0], nodes[1], ());
        let edge_one = $add_edge(&mut graph, nodes[0], nodes[2], ());
        let edge_two = $add_edge(&mut graph, nodes[1], nodes[3], ());
        let edge_three = $add_edge(&mut graph, nodes[3], nodes[2], ());

        let edges = [edge_zero, edge_one, edge_two, edge_three];

        assert_eq!(
            nodes.len(),
            $crate::utils::testing::undirected::UNDIRECTED_TEST_GRAPH_NODE_COUNT
        );
        assert_eq!(
            edges.len(),
            $crate::utils::testing::undirected::UNDIRECTED_TEST_GRAPH_EDGE_COUNT
        );

        (graph, nodes, edges)
    }};
}

/// Generates a suite of tests for [`UndirectedGraph`][crate::graph::UndirectedGraph]
/// implementations.
///
/// One test is generated for each method in the [`UndirectedGraph`][crate::graph::UndirectedGraph]
/// trait. The following invariants are expected from the graph implementation:
/// - If the most recently added node or edge is removed, its ID will no longer be valid. I.e.,
///   calling methods with that ID should return `None` or otherwise indicate non-existence.
///
/// The arguments to this macro are as follows (`G` is used to denote the graph type being tested).
/// - `$graph_constructor`: An expression that constructs a new instance of the graph type to be
///   tested. The generated graph must be empty (e.g. `G::new()`).
/// - `$add_node`: An expression that adds a node to the graph. It must take two arguments: a
///   mutable reference to the graph and the node weight. It must return the `<G as Graph>::NodeId`
///   of the added node.
/// - `$remove_node`: An expression that removes a node from the graph. It must take two arguments:
///   a mutable reference to the graph and the `<G as Graph>::NodeId` of the node to be removed. The
///   method should not return anything, i.e. it should panic on failure.
/// - `$add_edge`: An expression that adds an edge to the graph. It must take four arguments: a
///   mutable reference to the graph, the `<G as Graph>::NodeId` of the two endpoint nodes, and the
///   edge weight. It must return the `<G as Graph>::EdgeId` of the added edge.
/// - `$remove_edge`: An expression that removes an edge from the graph. It must take two arguments:
///   a mutable reference to the graph and the `<G as Graph>::EdgeId` of the edge to be removed. The
///   method should not return anything, i.e. it should panic on failure.
#[macro_export]
macro_rules! test_undirected_graph {
    (
        $graph_constructor:expr,
        $add_node:expr,
        $remove_node:expr,
        $add_edge:expr,
        $remove_edge:expr
    ) => {
        #[test]
        fn test_density_hint() {
            let (graph, _, _) =
                $crate::create_undirected_test_graph!($graph_constructor, $add_node, $add_edge);

            // DensityHint is a hint, so this is intentionally only a smoke test.
            let _density_hint = $crate::graph::UndirectedGraph::density_hint(&graph);
        }

        #[test]
        fn test_cardinality() {
            let (mut graph, nodes, _) =
                $crate::create_undirected_test_graph!($graph_constructor, $add_node, $add_edge);

            assert_eq!(
                $crate::graph::UndirectedGraph::node_count(&graph),
                $crate::utils::testing::undirected::UNDIRECTED_TEST_GRAPH_NODE_COUNT,
                "UndirectedGraph::node_count() did not match expected value"
            );
            assert_eq!(
                $crate::graph::UndirectedGraph::edge_count(&graph),
                $crate::utils::testing::undirected::UNDIRECTED_TEST_GRAPH_EDGE_COUNT,
                "UndirectedGraph::edge_count() did not match expected value"
            );

            let cardinality = $crate::graph::UndirectedGraph::cardinality(&graph);
            assert_eq!(
                cardinality.order,
                $crate::utils::testing::undirected::UNDIRECTED_TEST_GRAPH_NODE_COUNT,
                "UndirectedGraph::cardinality().order did not match expected value"
            );
            assert_eq!(
                cardinality.size,
                $crate::utils::testing::undirected::UNDIRECTED_TEST_GRAPH_EDGE_COUNT,
                "UndirectedGraph::cardinality().size did not match expected value"
            );

            $remove_node(&mut graph, nodes[0]);

            assert_eq!(
                $crate::graph::UndirectedGraph::node_count(&graph),
                $crate::utils::testing::undirected::UNDIRECTED_TEST_GRAPH_NODE_COUNT - 1,
                "UndirectedGraph::node_count() did not match expected value after removing node 0"
            );
            assert_eq!(
                $crate::graph::UndirectedGraph::edge_count(&graph),
                $crate::utils::testing::undirected::UNDIRECTED_TEST_GRAPH_EDGE_COUNT - 2,
                "UndirectedGraph::edge_count() did not match expected value after removing node 0"
            );
        }

        #[test]
        fn test_nodes() {
            let (graph, nodes, _) =
                $crate::create_undirected_test_graph!($graph_constructor, $add_node, $add_edge);

            assert_eq!(
                $crate::graph::UndirectedGraph::nodes(&graph).count(),
                $crate::utils::testing::undirected::UNDIRECTED_TEST_GRAPH_NODE_COUNT,
                "UndirectedGraph::nodes().count() did not match expected value"
            );

            $crate::utils::testing::check_if_nodes_match(
                nodes.iter().copied(),
                $crate::graph::UndirectedGraph::nodes(&graph).map(|node| node.id),
                "nodes",
                "graph",
            );
        }

        #[test]
        fn test_nodes_mut() {
            let (mut graph, nodes, _) =
                $crate::create_undirected_test_graph!($graph_constructor, $add_node, $add_edge);

            assert_eq!(
                $crate::graph::UndirectedGraph::nodes_mut(&mut graph).count(),
                $crate::utils::testing::undirected::UNDIRECTED_TEST_GRAPH_NODE_COUNT,
                "UndirectedGraph::nodes_mut().count() did not match expected value"
            );

            $crate::utils::testing::check_if_nodes_match(
                nodes.iter().copied(),
                $crate::graph::UndirectedGraph::nodes_mut(&mut graph).map(|node| node.id),
                "nodes_mut",
                "graph",
            );
        }

        #[test]
        fn test_isolated_nodes() {
            let (graph, nodes, _) =
                $crate::create_undirected_test_graph!($graph_constructor, $add_node, $add_edge);

            $crate::utils::testing::check_if_nodes_match(
                [nodes[4]],
                $crate::graph::UndirectedGraph::isolated_nodes(&graph).map(|node| node.id),
                "isolated_nodes",
                "graph",
            );
        }

        #[test]
        fn test_edges() {
            let (graph, _, edges) =
                $crate::create_undirected_test_graph!($graph_constructor, $add_node, $add_edge);

            assert_eq!(
                $crate::graph::UndirectedGraph::edges(&graph).count(),
                $crate::utils::testing::undirected::UNDIRECTED_TEST_GRAPH_EDGE_COUNT,
                "UndirectedGraph::edges().count() did not match expected value"
            );

            $crate::utils::testing::check_if_edges_match(
                edges.iter().copied(),
                $crate::graph::UndirectedGraph::edges(&graph).map(|edge| edge.id),
                "edges",
                "UndirectedGraph",
                "graph",
            );
        }

        #[test]
        fn test_edges_mut() {
            let (mut graph, _, edges) =
                $crate::create_undirected_test_graph!($graph_constructor, $add_node, $add_edge);

            assert_eq!(
                $crate::graph::UndirectedGraph::edges_mut(&mut graph).count(),
                $crate::utils::testing::undirected::UNDIRECTED_TEST_GRAPH_EDGE_COUNT,
                "UndirectedGraph::edges_mut().count() did not match expected value"
            );

            $crate::utils::testing::check_if_edges_match(
                edges.iter().copied(),
                $crate::graph::UndirectedGraph::edges_mut(&mut graph).map(|edge| edge.id),
                "edges_mut",
                "UndirectedGraph",
                "graph",
            );
        }

        #[test]
        fn test_node() {
            let (mut graph, nodes, _) =
                $crate::create_undirected_test_graph!($graph_constructor, $add_node, $add_edge);

            for &node_id in &nodes {
                let node = $crate::graph::UndirectedGraph::node(&graph, node_id)
                    .expect("Expected node not found in test_node");
                assert_eq!(
                    node.id, node_id,
                    "UndirectedGraph::node() did not return expected node id"
                );
            }

            // We remove node 4 here, as some graph implementations might not have stable node ids,
            // but the newest node added is likely to be removable without another node taking its
            // id.
            $remove_node(&mut graph, nodes[4]);

            assert!(
                $crate::graph::UndirectedGraph::node(&graph, nodes[4]).is_none(),
                "UndirectedGraph::node() did not return None for removed node id"
            );
        }

        #[test]
        fn test_node_mut() {
            let (mut graph, nodes, _) =
                $crate::create_undirected_test_graph!($graph_constructor, $add_node, $add_edge);

            for &node_id in &nodes {
                let node = $crate::graph::UndirectedGraph::node_mut(&mut graph, node_id)
                    .expect("Expected node not found in test_node_mut");
                assert_eq!(
                    node.id, node_id,
                    "UndirectedGraph::node_mut() did not return expected node id"
                );
            }

            // We remove node 4 here, as some graph implementations might not have stable node ids,
            // but the newest node added is likely to be removable without another node taking its
            // id.
            $remove_node(&mut graph, nodes[4]);

            assert!(
                $crate::graph::UndirectedGraph::node_mut(&mut graph, nodes[4]).is_none(),
                "UndirectedGraph::node_mut() did not return None for removed node id"
            );
        }

        #[test]
        fn test_edge() {
            let (mut graph, _, edges) =
                $crate::create_undirected_test_graph!($graph_constructor, $add_node, $add_edge);

            for &edge_id in &edges {
                let edge = $crate::graph::UndirectedGraph::edge(&graph, edge_id)
                    .expect("Expected edge not found in test_edge");
                assert_eq!(
                    edge.id, edge_id,
                    "UndirectedGraph::edge() did not return expected edge id"
                );
            }

            // We remove edge 3 here, as some graph implementations might not have stable edge ids,
            // but the newest edge added is likely to be removable without another edge taking its
            // id.
            $remove_edge(&mut graph, edges[3]);

            assert!(
                $crate::graph::UndirectedGraph::edge(&graph, edges[3]).is_none(),
                "UndirectedGraph::edge() did not return None for removed edge id"
            );
        }

        #[test]
        fn test_edge_mut() {
            let (mut graph, _, edges) =
                $crate::create_undirected_test_graph!($graph_constructor, $add_node, $add_edge);

            for &edge_id in &edges {
                let edge = $crate::graph::UndirectedGraph::edge_mut(&mut graph, edge_id)
                    .expect("Expected edge not found in test_edge_mut");
                assert_eq!(
                    edge.id, edge_id,
                    "UndirectedGraph::edge_mut() did not return expected edge id"
                );
            }

            // We remove edge 3 here, as some graph implementations might not have stable edge ids,
            // but the newest edge added is likely to be removable without another edge taking its
            // id.
            $remove_edge(&mut graph, edges[3]);

            assert!(
                $crate::graph::UndirectedGraph::edge_mut(&mut graph, edges[3]).is_none(),
                "UndirectedGraph::edge_mut() did not return None for removed edge id"
            );
        }

        #[test]
        fn test_degree() {
            let (graph, nodes, _) =
                $crate::create_undirected_test_graph!($graph_constructor, $add_node, $add_edge);

            let expected_degrees = [
                (nodes[0], 0, 2),
                (nodes[1], 1, 2),
                (nodes[2], 2, 2),
                (nodes[3], 3, 2),
                (nodes[4], 4, 0),
            ];

            for &(node_id, node_number, expected_degree) in &expected_degrees {
                assert_eq!(
                    $crate::graph::UndirectedGraph::degree(&graph, node_id),
                    expected_degree,
                    "UndirectedGraph::degree() did not return expected value for node {}",
                    node_number
                );
            }
        }

        #[test]
        fn test_incident_edges() {
            let (graph, nodes, edges) =
                $crate::create_undirected_test_graph!($graph_constructor, $add_node, $add_edge);

            for node_number in 0..nodes.len() {
                let expected_edges = match node_number {
                    0 => [Some(edges[0]), Some(edges[1])],
                    1 => [Some(edges[0]), Some(edges[2])],
                    2 => [Some(edges[1]), Some(edges[3])],
                    3 => [Some(edges[2]), Some(edges[3])],
                    4 => [None, None],
                    _ => unreachable!(),
                };

                $crate::utils::testing::check_if_edges_match(
                    core::iter::IntoIterator::into_iter(expected_edges).flatten(),
                    $crate::graph::UndirectedGraph::incident_edges(&graph, nodes[node_number])
                        .map(|edge| edge.id),
                    "incident_edges",
                    "UndirectedGraph",
                    format_args!("node {}", node_number),
                );
            }
        }

        #[test]
        fn test_incident_edges_mut() {
            let (mut graph, nodes, edges) =
                $crate::create_undirected_test_graph!($graph_constructor, $add_node, $add_edge);

            for node_number in 0..nodes.len() {
                let expected_edges = match node_number {
                    0 => [Some(edges[0]), Some(edges[1])],
                    1 => [Some(edges[0]), Some(edges[2])],
                    2 => [Some(edges[1]), Some(edges[3])],
                    3 => [Some(edges[2]), Some(edges[3])],
                    4 => [None, None],
                    _ => unreachable!(),
                };

                $crate::utils::testing::check_if_edges_match(
                    core::iter::IntoIterator::into_iter(expected_edges).flatten(),
                    $crate::graph::UndirectedGraph::incident_edges_mut(
                        &mut graph,
                        nodes[node_number],
                    )
                    .map(|edge| edge.id),
                    "incident_edges_mut",
                    "UndirectedGraph",
                    format_args!("node {}", node_number),
                );
            }
        }

        #[test]
        fn test_adjacencies() {
            let (graph, nodes, _) =
                $crate::create_undirected_test_graph!($graph_constructor, $add_node, $add_edge);

            for node_number in 0..nodes.len() {
                let expected_nodes = match node_number {
                    0 => [Some(nodes[1]), Some(nodes[2])],
                    1 => [Some(nodes[0]), Some(nodes[3])],
                    2 => [Some(nodes[0]), Some(nodes[3])],
                    3 => [Some(nodes[1]), Some(nodes[2])],
                    4 => [None, None],
                    _ => unreachable!(),
                };

                $crate::utils::testing::check_if_nodes_match(
                    core::iter::IntoIterator::into_iter(expected_nodes).flatten(),
                    $crate::graph::UndirectedGraph::adjacencies(&graph, nodes[node_number]),
                    "adjacencies",
                    format_args!("node {}", node_number),
                );
            }
        }

        #[test]
        fn test_edges_connecting() {
            let (graph, nodes, edges) =
                $crate::create_undirected_test_graph!($graph_constructor, $add_node, $add_edge);

            let expected_edge_matrix = [
                [None, Some(edges[0]), Some(edges[1]), None, None],
                [Some(edges[0]), None, None, Some(edges[2]), None],
                [Some(edges[1]), None, None, Some(edges[3]), None],
                [None, Some(edges[2]), Some(edges[3]), None, None],
                [None, None, None, None, None],
            ];

            for lhs_number in 0..nodes.len() {
                for rhs_number in 0..nodes.len() {
                    $crate::utils::testing::check_if_edges_match(
                        expected_edge_matrix[lhs_number][rhs_number],
                        $crate::graph::UndirectedGraph::edges_connecting(
                            &graph,
                            nodes[lhs_number],
                            nodes[rhs_number],
                        )
                        .map(|edge| edge.id),
                        "edges_connecting",
                        "UndirectedGraph",
                        format_args!("node pair ({}, {})", lhs_number, rhs_number),
                    );
                }
            }
        }

        #[test]
        fn test_edges_connecting_mut() {
            let (mut graph, nodes, edges) =
                $crate::create_undirected_test_graph!($graph_constructor, $add_node, $add_edge);

            let expected_edge_matrix = [
                [None, Some(edges[0]), Some(edges[1]), None, None],
                [Some(edges[0]), None, None, Some(edges[2]), None],
                [Some(edges[1]), None, None, Some(edges[3]), None],
                [None, Some(edges[2]), Some(edges[3]), None, None],
                [None, None, None, None, None],
            ];

            for lhs_number in 0..nodes.len() {
                for rhs_number in 0..nodes.len() {
                    $crate::utils::testing::check_if_edges_match(
                        expected_edge_matrix[lhs_number][rhs_number],
                        $crate::graph::UndirectedGraph::edges_connecting_mut(
                            &mut graph,
                            nodes[lhs_number],
                            nodes[rhs_number],
                        )
                        .map(|edge| edge.id),
                        "edges_connecting_mut",
                        "UndirectedGraph",
                        format_args!("node pair ({}, {})", lhs_number, rhs_number),
                    );
                }
            }
        }

        #[test]
        fn test_contains_node() {
            let (mut graph, nodes, _) =
                $crate::create_undirected_test_graph!($graph_constructor, $add_node, $add_edge);

            for &node in &nodes {
                assert!(
                    $crate::graph::UndirectedGraph::contains_node(&graph, node),
                    "UndirectedGraph::contains_node() returned false for existing node: {:?}",
                    node
                );
            }

            // We remove node 4 here, as some graph implementations might not have stable node ids,
            // but the newest node added is likely to be removable without another node taking its
            // id.
            $remove_node(&mut graph, nodes[4]);

            assert!(
                !$crate::graph::UndirectedGraph::contains_node(&graph, nodes[4]),
                "UndirectedGraph::contains_node() returned true for removed node: {:?}",
                nodes[4]
            );
        }

        #[test]
        fn test_contains_edge() {
            let (mut graph, _, edges) =
                $crate::create_undirected_test_graph!($graph_constructor, $add_node, $add_edge);

            for &edge in &edges {
                assert!(
                    $crate::graph::UndirectedGraph::contains_edge(&graph, edge),
                    "UndirectedGraph::contains_edge() returned false for existing edge: {:?}",
                    edge
                );
            }

            // We remove edge 3 here, as some graph implementations might not have stable edge ids,
            // but the newest edge added is likely to be removable without another edge taking its
            // id.
            $remove_edge(&mut graph, edges[3]);

            assert!(
                !$crate::graph::UndirectedGraph::contains_edge(&graph, edges[3]),
                "UndirectedGraph::contains_edge() returned true for removed edge: {:?}",
                edges[3]
            );
        }

        #[test]
        fn test_is_adjacent() {
            let (graph, nodes, _) =
                $crate::create_undirected_test_graph!($graph_constructor, $add_node, $add_edge);

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
                        $crate::graph::UndirectedGraph::is_adjacent(
                            &graph,
                            nodes[lhs_number],
                            nodes[rhs_number],
                        ),
                        expected_adjacency_matrix[lhs_number][rhs_number],
                        "UndirectedGraph::is_adjacent() returned incorrect result for nodes {:?} \
                         and {:?}",
                        nodes[lhs_number],
                        nodes[rhs_number]
                    );
                }
            }
        }

        #[test]
        fn test_is_empty() {
            let (graph, _, _) =
                $crate::create_undirected_test_graph!($graph_constructor, $add_node, $add_edge);

            assert!(
                !$crate::graph::UndirectedGraph::is_empty(&graph),
                "UndirectedGraph::is_empty() returned true for a non-empty graph"
            );

            let mut new_graph = $graph_constructor();

            assert!(
                $crate::graph::UndirectedGraph::is_empty(&new_graph),
                "UndirectedGraph::is_empty() returned false for an empty graph"
            );

            let node_one = $add_node(&mut new_graph, ());

            assert!(
                !$crate::graph::UndirectedGraph::is_empty(&new_graph),
                "UndirectedGraph::is_empty() returned true for a graph with one node"
            );

            $remove_node(&mut new_graph, node_one);

            assert!(
                $crate::graph::UndirectedGraph::is_empty(&new_graph),
                "UndirectedGraph::is_empty() returned false for an empty graph after removing the \
                 only node"
            );
        }
    };
}
