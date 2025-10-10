use petgraph::algo::{dsatur_coloring, recursive_largest_first_coloring};
use petgraph::{visit::EdgeRef, Directed, Graph, Undirected};

#[test]
fn recursive_largest_first_coloring_cycle6() {
    let mut graph: Graph<(), (), Undirected> = Graph::new_undirected();
    let a = graph.add_node(());
    let b = graph.add_node(());
    let c = graph.add_node(());
    let d = graph.add_node(());
    let e = graph.add_node(());
    let f = graph.add_node(());
    graph.extend_with_edges([(a, b), (b, c), (c, d), (d, e), (e, f), (f, a)]);

    // a -- b -- c -- d -- e -- f
    // |                        |
    // \------------------------/

    let color = recursive_largest_first_coloring(&graph);
    assert_eq!(color.color_count, 2);
    assert_eq!(color.nodes_to_colors.len(), 6);
}

#[test]
fn recursive_largest_first_coloring_graph() {
    let mut graph: Graph<(), (), Undirected> = Graph::new_undirected();
    let a = graph.add_node(());
    let b = graph.add_node(());
    let c = graph.add_node(());
    let d = graph.add_node(());
    let e = graph.add_node(());
    let f = graph.add_node(());
    let g = graph.add_node(());
    let h = graph.add_node(());

    graph.extend_with_edges([
        (a, b),
        (b, c),
        (c, d),
        (d, a),
        (e, f),
        (b, e),
        (f, g),
        (g, h),
        (h, e),
    ]);
    // a ----- b ----- e ----- f
    // |       |       |       |
    // |       |       |       |
    // d ----- c       h ----- g

    let res = recursive_largest_first_coloring(&graph);
    assert_eq!(res.color_count, 2);
    assert_eq!(res.nodes_to_colors.len(), 8);
}

#[test]
fn recursive_largest_first_coloring_self_edges() {
    let mut graph: Graph<(), (), Undirected> = Graph::new_undirected();
    let a = graph.add_node(());
    let b = graph.add_node(());
    let c = graph.add_node(());
    let d = graph.add_node(());

    graph.extend_with_edges([
        (a, b),
        (b, c),
        (c, d),
        (d, a),
        (a, a), // self-edge on a
        (c, c), // self-edge on c
    ]);

    // Graph with self-edges:
    // a ----- b
    // |↻      |
    // |       |
    // d ----- c↻

    let color = recursive_largest_first_coloring(&graph);

    // Self-edges shouldn't affect the coloring of the rest of the graph
    assert_eq!(color.color_count, 2);
    assert_eq!(color.nodes_to_colors.len(), 4);
}
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
