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

/// TODO Check where it would be useful to have a remove_node and remove_edge function
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

        fn check_if_edges_match<T: core::hash::Hash + Eq + core::fmt::Debug>(
            mut expected_edges: hashbrown::hash_set::HashSet<T, foldhash::fast::RandomState>,
            actual_edges: impl Iterator<Item = T>,
            method_name: &'static str,
            node_number: usize,
        ) {
            for edge in actual_edges {
                assert!(
                    expected_edges.contains(&edge),
                    "graph.{}() contained unexpected edge id: {:?} for node {}",
                    method_name,
                    edge,
                    node_number
                );
                expected_edges.remove(&edge);
            }
            assert!(
                expected_edges.is_empty(),
                "graph.{}() did not return all expected edges for node {}",
                method_name,
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
            check_if_edges_match(
                expected_edges_one,
                graph.incoming_edges(nodes[1]).map(|edge| edge.id),
                "incoming_edges",
                1,
            );

            let expected_edges_two =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[1], edges[3],
                ]);
            check_if_edges_match(
                expected_edges_two,
                graph.incoming_edges(nodes[2]).map(|edge| edge.id),
                "incoming_edges",
                2,
            );

            let expected_edges_three =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[2]
                ]);
            check_if_edges_match(
                expected_edges_three,
                graph.incoming_edges(nodes[3]).map(|edge| edge.id),
                "incoming_edges",
                3,
            );

            assert!(
                graph.incoming_edges(nodes[4]).next().is_none(),
                "graph.incoming_edges() did not return an empty iterator for node 4"
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
            check_if_edges_match(
                expected_edges_one,
                graph.incoming_edges_mut(nodes[1]).map(|edge| edge.id),
                "incoming_edges_mut",
                1,
            );

            let expected_edges_two =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[1], edges[3],
                ]);
            check_if_edges_match(
                expected_edges_two,
                graph.incoming_edges_mut(nodes[2]).map(|edge| edge.id),
                "incoming_edges_mut",
                2,
            );

            let expected_edges_three =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[2]
                ]);
            check_if_edges_match(
                expected_edges_three,
                graph.incoming_edges_mut(nodes[3]).map(|edge| edge.id),
                "incoming_edges_mut",
                3,
            );

            assert!(
                graph.incoming_edges_mut(nodes[4]).next().is_none(),
                "graph.incoming_edges_mut() did not return an empty iterator for node 4"
            );
        }

        #[test]
        fn test_outgoing_edges() {
            let (graph, nodes, edges) =
                $crate::create_directed_test_graph!($graph_constructer, $add_node, $add_edge);
            let expected_edges_zero =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[0], edges[1],
                ]);
            check_if_edges_match(
                expected_edges_zero,
                graph.outgoing_edges(nodes[0]).map(|edge| edge.id),
                "outgoing_edges",
                0,
            );

            let expected_edges_one =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[2]
                ]);
            check_if_edges_match(
                expected_edges_one,
                graph.outgoing_edges(nodes[1]).map(|edge| edge.id),
                "outgoing_edges",
                1,
            );

            assert!(
                graph.outgoing_edges(nodes[2]).next().is_none(),
                "graph.outgoing_edges() did not return an empty iterator for node 2"
            );

            let expected_edges_three =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[3]
                ]);
            check_if_edges_match(
                expected_edges_three,
                graph.outgoing_edges(nodes[3]).map(|edge| edge.id),
                "outgoing_edges",
                3,
            );

            assert!(
                graph.outgoing_edges(nodes[4]).next().is_none(),
                "graph.outgoing_edges() did not return an empty iterator for node 4"
            );
        }

        #[test]
        fn test_outgoing_edges_mut() {
            let (mut graph, nodes, edges) =
                $crate::create_directed_test_graph!($graph_constructer, $add_node, $add_edge);
            let expected_edges_zero =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[0], edges[1],
                ]);
            check_if_edges_match(
                expected_edges_zero,
                graph.outgoing_edges_mut(nodes[0]).map(|edge| edge.id),
                "outgoing_edges_mut",
                0,
            );

            let expected_edges_one =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[2]
                ]);
            check_if_edges_match(
                expected_edges_one,
                graph.outgoing_edges_mut(nodes[1]).map(|edge| edge.id),
                "outgoing_edges_mut",
                1,
            );

            assert!(
                graph.outgoing_edges_mut(nodes[2]).next().is_none(),
                "graph.outgoing_edges_mut() did not return an empty iterator for node 2"
            );

            let expected_edges_three =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[3]
                ]);
            check_if_edges_match(
                expected_edges_three,
                graph.outgoing_edges_mut(nodes[3]).map(|edge| edge.id),
                "outgoing_edges_mut",
                3,
            );

            assert!(
                graph.outgoing_edges_mut(nodes[4]).next().is_none(),
                "graph.outgoing_edges_mut() did not return an empty iterator for node 4"
            );
        }

        #[test]
        fn test_incident_edges() {
            let (graph, nodes, edges) =
                $crate::create_directed_test_graph!($graph_constructer, $add_node, $add_edge);
            let expected_edges_zero =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[0], edges[1],
                ]);
            check_if_edges_match(
                expected_edges_zero,
                graph.incident_edges(nodes[0]).map(|edge| edge.id),
                "incident_edges",
                0,
            );

            let expected_edges_one =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[0], edges[2],
                ]);
            check_if_edges_match(
                expected_edges_one,
                graph.incident_edges(nodes[1]).map(|edge| edge.id),
                "incident_edges",
                1,
            );

            let expected_edges_two =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[1], edges[3],
                ]);
            check_if_edges_match(
                expected_edges_two,
                graph.incident_edges(nodes[2]).map(|edge| edge.id),
                "incident_edges",
                2,
            );

            let expected_edges_three =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[2], edges[3],
                ]);
            check_if_edges_match(
                expected_edges_three,
                graph.incident_edges(nodes[3]).map(|edge| edge.id),
                "incident_edges",
                3,
            );

            assert!(
                graph.incident_edges(nodes[4]).next().is_none(),
                "graph.incident_edges() did not return an empty iterator for node 4"
            );
        }

        #[test]
        fn test_incident_edges_mut() {
            let (mut graph, nodes, edges) =
                $crate::create_directed_test_graph!($graph_constructer, $add_node, $add_edge);
            let expected_edges_zero =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[0], edges[1],
                ]);
            check_if_edges_match(
                expected_edges_zero,
                graph.incident_edges_mut(nodes[0]).map(|edge| edge.id),
                "incident_edges_mut",
                0,
            );

            let expected_edges_one =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[0], edges[2],
                ]);
            check_if_edges_match(
                expected_edges_one,
                graph.incident_edges_mut(nodes[1]).map(|edge| edge.id),
                "incident_edges_mut",
                1,
            );

            let expected_edges_two =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[1], edges[3],
                ]);
            check_if_edges_match(
                expected_edges_two,
                graph.incident_edges_mut(nodes[2]).map(|edge| edge.id),
                "incident_edges_mut",
                2,
            );

            let expected_edges_three =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[2], edges[3],
                ]);
            check_if_edges_match(
                expected_edges_three,
                graph.incident_edges_mut(nodes[3]).map(|edge| edge.id),
                "incident_edges_mut",
                3,
            );

            assert!(
                graph.incident_edges_mut(nodes[4]).next().is_none(),
                "graph.incident_edges_mut() did not return an empty iterator for node 4"
            );
        }

        // TODO: Adjust to take node_number: Option<usize> and adjust messages accordingly
        fn check_if_nodes_match<T: core::hash::Hash + Eq + core::fmt::Debug>(
            mut expected_nodes: hashbrown::hash_set::HashSet<T, foldhash::fast::RandomState>,
            actual_nodes: impl Iterator<Item = T>,
            method_name: &'static str,
            node_number: usize,
        ) {
            for node in actual_nodes {
                assert!(
                    expected_nodes.contains(&node),
                    "graph.{}() contained unexpected node id: {:?} for node {}",
                    method_name,
                    node,
                    node_number
                );
                expected_nodes.remove(&node);
            }
            assert!(
                expected_nodes.is_empty(),
                "graph.{}() did not return all expected nodes for node {}",
                method_name,
                node_number
            );
        }

        #[test]
        fn test_predecessors() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructer, $add_node, $add_edge);
            assert!(
                graph.predecessors(nodes[0]).next().is_none(),
                "graph.predecessors() did not return an empty iterator for node 0"
            );

            let expected_predecessors_one = hashbrown::hash_set::HashSet::<
                _,
                foldhash::fast::RandomState,
            >::from_iter([nodes[0]]);
            check_if_nodes_match(
                expected_predecessors_one,
                graph.predecessors(nodes[1]),
                "predecessors",
                1,
            );

            let expected_predecessors_two = hashbrown::hash_set::HashSet::<
                _,
                foldhash::fast::RandomState,
            >::from_iter([nodes[0], nodes[3]]);
            check_if_nodes_match(
                expected_predecessors_two,
                graph.predecessors(nodes[2]),
                "predecessors",
                2,
            );

            let expected_predecessors_three = hashbrown::hash_set::HashSet::<
                _,
                foldhash::fast::RandomState,
            >::from_iter([nodes[1]]);
            check_if_nodes_match(
                expected_predecessors_three,
                graph.predecessors(nodes[3]),
                "predecessors",
                3,
            );

            assert!(
                graph.predecessors(nodes[4]).next().is_none(),
                "graph.predecessors() did not return an empty iterator for node 4"
            );
        }

        #[test]
        fn test_successors() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructer, $add_node, $add_edge);
            let expected_successors_zero = hashbrown::hash_set::HashSet::<
                _,
                foldhash::fast::RandomState,
            >::from_iter([nodes[1], nodes[2]]);
            check_if_nodes_match(
                expected_successors_zero,
                graph.successors(nodes[0]),
                "successors",
                0,
            );

            let expected_successors_one = hashbrown::hash_set::HashSet::<
                _,
                foldhash::fast::RandomState,
            >::from_iter([nodes[3]]);
            check_if_nodes_match(
                expected_successors_one,
                graph.successors(nodes[1]),
                "successors",
                1,
            );

            assert!(
                graph.successors(nodes[2]).next().is_none(),
                "graph.successors() did not return an empty iterator for node 2"
            );

            let expected_successors_three = hashbrown::hash_set::HashSet::<
                _,
                foldhash::fast::RandomState,
            >::from_iter([nodes[2]]);
            check_if_nodes_match(
                expected_successors_three,
                graph.successors(nodes[3]),
                "successors",
                3,
            );

            assert!(
                graph.successors(nodes[4]).next().is_none(),
                "graph.successors() did not return an empty iterator for node 4"
            );
        }

        #[test]
        fn test_adjacencies() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructer, $add_node, $add_edge);
            let expected_adjacencies_zero = hashbrown::hash_set::HashSet::<
                _,
                foldhash::fast::RandomState,
            >::from_iter([nodes[1], nodes[2]]);
            check_if_nodes_match(
                expected_adjacencies_zero,
                graph.adjacencies(nodes[0]),
                "adjacencies",
                0,
            );

            let expected_adjacencies_one = hashbrown::hash_set::HashSet::<
                _,
                foldhash::fast::RandomState,
            >::from_iter([nodes[0], nodes[3]]);
            check_if_nodes_match(
                expected_adjacencies_one,
                graph.adjacencies(nodes[1]),
                "adjacencies",
                1,
            );

            let expected_adjacencies_two = hashbrown::hash_set::HashSet::<
                _,
                foldhash::fast::RandomState,
            >::from_iter([nodes[0], nodes[3]]);
            check_if_nodes_match(
                expected_adjacencies_two,
                graph.adjacencies(nodes[2]),
                "adjacencies",
                2,
            );

            let expected_adjacencies_three = hashbrown::hash_set::HashSet::<
                _,
                foldhash::fast::RandomState,
            >::from_iter([nodes[1], nodes[2]]);
            check_if_nodes_match(
                expected_adjacencies_three,
                graph.adjacencies(nodes[3]),
                "adjacencies",
                3,
            );

            assert!(
                graph.adjacencies(nodes[4]).next().is_none(),
                "graph.adjacencies() did not return an empty iterator for node 4"
            );
        }

        #[test]
        #[allow(clippy::too_many_lines)]
        fn test_edges_between() {
            let (graph, nodes, edges) =
                $crate::create_directed_test_graph!($graph_constructer, $add_node, $add_edge);

            // Source 0
            assert!(
                graph.edges_between(nodes[0], nodes[0]).next().is_none(),
                "graph.edges_between() did not return an empty iterator for nodes 0 and 0"
            );

            let expected_edges_0_1 =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[0]
                ]);
            check_if_edges_match(
                expected_edges_0_1,
                graph.edges_between(nodes[0], nodes[1]).map(|edge| edge.id),
                "edges_between",
                0,
            );

            let expected_edges_0_2 =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[1]
                ]);
            check_if_edges_match(
                expected_edges_0_2,
                graph.edges_between(nodes[0], nodes[2]).map(|edge| edge.id),
                "edges_between",
                0,
            );

            assert!(
                graph.edges_between(nodes[0], nodes[3]).next().is_none(),
                "graph.edges_between() did not return an empty iterator for nodes 0 and 3"
            );
            assert!(
                graph.edges_between(nodes[0], nodes[4]).next().is_none(),
                "graph.edges_between() did not return an empty iterator for nodes 0 and 4"
            );

            // Source 1
            assert!(
                graph.edges_between(nodes[1], nodes[0]).next().is_none(),
                "graph.edges_between() did not return an empty iterator for nodes 1 and 0"
            );

            assert!(
                graph.edges_between(nodes[1], nodes[1]).next().is_none(),
                "graph.edges_between() did not return an empty iterator for nodes 1 and 1"
            );

            assert!(
                graph.edges_between(nodes[1], nodes[2]).next().is_none(),
                "graph.edges_between() did not return an empty iterator for nodes 1 and 2"
            );

            let expected_edges_1_3 =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[2]
                ]);
            check_if_edges_match(
                expected_edges_1_3,
                graph.edges_between(nodes[1], nodes[3]).map(|edge| edge.id),
                "edges_between",
                1,
            );

            assert!(
                graph.edges_between(nodes[1], nodes[4]).next().is_none(),
                "graph.edges_between() did not return an empty iterator for nodes 1 and 4"
            );

            // Source 2
            assert!(
                graph.edges_between(nodes[2], nodes[0]).next().is_none(),
                "graph.edges_between() did not return an empty iterator for nodes 2 and 0"
            );

            assert!(
                graph.edges_between(nodes[2], nodes[1]).next().is_none(),
                "graph.edges_between() did not return an empty iterator for nodes 2 and 1"
            );

            assert!(
                graph.edges_between(nodes[2], nodes[2]).next().is_none(),
                "graph.edges_between() did not return an empty iterator for nodes 2 and 2"
            );

            assert!(
                graph.edges_between(nodes[2], nodes[3]).next().is_none(),
                "graph.edges_between() did not return an empty iterator for nodes 2 and 3"
            );

            assert!(
                graph.edges_between(nodes[2], nodes[4]).next().is_none(),
                "graph.edges_between() did not return an empty iterator for nodes 2 and 4"
            );

            // Source 3
            assert!(
                graph.edges_between(nodes[3], nodes[0]).next().is_none(),
                "graph.edges_between() did not return an empty iterator for nodes 3 and 0"
            );

            assert!(
                graph.edges_between(nodes[3], nodes[1]).next().is_none(),
                "graph.edges_between() did not return an empty iterator for nodes 3 and 1"
            );

            let expected_edges_3_2 =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[3]
                ]);
            check_if_edges_match(
                expected_edges_3_2,
                graph.edges_between(nodes[3], nodes[2]).map(|edge| edge.id),
                "edges_between",
                3,
            );

            assert!(
                graph.edges_between(nodes[3], nodes[3]).next().is_none(),
                "graph.edges_between() did not return an empty iterator for nodes 3 and 3"
            );

            assert!(
                graph.edges_between(nodes[3], nodes[4]).next().is_none(),
                "graph.edges_between() did not return an empty iterator for nodes 3 and 4"
            );

            // Source 4
            assert!(
                graph.edges_between(nodes[4], nodes[0]).next().is_none(),
                "graph.edges_between() did not return an empty iterator for nodes 4 and 0"
            );

            assert!(
                graph.edges_between(nodes[4], nodes[1]).next().is_none(),
                "graph.edges_between() did not return an empty iterator for nodes 4 and 1"
            );

            assert!(
                graph.edges_between(nodes[4], nodes[2]).next().is_none(),
                "graph.edges_between() did not return an empty iterator for nodes 4 and 2"
            );

            assert!(
                graph.edges_between(nodes[4], nodes[3]).next().is_none(),
                "graph.edges_between() did not return an empty iterator for nodes 4 and 3"
            );

            assert!(
                graph.edges_between(nodes[4], nodes[4]).next().is_none(),
                "graph.edges_between() did not return an empty iterator for nodes 4 and 4"
            );
        }

        #[test]
        #[allow(clippy::too_many_lines)]
        fn test_edges_between_mut() {
            let (mut graph, nodes, edges) =
                $crate::create_directed_test_graph!($graph_constructer, $add_node, $add_edge);

            // Source 0
            assert!(
                graph.edges_between_mut(nodes[0], nodes[0]).next().is_none(),
                "graph.edges_between_mut() did not return an empty iterator for nodes 0 and 0"
            );

            let expected_edges_0_1 =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[0]
                ]);
            check_if_edges_match(
                expected_edges_0_1,
                graph
                    .edges_between_mut(nodes[0], nodes[1])
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
                graph
                    .edges_between_mut(nodes[0], nodes[2])
                    .map(|edge| edge.id),
                "edges_between_mut",
                0,
            );

            assert!(
                graph.edges_between_mut(nodes[0], nodes[3]).next().is_none(),
                "graph.edges_between_mut() did not return an empty iterator for nodes 0 and 3"
            );
            assert!(
                graph.edges_between_mut(nodes[0], nodes[4]).next().is_none(),
                "graph.edges_between_mut() did not return an empty iterator for nodes 0 and 4"
            );

            // Source 1
            assert!(
                graph.edges_between_mut(nodes[1], nodes[0]).next().is_none(),
                "graph.edges_between_mut() did not return an empty iterator for nodes 1 and 0"
            );

            assert!(
                graph.edges_between_mut(nodes[1], nodes[1]).next().is_none(),
                "graph.edges_between_mut() did not return an empty iterator for nodes 1 and 1"
            );

            assert!(
                graph.edges_between_mut(nodes[1], nodes[2]).next().is_none(),
                "graph.edges_between_mut() did not return an empty iterator for nodes 1 and 2"
            );

            let expected_edges_1_3 =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[2]
                ]);
            check_if_edges_match(
                expected_edges_1_3,
                graph
                    .edges_between_mut(nodes[1], nodes[3])
                    .map(|edge| edge.id),
                "edges_between_mut",
                1,
            );

            assert!(
                graph.edges_between_mut(nodes[1], nodes[4]).next().is_none(),
                "graph.edges_between_mut() did not return an empty iterator for nodes 1 and 4"
            );

            // Source 2
            assert!(
                graph.edges_between_mut(nodes[2], nodes[0]).next().is_none(),
                "graph.edges_between_mut() did not return an empty iterator for nodes 2 and 0"
            );

            assert!(
                graph.edges_between_mut(nodes[2], nodes[1]).next().is_none(),
                "graph.edges_between_mut() did not return an empty iterator for nodes 2 and 1"
            );

            assert!(
                graph.edges_between_mut(nodes[2], nodes[2]).next().is_none(),
                "graph.edges_between_mut() did not return an empty iterator for nodes 2 and 2"
            );

            assert!(
                graph.edges_between_mut(nodes[2], nodes[3]).next().is_none(),
                "graph.edges_between_mut() did not return an empty iterator for nodes 2 and 3"
            );

            assert!(
                graph.edges_between_mut(nodes[2], nodes[4]).next().is_none(),
                "graph.edges_between_mut() did not return an empty iterator for nodes 2 and 4"
            );

            // Source 3
            assert!(
                graph.edges_between_mut(nodes[3], nodes[0]).next().is_none(),
                "graph.edges_between_mut() did not return an empty iterator for nodes 3 and 0"
            );

            assert!(
                graph.edges_between_mut(nodes[3], nodes[1]).next().is_none(),
                "graph.edges_between_mut() did not return an empty iterator for nodes 3 and 1"
            );

            let expected_edges_3_2 =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[3]
                ]);
            check_if_edges_match(
                expected_edges_3_2,
                graph
                    .edges_between_mut(nodes[3], nodes[2])
                    .map(|edge| edge.id),
                "edges_between_mut",
                3,
            );

            assert!(
                graph.edges_between_mut(nodes[3], nodes[3]).next().is_none(),
                "graph.edges_between_mut() did not return an empty iterator for nodes 3 and 3"
            );

            assert!(
                graph.edges_between_mut(nodes[3], nodes[4]).next().is_none(),
                "graph.edges_between_mut() did not return an empty iterator for nodes 3 and 4"
            );

            // Source 4
            assert!(
                graph.edges_between_mut(nodes[4], nodes[0]).next().is_none(),
                "graph.edges_between_mut() did not return an empty iterator for nodes 4 and 0"
            );

            assert!(
                graph.edges_between_mut(nodes[4], nodes[1]).next().is_none(),
                "graph.edges_between_mut() did not return an empty iterator for nodes 4 and 1"
            );

            assert!(
                graph.edges_between_mut(nodes[4], nodes[2]).next().is_none(),
                "graph.edges_between_mut() did not return an empty iterator for nodes 4 and 2"
            );

            assert!(
                graph.edges_between_mut(nodes[4], nodes[3]).next().is_none(),
                "graph.edges_between_mut() did not return an empty iterator for nodes 4 and 3"
            );

            assert!(
                graph.edges_between_mut(nodes[4], nodes[4]).next().is_none(),
                "graph.edges_between_mut() did not return an empty iterator for nodes 4 and 4"
            );
        }

        #[test]
        #[allow(clippy::too_many_lines)]
        fn test_edges_connecting() {
            let (graph, nodes, edges) =
                $crate::create_directed_test_graph!($graph_constructer, $add_node, $add_edge);

            // Source 0
            assert!(
                graph.edges_connecting(nodes[0], nodes[0]).next().is_none(),
                "graph.edges_connecting() did not return an empty iterator for nodes 0 and 0"
            );

            let expected_edges_0_1 =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[0]
                ]);
            check_if_edges_match(
                expected_edges_0_1,
                graph
                    .edges_connecting(nodes[0], nodes[1])
                    .map(|edge| edge.id),
                "edges_connecting",
                0,
            );

            let expected_edges_0_2 =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[1]
                ]);

            check_if_edges_match(
                expected_edges_0_2,
                graph
                    .edges_connecting(nodes[0], nodes[2])
                    .map(|edge| edge.id),
                "edges_connecting",
                0,
            );

            assert!(
                graph.edges_connecting(nodes[0], nodes[3]).next().is_none(),
                "graph.edges_connecting() did not return an empty iterator for nodes 0 and 3"
            );

            assert!(
                graph.edges_connecting(nodes[0], nodes[4]).next().is_none(),
                "graph.edges_connecting() did not return an empty iterator for nodes 0 and 4"
            );

            // Source 1
            assert!(
                graph.edges_connecting(nodes[1], nodes[1]).next().is_none(),
                "graph.edges_connecting() did not return an empty iterator for nodes 1 and
                1"
            );

            assert!(
                graph.edges_connecting(nodes[1], nodes[2]).next().is_none(),
                "graph.edges_connecting() did not return an empty iterator for nodes 1 and
                2"
            );

            let expected_edges_1_3 =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[2]
                ]);
            check_if_edges_match(
                expected_edges_1_3,
                graph
                    .edges_connecting(nodes[1], nodes[3])
                    .map(|edge| edge.id),
                "edges_connecting",
                1,
            );

            assert!(
                graph.edges_connecting(nodes[1], nodes[4]).next().is_none(),
                "graph.edges_connecting() did not return an empty iterator for nodes 1 and
                4"
            );

            // Source 2
            assert!(
                graph.edges_connecting(nodes[2], nodes[2]).next().is_none(),
                "graph.edges_connecting() did not return an empty iterator for nodes 2 and
                2"
            );

            let expected_edges_2_3 =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[3]
                ]);
            check_if_edges_match(
                expected_edges_2_3,
                graph
                    .edges_connecting(nodes[2], nodes[3])
                    .map(|edge| edge.id),
                "edges_connecting",
                2,
            );

            assert!(
                graph.edges_connecting(nodes[2], nodes[4]).next().is_none(),
                "graph.edges_connecting() did not return an empty iterator for nodes 2 and
                4"
            );

            // Source 3
            assert!(
                graph.edges_connecting(nodes[3], nodes[3]).next().is_none(),
                "graph.edges_connecting() did not return an empty iterator for nodes 3 and
                3"
            );

            assert!(
                graph.edges_connecting(nodes[3], nodes[4]).next().is_none(),
                "graph.edges_connecting() did not return an empty iterator for nodes 3 and
                4"
            );

            // Source 4
            assert!(
                graph.edges_connecting(nodes[4], nodes[4]).next().is_none(),
                "graph.edges_connecting() did not return an empty iterator for nodes 4 and
                4"
            );

            // Check if swapping lhs and rhs matters
            for i in nodes.iter() {
                for j in nodes.iter() {
                    let edges_lhs_rhs: hashbrown::HashSet<_, foldhash::fast::RandomState> =
                        graph.edges_connecting(*i, *j).map(|edge| edge.id).collect();
                    let edges_rhs_lhs: hashbrown::HashSet<_, foldhash::fast::RandomState> =
                        graph.edges_connecting(*j, *i).map(|edge| edge.id).collect();
                    assert_eq!(
                        edges_lhs_rhs, edges_rhs_lhs,
                        "graph.edges_connecting() returned different edges when swapping source \
                         and target nodes: {:?} and {:?}",
                        i, j
                    );
                }
            }
        }

        #[test]
        #[allow(clippy::too_many_lines)]
        fn test_edges_connecting_mut() {
            let (mut graph, nodes, edges) =
                $crate::create_directed_test_graph!($graph_constructer, $add_node, $add_edge);

            // Source 0
            assert!(
                graph
                    .edges_connecting_mut(nodes[0], nodes[0])
                    .next()
                    .is_none(),
                "graph.edges_connecting_mut() did not return an empty iterator for nodes 0 and 0"
            );

            let expected_edges_0_1 =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[0]
                ]);
            check_if_edges_match(
                expected_edges_0_1,
                graph
                    .edges_connecting_mut(nodes[0], nodes[1])
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
                graph
                    .edges_connecting_mut(nodes[0], nodes[2])
                    .map(|edge| edge.id),
                "edges_connecting_mut",
                0,
            );

            assert!(
                graph
                    .edges_connecting_mut(nodes[0], nodes[3])
                    .next()
                    .is_none(),
                "graph.edges_connecting_mut() did not return an empty iterator for nodes 0 and 3"
            );

            assert!(
                graph
                    .edges_connecting_mut(nodes[0], nodes[4])
                    .next()
                    .is_none(),
                "graph.edges_connecting_mut() did not return an empty iterator for nodes 0 and 4"
            );

            // Source 1
            assert!(
                graph
                    .edges_connecting_mut(nodes[1], nodes[1])
                    .next()
                    .is_none(),
                "graph.edges_connecting_mut() did not return an empty iterator for nodes 1 and
                1"
            );

            assert!(
                graph
                    .edges_connecting_mut(nodes[1], nodes[2])
                    .next()
                    .is_none(),
                "graph.edges_connecting_mut() did not return an empty iterator for nodes 1 and
                2"
            );

            let expected_edges_1_3 =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[2]
                ]);
            check_if_edges_match(
                expected_edges_1_3,
                graph
                    .edges_connecting_mut(nodes[1], nodes[3])
                    .map(|edge| edge.id),
                "edges_connecting_mut",
                1,
            );

            assert!(
                graph
                    .edges_connecting_mut(nodes[1], nodes[4])
                    .next()
                    .is_none(),
                "graph.edges_connecting_mut() did not return an empty iterator for nodes 1 and
                4"
            );

            // Source 2
            assert!(
                graph
                    .edges_connecting_mut(nodes[2], nodes[2])
                    .next()
                    .is_none(),
                "graph.edges_connecting_mut() did not return an empty iterator for nodes 2 and
                2"
            );

            let expected_edges_2_3 =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    edges[3]
                ]);
            check_if_edges_match(
                expected_edges_2_3,
                graph
                    .edges_connecting_mut(nodes[2], nodes[3])
                    .map(|edge| edge.id),
                "edges_connecting_mut",
                2,
            );

            assert!(
                graph
                    .edges_connecting_mut(nodes[2], nodes[4])
                    .next()
                    .is_none(),
                "graph.edges_connecting_mut() did not return an empty iterator for nodes 2 and
                4"
            );

            // Source 3
            assert!(
                graph
                    .edges_connecting_mut(nodes[3], nodes[3])
                    .next()
                    .is_none(),
                "graph.edges_connecting_mut() did not return an empty iterator for nodes 3 and
                3"
            );

            assert!(
                graph
                    .edges_connecting_mut(nodes[3], nodes[4])
                    .next()
                    .is_none(),
                "graph.edges_connecting_mut() did not return an empty iterator for nodes 3 and
                4"
            );

            // Source 4
            assert!(
                graph
                    .edges_connecting_mut(nodes[4], nodes[4])
                    .next()
                    .is_none(),
                "graph.edges_connecting_mut() did not return an empty iterator for nodes 4 and
                4"
            );

            // Check if swapping lhs and rhs matters
            for i in nodes.iter() {
                for j in nodes.iter() {
                    let edges_lhs_rhs: hashbrown::HashSet<_, foldhash::fast::RandomState> = graph
                        .edges_connecting_mut(*i, *j)
                        .map(|edge| edge.id)
                        .collect();
                    let edges_rhs_lhs: hashbrown::HashSet<_, foldhash::fast::RandomState> = graph
                        .edges_connecting_mut(*j, *i)
                        .map(|edge| edge.id)
                        .collect();
                    assert_eq!(
                        edges_lhs_rhs, edges_rhs_lhs,
                        "graph.edges_connecting_mut() returned different edges when swapping \
                         source and target nodes: {:?} and {:?}",
                        i, j
                    );
                }
            }
        }

        #[test]
        fn test_contains_node() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructer, $add_node, $add_edge);

            for node in nodes.iter() {
                assert!(
                    graph.contains_node(*node),
                    "graph.contains_node() returned false for existing node: {:?}",
                    node
                );
            }
        }

        #[test]
        fn test_contains_edge() {
            let (graph, _, edges) =
                $crate::create_directed_test_graph!($graph_constructer, $add_node, $add_edge);

            for edge in edges.iter() {
                assert!(
                    graph.contains_edge(*edge),
                    "graph.contains_edge() returned false for existing edge: {:?}",
                    edge
                );
            }
        }

        #[test]
        fn test_is_adjacent() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructer, $add_node, $add_edge);

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
                        graph.is_adjacent(*source, *target),
                        expected_adjacency,
                        "graph.is_adjacent() returned incorrect result for nodes {:?} and {:?}",
                        source,
                        target
                    );
                }
            }
        }

        #[test]
        fn test_is_empty() {
            let (graph, _, _) =
                $crate::create_directed_test_graph!($graph_constructer, $add_node, $add_edge);

            assert!(
                !graph.is_empty(),
                "graph.is_empty() returned true for a non-empty graph"
            );

            let new_graph = $graph_constructer();
            assert!(
                new_graph.is_empty(),
                "graph.is_empty() returned false for an empty graph"
            );
        }

        #[test]
        fn test_sources() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructer, $add_node, $add_edge);

            let expected_sources =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    nodes[0], nodes[1], nodes[3],
                ]);
            check_if_nodes_match(
                expected_sources,
                graph.sources().map(|n| n.id),
                "sources",
                usize::MAX,
            );
        }

        #[test]
        fn test_sinks() {
            let (graph, nodes, _) =
                $crate::create_directed_test_graph!($graph_constructer, $add_node, $add_edge);

            let expected_sinks =
                hashbrown::hash_set::HashSet::<_, foldhash::fast::RandomState>::from_iter([
                    nodes[2], nodes[4],
                ]);
            check_if_nodes_match(
                expected_sinks,
                graph.sinks().map(|n| n.id),
                "sinks",
                usize::MAX,
            );
        }
    };
}
