mod flows;

#[macro_export]
macro_rules! run_macro_for_all_dir_graphs {
    ($macro:ident) => {
        $macro!(
            DirectedTestGraph::<
                _,
                _,
                petgraph_core::utils::test_graphs::directed::DirNodeId,
                petgraph_core::utils::test_graphs::directed::DirEdgeId,
            >::new
        );
        // Add more graphs here as available
    };
}
