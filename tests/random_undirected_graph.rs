use petgraph::{generators::random_undirected_graph, Graph, Undirected};

#[test]
fn test_random_undirected_graph() {
    let n = 9;
    let p = 0.;

    let g: Graph<(), (), Undirected, u32> = random_undirected_graph(n, p);

    assert_eq!(n, g.node_count());
}

#[test]
fn test_random_complete_graph() {
    let n = 10;
    let p = 1.;
    let g: Graph<(), (), Undirected, u32> = random_undirected_graph(n, p);

    assert_eq!(n, g.node_count());
    assert_eq!((n * (n - 1)) / 2, g.edge_count());
}

#[test]
fn test_empty_random_undirected_graph() {
    let n = 8;
    let p = 0.;

    let g: Graph<(), (), Undirected, u32> = random_undirected_graph(n, p);

    assert_eq!(n, g.node_count());
    assert_eq!(0, g.edge_count());
}
