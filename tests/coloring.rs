use petgraph::algo::{dsatur_coloring, wfc_coloring};
use petgraph::{Graph, Undirected};

#[test]
fn dsatur_coloring_cycle6() {
    let mut graph: Graph<(), (), Undirected> = Graph::new_undirected();
    let a = graph.add_node(());
    let d = graph.add_node(());
    let b = graph.add_node(());
    let c = graph.add_node(());
    let e = graph.add_node(());
    let f = graph.add_node(());
    graph.extend_with_edges([(a, b), (b, c), (c, d), (d, e), (e, f), (f, e)]);

    let (coloring, nb_colors) = dsatur_coloring(&graph);
    assert_eq!(nb_colors, 2);
    assert_eq!(coloring.len(), 6);
}

#[test]
fn dsatur_coloring_bipartite() {
    let mut graph: Graph<(), (), Undirected> = Graph::new_undirected();
    let a = graph.add_node(());
    let d = graph.add_node(());
    let b = graph.add_node(());
    let c = graph.add_node(());
    let e = graph.add_node(());
    let f = graph.add_node(());
    let g = graph.add_node(());
    let h = graph.add_node(());
    let i = graph.add_node(());
    let j = graph.add_node(());
    let k = graph.add_node(());
    let l = graph.add_node(());
    graph.extend_with_edges([
        (a, b),
        (a, g),
        (a, l),
        (b, d),
        (b, h),
        (b, k),
        (c, d),
        (c, k),
        (d, l),
        (e, f),
        (e, j),
        (f, i),
        (f, l),
        (g, h),
        (g, j),
        (g, k),
        (h, i),
        (i, j),
        (i, k),
    ]);

    let (_, nb_colors) = dsatur_coloring(&graph);
    assert_eq!(nb_colors, 2);
}

#[test]
fn wfc_coloring_random_graph() {
    let mut graph: Graph<(), (), Undirected> = Graph::new_undirected();
    let a = graph.add_node(());
    let b = graph.add_node(());
    let c = graph.add_node(());
    let d = graph.add_node(());
    let e = graph.add_node(());
    let f = graph.add_node(());

    graph.extend_with_edges(&[
        (a, b),
        (a, c),
        (a, d),
        (b, c),
        (b, e),
        (c, f),
        (d, e),
        (d, f),
        (e, f),
    ]);

    let coloring = wfc_coloring(&graph).expect("Coloring failed");

    for (u, v) in graph
        .edge_indices()
        .map(|e| graph.edge_endpoints(e).unwrap())
    {
        assert_ne!(
            coloring[&u], coloring[&v],
            "Adjacent nodes have the same color"
        );
    }
}

#[test]
fn wfc_coloring_crown_graph() {
    let mut graph: Graph<(), (), Undirected> = Graph::new_undirected();
    let a = graph.add_node(());
    let b = graph.add_node(());
    let c = graph.add_node(());
    let d = graph.add_node(());
    let e = graph.add_node(());
    let f = graph.add_node(());
    let g = graph.add_node(());
    let h = graph.add_node(());

    graph.extend_with_edges(&[
        (a, f),
        (a, g),
        (a, h),
        (b, e),
        (b, g),
        (b, h),
        (c, e),
        (c, f),
        (c, h),
        (d, e),
        (d, f),
        (d, g),
    ]);

    let coloring = wfc_coloring(&graph).expect("Coloring failed");

    for (u, v) in graph
        .edge_indices()
        .map(|e| graph.edge_endpoints(e).unwrap())
    {
        assert_ne!(
            coloring[&u], coloring[&v],
            "Adjacent nodes have the same color"
        );
    }

    let unique_colors: std::collections::HashSet<_> = coloring.values().collect();
    assert_eq!(unique_colors.len(), 2, "More than 2 colors were used");
}

#[test]
fn wfc_coloring_directed() {
    let mut graph: Graph<(), (), petgraph::Directed> = Graph::new();
    let a = graph.add_node(());
    let b = graph.add_node(());
    let c = graph.add_node(());
    let d = graph.add_node(());

    graph.extend_with_edges(&[(a, b), (b, c), (c, d), (d, a), (a, c)]);

    let result = wfc_coloring(&graph);
    assert!(result.is_err(), "Expected an error for directed graph");
}
