mod flows;

#[macro_export]
macro_rules! run_macro_for_all_graphs {
    ($macro:ident) => {
        $macro!(
            DirectedTestGraph::<(), u32>::new,
            DirectedTestGraph::<(), u32>::add_node,
            DirectedTestGraph::<(), u32>::add_edge,
            DirectedTestGraph::<(), u32>::remove_node,
            DirectedTestGraph::<(), u32>::remove_edge
        );
        // Add more graphs here as available
    };
}
