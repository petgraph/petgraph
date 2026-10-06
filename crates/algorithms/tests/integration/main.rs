mod flows;

fn directed_test_graph_remove_node_with_unwrap<N, E>(
    graph: &mut petgraph_core::utils::test_graphs::directed::DirectedTestGraph<
        N,
        E,
        petgraph_core::utils::test_graphs::directed::DirNodeId,
        petgraph_core::utils::test_graphs::directed::DirEdgeId,
    >,
    node_id: petgraph_core::utils::test_graphs::directed::DirNodeId,
) {
    graph.remove_node(node_id).unwrap();
}

fn directed_test_graph_remove_edge_with_unwrap<N, E>(
    graph: &mut petgraph_core::utils::test_graphs::directed::DirectedTestGraph<
        N,
        E,
        petgraph_core::utils::test_graphs::directed::DirNodeId,
        petgraph_core::utils::test_graphs::directed::DirEdgeId,
    >,
    edge_id: petgraph_core::utils::test_graphs::directed::DirEdgeId,
) {
    graph.remove_edge(edge_id).unwrap();
}

#[macro_export]
macro_rules! run_macro_for_all_dir_graphs {
    ($macro:ident) => {
        $macro!(
            DirectedTestGraph::<
                _,
                _,
                petgraph_core::utils::test_graphs::directed::DirNodeId,
                petgraph_core::utils::test_graphs::directed::DirEdgeId,
            >::new,
            DirectedTestGraph::<
                _,
                _,
                petgraph_core::utils::test_graphs::directed::DirNodeId,
                petgraph_core::utils::test_graphs::directed::DirEdgeId,
            >::add_node,
            DirectedTestGraph::<
                _,
                _,
                petgraph_core::utils::test_graphs::directed::DirNodeId,
                petgraph_core::utils::test_graphs::directed::DirEdgeId,
            >::add_edge,
            $crate::directed_test_graph_remove_node_with_unwrap,
            $crate::directed_test_graph_remove_edge_with_unwrap
        );
        // Add more graphs here as available
    };
}
