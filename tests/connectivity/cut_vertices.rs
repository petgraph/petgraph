use std::collections::HashSet;

use petgraph::{
    algo::connectivity::CutVerticesSearch, dot::Dot, graph::NodeIndex, Graph, Undirected,
};

#[test]
fn cut_vertices_test_empty() {
    let gr: Graph<(), (), Undirected> = Graph::new_undirected();

    let mut iter = CutVerticesSearch::new(&gr);

    assert_eq!(iter.next(&gr), None);
    assert_eq!(iter.next(&gr), None);
}

#[test]
fn cut_vertices_test_k1() {
    let mut gr: Graph<&str, (), Undirected> = Graph::new_undirected();
    gr.add_node("A");

    let mut iter = CutVerticesSearch::new(&gr);

    assert_eq!(iter.next(&gr), None);
    assert_eq!(iter.next(&gr), None);
}

#[test]
fn cut_vertices_test_k2() {
    let mut gr = Graph::new_undirected();
    let a = gr.add_node("A");
    let b = gr.add_node("B");

    gr.add_edge(a, b, 1.);

    let mut iter = CutVerticesSearch::new(&gr);

    assert_eq!(iter.next(&gr), None);
    assert_eq!(iter.next(&gr), None);
}

#[test]
// A - B - C - D
//         | /
//         E
fn cut_vertices_test_a() {
    let mut gr = Graph::new_undirected();
    let a = gr.add_node("A");
    let b = gr.add_node("B");
    let c = gr.add_node("C");
    let d = gr.add_node("D");
    let e = gr.add_node("E");

    gr.add_edge(a, b, 1.);
    gr.add_edge(b, c, 2.);
    gr.add_edge(c, d, 3.);
    gr.add_edge(c, e, 4.);
    gr.add_edge(d, e, 5.);

    println!("{}", Dot::new(&gr));

    let expected_cut_vertices = HashSet::from([b, c]);
    let mut iter = CutVerticesSearch::new(&gr);
    let mut cut_vertices = HashSet::new();
    while let Some(cut_vertex) = iter.next(&gr) {
        cut_vertices.insert(cut_vertex);
    }

    assert_eq!(expected_cut_vertices, cut_vertices);

    assert_eq!(iter.next(&gr), None);
}

#[test]
// A - B - C - D
//         | /
//     F - E
fn cut_vertices_test_b() {
    let mut gr = Graph::new_undirected();
    let a = gr.add_node("A");
    let b = gr.add_node("B");
    let c = gr.add_node("C");
    let d = gr.add_node("D");
    let e = gr.add_node("E");
    let f = gr.add_node("F");

    gr.add_edge(a, b, 1.);
    gr.add_edge(b, c, 2.);
    gr.add_edge(c, d, 3.);
    gr.add_edge(c, e, 4.);
    gr.add_edge(d, e, 5.);
    gr.add_edge(e, f, 6.);

    println!("{}", Dot::new(&gr));

    let expected_cut_vertices = HashSet::from([b, c, e]);
    let mut iter = CutVerticesSearch::new(&gr);
    let mut cut_vertices = HashSet::new();
    while let Some(cut_vertex) = iter.next(&gr) {
        cut_vertices.insert(cut_vertex);
    }

    assert_eq!(expected_cut_vertices, cut_vertices);

    assert_eq!(iter.next(&gr), None);
}

#[test]
// A - B - C
// | /   \ |
// D      E
fn cut_vertices_test_c() {
    let mut gr = Graph::new_undirected();
    let a = gr.add_node("A");
    let b = gr.add_node("B");
    let c = gr.add_node("C");
    let d = gr.add_node("D");
    let e = gr.add_node("E");

    gr.add_edge(a, b, 1.);
    gr.add_edge(a, d, 2.);
    gr.add_edge(b, c, 3.);
    gr.add_edge(b, d, 4.);
    gr.add_edge(b, e, 4.);
    gr.add_edge(c, e, 5.);

    println!("{}", Dot::new(&gr));

    let expected_cut_vertices = HashSet::from([b]);
    let mut iter = CutVerticesSearch::new(&gr);
    let mut cut_vertices = HashSet::new();
    while let Some(cut_vertex) = iter.next(&gr) {
        cut_vertices.insert(cut_vertex);
    }
    assert_eq!(expected_cut_vertices, cut_vertices);

    assert_eq!(iter.next(&gr), None);
}

#[test]
// A - B - D - E - F
//     | /   \
//     C       G - H
//             | /
//             I
fn cut_vertices_test_d() {
    let mut gr = Graph::new_undirected();
    let a = gr.add_node("A");
    let b = gr.add_node("B");
    let c = gr.add_node("C");
    let d = gr.add_node("D");
    let e = gr.add_node("E");
    let f = gr.add_node("F");
    let g = gr.add_node("G");
    let h = gr.add_node("H");
    let i = gr.add_node("I");

    gr.add_edge(a, b, 1.);
    gr.add_edge(b, c, 2.);
    gr.add_edge(b, d, 3.);
    gr.add_edge(c, d, 4.);
    gr.add_edge(d, e, 5.);
    gr.add_edge(d, g, 6.);
    gr.add_edge(e, f, 7.);
    gr.add_edge(g, h, 8.);
    gr.add_edge(g, i, 9.);
    gr.add_edge(h, i, 10.);

    println!("{}", Dot::new(&gr));

    let expected_cut_vertices = HashSet::from([b, d, e, g]);
    let mut iter = CutVerticesSearch::new(&gr);
    let mut cut_vertices = HashSet::new();
    while let Some(cut_vertex) = iter.next(&gr) {
        cut_vertices.insert(cut_vertex);
    }
    assert_eq!(expected_cut_vertices, cut_vertices);

    assert_eq!(iter.next(&gr), None);
}

#[test]
fn cut_vertices_test_hard() {
    let mut gr = Graph::new_undirected();
    let mut nodes = Vec::new();
    for _ in 0..26 {
        nodes.push(gr.add_node(1));
    }

    let edges = vec![
        (1, 3, 1),
        (2, 3, 1),
        (3, 4, 1),
        (4, 6, 1),
        (4, 5, 1),
        (6, 7, 1),
        (7, 8, 1),
        (7, 9, 1),
        (7, 10, 1),
        (8, 9, 1),
        (8, 10, 1),
        (8, 18, 1),
        (9, 10, 1),
        (9, 11, 1),
        (10, 22, 1),
        (11, 12, 1),
        (12, 13, 1),
        (13, 14, 1),
        (12, 14, 1),
        (15, 17, 1),
        (16, 17, 1),
        (17, 18, 1),
        (17, 19, 1),
        (18, 19, 1),
        (19, 20, 1),
        (19, 21, 1),
        (0, 22, 1),
        (0, 25, 1),
        (22, 23, 1),
        (23, 24, 1),
        (24, 25, 1),
    ];

    for (u, v, weight) in &edges {
        gr.add_edge(nodes[*u], nodes[*v], weight);
    }

    println!("{}", Dot::new(&gr));

    let cut_vertices = vec![3, 4, 6, 7, 8, 9, 10, 22, 11, 12, 18, 17, 19];

    let expected_cut_vertices: HashSet<NodeIndex> =
        cut_vertices.iter().map(|&index| nodes[index]).collect();

    let mut iter = CutVerticesSearch::new(&gr);
    let mut cut_vertices = HashSet::new();
    while let Some(cut_vertex) = iter.next(&gr) {
        cut_vertices.insert(cut_vertex);
    }
    assert_eq!(expected_cut_vertices, cut_vertices);

    assert_eq!(iter.next(&gr), None);
}
