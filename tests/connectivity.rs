use std::{
    collections::HashSet,
    fmt::{Debug, Display},
    hash::Hash,
};

use petgraph::{
    algo::connectivity::{BiconnectedComponentsSearch, CutEdgesSearch, CutVerticesSearch},
    dot::Dot,
    graph::NodeIndex,
    graph6::FromGraph6,
    visit::{GraphProp, IntoEdgeReferences, IntoNeighbors, IntoNodeReferences, NodeIndexable},
    Graph, Undirected,
};

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

    let edges = vec![
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

#[test]
fn biconnected_components_test_empty() {
    let gr: Graph<(), (), Undirected> = Graph::new_undirected();

    let mut iter = BiconnectedComponentsSearch::new(&gr);

    assert_eq!(iter.next(&gr), None);
    assert_eq!(iter.next(&gr), None);
}

#[test]
fn biconnected_components_test_k1() {
    let mut gr: Graph<&str, (), Undirected> = Graph::new_undirected();
    let a = gr.add_node("A");

    let mut iter = BiconnectedComponentsSearch::new(&gr);

    assert_eq!(iter.next(&gr), Some(HashSet::from([a])));
    assert_eq!(iter.next(&gr), None);
}

#[test]
fn biconnected_components_test_k2() {
    let mut gr = Graph::new_undirected();
    let a = gr.add_node("A");
    let b = gr.add_node("B");

    gr.add_edge(a, b, 1.);

    let mut iter = BiconnectedComponentsSearch::new(&gr);
    assert_eq!(iter.next(&gr), Some(HashSet::from([a, b])));
    assert_eq!(iter.next(&gr), None);
}

#[test]
fn biconnected_components_test_k3() {
    let mut gr = Graph::new_undirected();
    let a = gr.add_node("A");
    let b = gr.add_node("B");
    let c = gr.add_node("C");

    gr.add_edge(a, b, 1.);
    gr.add_edge(a, c, 2.);
    gr.add_edge(b, c, 3.);

    let mut iter = BiconnectedComponentsSearch::new(&gr);
    assert_eq!(iter.next(&gr), Some(HashSet::from([a, b, c])));
    assert_eq!(iter.next(&gr), None);
}

#[test]
// A - B - C - D
//         | /
//         E
fn biconnected_components_test_a() {
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

    let expected_biconnected_components = vec![
        HashSet::from([a, b]),
        HashSet::from([b, c]),
        HashSet::from([c, d, e]),
    ];

    test_biconnected_components(&gr, expected_biconnected_components);
}

#[test]
// A - B - C - D
//         | /
//     F - E
fn biconnected_components_test_b() {
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

    let expected_biconnected_components = vec![
        HashSet::from([a, b]),
        HashSet::from([b, c]),
        HashSet::from([c, d, e]),
        HashSet::from([e, f]),
    ];

    test_biconnected_components(&gr, expected_biconnected_components);
}

#[test]
// A - B - C
// | /   \ |
// D      E
fn biconnected_components_test_c() {
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

    let expected_biconnected_components = vec![HashSet::from([a, b, d]), HashSet::from([b, c, e])];

    test_biconnected_components(&gr, expected_biconnected_components);
}

#[test]
// A - B - D - E - F
//     | /   \
//     C       G - H
//             | /
//             I
fn biconnected_components_test_d() {
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

    let expected_biconnected_components = vec![
        HashSet::from([a, b]),
        HashSet::from([b, c, d]),
        HashSet::from([d, e]),
        HashSet::from([e, f]),
        HashSet::from([d, g]),
        HashSet::from([g, h, i]),
    ];

    test_biconnected_components(&gr, expected_biconnected_components);
}

#[test]
// 0 - 1 ---- 2
// |   | \  / |
// |   |  3 - 4
// |   |
// 6 - 5 - 7
//      \ /
//       8 - 9
fn biconnected_components_test_e() {
    let mut gr = Graph::new_undirected();
    let _0 = gr.add_node("0");
    let _1 = gr.add_node("A");
    let _2 = gr.add_node("B");
    let _3 = gr.add_node("C");
    let _4 = gr.add_node("D");
    let _5 = gr.add_node("E");
    let _6 = gr.add_node("F");
    let _7 = gr.add_node("G");
    let _8 = gr.add_node("H");
    let _9 = gr.add_node("I");

    gr.add_edge(_0, _1, 1.);
    gr.add_edge(_0, _6, 2.);
    gr.add_edge(_1, _2, 3.);
    gr.add_edge(_1, _3, 4.);
    gr.add_edge(_1, _5, 5.);
    gr.add_edge(_2, _3, 6.);
    gr.add_edge(_2, _4, 7.);
    gr.add_edge(_3, _4, 8.);
    gr.add_edge(_5, _6, 9.);
    gr.add_edge(_5, _7, 10.);
    gr.add_edge(_5, _8, 11.);
    gr.add_edge(_7, _8, 12.);
    gr.add_edge(_8, _9, 13.);

    let expected_biconnected_components = vec![
        HashSet::from([_1, _2, _3, _4]),
        HashSet::from([_8, _9]),
        HashSet::from([_5, _7, _8]),
        HashSet::from([_0, _1, _5, _6]),
    ];

    test_biconnected_components(&gr, expected_biconnected_components);
}

#[test]
fn biconnected_components_test_hard() {
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

    let biconnected_components = vec![
        vec![1, 3],
        vec![2, 3],
        vec![3, 4],
        vec![4, 5],
        vec![4, 6],
        vec![6, 7],
        vec![7, 8, 9, 10],
        vec![8, 18],
        vec![18, 19, 17],
        vec![17, 16],
        vec![17, 15],
        vec![21, 19],
        vec![19, 20],
        vec![9, 11],
        vec![11, 12],
        vec![12, 13, 14],
        vec![22, 10],
        vec![0, 25, 24, 23, 22],
    ];

    let expected_biconnected_components = biconnected_components
        .iter()
        .map(|component| {
            let mut set = HashSet::new();
            for &index in component {
                set.insert(nodes[index]);
            }
            set
        })
        .collect();

    test_biconnected_components(&gr, expected_biconnected_components);
}

fn test_biconnected_components<G, N>(gr: G, expected_biconnected_components: Vec<HashSet<N>>)
where
    N: Debug + Hash + Eq + Copy,
    G: IntoNodeReferences
        + IntoEdgeReferences
        + IntoNeighbors<NodeId = N>
        + NodeIndexable
        + GraphProp,
    G::NodeWeight: Display,
    G::EdgeWeight: Display,
{
    println!("{}", Dot::new(&gr));

    let mut iter = BiconnectedComponentsSearch::new(&gr);
    let mut biconnected_components = Vec::new();
    while let Some(biconnected_component) = iter.next(&gr) {
        biconnected_components.push(biconnected_component);
    }
    assert_eq!(iter.next(&gr), None);

    println!("actual: {:?}", biconnected_components);
    println!("expected: {:?}", expected_biconnected_components);
    let expected_len = expected_biconnected_components.len();
    for expected_biconnected_component in expected_biconnected_components {
        assert!(biconnected_components.contains(&expected_biconnected_component));
    }

    assert_eq!(biconnected_components.len(), expected_len);
}
