use hashbrown::HashSet;

use petgraph::algo::min_st_cut;
use petgraph::prelude::Graph;
use petgraph::visit::EdgeRef;

#[test]
fn test_min_cut_trivial() {
    let mut graph = Graph::<(), u32>::new();
    let source = graph.add_node(());
    let a = graph.add_node(());
    let sink = graph.add_node(());

    let _s_a = graph.add_edge(source, a, 5);
    let a_t = graph.add_edge(a, sink, 3);

    let (cut_capacity, cut_edges) = min_st_cut(&graph, source, sink);
    assert_eq!(cut_capacity, 3);
    assert_eq!(cut_edges.len(), 1);
    assert_eq!(cut_edges[0].id(), a_t);
}

#[test]
fn test_min_st_cut_a() {
    // Example from https://downey.io/blog/max-flow-ford-fulkerson-algorithm-explanation/
    // Graph Image: https://images.downey.io/max-flow/max-flow-3.png
    let mut graph = Graph::<usize, u16>::new();
    let source = graph.add_node(0);
    let a = graph.add_node(1);
    let b = graph.add_node(2);
    let sink = graph.add_node(3);

    let s_a = graph.add_edge(source, a, 3);
    let s_b = graph.add_edge(source, b, 2);
    graph.extend_with_edges([(1, 2, 5), (1, 3, 2), (2, 3, 3)]);

    let (cut_capacity, cut_edges) = min_st_cut(&graph, source, sink);
    assert_eq!(5, cut_capacity);

    let expected_cut_edges: HashSet<_> = HashSet::from([s_a, s_b]);
    let actual_cut_edges: HashSet<_> = HashSet::from_iter(cut_edges.iter().map(|edge| edge.id()));
    assert_eq!(actual_cut_edges, expected_cut_edges);
}

#[test]
fn test_min_st_cut_b() {
    // Example from https://brilliant.org/wiki/ford-fulkerson-algorithm/
    let mut graph = Graph::<usize, f32>::new();
    let source = graph.add_node(0);
    let a = graph.add_node(1);
    let b = graph.add_node(2);
    let _ = graph.add_node(3);
    let _ = graph.add_node(4);
    let sink = graph.add_node(5);

    let s_a = graph.add_edge(source, a, 4.0);
    let s_b = graph.add_edge(source, b, 3.0);
    graph.extend_with_edges([
        (1, 3, 4.0),
        (2, 4, 6.0),
        (3, 2, 3.0),
        (3, 5, 2.0),
        (4, 5, 6.0),
    ]);

    let (cut_capacity, cut_edges) = min_st_cut(&graph, source, sink);
    assert_eq!(7.0, cut_capacity);

    let expected_cut_edges: HashSet<_> = HashSet::from([s_a, s_b]);
    let actual_cut_edges: HashSet<_> = cut_edges.iter().map(|e| e.id()).collect();
    assert_eq!(expected_cut_edges, actual_cut_edges);
}

#[test]
fn test_min_st_cut_c() {
    // Example from https://cp-algorithms.com/graph/edmonds_karp.html
    let mut graph = Graph::<usize, f32>::new();
    let source = graph.add_node(0);
    let a = graph.add_node(1);
    let b = graph.add_node(2);
    let c = graph.add_node(3);
    let d = graph.add_node(4);
    let sink = graph.add_node(5);

    let a_b = graph.add_edge(a, b, 5.0);
    let a_c = graph.add_edge(a, c, 3.0);
    let d_c = graph.add_edge(d, c, 2.0);
    graph.extend_with_edges([
        (source, a, 7.0),
        (source, d, 4.0),
        (d, a, 3.0),
        (b, sink, 8.0),
        (c, b, 3.0),
        (c, sink, 5.0),
    ]);

    let (cut_capacity, cut_edges) = min_st_cut(&graph, source, sink);
    assert_eq!(10.0, cut_capacity);

    let expected_cut_edges: HashSet<_> = HashSet::from([a_b, a_c, d_c]);
    let actual_cut_edges: HashSet<_> = cut_edges.iter().map(|e| e.id()).collect();
    assert_eq!(expected_cut_edges, actual_cut_edges);
}

