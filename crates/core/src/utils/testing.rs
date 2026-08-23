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
            $crate::utils::testing::DIRECTED_TEST_GRAPH_NODE_COUNT
        );
        assert_eq!(
            edges.len(),
            $crate::utils::testing::DIRECTED_TEST_GRAPH_EDGE_COUNT
        );

        (graph, nodes, edges)
    }};
}

/// Generates a suite of tests for [`DirectedGraph`][crate::graph::DirectedGraph]
/// implementations.
///
/// One test is generated for each method in the [`DirectedGraph`][crate::graph::DirectedGraph]
/// trait. The following invariants are expected from the graph implementation:
/// - If the most recently added node or edge is removed, its ID will no longer be valid. I.e.,
///   calling methods with that ID should return `None` or indicate non-existence.
///
/// The arguments to this macro are as follows (`G` is used to denote the graph type being tested).
/// For a reference usage, see the tests in [`crate::utils::test_graphs::directed`].
/// - `$graph_constructor`: An expression that constructs a new instance of the graph type to be
///   tested. The generated graph must be empty (e.g. `G::new()`).
/// - `$add_node`: An expression that adds a node to the graph. It must take two arguments: a
///   mutable reference to the graph and the node weight. It must return the `<G as Graph>::NodeId`
///   of the added node.
/// - `$remove_node`: An expression that removes a node from the graph. It must take two arguments:
///   a mutable reference to the graph and the `<G as Graph>::NodeId` of the node to be removed. The
///   method should not return anything, i.e., it should panic on failure.
/// - `$add_edge`: An expression that adds an edge to the graph. It must take four arguments: a
///   mutable reference to the graph, the `<G as Graph>::NodeId` of the source and target nodes, and
///   the edge weight. It must return the `<G as Graph>::EdgeId` of the added edge.
/// - `$remove_edge`: An expression that removes an edge from the graph. It must take two arguments:
///   a mutable reference to the graph and the `<G as Graph>::EdgeId` of the edge to be removed. The
///   method should not return anything, i.e., it should panic on failure.
#[macro_export]
macro_rules! test_directed_graph {
    (
        $graph_constructor:expr,
        $add_node:expr,
        $remove_node:expr,
        $add_edge:expr,
        $remove_edge:expr
    ) => {
        #[test]
        fn test_cardinality() {
            let (mut graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            assert_eq!(
                DirectedGraph::node_count(&graph),
                $crate::utils::testing::DIRECTED_TEST_GRAPH_NODE_COUNT,
                "DirectedGraph::node_count() did not match expected value"
            );
            assert_eq!(
                DirectedGraph::edge_count(&graph),
                $crate::utils::testing::DIRECTED_TEST_GRAPH_EDGE_COUNT,
                "DirectedGraph::edge_count() did not match expected value"
            );

            let cardinality = DirectedGraph::cardinality(&graph);
            assert_eq!(
                cardinality.order,
                $crate::utils::testing::DIRECTED_TEST_GRAPH_NODE_COUNT,
                "DirectedGraph::cardinality().order did not match expected value"
            );
            assert_eq!(
                cardinality.size,
                $crate::utils::testing::DIRECTED_TEST_GRAPH_EDGE_COUNT,
                "DirectedGraph::cardinality().size did not match expected value"
            );

            $remove_node(&mut graph, nodes[0]);
            assert_eq!(
                DirectedGraph::node_count(&graph),
                $crate::utils::testing::DIRECTED_TEST_GRAPH_NODE_COUNT - 1,
                "DirectedGraph::node_count() did not match expected value after removing node 0"
            );
            assert_eq!(
                DirectedGraph::edge_count(&graph),
                $crate::utils::testing::DIRECTED_TEST_GRAPH_EDGE_COUNT - 2,
                "DirectedGraph::edge_count() did not match expected value after removing node 0"
            );
        }

        #[test]
        fn test_nodes() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            let nodes_count = DirectedGraph::nodes(&graph).count();
            assert_eq!(
                nodes_count,
                $crate::utils::testing::DIRECTED_TEST_GRAPH_NODE_COUNT,
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

        #[test]
        fn test_nodes_mut() {
            let (mut graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            let nodes_count = DirectedGraph::nodes_mut(&mut graph).count();
            assert_eq!(
                nodes_count,
                $crate::utils::testing::DIRECTED_TEST_GRAPH_NODE_COUNT,
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

        #[test]
        fn test_isolated_nodes() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
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

        #[test]
        fn test_edges() {
            let (graph, _, edges) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            let edges_count = DirectedGraph::edges(&graph).count();
            assert_eq!(
                edges_count,
                $crate::utils::testing::DIRECTED_TEST_GRAPH_EDGE_COUNT,
                "DirectedGraph::edges().count() did not match expected value"
            );
            for edge in DirectedGraph::edges(&graph) {
                assert!(
                    edges.contains(&edge.id),
                    "DirectedGraph::edges() contained unexpected edge id: {:?}",
                    edge.id
                );
            }
        }

        #[test]
        fn test_edges_mut() {
            let (mut graph, _, edges) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            let edges_count = DirectedGraph::edges_mut(&mut graph).count();
            assert_eq!(
                edges_count,
                $crate::utils::testing::DIRECTED_TEST_GRAPH_EDGE_COUNT,
                "DirectedGraph::edges_mut().count() did not match expected value"
            );
            for edge in DirectedGraph::edges_mut(&mut graph) {
                assert!(
                    edges.contains(&edge.id),
                    "DirectedGraph::edges_mut() contained unexpected edge id: {:?}",
                    edge.id
                );
            }
        }

        #[test]
        fn test_node() {
            let (mut graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            for &node_id in &nodes {
                let node = DirectedGraph::node(&graph, node_id)
                    .expect("Expected node not found in test_node");
                assert_eq!(
                    node.id, node_id,
                    "DirectedGraph::node() did not return expected node id"
                );
            }

            // We remove node 4 here, as some graph implementations might not have stable node ids,
            // but the newest node added is likely to be removable without another node taking its
            // id.
            $remove_node(&mut graph, nodes[4]);
            assert!(
                DirectedGraph::node(&graph, nodes[4]).is_none(),
                "DirectedGraph::node() did not return None for removed node id"
            );
        }

        #[test]
        fn test_node_mut() {
            let (mut graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
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
            $remove_node(&mut graph, nodes[4]);
            assert!(
                DirectedGraph::node_mut(&mut graph, nodes[4]).is_none(),
                "DirectedGraph::node_mut() did not return None for removed node id"
            );
        }

        #[test]
        fn test_edge() {
            let (mut graph, _, edges) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            for &edge_id in &edges {
                let edge = DirectedGraph::edge(&graph, edge_id)
                    .expect("Expected edge not found in test_edge");
                assert_eq!(
                    edge.id, edge_id,
                    "DirectedGraph::edge() did not return expected edge id"
                );
            }

            $remove_edge(&mut graph, edges[3]);
            assert!(
                DirectedGraph::edge(&graph, edges[3]).is_none(),
                "DirectedGraph::edge() did not return None for removed edge id"
            );
        }

        #[test]
        fn test_edge_mut() {
            let (mut graph, _, edges) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            for &edge_id in &edges {
                let edge = DirectedGraph::edge_mut(&mut graph, edge_id)
                    .expect("Expected edge not found in test_edge_mut");
                assert_eq!(
                    edge.id, edge_id,
                    "DirectedGraph::edge_mut() did not return expected edge id"
                );
            }

            $remove_edge(&mut graph, edges[3]);
            assert!(
                DirectedGraph::edge_mut(&mut graph, edges[3]).is_none(),
                "DirectedGraph::edge_mut() did not return None for removed edge id"
            );
        }

        #[test]
        fn test_in_degree() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            assert_eq!(
                DirectedGraph::in_degree(&graph, nodes[0]),
                0,
                "DirectedGraph::in_degree() did not return expected value for node 0"
            );
            assert_eq!(
                DirectedGraph::in_degree(&graph, nodes[1]),
                1,
                "DirectedGraph::in_degree() did not return expected value for node 1"
            );
            assert_eq!(
                DirectedGraph::in_degree(&graph, nodes[2]),
                2,
                "DirectedGraph::in_degree() did not return expected value for node 2"
            );
            assert_eq!(
                DirectedGraph::in_degree(&graph, nodes[3]),
                1,
                "DirectedGraph::in_degree() did not return expected value for node 3"
            );
            assert_eq!(
                DirectedGraph::in_degree(&graph, nodes[4]),
                0,
                "DirectedGraph::in_degree() did not return expected value for node 4"
            );
        }

        #[test]
        fn test_out_degree() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            assert_eq!(
                DirectedGraph::out_degree(&graph, nodes[0]),
                2,
                "DirectedGraph::out_degree() did not return expected value for node 0"
            );
            assert_eq!(
                DirectedGraph::out_degree(&graph, nodes[1]),
                1,
                "DirectedGraph::out_degree() did not return expected value for node 1"
            );
            assert_eq!(
                DirectedGraph::out_degree(&graph, nodes[2]),
                0,
                "DirectedGraph::out_degree() did not return expected value for node 2"
            );
            assert_eq!(
                DirectedGraph::out_degree(&graph, nodes[3]),
                1,
                "DirectedGraph::out_degree() did not return expected value for node 3"
            );
            assert_eq!(
                DirectedGraph::out_degree(&graph, nodes[4]),
                0,
                "DirectedGraph::out_degree() did not return expected value for node 4"
            );
        }

        #[test]
        fn test_degree() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            assert_eq!(
                DirectedGraph::degree(&graph, nodes[0]),
                2,
                "DirectedGraph::degree() did not return expected value for node 0"
            );
            assert_eq!(
                DirectedGraph::degree(&graph, nodes[1]),
                2,
                "DirectedGraph::degree() did not return expected value for node 1"
            );
            assert_eq!(
                DirectedGraph::degree(&graph, nodes[2]),
                2,
                "DirectedGraph::degree() did not return expected value for node 2"
            );
            assert_eq!(
                DirectedGraph::degree(&graph, nodes[3]),
                2,
                "DirectedGraph::degree() did not return expected value for node 3"
            );
            assert_eq!(
                DirectedGraph::degree(&graph, nodes[4]),
                0,
                "DirectedGraph::degree() did not return expected value for node 4"
            );
        }

        /// Helper function to check if the edges returned by an iterator match the expected edges.
        ///
        /// The additional arguments are just for better error messages.
        fn check_if_edges_match<T: core::hash::Hash + Eq + core::fmt::Debug>(
            mut expected_edges: hashbrown::hash_set::HashSet<T, foldhash::fast::RandomState>,
            actual_edges: impl Iterator<Item = T>,
            method_name: &'static str,
            node_number: usize,
        ) {
            for edge in actual_edges {
                assert!(
                    expected_edges.contains(&edge),
                    "DirectedGraph::{}() contained unexpected edge id: {:?} for node {}",
                    method_name,
                    edge,
                    node_number
                );
                expected_edges.remove(&edge);
            }
            assert!(
                expected_edges.is_empty(),
                "DirectedGraph::{}() did not return all expected edges for node {}",
                method_name,
                node_number
            );
        }

        #[test]
        fn test_incoming_edges() {
            let (graph, nodes, edges) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            assert!(
                DirectedGraph::incoming_edges(&graph, nodes[0])
                    .next()
                    .is_none(),
                "DirectedGraph::incoming_edges() did not return an empty iterator for node 0"
            );

            let expected_edges_one =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[0]
                ]);
            check_if_edges_match(
                expected_edges_one,
                DirectedGraph::incoming_edges(&graph, nodes[1]).map(|edge| edge.id),
                "incoming_edges",
                1,
            );

            let expected_edges_two =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[1], edges[3],
                ]);
            check_if_edges_match(
                expected_edges_two,
                DirectedGraph::incoming_edges(&graph, nodes[2]).map(|edge| edge.id),
                "incoming_edges",
                2,
            );

            let expected_edges_three =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[2]
                ]);
            check_if_edges_match(
                expected_edges_three,
                DirectedGraph::incoming_edges(&graph, nodes[3]).map(|edge| edge.id),
                "incoming_edges",
                3,
            );

            assert!(
                DirectedGraph::incoming_edges(&graph, nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::incoming_edges() did not return an empty iterator for node 4"
            );
        }

        #[test]
        fn test_incoming_edges_mut() {
            let (mut graph, nodes, edges) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            assert!(
                DirectedGraph::incoming_edges_mut(&mut graph, nodes[0])
                    .next()
                    .is_none(),
                "DirectedGraph::incoming_edges_mut() did not return an empty iterator for node 0"
            );

            let expected_edges_one =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[0]
                ]);
            check_if_edges_match(
                expected_edges_one,
                DirectedGraph::incoming_edges_mut(&mut graph, nodes[1]).map(|edge| edge.id),
                "incoming_edges_mut",
                1,
            );

            let expected_edges_two =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[1], edges[3],
                ]);
            check_if_edges_match(
                expected_edges_two,
                DirectedGraph::incoming_edges_mut(&mut graph, nodes[2]).map(|edge| edge.id),
                "incoming_edges_mut",
                2,
            );

            let expected_edges_three =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[2]
                ]);
            check_if_edges_match(
                expected_edges_three,
                DirectedGraph::incoming_edges_mut(&mut graph, nodes[3]).map(|edge| edge.id),
                "incoming_edges_mut",
                3,
            );

            assert!(
                DirectedGraph::incoming_edges_mut(&mut graph, nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::incoming_edges_mut() did not return an empty iterator for node 4"
            );
        }

        #[test]
        fn test_outgoing_edges() {
            let (graph, nodes, edges) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            let expected_edges_zero =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[0], edges[1],
                ]);
            check_if_edges_match(
                expected_edges_zero,
                DirectedGraph::outgoing_edges(&graph, nodes[0]).map(|edge| edge.id),
                "outgoing_edges",
                0,
            );

            let expected_edges_one =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[2]
                ]);
            check_if_edges_match(
                expected_edges_one,
                DirectedGraph::outgoing_edges(&graph, nodes[1]).map(|edge| edge.id),
                "outgoing_edges",
                1,
            );

            assert!(
                DirectedGraph::outgoing_edges(&graph, nodes[2])
                    .next()
                    .is_none(),
                "DirectedGraph::outgoing_edges() did not return an empty iterator for node 2"
            );

            let expected_edges_three =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[3]
                ]);
            check_if_edges_match(
                expected_edges_three,
                DirectedGraph::outgoing_edges(&graph, nodes[3]).map(|edge| edge.id),
                "outgoing_edges",
                3,
            );

            assert!(
                DirectedGraph::outgoing_edges(&graph, nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::outgoing_edges() did not return an empty iterator for node 4"
            );
        }

        #[test]
        fn test_outgoing_edges_mut() {
            let (mut graph, nodes, edges) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            let expected_edges_zero =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[0], edges[1],
                ]);
            check_if_edges_match(
                expected_edges_zero,
                DirectedGraph::outgoing_edges_mut(&mut graph, nodes[0]).map(|edge| edge.id),
                "outgoing_edges_mut",
                0,
            );

            let expected_edges_one =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[2]
                ]);
            check_if_edges_match(
                expected_edges_one,
                DirectedGraph::outgoing_edges_mut(&mut graph, nodes[1]).map(|edge| edge.id),
                "outgoing_edges_mut",
                1,
            );

            assert!(
                DirectedGraph::outgoing_edges_mut(&mut graph, nodes[2])
                    .next()
                    .is_none(),
                "DirectedGraph::outgoing_edges_mut() did not return an empty iterator for node 2"
            );

            let expected_edges_three =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[3]
                ]);
            check_if_edges_match(
                expected_edges_three,
                DirectedGraph::outgoing_edges_mut(&mut graph, nodes[3]).map(|edge| edge.id),
                "outgoing_edges_mut",
                3,
            );

            assert!(
                DirectedGraph::outgoing_edges_mut(&mut graph, nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::outgoing_edges_mut() did not return an empty iterator for node 4"
            );
        }

        #[test]
        fn test_incident_edges() {
            let (graph, nodes, edges) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            let expected_edges_zero =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[0], edges[1],
                ]);
            check_if_edges_match(
                expected_edges_zero,
                DirectedGraph::incident_edges(&graph, nodes[0]).map(|edge| edge.id),
                "incident_edges",
                0,
            );

            let expected_edges_one =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[0], edges[2],
                ]);
            check_if_edges_match(
                expected_edges_one,
                DirectedGraph::incident_edges(&graph, nodes[1]).map(|edge| edge.id),
                "incident_edges",
                1,
            );

            let expected_edges_two =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[1], edges[3],
                ]);
            check_if_edges_match(
                expected_edges_two,
                DirectedGraph::incident_edges(&graph, nodes[2]).map(|edge| edge.id),
                "incident_edges",
                2,
            );

            let expected_edges_three =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[2], edges[3],
                ]);
            check_if_edges_match(
                expected_edges_three,
                DirectedGraph::incident_edges(&graph, nodes[3]).map(|edge| edge.id),
                "incident_edges",
                3,
            );

            assert!(
                DirectedGraph::incident_edges(&graph, nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::incident_edges() did not return an empty iterator for node 4"
            );
        }

        #[test]
        fn test_incident_edges_mut() {
            let (mut graph, nodes, edges) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            let expected_edges_zero =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[0], edges[1],
                ]);
            check_if_edges_match(
                expected_edges_zero,
                DirectedGraph::incident_edges_mut(&mut graph, nodes[0]).map(|edge| edge.id),
                "incident_edges_mut",
                0,
            );

            let expected_edges_one =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[0], edges[2],
                ]);
            check_if_edges_match(
                expected_edges_one,
                DirectedGraph::incident_edges_mut(&mut graph, nodes[1]).map(|edge| edge.id),
                "incident_edges_mut",
                1,
            );

            let expected_edges_two =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[1], edges[3],
                ]);
            check_if_edges_match(
                expected_edges_two,
                DirectedGraph::incident_edges_mut(&mut graph, nodes[2]).map(|edge| edge.id),
                "incident_edges_mut",
                2,
            );

            let expected_edges_three =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[2], edges[3],
                ]);
            check_if_edges_match(
                expected_edges_three,
                DirectedGraph::incident_edges_mut(&mut graph, nodes[3]).map(|edge| edge.id),
                "incident_edges_mut",
                3,
            );

            assert!(
                DirectedGraph::incident_edges_mut(&mut graph, nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::incident_edges_mut() did not return an empty iterator for node 4"
            );
        }

        /// Helper function to check if the nodes returned by an iterator match the expected nodes
        ///
        /// The additional arguments are just for better error messages.
        fn check_if_nodes_match<T: core::hash::Hash + Eq + core::fmt::Debug>(
            mut expected_nodes: hashbrown::hash_set::HashSet<T, foldhash::fast::RandomState>,
            actual_nodes: impl Iterator<Item = T>,
            method_name: &'static str,
            node_number: Option<usize>,
        ) {
            for node in actual_nodes {
                if let Some(node_number) = node_number {
                    assert!(
                        expected_nodes.contains(&node),
                        "DirectedGraph::{}() contained unexpected node id: {:?} for node {}",
                        method_name,
                        node,
                        node_number
                    );
                } else {
                    assert!(
                        expected_nodes.contains(&node),
                        "DirectedGraph::{}() contained unexpected node id: {:?}",
                        method_name,
                        node,
                    );
                }
                expected_nodes.remove(&node);
            }
            if let Some(node_number) = node_number {
                assert!(
                    expected_nodes.is_empty(),
                    "DirectedGraph::{}() did not return all expected nodes for node {}: {:?}",
                    method_name,
                    node_number,
                    expected_nodes
                );
            } else {
                assert!(
                    expected_nodes.is_empty(),
                    "DirectedGraph::{}() did not return all expected nodes: {:?}",
                    method_name,
                    expected_nodes
                );
            }
        }

        #[test]
        fn test_predecessors() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            assert!(
                DirectedGraph::predecessors(&graph, nodes[0])
                    .next()
                    .is_none(),
                "DirectedGraph::predecessors() did not return an empty iterator for node 0"
            );

            let expected_predecessors_one = hashbrown::hash_set::HashSet::<
                _,
                foldhash::fast::RandomState,
            >::from_iter([nodes[0]]);
            check_if_nodes_match(
                expected_predecessors_one,
                DirectedGraph::predecessors(&graph, nodes[1]),
                "predecessors",
                Some(1),
            );

            let expected_predecessors_two = hashbrown::hash_set::HashSet::<
                _,
                foldhash::fast::RandomState,
            >::from_iter([nodes[0], nodes[3]]);
            check_if_nodes_match(
                expected_predecessors_two,
                DirectedGraph::predecessors(&graph, nodes[2]),
                "predecessors",
                Some(2),
            );

            let expected_predecessors_three = hashbrown::hash_set::HashSet::<
                _,
                foldhash::fast::RandomState,
            >::from_iter([nodes[1]]);
            check_if_nodes_match(
                expected_predecessors_three,
                DirectedGraph::predecessors(&graph, nodes[3]),
                "predecessors",
                Some(3),
            );

            assert!(
                DirectedGraph::predecessors(&graph, nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::predecessors() did not return an empty iterator for node 4"
            );
        }

        #[test]
        fn test_successors() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            let expected_successors_zero = hashbrown::hash_set::HashSet::<
                _,
                foldhash::fast::RandomState,
            >::from_iter([nodes[1], nodes[2]]);
            check_if_nodes_match(
                expected_successors_zero,
                DirectedGraph::successors(&graph, nodes[0]),
                "successors",
                Some(0),
            );

            let expected_successors_one = hashbrown::hash_set::HashSet::<
                _,
                foldhash::fast::RandomState,
            >::from_iter([nodes[3]]);
            check_if_nodes_match(
                expected_successors_one,
                DirectedGraph::successors(&graph, nodes[1]),
                "successors",
                Some(1),
            );

            assert!(
                DirectedGraph::successors(&graph, nodes[2]).next().is_none(),
                "DirectedGraph::successors() did not return an empty iterator for node 2"
            );

            let expected_successors_three = hashbrown::hash_set::HashSet::<
                _,
                foldhash::fast::RandomState,
            >::from_iter([nodes[2]]);
            check_if_nodes_match(
                expected_successors_three,
                DirectedGraph::successors(&graph, nodes[3]),
                "successors",
                Some(3),
            );

            assert!(
                DirectedGraph::successors(&graph, nodes[4]).next().is_none(),
                "DirectedGraph::successors() did not return an empty iterator for node 4"
            );
        }

        #[test]
        fn test_adjacencies() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            let expected_adjacencies_zero = hashbrown::hash_set::HashSet::<
                _,
                foldhash::fast::RandomState,
            >::from_iter([nodes[1], nodes[2]]);
            check_if_nodes_match(
                expected_adjacencies_zero,
                DirectedGraph::adjacencies(&graph, nodes[0]),
                "adjacencies",
                Some(0),
            );

            let expected_adjacencies_one = hashbrown::hash_set::HashSet::<
                _,
                foldhash::fast::RandomState,
            >::from_iter([nodes[0], nodes[3]]);
            check_if_nodes_match(
                expected_adjacencies_one,
                DirectedGraph::adjacencies(&graph, nodes[1]),
                "adjacencies",
                Some(1),
            );

            let expected_adjacencies_two = hashbrown::hash_set::HashSet::<
                _,
                foldhash::fast::RandomState,
            >::from_iter([nodes[0], nodes[3]]);
            check_if_nodes_match(
                expected_adjacencies_two,
                DirectedGraph::adjacencies(&graph, nodes[2]),
                "adjacencies",
                Some(2),
            );

            let expected_adjacencies_three = hashbrown::hash_set::HashSet::<
                _,
                foldhash::fast::RandomState,
            >::from_iter([nodes[1], nodes[2]]);
            check_if_nodes_match(
                expected_adjacencies_three,
                DirectedGraph::adjacencies(&graph, nodes[3]),
                "adjacencies",
                Some(3),
            );

            assert!(
                DirectedGraph::adjacencies(&graph, nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::adjacencies() did not return an empty iterator for node 4"
            );
        }

        #[test]
        #[allow(clippy::too_many_lines)]
        fn test_edges_between() {
            let (graph, nodes, edges) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);

            // Source 0
            assert!(
                DirectedGraph::edges_between(&graph, nodes[0], nodes[0])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 0 and 0"
            );

            let expected_edges_0_1 =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[0]
                ]);
            check_if_edges_match(
                expected_edges_0_1,
                DirectedGraph::edges_between(&graph, nodes[0], nodes[1]).map(|edge| edge.id),
                "edges_between",
                0,
            );

            let expected_edges_0_2 =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[1]
                ]);
            check_if_edges_match(
                expected_edges_0_2,
                DirectedGraph::edges_between(&graph, nodes[0], nodes[2]).map(|edge| edge.id),
                "edges_between",
                0,
            );

            assert!(
                DirectedGraph::edges_between(&graph, nodes[0], nodes[3])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 0 and 3"
            );
            assert!(
                DirectedGraph::edges_between(&graph, nodes[0], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 0 and 4"
            );

            // Source 1
            assert!(
                DirectedGraph::edges_between(&graph, nodes[1], nodes[0])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 1 and 0"
            );

            assert!(
                DirectedGraph::edges_between(&graph, nodes[1], nodes[1])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 1 and 1"
            );

            assert!(
                DirectedGraph::edges_between(&graph, nodes[1], nodes[2])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 1 and 2"
            );

            let expected_edges_1_3 =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[2]
                ]);
            check_if_edges_match(
                expected_edges_1_3,
                DirectedGraph::edges_between(&graph, nodes[1], nodes[3]).map(|edge| edge.id),
                "edges_between",
                1,
            );

            assert!(
                DirectedGraph::edges_between(&graph, nodes[1], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 1 and 4"
            );

            // Source 2
            assert!(
                DirectedGraph::edges_between(&graph, nodes[2], nodes[0])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 2 and 0"
            );

            assert!(
                DirectedGraph::edges_between(&graph, nodes[2], nodes[1])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 2 and 1"
            );

            assert!(
                DirectedGraph::edges_between(&graph, nodes[2], nodes[2])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 2 and 2"
            );

            assert!(
                DirectedGraph::edges_between(&graph, nodes[2], nodes[3])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 2 and 3"
            );

            assert!(
                DirectedGraph::edges_between(&graph, nodes[2], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 2 and 4"
            );

            // Source 3
            assert!(
                DirectedGraph::edges_between(&graph, nodes[3], nodes[0])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 3 and 0"
            );

            assert!(
                DirectedGraph::edges_between(&graph, nodes[3], nodes[1])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 3 and 1"
            );

            let expected_edges_3_2 =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[3]
                ]);
            check_if_edges_match(
                expected_edges_3_2,
                DirectedGraph::edges_between(&graph, nodes[3], nodes[2]).map(|edge| edge.id),
                "edges_between",
                3,
            );

            assert!(
                DirectedGraph::edges_between(&graph, nodes[3], nodes[3])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 3 and 3"
            );

            assert!(
                DirectedGraph::edges_between(&graph, nodes[3], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 3 and 4"
            );

            // Source 4
            assert!(
                DirectedGraph::edges_between(&graph, nodes[4], nodes[0])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 4 and 0"
            );

            assert!(
                DirectedGraph::edges_between(&graph, nodes[4], nodes[1])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 4 and 1"
            );

            assert!(
                DirectedGraph::edges_between(&graph, nodes[4], nodes[2])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 4 and 2"
            );

            assert!(
                DirectedGraph::edges_between(&graph, nodes[4], nodes[3])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 4 and 3"
            );

            assert!(
                DirectedGraph::edges_between(&graph, nodes[4], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 4 and 4"
            );
        }

        #[test]
        #[allow(clippy::too_many_lines)]
        fn test_edges_between_mut() {
            let (mut graph, nodes, edges) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);

            // Source 0
            assert!(
                DirectedGraph::edges_between_mut(&mut graph, nodes[0], nodes[0])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 0 \
                 and 0"
            );

            let expected_edges_0_1 =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[0]
                ]);
            check_if_edges_match(
                expected_edges_0_1,
                DirectedGraph::edges_between_mut(&mut graph, nodes[0], nodes[1])
                    .map(|edge| edge.id),
                "edges_between_mut",
                0,
            );

            let expected_edges_0_2 =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[1]
                ]);
            check_if_edges_match(
                expected_edges_0_2,
                DirectedGraph::edges_between_mut(&mut graph, nodes[0], nodes[2])
                    .map(|edge| edge.id),
                "edges_between_mut",
                0,
            );

            assert!(
                DirectedGraph::edges_between_mut(&mut graph, nodes[0], nodes[3])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 0 \
                 and 3"
            );
            assert!(
                DirectedGraph::edges_between_mut(&mut graph, nodes[0], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 0 \
                 and 4"
            );

            // Source 1
            assert!(
                DirectedGraph::edges_between_mut(&mut graph, nodes[1], nodes[0])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 1 \
                 and 0"
            );

            assert!(
                DirectedGraph::edges_between_mut(&mut graph, nodes[1], nodes[1])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 1 \
                 and 1"
            );

            assert!(
                DirectedGraph::edges_between_mut(&mut graph, nodes[1], nodes[2])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 1 \
                 and 2"
            );

            let expected_edges_1_3 =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[2]
                ]);
            check_if_edges_match(
                expected_edges_1_3,
                DirectedGraph::edges_between_mut(&mut graph, nodes[1], nodes[3])
                    .map(|edge| edge.id),
                "edges_between_mut",
                1,
            );

            assert!(
                DirectedGraph::edges_between_mut(&mut graph, nodes[1], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 1 \
                 and 4"
            );

            // Source 2
            assert!(
                DirectedGraph::edges_between_mut(&mut graph, nodes[2], nodes[0])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 2 \
                 and 0"
            );

            assert!(
                DirectedGraph::edges_between_mut(&mut graph, nodes[2], nodes[1])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 2 \
                 and 1"
            );

            assert!(
                DirectedGraph::edges_between_mut(&mut graph, nodes[2], nodes[2])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 2 \
                 and 2"
            );

            assert!(
                DirectedGraph::edges_between_mut(&mut graph, nodes[2], nodes[3])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 2 \
                 and 3"
            );

            assert!(
                DirectedGraph::edges_between_mut(&mut graph, nodes[2], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 2 \
                 and 4"
            );

            // Source 3
            assert!(
                DirectedGraph::edges_between_mut(&mut graph, nodes[3], nodes[0])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 3 \
                 and 0"
            );

            assert!(
                DirectedGraph::edges_between_mut(&mut graph, nodes[3], nodes[1])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 3 \
                 and 1"
            );

            let expected_edges_3_2 =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[3]
                ]);
            check_if_edges_match(
                expected_edges_3_2,
                DirectedGraph::edges_between_mut(&mut graph, nodes[3], nodes[2])
                    .map(|edge| edge.id),
                "edges_between_mut",
                3,
            );

            assert!(
                DirectedGraph::edges_between_mut(&mut graph, nodes[3], nodes[3])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 3 \
                 and 3"
            );

            assert!(
                DirectedGraph::edges_between_mut(&mut graph, nodes[3], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 3 \
                 and 4"
            );

            // Source 4
            assert!(
                DirectedGraph::edges_between_mut(&mut graph, nodes[4], nodes[0])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 4 \
                 and 0"
            );

            assert!(
                DirectedGraph::edges_between_mut(&mut graph, nodes[4], nodes[1])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 4 \
                 and 1"
            );

            assert!(
                DirectedGraph::edges_between_mut(&mut graph, nodes[4], nodes[2])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 4 \
                 and 2"
            );

            assert!(
                DirectedGraph::edges_between_mut(&mut graph, nodes[4], nodes[3])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 4 \
                 and 3"
            );

            assert!(
                DirectedGraph::edges_between_mut(&mut graph, nodes[4], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 4 \
                 and 4"
            );
        }

        #[test]
        #[allow(clippy::too_many_lines)]
        fn test_edges_connecting() {
            let (graph, nodes, edges) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);

            // Source 0
            assert!(
                DirectedGraph::edges_connecting(&graph, nodes[0], nodes[0])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting() did not return an empty iterator for nodes 0 \
                 and 0"
            );

            let expected_edges_0_1 =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[0]
                ]);
            check_if_edges_match(
                expected_edges_0_1,
                DirectedGraph::edges_connecting(&graph, nodes[0], nodes[1]).map(|edge| edge.id),
                "edges_connecting",
                0,
            );

            let expected_edges_0_2 =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[1]
                ]);

            check_if_edges_match(
                expected_edges_0_2,
                DirectedGraph::edges_connecting(&graph, nodes[0], nodes[2]).map(|edge| edge.id),
                "edges_connecting",
                0,
            );

            assert!(
                DirectedGraph::edges_connecting(&graph, nodes[0], nodes[3])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting() did not return an empty iterator for nodes 0 \
                 and 3"
            );

            assert!(
                DirectedGraph::edges_connecting(&graph, nodes[0], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting() did not return an empty iterator for nodes 0 \
                 and 4"
            );

            // Source 1
            assert!(
                DirectedGraph::edges_connecting(&graph, nodes[1], nodes[1])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting() did not return an empty iterator for nodes 1 \
                 and 1"
            );

            assert!(
                DirectedGraph::edges_connecting(&graph, nodes[1], nodes[2])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting() did not return an empty iterator for nodes 1 \
                 and 2"
            );

            let expected_edges_1_3 =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[2]
                ]);
            check_if_edges_match(
                expected_edges_1_3,
                DirectedGraph::edges_connecting(&graph, nodes[1], nodes[3]).map(|edge| edge.id),
                "edges_connecting",
                1,
            );

            assert!(
                DirectedGraph::edges_connecting(&graph, nodes[1], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting() did not return an empty iterator for nodes 1 \
                 and 4"
            );

            // Source 2
            assert!(
                DirectedGraph::edges_connecting(&graph, nodes[2], nodes[2])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting() did not return an empty iterator for nodes 2 \
                 and 2"
            );

            let expected_edges_2_3 =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[3]
                ]);
            check_if_edges_match(
                expected_edges_2_3,
                DirectedGraph::edges_connecting(&graph, nodes[2], nodes[3]).map(|edge| edge.id),
                "edges_connecting",
                2,
            );

            assert!(
                DirectedGraph::edges_connecting(&graph, nodes[2], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting() did not return an empty iterator for nodes 2 \
                 and 4"
            );

            // Source 3
            assert!(
                DirectedGraph::edges_connecting(&graph, nodes[3], nodes[3])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting() did not return an empty iterator for nodes 3 \
                 and 3"
            );

            assert!(
                DirectedGraph::edges_connecting(&graph, nodes[3], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting() did not return an empty iterator for nodes 3 \
                 and 4"
            );

            // Source 4
            assert!(
                DirectedGraph::edges_connecting(&graph, nodes[4], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting() did not return an empty iterator for nodes 4 \
                 and 4"
            );

            // Check if swapping lhs and rhs matters
            for i in nodes.iter() {
                for j in nodes.iter() {
                    let edges_lhs_rhs: hashbrown::HashSet<_, foldhash::fast::RandomState> =
                        DirectedGraph::edges_connecting(&graph, *i, *j)
                            .map(|edge| edge.id)
                            .collect();
                    let edges_rhs_lhs: hashbrown::HashSet<_, foldhash::fast::RandomState> =
                        DirectedGraph::edges_connecting(&graph, *j, *i)
                            .map(|edge| edge.id)
                            .collect();
                    assert_eq!(
                        edges_lhs_rhs, edges_rhs_lhs,
                        "DirectedGraph::edges_connecting() returned different edges when swapping \
                         source and target nodes: {:?} and {:?}",
                        i, j
                    );
                }
            }
        }

        #[test]
        #[allow(clippy::too_many_lines)]
        fn test_edges_connecting_mut() {
            let (mut graph, nodes, edges) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);

            // Source 0
            assert!(
                DirectedGraph::edges_connecting_mut(&mut graph, nodes[0], nodes[0])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting_mut() did not return an empty iterator for nodes \
                 0 and 0"
            );

            let expected_edges_0_1 =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[0]
                ]);
            check_if_edges_match(
                expected_edges_0_1,
                DirectedGraph::edges_connecting_mut(&mut graph, nodes[0], nodes[1])
                    .map(|edge| edge.id),
                "edges_connecting_mut",
                0,
            );

            let expected_edges_0_2 =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[1]
                ]);

            check_if_edges_match(
                expected_edges_0_2,
                DirectedGraph::edges_connecting_mut(&mut graph, nodes[0], nodes[2])
                    .map(|edge| edge.id),
                "edges_connecting_mut",
                0,
            );

            assert!(
                DirectedGraph::edges_connecting_mut(&mut graph, nodes[0], nodes[3])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting_mut() did not return an empty iterator for nodes \
                 0 and 3"
            );

            assert!(
                DirectedGraph::edges_connecting_mut(&mut graph, nodes[0], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting_mut() did not return an empty iterator for nodes \
                 0 and 4"
            );

            // Source 1
            assert!(
                DirectedGraph::edges_connecting_mut(&mut graph, nodes[1], nodes[1])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting_mut() did not return an empty iterator for nodes \
                 1 and 1"
            );

            assert!(
                DirectedGraph::edges_connecting_mut(&mut graph, nodes[1], nodes[2])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting_mut() did not return an empty iterator for nodes \
                 1 and 2"
            );

            let expected_edges_1_3 =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[2]
                ]);
            check_if_edges_match(
                expected_edges_1_3,
                DirectedGraph::edges_connecting_mut(&mut graph, nodes[1], nodes[3])
                    .map(|edge| edge.id),
                "edges_connecting_mut",
                1,
            );

            assert!(
                DirectedGraph::edges_connecting_mut(&mut graph, nodes[1], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting_mut() did not return an empty iterator for nodes \
                 1 and 4"
            );

            // Source 2
            assert!(
                DirectedGraph::edges_connecting_mut(&mut graph, nodes[2], nodes[2])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting_mut() did not return an empty iterator for nodes \
                 2 and 2"
            );

            let expected_edges_2_3 =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[3]
                ]);
            check_if_edges_match(
                expected_edges_2_3,
                DirectedGraph::edges_connecting_mut(&mut graph, nodes[2], nodes[3])
                    .map(|edge| edge.id),
                "edges_connecting_mut",
                2,
            );

            assert!(
                DirectedGraph::edges_connecting_mut(&mut graph, nodes[2], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting_mut() did not return an empty iterator for nodes \
                 2 and 4"
            );

            // Source 3
            assert!(
                DirectedGraph::edges_connecting_mut(&mut graph, nodes[3], nodes[3])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting_mut() did not return an empty iterator for nodes \
                 3 and 3"
            );

            assert!(
                DirectedGraph::edges_connecting_mut(&mut graph, nodes[3], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting_mut() did not return an empty iterator for nodes \
                 3 and 4"
            );

            // Source 4
            assert!(
                DirectedGraph::edges_connecting_mut(&mut graph, nodes[4], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting_mut() did not return an empty iterator for nodes \
                 4 and 4"
            );

            // Check if swapping lhs and rhs matters
            for i in nodes.iter() {
                for j in nodes.iter() {
                    let edges_lhs_rhs: hashbrown::HashSet<_, foldhash::fast::RandomState> =
                        DirectedGraph::edges_connecting_mut(&mut graph, *i, *j)
                            .map(|edge| edge.id)
                            .collect();
                    let edges_rhs_lhs: hashbrown::HashSet<_, foldhash::fast::RandomState> =
                        DirectedGraph::edges_connecting_mut(&mut graph, *j, *i)
                            .map(|edge| edge.id)
                            .collect();
                    assert_eq!(
                        edges_lhs_rhs, edges_rhs_lhs,
                        "DirectedGraph::edges_connecting_mut() returned different edges when \
                         swapping source and target nodes: {:?} and {:?}",
                        i, j
                    );
                }
            }
        }

        #[test]
        fn test_contains_node() {
            let (mut graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);

            for node in nodes.iter() {
                assert!(
                    DirectedGraph::contains_node(&graph, *node),
                    "DirectedGraph::contains_node() returned false for existing node: {:?}",
                    node
                );
            }

            // We remove node 4 here, as some graph implementations might not have stable node ids,
            // but the newest node added is likely to be removable without another node taking its
            // id.
            $remove_node(&mut graph, nodes[4]);
            assert!(
                !DirectedGraph::contains_node(&graph, nodes[4]),
                "DirectedGraph::contains_node() returned true for removed node: {:?}",
                nodes[4]
            );
        }

        #[test]
        fn test_contains_edge() {
            let (mut graph, _, edges) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);

            for edge in edges.iter() {
                assert!(
                    DirectedGraph::contains_edge(&graph, *edge),
                    "DirectedGraph::contains_edge() returned false for existing edge: {:?}",
                    edge
                );
            }
            // We remove edge 3 here, as some graph implementations might not have stable edge ids,
            // but the newest edge added is likely to be removable without another edge taking its
            // id.
            $remove_edge(&mut graph, edges[3]);
            assert!(
                !DirectedGraph::contains_edge(&graph, edges[3]),
                "DirectedGraph::contains_edge() returned true for removed edge: {:?}",
                edges[3]
            );
        }

        #[test]
        fn test_is_adjacent() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);

            let adjacent_node_pairs = [
                (nodes[0], nodes[1]),
                (nodes[0], nodes[2]),
                (nodes[1], nodes[3]),
                (nodes[3], nodes[2]),
            ];

            for source in nodes.iter() {
                for target in nodes.iter() {
                    let expected_adjacency = adjacent_node_pairs.contains(&(*source, *target));
                    assert_eq!(
                        DirectedGraph::is_adjacent(&graph, *source, *target),
                        expected_adjacency,
                        "DirectedGraph::is_adjacent() returned incorrect result for nodes {:?} \
                         and {:?}",
                        source,
                        target
                    );
                }
            }
        }

        #[test]
        fn test_is_empty() {
            let (graph, _, _) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);

            assert!(
                !DirectedGraph::is_empty(&graph),
                "DirectedGraph::is_empty() returned true for a non-empty graph"
            );

            let mut new_graph = $graph_constructor();
            assert!(
                DirectedGraph::is_empty(&new_graph),
                "DirectedGraph::is_empty() returned false for an empty graph"
            );

            let node_one = $add_node(&mut new_graph, ());
            assert!(
                !DirectedGraph::is_empty(&new_graph),
                "DirectedGraph::is_empty() returned true for a graph with one node"
            );

            $remove_node(&mut new_graph, node_one);
            assert!(
                DirectedGraph::is_empty(&new_graph),
                "DirectedGraph::is_empty() returned false for an empty graph after removing the \
                 only node"
            );
        }

        #[test]
        fn test_sources() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);

            let expected_sources =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    nodes[0], nodes[4],
                ]);
            check_if_nodes_match(
                expected_sources,
                DirectedGraph::sources(&graph).map(|n| n.id),
                "sources",
                None,
            );
        }

        #[test]
        fn test_sinks() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);

            let expected_sinks =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    nodes[2], nodes[4],
                ]);
            check_if_nodes_match(
                expected_sinks,
                DirectedGraph::sinks(&graph).map(|n| n.id),
                "sinks",
                None,
            );
        }
    };
}
