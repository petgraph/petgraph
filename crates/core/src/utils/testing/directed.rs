pub const DIRECTED_TEST_GRAPH_NODE_COUNT: usize = 5;
pub const DIRECTED_TEST_GRAPH_EDGE_COUNT: usize = 4;

/// A macro to create a simple directed graph for testing purposes.
///
/// The graph looks as follows:
/// ```text
/// 0 --> 1
/// |      |
/// v      v
/// 2 <----3     4
/// ```
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
            $crate::utils::testing::directed::DIRECTED_TEST_GRAPH_NODE_COUNT
        );
        assert_eq!(
            edges.len(),
            $crate::utils::testing::directed::DIRECTED_TEST_GRAPH_EDGE_COUNT
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
///   calling methods with that ID should return `None` or otherwise indicate non-existence.
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
        fn test_density_hint() {
            let (graph, _, _) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);

            // DensityHint is a hint, so this is intentionally only a smoke test.
            let _density_hint = $crate::graph::DirectedGraph::density_hint(&graph);
        }

        #[test]
        fn test_cardinality() {
            let (mut graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            assert_eq!(
                $crate::graph::DirectedGraph::node_count(&graph),
                $crate::utils::testing::directed::DIRECTED_TEST_GRAPH_NODE_COUNT,
                "DirectedGraph::node_count() did not match expected value"
            );
            assert_eq!(
                $crate::graph::DirectedGraph::edge_count(&graph),
                $crate::utils::testing::directed::DIRECTED_TEST_GRAPH_EDGE_COUNT,
                "DirectedGraph::edge_count() did not match expected value"
            );

            let cardinality = $crate::graph::DirectedGraph::cardinality(&graph);
            assert_eq!(
                cardinality.order,
                $crate::utils::testing::directed::DIRECTED_TEST_GRAPH_NODE_COUNT,
                "DirectedGraph::cardinality().order did not match expected value"
            );
            assert_eq!(
                cardinality.size,
                $crate::utils::testing::directed::DIRECTED_TEST_GRAPH_EDGE_COUNT,
                "DirectedGraph::cardinality().size did not match expected value"
            );

            $remove_node(&mut graph, nodes[0]);
            assert_eq!(
                $crate::graph::DirectedGraph::node_count(&graph),
                $crate::utils::testing::directed::DIRECTED_TEST_GRAPH_NODE_COUNT - 1,
                "DirectedGraph::node_count() did not match expected value after removing node 0"
            );
            assert_eq!(
                $crate::graph::DirectedGraph::edge_count(&graph),
                $crate::utils::testing::directed::DIRECTED_TEST_GRAPH_EDGE_COUNT - 2,
                "DirectedGraph::edge_count() did not match expected value after removing node 0"
            );
        }

        #[test]
        fn test_nodes() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            let nodes_count = $crate::graph::DirectedGraph::nodes(&graph).count();
            assert_eq!(
                nodes_count,
                $crate::utils::testing::directed::DIRECTED_TEST_GRAPH_NODE_COUNT,
                "DirectedGraph::nodes().count() did not match expected value"
            );
            for node in $crate::graph::DirectedGraph::nodes(&graph) {
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
            let nodes_count = $crate::graph::DirectedGraph::nodes_mut(&mut graph).count();
            assert_eq!(
                nodes_count,
                $crate::utils::testing::directed::DIRECTED_TEST_GRAPH_NODE_COUNT,
                "DirectedGraph::nodes_mut().count() did not match expected value"
            );
            for node in $crate::graph::DirectedGraph::nodes_mut(&mut graph) {
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
            let isolated_nodes_count = $crate::graph::DirectedGraph::isolated_nodes(&graph).count();
            assert_eq!(
                isolated_nodes_count, 1,
                "DirectedGraph::isolated_nodes().count() did not match expected value"
            );
            let mut isolated_nodes_iter = $crate::graph::DirectedGraph::isolated_nodes(&graph);
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
            let edges_count = $crate::graph::DirectedGraph::edges(&graph).count();
            assert_eq!(
                edges_count,
                $crate::utils::testing::directed::DIRECTED_TEST_GRAPH_EDGE_COUNT,
                "DirectedGraph::edges().count() did not match expected value"
            );
            for edge in $crate::graph::DirectedGraph::edges(&graph) {
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
            let edges_count = $crate::graph::DirectedGraph::edges_mut(&mut graph).count();
            assert_eq!(
                edges_count,
                $crate::utils::testing::directed::DIRECTED_TEST_GRAPH_EDGE_COUNT,
                "DirectedGraph::edges_mut().count() did not match expected value"
            );
            for edge in $crate::graph::DirectedGraph::edges_mut(&mut graph) {
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
                let node = $crate::graph::DirectedGraph::node(&graph, node_id)
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
                $crate::graph::DirectedGraph::node(&graph, nodes[4]).is_none(),
                "DirectedGraph::node() did not return None for removed node id"
            );
        }

        #[test]
        fn test_node_mut() {
            let (mut graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            for &node_id in &nodes {
                let node = $crate::graph::DirectedGraph::node_mut(&mut graph, node_id)
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
                $crate::graph::DirectedGraph::node_mut(&mut graph, nodes[4]).is_none(),
                "DirectedGraph::node_mut() did not return None for removed node id"
            );
        }

        #[test]
        fn test_edge() {
            let (mut graph, _, edges) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            for &edge_id in &edges {
                let edge = $crate::graph::DirectedGraph::edge(&graph, edge_id)
                    .expect("Expected edge not found in test_edge");
                assert_eq!(
                    edge.id, edge_id,
                    "DirectedGraph::edge() did not return expected edge id"
                );
            }

            $remove_edge(&mut graph, edges[3]);
            assert!(
                $crate::graph::DirectedGraph::edge(&graph, edges[3]).is_none(),
                "DirectedGraph::edge() did not return None for removed edge id"
            );
        }

        #[test]
        fn test_edge_mut() {
            let (mut graph, _, edges) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            for &edge_id in &edges {
                let edge = $crate::graph::DirectedGraph::edge_mut(&mut graph, edge_id)
                    .expect("Expected edge not found in test_edge_mut");
                assert_eq!(
                    edge.id, edge_id,
                    "DirectedGraph::edge_mut() did not return expected edge id"
                );
            }

            $remove_edge(&mut graph, edges[3]);
            assert!(
                $crate::graph::DirectedGraph::edge_mut(&mut graph, edges[3]).is_none(),
                "DirectedGraph::edge_mut() did not return None for removed edge id"
            );
        }

        #[test]
        fn test_in_degree() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            assert_eq!(
                $crate::graph::DirectedGraph::in_degree(&graph, nodes[0]),
                0,
                "DirectedGraph::in_degree() did not return expected value for node 0"
            );
            assert_eq!(
                $crate::graph::DirectedGraph::in_degree(&graph, nodes[1]),
                1,
                "DirectedGraph::in_degree() did not return expected value for node 1"
            );
            assert_eq!(
                $crate::graph::DirectedGraph::in_degree(&graph, nodes[2]),
                2,
                "DirectedGraph::in_degree() did not return expected value for node 2"
            );
            assert_eq!(
                $crate::graph::DirectedGraph::in_degree(&graph, nodes[3]),
                1,
                "DirectedGraph::in_degree() did not return expected value for node 3"
            );
            assert_eq!(
                $crate::graph::DirectedGraph::in_degree(&graph, nodes[4]),
                0,
                "DirectedGraph::in_degree() did not return expected value for node 4"
            );
        }

        #[test]
        fn test_out_degree() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            assert_eq!(
                $crate::graph::DirectedGraph::out_degree(&graph, nodes[0]),
                2,
                "DirectedGraph::out_degree() did not return expected value for node 0"
            );
            assert_eq!(
                $crate::graph::DirectedGraph::out_degree(&graph, nodes[1]),
                1,
                "DirectedGraph::out_degree() did not return expected value for node 1"
            );
            assert_eq!(
                $crate::graph::DirectedGraph::out_degree(&graph, nodes[2]),
                0,
                "DirectedGraph::out_degree() did not return expected value for node 2"
            );
            assert_eq!(
                $crate::graph::DirectedGraph::out_degree(&graph, nodes[3]),
                1,
                "DirectedGraph::out_degree() did not return expected value for node 3"
            );
            assert_eq!(
                $crate::graph::DirectedGraph::out_degree(&graph, nodes[4]),
                0,
                "DirectedGraph::out_degree() did not return expected value for node 4"
            );
        }

        #[test]
        fn test_degree() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            assert_eq!(
                $crate::graph::DirectedGraph::degree(&graph, nodes[0]),
                2,
                "DirectedGraph::degree() did not return expected value for node 0"
            );
            assert_eq!(
                $crate::graph::DirectedGraph::degree(&graph, nodes[1]),
                2,
                "DirectedGraph::degree() did not return expected value for node 1"
            );
            assert_eq!(
                $crate::graph::DirectedGraph::degree(&graph, nodes[2]),
                2,
                "DirectedGraph::degree() did not return expected value for node 2"
            );
            assert_eq!(
                $crate::graph::DirectedGraph::degree(&graph, nodes[3]),
                2,
                "DirectedGraph::degree() did not return expected value for node 3"
            );
            assert_eq!(
                $crate::graph::DirectedGraph::degree(&graph, nodes[4]),
                0,
                "DirectedGraph::degree() did not return expected value for node 4"
            );
        }

        #[test]
        fn test_incoming_edges() {
            let (graph, nodes, edges) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            assert!(
                $crate::graph::DirectedGraph::incoming_edges(&graph, nodes[0])
                    .next()
                    .is_none(),
                "DirectedGraph::incoming_edges() did not return an empty iterator for node 0"
            );

            let expected_edges_one = [edges[0]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_one,
                $crate::graph::DirectedGraph::incoming_edges(&graph, nodes[1]).map(|edge| edge.id),
                "incoming_edges",
                "DirectedGraph",
                1,
            );

            let expected_edges_two = [edges[1], edges[3]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_two,
                $crate::graph::DirectedGraph::incoming_edges(&graph, nodes[2]).map(|edge| edge.id),
                "incoming_edges",
                "DirectedGraph",
                2,
            );

            let expected_edges_three = [edges[2]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_three,
                $crate::graph::DirectedGraph::incoming_edges(&graph, nodes[3]).map(|edge| edge.id),
                "incoming_edges",
                "DirectedGraph",
                3,
            );

            assert!(
                $crate::graph::DirectedGraph::incoming_edges(&graph, nodes[4])
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
                $crate::graph::DirectedGraph::incoming_edges_mut(&mut graph, nodes[0])
                    .next()
                    .is_none(),
                "DirectedGraph::incoming_edges_mut() did not return an empty iterator for node 0"
            );

            let expected_edges_one = [edges[0]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_one,
                $crate::graph::DirectedGraph::incoming_edges_mut(&mut graph, nodes[1])
                    .map(|edge| edge.id),
                "incoming_edges_mut",
                "DirectedGraph",
                1,
            );

            let expected_edges_two = [edges[1], edges[3]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_two,
                $crate::graph::DirectedGraph::incoming_edges_mut(&mut graph, nodes[2])
                    .map(|edge| edge.id),
                "incoming_edges_mut",
                "DirectedGraph",
                2,
            );

            let expected_edges_three = [edges[2]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_three,
                $crate::graph::DirectedGraph::incoming_edges_mut(&mut graph, nodes[3])
                    .map(|edge| edge.id),
                "incoming_edges_mut",
                "DirectedGraph",
                3,
            );

            assert!(
                $crate::graph::DirectedGraph::incoming_edges_mut(&mut graph, nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::incoming_edges_mut() did not return an empty iterator for node 4"
            );
        }

        #[test]
        fn test_outgoing_edges() {
            let (graph, nodes, edges) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            let expected_edges_zero = [edges[0], edges[1]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_zero,
                $crate::graph::DirectedGraph::outgoing_edges(&graph, nodes[0]).map(|edge| edge.id),
                "outgoing_edges",
                "DirectedGraph",
                0,
            );

            let expected_edges_one = [edges[2]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_one,
                $crate::graph::DirectedGraph::outgoing_edges(&graph, nodes[1]).map(|edge| edge.id),
                "outgoing_edges",
                "DirectedGraph",
                1,
            );

            assert!(
                $crate::graph::DirectedGraph::outgoing_edges(&graph, nodes[2])
                    .next()
                    .is_none(),
                "DirectedGraph::outgoing_edges() did not return an empty iterator for node 2"
            );

            let expected_edges_three = [edges[3]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_three,
                $crate::graph::DirectedGraph::outgoing_edges(&graph, nodes[3]).map(|edge| edge.id),
                "outgoing_edges",
                "DirectedGraph",
                3,
            );

            assert!(
                $crate::graph::DirectedGraph::outgoing_edges(&graph, nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::outgoing_edges() did not return an empty iterator for node 4"
            );
        }

        #[test]
        fn test_outgoing_edges_mut() {
            let (mut graph, nodes, edges) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            let expected_edges_zero = [edges[0], edges[1]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_zero,
                $crate::graph::DirectedGraph::outgoing_edges_mut(&mut graph, nodes[0])
                    .map(|edge| edge.id),
                "outgoing_edges_mut",
                "DirectedGraph",
                0,
            );

            let expected_edges_one = [edges[2]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_one,
                $crate::graph::DirectedGraph::outgoing_edges_mut(&mut graph, nodes[1])
                    .map(|edge| edge.id),
                "outgoing_edges_mut",
                "DirectedGraph",
                1,
            );

            assert!(
                $crate::graph::DirectedGraph::outgoing_edges_mut(&mut graph, nodes[2])
                    .next()
                    .is_none(),
                "DirectedGraph::outgoing_edges_mut() did not return an empty iterator for node 2"
            );

            let expected_edges_three = [edges[3]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_three,
                $crate::graph::DirectedGraph::outgoing_edges_mut(&mut graph, nodes[3])
                    .map(|edge| edge.id),
                "outgoing_edges_mut",
                "DirectedGraph",
                3,
            );

            assert!(
                $crate::graph::DirectedGraph::outgoing_edges_mut(&mut graph, nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::outgoing_edges_mut() did not return an empty iterator for node 4"
            );
        }

        #[test]
        fn test_incident_edges() {
            let (graph, nodes, edges) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            let expected_edges_zero = [edges[0], edges[1]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_zero,
                $crate::graph::DirectedGraph::incident_edges(&graph, nodes[0]).map(|edge| edge.id),
                "incident_edges",
                "DirectedGraph",
                0,
            );

            let expected_edges_one = [edges[0], edges[2]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_one,
                $crate::graph::DirectedGraph::incident_edges(&graph, nodes[1]).map(|edge| edge.id),
                "incident_edges",
                "DirectedGraph",
                1,
            );

            let expected_edges_two = [edges[1], edges[3]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_two,
                $crate::graph::DirectedGraph::incident_edges(&graph, nodes[2]).map(|edge| edge.id),
                "incident_edges",
                "DirectedGraph",
                2,
            );

            let expected_edges_three = [edges[2], edges[3]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_three,
                $crate::graph::DirectedGraph::incident_edges(&graph, nodes[3]).map(|edge| edge.id),
                "incident_edges",
                "DirectedGraph",
                3,
            );

            assert!(
                $crate::graph::DirectedGraph::incident_edges(&graph, nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::incident_edges() did not return an empty iterator for node 4"
            );
        }

        #[test]
        fn test_incident_edges_mut() {
            let (mut graph, nodes, edges) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            let expected_edges_zero = [edges[0], edges[1]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_zero,
                $crate::graph::DirectedGraph::incident_edges_mut(&mut graph, nodes[0])
                    .map(|edge| edge.id),
                "incident_edges_mut",
                "DirectedGraph",
                0,
            );

            let expected_edges_one = [edges[0], edges[2]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_one,
                $crate::graph::DirectedGraph::incident_edges_mut(&mut graph, nodes[1])
                    .map(|edge| edge.id),
                "incident_edges_mut",
                "DirectedGraph",
                1,
            );

            let expected_edges_two = [edges[1], edges[3]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_two,
                $crate::graph::DirectedGraph::incident_edges_mut(&mut graph, nodes[2])
                    .map(|edge| edge.id),
                "incident_edges_mut",
                "DirectedGraph",
                2,
            );

            let expected_edges_three = [edges[2], edges[3]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_three,
                $crate::graph::DirectedGraph::incident_edges_mut(&mut graph, nodes[3])
                    .map(|edge| edge.id),
                "incident_edges_mut",
                "DirectedGraph",
                3,
            );

            assert!(
                $crate::graph::DirectedGraph::incident_edges_mut(&mut graph, nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::incident_edges_mut() did not return an empty iterator for node 4"
            );
        }

        #[test]
        fn test_predecessors() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            assert!(
                $crate::graph::DirectedGraph::predecessors(&graph, nodes[0])
                    .next()
                    .is_none(),
                "DirectedGraph::predecessors() did not return an empty iterator for node 0"
            );

            let expected_predecessors_one = [nodes[0]];
            $crate::utils::testing::check_if_nodes_match(
                expected_predecessors_one,
                $crate::graph::DirectedGraph::predecessors(&graph, nodes[1]),
                "predecessors",
                "DirectedGraph",
                Some(1),
            );

            let expected_predecessors_two = [nodes[0], nodes[3]];
            $crate::utils::testing::check_if_nodes_match(
                expected_predecessors_two,
                $crate::graph::DirectedGraph::predecessors(&graph, nodes[2]),
                "predecessors",
                "DirectedGraph",
                Some(2),
            );

            let expected_predecessors_three = [nodes[1]];
            $crate::utils::testing::check_if_nodes_match(
                expected_predecessors_three,
                $crate::graph::DirectedGraph::predecessors(&graph, nodes[3]),
                "predecessors",
                "DirectedGraph",
                Some(3),
            );

            assert!(
                $crate::graph::DirectedGraph::predecessors(&graph, nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::predecessors() did not return an empty iterator for node 4"
            );
        }

        #[test]
        fn test_successors() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            let expected_successors_zero = [nodes[1], nodes[2]];
            $crate::utils::testing::check_if_nodes_match(
                expected_successors_zero,
                $crate::graph::DirectedGraph::successors(&graph, nodes[0]),
                "successors",
                "DirectedGraph",
                Some(0),
            );

            let expected_successors_one = [nodes[3]];
            $crate::utils::testing::check_if_nodes_match(
                expected_successors_one,
                $crate::graph::DirectedGraph::successors(&graph, nodes[1]),
                "successors",
                "DirectedGraph",
                Some(1),
            );

            assert!(
                $crate::graph::DirectedGraph::successors(&graph, nodes[2])
                    .next()
                    .is_none(),
                "DirectedGraph::successors() did not return an empty iterator for node 2"
            );

            let expected_successors_three = [nodes[2]];
            $crate::utils::testing::check_if_nodes_match(
                expected_successors_three,
                $crate::graph::DirectedGraph::successors(&graph, nodes[3]),
                "successors",
                "DirectedGraph",
                Some(3),
            );

            assert!(
                $crate::graph::DirectedGraph::successors(&graph, nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::successors() did not return an empty iterator for node 4"
            );
        }

        #[test]
        fn test_adjacencies() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);
            let expected_adjacencies_zero = [nodes[1], nodes[2]];
            $crate::utils::testing::check_if_nodes_match(
                expected_adjacencies_zero,
                $crate::graph::DirectedGraph::adjacencies(&graph, nodes[0]),
                "adjacencies",
                "DirectedGraph",
                Some(0),
            );

            let expected_adjacencies_one = [nodes[0], nodes[3]];
            $crate::utils::testing::check_if_nodes_match(
                expected_adjacencies_one,
                $crate::graph::DirectedGraph::adjacencies(&graph, nodes[1]),
                "adjacencies",
                "DirectedGraph",
                Some(1),
            );

            let expected_adjacencies_two = [nodes[0], nodes[3]];
            $crate::utils::testing::check_if_nodes_match(
                expected_adjacencies_two,
                $crate::graph::DirectedGraph::adjacencies(&graph, nodes[2]),
                "adjacencies",
                "DirectedGraph",
                Some(2),
            );

            let expected_adjacencies_three = [nodes[1], nodes[2]];
            $crate::utils::testing::check_if_nodes_match(
                expected_adjacencies_three,
                $crate::graph::DirectedGraph::adjacencies(&graph, nodes[3]),
                "adjacencies",
                "DirectedGraph",
                Some(3),
            );

            assert!(
                $crate::graph::DirectedGraph::adjacencies(&graph, nodes[4])
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
                $crate::graph::DirectedGraph::edges_between(&graph, nodes[0], nodes[0])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 0 and 0"
            );

            let expected_edges_0_1 = [edges[0]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_0_1,
                $crate::graph::DirectedGraph::edges_between(&graph, nodes[0], nodes[1])
                    .map(|edge| edge.id),
                "edges_between",
                "DirectedGraph",
                0,
            );

            let expected_edges_0_2 = [edges[1]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_0_2,
                $crate::graph::DirectedGraph::edges_between(&graph, nodes[0], nodes[2])
                    .map(|edge| edge.id),
                "edges_between",
                "DirectedGraph",
                0,
            );

            assert!(
                $crate::graph::DirectedGraph::edges_between(&graph, nodes[0], nodes[3])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 0 and 3"
            );
            assert!(
                $crate::graph::DirectedGraph::edges_between(&graph, nodes[0], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 0 and 4"
            );

            // Source 1
            assert!(
                $crate::graph::DirectedGraph::edges_between(&graph, nodes[1], nodes[0])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 1 and 0"
            );

            assert!(
                $crate::graph::DirectedGraph::edges_between(&graph, nodes[1], nodes[1])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 1 and 1"
            );

            assert!(
                $crate::graph::DirectedGraph::edges_between(&graph, nodes[1], nodes[2])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 1 and 2"
            );

            let expected_edges_1_3 = [edges[2]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_1_3,
                $crate::graph::DirectedGraph::edges_between(&graph, nodes[1], nodes[3])
                    .map(|edge| edge.id),
                "edges_between",
                "DirectedGraph",
                1,
            );

            assert!(
                $crate::graph::DirectedGraph::edges_between(&graph, nodes[1], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 1 and 4"
            );

            // Source 2
            assert!(
                $crate::graph::DirectedGraph::edges_between(&graph, nodes[2], nodes[0])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 2 and 0"
            );

            assert!(
                $crate::graph::DirectedGraph::edges_between(&graph, nodes[2], nodes[1])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 2 and 1"
            );

            assert!(
                $crate::graph::DirectedGraph::edges_between(&graph, nodes[2], nodes[2])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 2 and 2"
            );

            assert!(
                $crate::graph::DirectedGraph::edges_between(&graph, nodes[2], nodes[3])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 2 and 3"
            );

            assert!(
                $crate::graph::DirectedGraph::edges_between(&graph, nodes[2], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 2 and 4"
            );

            // Source 3
            assert!(
                $crate::graph::DirectedGraph::edges_between(&graph, nodes[3], nodes[0])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 3 and 0"
            );

            assert!(
                $crate::graph::DirectedGraph::edges_between(&graph, nodes[3], nodes[1])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 3 and 1"
            );

            let expected_edges_3_2 = [edges[3]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_3_2,
                $crate::graph::DirectedGraph::edges_between(&graph, nodes[3], nodes[2])
                    .map(|edge| edge.id),
                "edges_between",
                "DirectedGraph",
                3,
            );

            assert!(
                $crate::graph::DirectedGraph::edges_between(&graph, nodes[3], nodes[3])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 3 and 3"
            );

            assert!(
                $crate::graph::DirectedGraph::edges_between(&graph, nodes[3], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 3 and 4"
            );

            // Source 4
            assert!(
                $crate::graph::DirectedGraph::edges_between(&graph, nodes[4], nodes[0])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 4 and 0"
            );

            assert!(
                $crate::graph::DirectedGraph::edges_between(&graph, nodes[4], nodes[1])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 4 and 1"
            );

            assert!(
                $crate::graph::DirectedGraph::edges_between(&graph, nodes[4], nodes[2])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 4 and 2"
            );

            assert!(
                $crate::graph::DirectedGraph::edges_between(&graph, nodes[4], nodes[3])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between() did not return an empty iterator for nodes 4 and 3"
            );

            assert!(
                $crate::graph::DirectedGraph::edges_between(&graph, nodes[4], nodes[4])
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
                $crate::graph::DirectedGraph::edges_between_mut(&mut graph, nodes[0], nodes[0])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 0 \
                 and 0"
            );

            let expected_edges_0_1 = [edges[0]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_0_1,
                $crate::graph::DirectedGraph::edges_between_mut(&mut graph, nodes[0], nodes[1])
                    .map(|edge| edge.id),
                "edges_between_mut",
                "DirectedGraph",
                0,
            );

            let expected_edges_0_2 = [edges[1]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_0_2,
                $crate::graph::DirectedGraph::edges_between_mut(&mut graph, nodes[0], nodes[2])
                    .map(|edge| edge.id),
                "edges_between_mut",
                "DirectedGraph",
                0,
            );

            assert!(
                $crate::graph::DirectedGraph::edges_between_mut(&mut graph, nodes[0], nodes[3])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 0 \
                 and 3"
            );
            assert!(
                $crate::graph::DirectedGraph::edges_between_mut(&mut graph, nodes[0], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 0 \
                 and 4"
            );

            // Source 1
            assert!(
                $crate::graph::DirectedGraph::edges_between_mut(&mut graph, nodes[1], nodes[0])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 1 \
                 and 0"
            );

            assert!(
                $crate::graph::DirectedGraph::edges_between_mut(&mut graph, nodes[1], nodes[1])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 1 \
                 and 1"
            );

            assert!(
                $crate::graph::DirectedGraph::edges_between_mut(&mut graph, nodes[1], nodes[2])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 1 \
                 and 2"
            );

            let expected_edges_1_3 = [edges[2]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_1_3,
                $crate::graph::DirectedGraph::edges_between_mut(&mut graph, nodes[1], nodes[3])
                    .map(|edge| edge.id),
                "edges_between_mut",
                "DirectedGraph",
                1,
            );

            assert!(
                $crate::graph::DirectedGraph::edges_between_mut(&mut graph, nodes[1], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 1 \
                 and 4"
            );

            // Source 2
            assert!(
                $crate::graph::DirectedGraph::edges_between_mut(&mut graph, nodes[2], nodes[0])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 2 \
                 and 0"
            );

            assert!(
                $crate::graph::DirectedGraph::edges_between_mut(&mut graph, nodes[2], nodes[1])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 2 \
                 and 1"
            );

            assert!(
                $crate::graph::DirectedGraph::edges_between_mut(&mut graph, nodes[2], nodes[2])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 2 \
                 and 2"
            );

            assert!(
                $crate::graph::DirectedGraph::edges_between_mut(&mut graph, nodes[2], nodes[3])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 2 \
                 and 3"
            );

            assert!(
                $crate::graph::DirectedGraph::edges_between_mut(&mut graph, nodes[2], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 2 \
                 and 4"
            );

            // Source 3
            assert!(
                $crate::graph::DirectedGraph::edges_between_mut(&mut graph, nodes[3], nodes[0])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 3 \
                 and 0"
            );

            assert!(
                $crate::graph::DirectedGraph::edges_between_mut(&mut graph, nodes[3], nodes[1])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 3 \
                 and 1"
            );

            let expected_edges_3_2 = [edges[3]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_3_2,
                $crate::graph::DirectedGraph::edges_between_mut(&mut graph, nodes[3], nodes[2])
                    .map(|edge| edge.id),
                "edges_between_mut",
                "DirectedGraph",
                3,
            );

            assert!(
                $crate::graph::DirectedGraph::edges_between_mut(&mut graph, nodes[3], nodes[3])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 3 \
                 and 3"
            );

            assert!(
                $crate::graph::DirectedGraph::edges_between_mut(&mut graph, nodes[3], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 3 \
                 and 4"
            );

            // Source 4
            assert!(
                $crate::graph::DirectedGraph::edges_between_mut(&mut graph, nodes[4], nodes[0])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 4 \
                 and 0"
            );

            assert!(
                $crate::graph::DirectedGraph::edges_between_mut(&mut graph, nodes[4], nodes[1])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 4 \
                 and 1"
            );

            assert!(
                $crate::graph::DirectedGraph::edges_between_mut(&mut graph, nodes[4], nodes[2])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 4 \
                 and 2"
            );

            assert!(
                $crate::graph::DirectedGraph::edges_between_mut(&mut graph, nodes[4], nodes[3])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_between_mut() did not return an empty iterator for nodes 4 \
                 and 3"
            );

            assert!(
                $crate::graph::DirectedGraph::edges_between_mut(&mut graph, nodes[4], nodes[4])
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
                $crate::graph::DirectedGraph::edges_connecting(&graph, nodes[0], nodes[0])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting() did not return an empty iterator for nodes 0 \
                 and 0"
            );

            let expected_edges_0_1 = [edges[0]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_0_1,
                $crate::graph::DirectedGraph::edges_connecting(&graph, nodes[0], nodes[1])
                    .map(|edge| edge.id),
                "edges_connecting",
                "DirectedGraph",
                0,
            );

            let expected_edges_0_2 = [edges[1]];

            $crate::utils::testing::check_if_edges_match(
                expected_edges_0_2,
                $crate::graph::DirectedGraph::edges_connecting(&graph, nodes[0], nodes[2])
                    .map(|edge| edge.id),
                "edges_connecting",
                "DirectedGraph",
                0,
            );

            assert!(
                $crate::graph::DirectedGraph::edges_connecting(&graph, nodes[0], nodes[3])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting() did not return an empty iterator for nodes 0 \
                 and 3"
            );

            assert!(
                $crate::graph::DirectedGraph::edges_connecting(&graph, nodes[0], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting() did not return an empty iterator for nodes 0 \
                 and 4"
            );

            // Source 1
            assert!(
                $crate::graph::DirectedGraph::edges_connecting(&graph, nodes[1], nodes[1])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting() did not return an empty iterator for nodes 1 \
                 and 1"
            );

            assert!(
                $crate::graph::DirectedGraph::edges_connecting(&graph, nodes[1], nodes[2])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting() did not return an empty iterator for nodes 1 \
                 and 2"
            );

            let expected_edges_1_3 = [edges[2]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_1_3,
                $crate::graph::DirectedGraph::edges_connecting(&graph, nodes[1], nodes[3])
                    .map(|edge| edge.id),
                "edges_connecting",
                "DirectedGraph",
                1,
            );

            assert!(
                $crate::graph::DirectedGraph::edges_connecting(&graph, nodes[1], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting() did not return an empty iterator for nodes 1 \
                 and 4"
            );

            // Source 2
            assert!(
                $crate::graph::DirectedGraph::edges_connecting(&graph, nodes[2], nodes[2])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting() did not return an empty iterator for nodes 2 \
                 and 2"
            );

            let expected_edges_2_3 = [edges[3]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_2_3,
                $crate::graph::DirectedGraph::edges_connecting(&graph, nodes[2], nodes[3])
                    .map(|edge| edge.id),
                "edges_connecting",
                "DirectedGraph",
                2,
            );

            assert!(
                $crate::graph::DirectedGraph::edges_connecting(&graph, nodes[2], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting() did not return an empty iterator for nodes 2 \
                 and 4"
            );

            // Source 3
            assert!(
                $crate::graph::DirectedGraph::edges_connecting(&graph, nodes[3], nodes[3])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting() did not return an empty iterator for nodes 3 \
                 and 3"
            );

            assert!(
                $crate::graph::DirectedGraph::edges_connecting(&graph, nodes[3], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting() did not return an empty iterator for nodes 3 \
                 and 4"
            );

            // Source 4
            assert!(
                $crate::graph::DirectedGraph::edges_connecting(&graph, nodes[4], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting() did not return an empty iterator for nodes 4 \
                 and 4"
            );

            // Check if swapping lhs and rhs matters
            for i in nodes.iter() {
                for j in nodes.iter() {
                    let edges_lhs_rhs: hashbrown::HashSet<_, foldhash::fast::RandomState> =
                        $crate::graph::DirectedGraph::edges_connecting(&graph, *i, *j)
                            .map(|edge| edge.id)
                            .collect();
                    let edges_rhs_lhs: hashbrown::HashSet<_, foldhash::fast::RandomState> =
                        $crate::graph::DirectedGraph::edges_connecting(&graph, *j, *i)
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
                $crate::graph::DirectedGraph::edges_connecting_mut(&mut graph, nodes[0], nodes[0])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting_mut() did not return an empty iterator for nodes \
                 0 and 0"
            );

            let expected_edges_0_1 = [edges[0]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_0_1,
                $crate::graph::DirectedGraph::edges_connecting_mut(&mut graph, nodes[0], nodes[1])
                    .map(|edge| edge.id),
                "edges_connecting_mut",
                "DirectedGraph",
                0,
            );

            let expected_edges_0_2 = [edges[1]];

            $crate::utils::testing::check_if_edges_match(
                expected_edges_0_2,
                $crate::graph::DirectedGraph::edges_connecting_mut(&mut graph, nodes[0], nodes[2])
                    .map(|edge| edge.id),
                "edges_connecting_mut",
                "DirectedGraph",
                0,
            );

            assert!(
                $crate::graph::DirectedGraph::edges_connecting_mut(&mut graph, nodes[0], nodes[3])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting_mut() did not return an empty iterator for nodes \
                 0 and 3"
            );

            assert!(
                $crate::graph::DirectedGraph::edges_connecting_mut(&mut graph, nodes[0], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting_mut() did not return an empty iterator for nodes \
                 0 and 4"
            );

            // Source 1
            assert!(
                $crate::graph::DirectedGraph::edges_connecting_mut(&mut graph, nodes[1], nodes[1])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting_mut() did not return an empty iterator for nodes \
                 1 and 1"
            );

            assert!(
                $crate::graph::DirectedGraph::edges_connecting_mut(&mut graph, nodes[1], nodes[2])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting_mut() did not return an empty iterator for nodes \
                 1 and 2"
            );

            let expected_edges_1_3 = [edges[2]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_1_3,
                $crate::graph::DirectedGraph::edges_connecting_mut(&mut graph, nodes[1], nodes[3])
                    .map(|edge| edge.id),
                "edges_connecting_mut",
                "DirectedGraph",
                1,
            );

            assert!(
                $crate::graph::DirectedGraph::edges_connecting_mut(&mut graph, nodes[1], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting_mut() did not return an empty iterator for nodes \
                 1 and 4"
            );

            // Source 2
            assert!(
                $crate::graph::DirectedGraph::edges_connecting_mut(&mut graph, nodes[2], nodes[2])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting_mut() did not return an empty iterator for nodes \
                 2 and 2"
            );

            let expected_edges_2_3 = [edges[3]];
            $crate::utils::testing::check_if_edges_match(
                expected_edges_2_3,
                $crate::graph::DirectedGraph::edges_connecting_mut(&mut graph, nodes[2], nodes[3])
                    .map(|edge| edge.id),
                "edges_connecting_mut",
                "DirectedGraph",
                2,
            );

            assert!(
                $crate::graph::DirectedGraph::edges_connecting_mut(&mut graph, nodes[2], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting_mut() did not return an empty iterator for nodes \
                 2 and 4"
            );

            // Source 3
            assert!(
                $crate::graph::DirectedGraph::edges_connecting_mut(&mut graph, nodes[3], nodes[3])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting_mut() did not return an empty iterator for nodes \
                 3 and 3"
            );

            assert!(
                $crate::graph::DirectedGraph::edges_connecting_mut(&mut graph, nodes[3], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting_mut() did not return an empty iterator for nodes \
                 3 and 4"
            );

            // Source 4
            assert!(
                $crate::graph::DirectedGraph::edges_connecting_mut(&mut graph, nodes[4], nodes[4])
                    .next()
                    .is_none(),
                "DirectedGraph::edges_connecting_mut() did not return an empty iterator for nodes \
                 4 and 4"
            );

            // Check if swapping lhs and rhs matters
            for i in nodes.iter() {
                for j in nodes.iter() {
                    let edges_lhs_rhs: hashbrown::HashSet<_, foldhash::fast::RandomState> =
                        $crate::graph::DirectedGraph::edges_connecting_mut(&mut graph, *i, *j)
                            .map(|edge| edge.id)
                            .collect();
                    let edges_rhs_lhs: hashbrown::HashSet<_, foldhash::fast::RandomState> =
                        $crate::graph::DirectedGraph::edges_connecting_mut(&mut graph, *j, *i)
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
                    $crate::graph::DirectedGraph::contains_node(&graph, *node),
                    "DirectedGraph::contains_node() returned false for existing node: {:?}",
                    node
                );
            }

            // We remove node 4 here, as some graph implementations might not have stable node ids,
            // but the newest node added is likely to be removable without another node taking its
            // id.
            $remove_node(&mut graph, nodes[4]);
            assert!(
                !$crate::graph::DirectedGraph::contains_node(&graph, nodes[4]),
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
                    $crate::graph::DirectedGraph::contains_edge(&graph, *edge),
                    "DirectedGraph::contains_edge() returned false for existing edge: {:?}",
                    edge
                );
            }
            // We remove edge 3 here, as some graph implementations might not have stable edge ids,
            // but the newest edge added is likely to be removable without another edge taking its
            // id.
            $remove_edge(&mut graph, edges[3]);
            assert!(
                !$crate::graph::DirectedGraph::contains_edge(&graph, edges[3]),
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
                        $crate::graph::DirectedGraph::is_adjacent(&graph, *source, *target),
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
                !$crate::graph::DirectedGraph::is_empty(&graph),
                "DirectedGraph::is_empty() returned true for a non-empty graph"
            );

            let mut new_graph = $graph_constructor();
            assert!(
                $crate::graph::DirectedGraph::is_empty(&new_graph),
                "DirectedGraph::is_empty() returned false for an empty graph"
            );

            let node_one = $add_node(&mut new_graph, ());
            assert!(
                !$crate::graph::DirectedGraph::is_empty(&new_graph),
                "DirectedGraph::is_empty() returned true for a graph with one node"
            );

            $remove_node(&mut new_graph, node_one);
            assert!(
                $crate::graph::DirectedGraph::is_empty(&new_graph),
                "DirectedGraph::is_empty() returned false for an empty graph after removing the \
                 only node"
            );
        }

        #[test]
        fn test_sources() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);

            let expected_sources = [nodes[0], nodes[4]];
            $crate::utils::testing::check_if_nodes_match(
                expected_sources,
                $crate::graph::DirectedGraph::sources(&graph).map(|n| n.id),
                "sources",
                "DirectedGraph",
                Option::<usize>::None,
            );
        }

        #[test]
        fn test_sinks() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructor, $add_node, $add_edge);

            let expected_sinks = [nodes[2], nodes[4]];
            $crate::utils::testing::check_if_nodes_match(
                expected_sinks,
                $crate::graph::DirectedGraph::sinks(&graph).map(|n| n.id),
                "sinks",
                "DirectedGraph",
                Option::<usize>::None,
            );
        }
    };
}
