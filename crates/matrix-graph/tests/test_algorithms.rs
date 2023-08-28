use petgraph::{
    algorithms::components::{kosaraju_scc, tarjan_scc},
    matrix_graph::NodeIndex,
};
use petgraph_matrix_graph::MatrixGraph;

/// From https://github.com/petgraph/petgraph/issues/523
#[test]
fn tarjan_scc_with_removed_node() {
    let mut graph: MatrixGraph<(), ()> = MatrixGraph::new();

    graph.add_node(());
    let b = graph.add_node(());
    graph.add_node(());

    graph.remove_node(b);

    assert_eq!(tarjan_scc(&graph), [[NodeIndex::new(0)], [NodeIndex::new(
        2
    )]]);
}

/// From https://github.com/petgraph/petgraph/issues/523
#[test]
fn kosaraju_scc_with_removed_node() {
    let mut graph: MatrixGraph<(), ()> = MatrixGraph::new();

    graph.add_node(());
    let b = graph.add_node(());
    graph.add_node(());

    graph.remove_node(b);

    assert_eq!(kosaraju_scc(&graph), [[NodeIndex::new(2)], [
        NodeIndex::new(0)
    ]]);
}
