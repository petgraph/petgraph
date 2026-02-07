mod flows;

#[macro_export]
macro_rules! run_macro_for_all_graphs {
    ($macro:ident) => {
        $macro!(
            DirectedTestGraph::<
                _,
                _,
                petgraph_core::utils::directed::NodeId,
                petgraph_core::utils::directed::EdgeId,
            >::new,
            DirectedTestGraph::<
                _,
                _,
                petgraph_core::utils::directed::NodeId,
                petgraph_core::utils::directed::EdgeId,
            >::add_node,
            DirectedTestGraph::<
                _,
                _,
                petgraph_core::utils::directed::NodeId,
                petgraph_core::utils::directed::EdgeId,
            >::add_edge,
            DirectedTestGraph::<
                _,
                _,
                petgraph_core::utils::directed::NodeId,
                petgraph_core::utils::directed::EdgeId,
            >::remove_node,
            DirectedTestGraph::<
                _,
                _,
                petgraph_core::utils::directed::NodeId,
                petgraph_core::utils::directed::EdgeId,
            >::remove_edge
        );
        // Add more graphs here as available
    };
}
