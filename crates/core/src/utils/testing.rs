pub const DIRECTED_TEST_GRAPH_NODE_COUNT: usize = 5;
pub const DIRECTED_TEST_GRAPH_EDGE_COUNT: usize = 4;

/// A macro to create a simple directed graph for testing purposes.
///
/// The graph looks as follows:
/// 0 --> 1
/// |      |
/// v      v
/// 2 <----3     4
///
/// The macro returns a tuple containing the constructed graph,
/// a vector of the indices of added nodes, and a vector of the indices of added edges.
///
/// For ordering of added nodes and edges, see the implementation of the macro.
#[macro_export]
macro_rules! create_directed_test_graph {
    ($graph_constructer:expr, $add_node:expr, $add_edge:expr) => {{
        let mut graph = $graph_constructer();

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
            $crate::utils::testing::DIRECTED_TEST_GRAPH_NODE_COUNT
        );
        assert_eq!(
            edges.len(),
            $crate::utils::testing::DIRECTED_TEST_GRAPH_EDGE_COUNT
        );

        (graph, nodes, edges)
    }};
}

#[macro_export]
macro_rules! test_directed_graph {
    ($graph_constructer:expr, $add_node:expr, $add_edge:expr) => {
        #[test]
        fn test_cardinality() {
            let (graph, _, _) =
                $crate::create_directed_test_graph!($graph_constructer, $add_node, $add_edge);
            assert_eq!(
                graph.node_count(),
                $crate::utils::testing::DIRECTED_TEST_GRAPH_NODE_COUNT,
                "graph.node_count() did not match expected value"
            );
            assert_eq!(
                graph.edge_count(),
                $crate::utils::testing::DIRECTED_TEST_GRAPH_EDGE_COUNT,
                "graph.edge_count() did not match expected value"
            );

            let cardinality = graph.cardinality();
            assert_eq!(
                cardinality.order,
                $crate::utils::testing::DIRECTED_TEST_GRAPH_NODE_COUNT,
                "graph.cardinality().order did not match expected value"
            );
            assert_eq!(
                cardinality.size,
                $crate::utils::testing::DIRECTED_TEST_GRAPH_EDGE_COUNT,
                "graph.cardinality().size did not match expected value"
            );
        }

        #[test]
        fn test_nodes() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructer, $add_node, $add_edge);
            let nodes_count = graph.nodes().count();
            assert_eq!(
                nodes_count,
                $crate::utils::testing::DIRECTED_TEST_GRAPH_NODE_COUNT,
                "graph.nodes().count() did not match expected value"
            );
            for node in graph.nodes() {
                assert!(
                    nodes.contains(&node.id),
                    "graph.nodes() contained unexpected node id: {:?}",
                    node.id
                );
            }
        }

        #[test]
        fn test_nodes_mut() {
            let (mut graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructer, $add_node, $add_edge);
            let nodes_count = graph.nodes_mut().count();
            assert_eq!(
                nodes_count,
                $crate::utils::testing::DIRECTED_TEST_GRAPH_NODE_COUNT,
                "graph.nodes_mut().count() did not match expected value"
            );
            for node in graph.nodes_mut() {
                assert!(
                    nodes.contains(&node.id),
                    "graph.nodes_mut() contained unexpected node id: {:?}",
                    node.id
                );
            }
        }

        #[test]
        fn test_isolated_nodes() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructer, $add_node, $add_edge);
            let isolated_nodes_count = graph.isolated_nodes().count();
            assert_eq!(
                isolated_nodes_count, 1,
                "graph.isolated_nodes().count() did not match expected value"
            );
            let mut isolated_nodes_iter = graph.isolated_nodes();
            let first_isolated_node = isolated_nodes_iter.next().unwrap();
            assert_eq!(
                first_isolated_node.id, nodes[4],
                "graph.isolated_nodes() did not return expected node id"
            );
            assert!(
                isolated_nodes_iter.next().is_none(),
                "graph.isolated_nodes() returned more nodes than expected"
            );
        }

        #[test]
        fn test_edges() {
            let (graph, _, edges) =
                $crate::create_directed_test_graph!($graph_constructer, $add_node, $add_edge);
            let edges_count = graph.edges().count();
            assert_eq!(
                edges_count,
                $crate::utils::testing::DIRECTED_TEST_GRAPH_EDGE_COUNT,
                "graph.edges().count() did not match expected value"
            );
            for edge in graph.edges() {
                assert!(
                    edges.contains(&edge.id),
                    "graph.edges() contained unexpected edge id: {:?}",
                    edge.id
                );
            }
        }

        #[test]
        fn test_edges_mut() {
            let (mut graph, _, edges) =
                $crate::create_directed_test_graph!($graph_constructer, $add_node, $add_edge);
            let edges_count = graph.edges_mut().count();
            assert_eq!(
                edges_count,
                $crate::utils::testing::DIRECTED_TEST_GRAPH_EDGE_COUNT,
                "graph.edges_mut().count() did not match expected value"
            );
            for edge in graph.edges_mut() {
                assert!(
                    edges.contains(&edge.id),
                    "graph.edges_mut() contained unexpected edge id: {:?}",
                    edge.id
                );
            }
        }

        #[test]
        fn test_node() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructer, $add_node, $add_edge);
            for &node_id in &nodes {
                let node = graph.node(node_id).unwrap();
                assert_eq!(
                    node.id, node_id,
                    "graph.node() did not return expected node id"
                );
            }
        }

        #[test]
        fn test_node_mut() {
            let (mut graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructer, $add_node, $add_edge);
            for &node_id in &nodes {
                let node = graph.node_mut(node_id).unwrap();
                assert_eq!(
                    node.id, node_id,
                    "graph.node_mut() did not return expected node id"
                );
            }
        }

        #[test]
        fn test_edge() {
            let (graph, _, edges) =
                $crate::create_directed_test_graph!($graph_constructer, $add_node, $add_edge);
            for &edge_id in &edges {
                let edge = graph.edge(edge_id).unwrap();
                assert_eq!(
                    edge.id, edge_id,
                    "graph.edge() did not return expected edge id"
                );
            }
        }

        #[test]
        fn test_edge_mut() {
            let (mut graph, _, edges) =
                $crate::create_directed_test_graph!($graph_constructer, $add_node, $add_edge);
            for &edge_id in &edges {
                let edge = graph.edge_mut(edge_id).unwrap();
                assert_eq!(
                    edge.id, edge_id,
                    "graph.edge_mut() did not return expected edge id"
                );
            }
        }

        #[test]
        fn test_in_degree() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructer, $add_node, $add_edge);
            assert_eq!(
                graph.in_degree(nodes[0]),
                0,
                "graph.in_degree() did not return expected value for node 0"
            );
            assert_eq!(
                graph.in_degree(nodes[1]),
                1,
                "graph.in_degree() did not return expected value for node 1"
            );
            assert_eq!(
                graph.in_degree(nodes[2]),
                2,
                "graph.in_degree() did not return expected value for node 2"
            );
            assert_eq!(
                graph.in_degree(nodes[3]),
                1,
                "graph.in_degree() did not return expected value for node 3"
            );
            assert_eq!(
                graph.in_degree(nodes[4]),
                0,
                "graph.in_degree() did not return expected value for node 4"
            );
        }

        #[test]
        fn test_out_degree() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructer, $add_node, $add_edge);
            assert_eq!(
                graph.out_degree(nodes[0]),
                2,
                "graph.out_degree() did not return expected value for node 0"
            );
            assert_eq!(
                graph.out_degree(nodes[1]),
                1,
                "graph.out_degree() did not return expected value for node 1"
            );
            assert_eq!(
                graph.out_degree(nodes[2]),
                0,
                "graph.out_degree() did not return expected value for node 2"
            );
            assert_eq!(
                graph.out_degree(nodes[3]),
                1,
                "graph.out_degree() did not return expected value for node 3"
            );
            assert_eq!(
                graph.out_degree(nodes[4]),
                0,
                "graph.out_degree() did not return expected value for node 4"
            );
        }

        #[test]
        fn test_degree() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructer, $add_node, $add_edge);
            assert_eq!(
                graph.degree(nodes[0]),
                2,
                "graph.degree() did not return expected value for node 0"
            );
            assert_eq!(
                graph.degree(nodes[1]),
                2,
                "graph.degree() did not return expected value for node 1"
            );
            assert_eq!(
                graph.degree(nodes[2]),
                2,
                "graph.degree() did not return expected value for node 2"
            );
            assert_eq!(
                graph.degree(nodes[3]),
                2,
                "graph.degree() did not return expected value for node 3"
            );
            assert_eq!(
                graph.degree(nodes[4]),
                0,
                "graph.degree() did not return expected value for node 4"
            );
        }

        fn check_if_incoming_edges_matches<T: core::hash::Hash + Eq + core::fmt::Debug>(
            mut expected_edges: hashbrown::hash_set::HashSet<T, foldhash::fast::RandomState>,
            actual_edges: impl Iterator<Item = T>,
            node_number: usize,
        ) {
            for edge in actual_edges {
                assert!(
                    expected_edges.contains(&edge),
                    "graph.incoming_edges() contained unexpected edge id: {:?} for node {}",
                    edge,
                    node_number
                );
                expected_edges.remove(&edge);
            }
            assert!(
                expected_edges.is_empty(),
                "graph.incoming_edges() did not return all expected edges for node {}",
                node_number
            );
        }

        #[test]
        fn test_incoming_edges() {
            let (graph, nodes, edges) =
                $crate::create_directed_test_graph!($graph_constructer, $add_node, $add_edge);
            assert!(
                graph.incoming_edges(nodes[0]).next().is_none(),
                "graph.incoming_edges() did not return an empty iterator for node 0"
            );

            let expected_edges_one =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[0]
                ]);
            check_if_incoming_edges_matches(
                expected_edges_one,
                graph.incoming_edges(nodes[1]).map(|edge| edge.id),
                1,
            );

            let expected_edges_two =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[1], edges[3],
                ]);
            check_if_incoming_edges_matches(
                expected_edges_two,
                graph.incoming_edges(nodes[2]).map(|edge| edge.id),
                2,
            );

            let expected_edges_three =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[2]
                ]);
            check_if_incoming_edges_matches(
                expected_edges_three,
                graph.incoming_edges(nodes[3]).map(|edge| edge.id),
                3,
            );

            assert!(
                graph.incoming_edges(nodes[4]).next().is_none(),
                "graph.incoming_edges() did not return an empty iterator for node 4"
            );
        }

        fn check_if_incoming_edges_mut_matches<T: core::hash::Hash + Eq + core::fmt::Debug>(
            mut expected_edges: hashbrown::hash_set::HashSet<T, foldhash::fast::RandomState>,
            actual_edges: impl Iterator<Item = T>,
            node_number: usize,
        ) {
            for edge in actual_edges {
                assert!(
                    expected_edges.contains(&edge),
                    "graph.incoming_edges() contained unexpected edge id: {:?} for node {}",
                    edge,
                    node_number
                );
                expected_edges.remove(&edge);
            }
            assert!(
                expected_edges.is_empty(),
                "graph.incoming_edges() did not return all expected edges for node {}",
                node_number
            );
        }

        #[test]
        fn test_incoming_edges_mut() {
            let (mut graph, nodes, edges) =
                $crate::create_directed_test_graph!($graph_constructer, $add_node, $add_edge);
            assert!(
                graph.incoming_edges_mut(nodes[0]).next().is_none(),
                "graph.incoming_edges_mut() did not return an empty iterator for node 0"
            );

            let expected_edges_one =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[0]
                ]);
            check_if_incoming_edges_mut_matches(
                expected_edges_one,
                graph.incoming_edges_mut(nodes[1]).map(|edge| edge.id),
                1,
            );

            let expected_edges_two =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[1], edges[3],
                ]);
            check_if_incoming_edges_mut_matches(
                expected_edges_two,
                graph.incoming_edges_mut(nodes[2]).map(|edge| edge.id),
                2,
            );

            let expected_edges_three =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[2]
                ]);
            check_if_incoming_edges_mut_matches(
                expected_edges_three,
                graph.incoming_edges_mut(nodes[3]).map(|edge| edge.id),
                3,
            );

            assert!(
                graph.incoming_edges_mut(nodes[4]).next().is_none(),
                "graph.incoming_edges_mut() did not return an empty iterator for node 4"
            );
        }
    };
}
