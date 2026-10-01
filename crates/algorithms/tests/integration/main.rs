mod flows;

#[macro_export]
macro_rules! run_macro_for_all_graphs {
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
            DirectedTestGraph::<
                _,
                _,
                petgraph_core::utils::test_graphs::directed::DirNodeId,
                petgraph_core::utils::test_graphs::directed::DirEdgeId,
            >::remove_node,
            DirectedTestGraph::<
                _,
                _,
                petgraph_core::utils::test_graphs::directed::DirNodeId,
                petgraph_core::utils::test_graphs::directed::DirEdgeId,
            >::remove_edge
        );
        // Add more graphs here as available
    };
}
