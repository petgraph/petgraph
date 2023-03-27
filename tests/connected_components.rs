extern crate petgraph;

use petgraph::algo::connected_components_vec;
use petgraph::{Directed, Graph};

#[test]
fn connected_components_vec_test_simple() {
    let mut graph: Graph<(), (), Directed> = Graph::new();
    let a = graph.add_node(()); // node with no weight
    let b = graph.add_node(());
    let c = graph.add_node(());
    let d = graph.add_node(());

    let e = graph.add_node(());
    let f = graph.add_node(());
    let g = graph.add_node(());

    let h = graph.add_node(());
    let i = graph.add_node(());
    let j = graph.add_node(());
    let k = graph.add_node(());

    graph.extend_with_edges(&[
        (a, b),
        (b, c),
        (c, d),
        (d, a),
        //
        (e, f),
        (f, g),
        (g, e),
        //
        (h, i),
        (i, j),
        (j, k),
        (k, h),
    ]);

    let res = connected_components_vec(&graph);
    assert_eq!(res.0, 3usize);
    assert_eq!(res.1, vec![0, 0, 0, 0, 1, 1, 1, 2, 2, 2, 2]);
}

#[test]
fn connected_components_vec_test_mixed() {
    let mut graph: Graph<(), (), Directed> = Graph::new();
    let a = graph.add_node(()); // node with no weight
    let b = graph.add_node(());
    let c = graph.add_node(());
    let d = graph.add_node(());
    let e = graph.add_node(());
    let f = graph.add_node(());
    let g = graph.add_node(());
    let h = graph.add_node(());
    let i = graph.add_node(());
    let j = graph.add_node(());
    let k = graph.add_node(());

    graph.extend_with_edges(&[
        (a, b),
        (b, j),
        (j, k),
        (k, a),
        //
        (e, g),
        (f, f),
        (g, e),
        //
        (c, d),
        (d, h),
        (h, i),
        (i, c),
    ]);

    let res = connected_components_vec(&graph);
    assert_eq!(res.0, 4usize);
    assert_eq!(res.1, vec![0, 0, 1, 1, 2, 3, 2, 1, 1, 0, 0]);
}
