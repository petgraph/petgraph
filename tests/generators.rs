use petgraph::generators::complete_graph;

#[test]
fn test_complete_graph_un_graph() {
    use petgraph::graph::UnGraph;
    let complete = complete_graph::<UnGraph<_, _>>(core::iter::repeat(()).take(4), |_, _| ());

    let expected = UnGraph::<(), ()>::from_edges([(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]);

    assert_eq!(format!("{:?}", complete), format!("{:?}", expected));
}

#[test]
fn test_complete_graph_stable_un_graph() {
    use petgraph::stable_graph::{node_index, StableUnGraph};
    let edge_map = std::collections::HashMap::from([((0, 1), "1"), ((0, 2), "x"), ((1, 2), "y")]);
    let complete = complete_graph::<StableUnGraph<_, _>>(["1", "x", "y"], |a, b| {
        edge_map[&(a.index(), b.index())]
    });

    let mut expected = StableUnGraph::<_, _>::from_edges([(0, 1, "1"), (0, 2, "x"), (1, 2, "y")]);
    *&mut expected[node_index(0)] = "1";
    *&mut expected[node_index(1)] = "x";
    *&mut expected[node_index(2)] = "y";

    assert_eq!(format!("{:?}", complete), format!("{:?}", expected));
}
