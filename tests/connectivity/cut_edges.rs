use petgraph::{
    algo::connectivity::CutEdgesSearch, dot::Dot, graph6::FromGraph6, Graph, Undirected,
};

use hashbrown::HashSet;

#[test]
fn cut_edges_test_empty() {
    let gr: Graph<(), (), Undirected> = Graph::new_undirected();

    let mut iter = CutEdgesSearch::new(&gr);

    assert_eq!(iter.next(&gr), None);
    assert_eq!(iter.next(&gr), None);
}

#[test]
fn cut_edges_test_k1() {
    let mut gr: Graph<&str, (), Undirected> = Graph::new_undirected();
    gr.add_node("A");

    let mut iter = CutEdgesSearch::new(&gr);

    assert_eq!(iter.next(&gr), None);
    assert_eq!(iter.next(&gr), None);
}

#[test]
fn cut_edges_test_k2() {
    let mut gr = Graph::new_undirected();
    let a = gr.add_node("A");
    let b = gr.add_node("B");

    gr.add_edge(a, b, 1.);

    let mut iter = CutEdgesSearch::new(&gr);

    assert_eq!(iter.next(&gr), Some((a, b)));
    assert_eq!(iter.next(&gr), None);
}

#[test]
// A - B - C - D
//         | /
//         E
fn cut_edges_test_a() {
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

    let expected_bridges = [(a, b), (b, c)];

    let mut iter = CutEdgesSearch::new(&gr);
    let mut bridges = HashSet::new();
    while let Some(bridge) = iter.next(&gr) {
        bridges.insert(bridge);
    }

    assert_eq!(bridges.len(), expected_bridges.len());
    for (a, b) in expected_bridges {
        assert!(bridges.contains(&(a, b)) || bridges.contains(&(b, a)))
    }

    assert_eq!(iter.next(&gr), None);
}

#[test]
// A - B - C - D
//         | /
//     F - E
fn cut_edges_test_b() {
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

    let expected_bridges = [(a, b), (b, c), (e, f)];

    let mut iter = CutEdgesSearch::new(&gr);
    let mut bridges = HashSet::new();
    while let Some(bridge) = iter.next(&gr) {
        bridges.insert(bridge);
    }

    assert_eq!(bridges.len(), expected_bridges.len());
    for (a, b) in expected_bridges {
        assert!(bridges.contains(&(a, b)) || bridges.contains(&(b, a)))
    }

    assert_eq!(iter.next(&gr), None);
}

#[test]
// A - B - D - E - F
//     | /   \
//     C       G - H
//             | /
//             I
fn cut_edges_test_c() {
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

    let expected_bridges = [(a, b), (d, e), (d, g), (e, f)];

    let mut iter = CutEdgesSearch::new(&gr);
    let mut bridges = HashSet::new();
    while let Some(bridge) = iter.next(&gr) {
        bridges.insert(bridge);
    }

    assert_eq!(bridges.len(), expected_bridges.len());
    for (a, b) in expected_bridges {
        assert!(bridges.contains(&(a, b)) || bridges.contains(&(b, a)))
    }
}

#[test]
fn cut_edges_test_c6() {
    let c6 = "EhEG".to_string(); // C_6 graph
    let gr: Graph<(), (), Undirected> = Graph::from_graph6_string(c6);

    let mut iter = CutEdgesSearch::new(&gr);

    assert_eq!(iter.next(&gr), None);
    assert_eq!(iter.next(&gr), None);
}

#[test]
fn cut_edges_test_butterfly() {
    let butterfly_graph = "DK{".to_string();
    let gr: Graph<(), (), Undirected> = Graph::from_graph6_string(butterfly_graph);

    let mut iter = CutEdgesSearch::new(&gr);

    assert_eq!(iter.next(&gr), None);
    assert_eq!(iter.next(&gr), None);
}

#[test]
fn cut_edges_test_star() {
    let star6rays = "FsaC?".to_string();
    let gr: Graph<(), (), Undirected> = Graph::from_graph6_string(star6rays);

    let mut bridges = HashSet::new();
    let mut iter = CutEdgesSearch::new(&gr);
    while let Some(bridge) = iter.next(&gr) {
        bridges.insert((bridge.0.index(), bridge.1.index()));
    }

    let expected_bridges = [(0, 1), (0, 2), (0, 3), (0, 4), (0, 5), (0, 6)];

    assert_eq!(bridges.len(), expected_bridges.len());
    for (a, b) in expected_bridges {
        assert!(bridges.contains(&(a, b)) || bridges.contains(&(b, a)))
    }
}

#[test]
fn cut_edges_test_small_tree() {
    let mut gr = Graph::new_undirected();
    let mut nodes = Vec::new();
    for _ in 0..13 {
        nodes.push(gr.add_node(1));
    }

    let edges = [
        (1, 2, 1.0),
        (2, 3, 1.0),
        (3, 4, 1.0),
        (4, 5, 1.0),
        (5, 6, 1.0),
        (5, 7, 1.0),
        (5, 9, 1.0),
        (9, 10, 1.0),
        (10, 11, 1.0),
        (10, 12, 1.0),
        (5, 0, 1.0),
    ];

    for (u, v, weight) in &edges {
        gr.add_edge(nodes[*u], nodes[*v], weight);
    }

    let mut bridges = HashSet::new();
    let mut iter = CutEdgesSearch::new(&gr);
    while let Some(bridge) = iter.next(&gr) {
        bridges.insert((bridge.0.index(), bridge.1.index()));
    }

    assert_eq!(bridges.len(), edges.len());
    for (a, b, _w) in &edges {
        assert!(bridges.contains(&(*a, *b)) || bridges.contains(&(*b, *a)))
    }
}
