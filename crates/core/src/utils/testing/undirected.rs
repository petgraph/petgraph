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