#[test]
fn test_min_st_cut_d() {
    // Example from https://www.programiz.com/dsa/ford-fulkerson-algorithm (corrected: result not 6 but 5)
    let mut graph = Graph::<_, _>::new();
    let source = graph.add_node("S");
    let a = graph.add_node("A");
    let b = graph.add_node("B");
    let c = graph.add_node("C");
    let d = graph.add_node("D");
    let sink = graph.add_node("T");

    let s_d = graph.add_edge(source, d, 3.);
    let b_t = graph.add_edge(b, sink, 2.);
    graph.extend_with_edges([
        (source, a, 8.),
        (a, b, 9.),
        (c, sink, 5.),
        (d, b, 7.),
        (d, c, 4.),
    ]);

    let (cut_capacity, cut_edges) = min_st_cut(&graph, source, sink);
    assert_eq!(5., cut_capacity);

    let expected_cut_edges: HashSet<_> = HashSet::from([s_d, b_t]);
    let actual_cut_edges: HashSet<_> = cut_edges.iter().map(|e| e.id()).collect();
    assert_eq!(expected_cut_edges, actual_cut_edges);
}

#[test]
fn test_min_st_cut_e() {
    let mut graph = Graph::<_, u8>::new();
    let source = graph.add_node("S");
    let a = graph.add_node("A");
    let b = graph.add_node("B");
    let c = graph.add_node("C");
    let d = graph.add_node("D");
    let sink = graph.add_node("T");

    let a_c = graph.add_edge(a, c, 12);
    let d_c = graph.add_edge(d, c, 7);
    let d_t = graph.add_edge(d, sink, 4);
    graph.extend_with_edges([
        (source, a, 16),
        (source, b, 13),
        (a, b, 10),
        (b, a, 4),
        (b, d, 14),
        (c, b, 9),
        (c, sink, 20),
    ]);

    let (cut_capacity, cut_edges) = min_st_cut(&graph, source, sink);
    assert_eq!(23, cut_capacity);

    let expected_cut_edges: HashSet<_> = HashSet::from([a_c, d_c, d_t]);
    let actual_cut_edges: HashSet<_> = cut_edges.iter().map(|e| e.id()).collect();
    assert_eq!(expected_cut_edges, actual_cut_edges);
}

#[test]
fn test_min_st_cut_f() {
    // Example taken from https://medium.com/@jithmisha/solving-the-maximum-flow-problem-with-ford-fulkerson-method-3fccc2883dc7
    let mut graph = Graph::<_, u8>::new();
    let source = graph.add_node("S");
    let a = graph.add_node("A");
    let b = graph.add_node("B");
    let c = graph.add_node("C");
    let d = graph.add_node("D");
    let sink = graph.add_node("T");

    let s_a = graph.add_edge(source, a, 10);
    let c_d = graph.add_edge(c, d, 9);
    graph.extend_with_edges([
        (source, c, 10),
        (a, b, 4),
        (a, d, 8),
        (a, c, 2),
        (b, sink, 10),
        (d, b, 6),
        (d, sink, 10),
    ]);

    let (cut_capacity, cut_edges) = min_st_cut(&graph, source, sink);
    assert_eq!(19, cut_capacity);

    let expected_cut_edges: HashSet<_> = HashSet::from([s_a, c_d]);
    let actual_cut_edges: HashSet<_> = cut_edges.iter().map(|e| e.id()).collect();
    assert_eq!(expected_cut_edges, actual_cut_edges);
}

#[test]
fn test_min_st_cut_g() {
    let mut g = Graph::<_, u32>::new();

    let source = g.add_node("S");
    let a = g.add_node("A");
    let b = g.add_node("B");
    let c = g.add_node("C");
    let d = g.add_node("D");
    let sink = g.add_node("T");

    let s_a = g.add_edge(source, a, 2);
    let s_b = g.add_edge(source, b, 2);
    g.add_edge(a, c, 1); // misleading edge
    g.add_edge(a, d, 2);
    g.add_edge(b, c, 2);
    g.add_edge(c, sink, 2);
    g.add_edge(d, sink, 2);

    let (flow, cut_edges) = min_st_cut(&g, source, sink);
    assert_eq!(flow, 4);

    let expected_cut_edges: HashSet<_> = HashSet::from([s_a, s_b]);
    let actual_cut_edges: HashSet<_> = cut_edges.iter().map(|e| e.id()).collect();
    assert_eq!(expected_cut_edges, actual_cut_edges);
}
