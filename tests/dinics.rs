use petgraph::algo::dinics;
use petgraph::prelude::Graph;

#[test]
fn test_dinics_a() {
    // Example from https://downey.io/blog/max-flow-ford-fulkerson-algorithm-explanation/
    // Graph Image: https://images.downey.io/max-flow/max-flow-3.png
    let mut graph = Graph::<usize, u16>::new();
    let source = graph.add_node(0);
    let _ = graph.add_node(1);
    let _ = graph.add_node(2);
    let sink = graph.add_node(3);
    graph.extend_with_edges([(0, 1, 3), (0, 2, 2), (1, 2, 5), (1, 3, 2), (2, 3, 3)]);
    let (max_flow, _) = dinics(&graph, source, sink);
    assert_eq!(5, max_flow);
}

#[test]
fn test_dinics_b() {
    // Example from https://brilliant.org/wiki/ford-fulkerson-algorithm/
    let mut graph = Graph::<usize, f32>::new();
    let source = graph.add_node(0);
    let _ = graph.add_node(1);
    let _ = graph.add_node(2);
    let _ = graph.add_node(3);
    let _ = graph.add_node(4);
    let sink = graph.add_node(5);
    graph.extend_with_edges([
        (0, 1, 4.),
        (0, 2, 3.),
        (1, 3, 4.),
        (2, 4, 6.),
        (3, 2, 3.),
        (3, 5, 2.),
        (4, 5, 6.),
    ]);
    let (max_flow, _) = dinics(&graph, source, sink);
    assert_eq!(7.0, max_flow);
}
#[test]
fn test_dinics_c() {
    // Example from https://cp-algorithms.com/graph/edmonds_karp.html
    let mut graph = Graph::<usize, f32>::new();
    let source = graph.add_node(0);
    let _ = graph.add_node(1);
    let _ = graph.add_node(2);
    let _ = graph.add_node(3);
    let _ = graph.add_node(4);
    let sink = graph.add_node(5);
    graph.extend_with_edges([
        (0, 1, 7.),
        (0, 2, 4.),
        (1, 3, 5.),
        (1, 4, 3.),
        (2, 1, 3.),
        (2, 4, 2.),
        (3, 5, 8.),
        (4, 3, 3.),
        (4, 5, 5.),
    ]);
    let (max_flow, _) = dinics(&graph, source, sink);
    assert_eq!(10.0, max_flow);
}

#[test]
fn test_dinics_d() {
    // Example from https://www.programiz.com/dsa/ford-fulkerson-algorithm (corrected: result not 6 but 5)
    let mut graph = Graph::<u8, f32>::new();
    let source = graph.add_node(0);
    let _ = graph.add_node(1);
    let _ = graph.add_node(2);
    let _ = graph.add_node(3);
    let _ = graph.add_node(4);
    let sink = graph.add_node(5);
    graph.extend_with_edges([
        (0, 1, 8.),
        (0, 2, 3.),
        (1, 3, 9.),
        (2, 3, 7.),
        (2, 4, 4.),
        (3, 5, 2.),
        (4, 5, 5.),
    ]);
    let (max_flow, _) = dinics(&graph, source, sink);
    assert_eq!(5.0, max_flow);
}

#[test]
fn test_dinics_e() {
    let mut graph = Graph::<u8, u8>::new();
    let source = graph.add_node(0);
    let _ = graph.add_node(1);
    let _ = graph.add_node(2);
    let _ = graph.add_node(3);
    let _ = graph.add_node(4);
    let sink = graph.add_node(5);
    graph.extend_with_edges([
        (0, 1, 16),
        (0, 2, 13),
        (1, 2, 10),
        (1, 3, 12),
        (2, 1, 4),
        (2, 4, 14),
        (3, 2, 9),
        (3, 5, 20),
        (4, 3, 7),
        (4, 5, 4),
    ]);
    let (max_flow, _) = dinics(&graph, source, sink);
    assert_eq!(23, max_flow);
}

#[test]
fn test_dinics_f() {
    // Example taken from https://medium.com/@jithmisha/solving-the-maximum-flow-problem-with-ford-fulkerson-method-3fccc2883dc7
    let mut graph = Graph::<u8, u8>::new();
    let source = graph.add_node(0);
    let _ = graph.add_node(1);
    let _ = graph.add_node(2);
    let _ = graph.add_node(3);
    let _ = graph.add_node(4);
    let sink = graph.add_node(5);
    graph.extend_with_edges([
        (0, 1, 10),
        (0, 2, 10),
        (1, 2, 2),
        (1, 3, 4),
        (1, 4, 8),
        (2, 4, 9),
        (3, 5, 10),
        (4, 3, 6),
        (4, 5, 10),
    ]);
    let (max_flow, _) = dinics(&graph, source, sink);
    assert_eq!(19, max_flow);
}

#[test]
fn test_dinics_g() {
    // Example that can lead to invalid answers if backward edges
    // in residual network are not considered, resulting in a flow of 3
    // instead of the maximum 4

    let mut g = Graph::<(), u32>::new();

    let s = g.add_node(());
    let a = g.add_node(());
    let b = g.add_node(());
    let c = g.add_node(());
    let d = g.add_node(());
    let t = g.add_node(());

    g.add_edge(s, a, 2);
    g.add_edge(s, b, 2);
    g.add_edge(a, c, 1); // misleading edge
    g.add_edge(a, d, 2);
    g.add_edge(b, c, 2);
    g.add_edge(c, t, 2);
    g.add_edge(d, t, 2);

    let (flow, _) = dinics(&g, s, t);

    assert_eq!(flow, 4);
}

#[cfg(feature = "stable_graph")]
#[test]
fn test_dinics_stable_graph() {
    use petgraph::prelude::StableGraph;

    // Example from https://downey.io/blog/max-flow-ford-fulkerson-algorithm-explanation/
    // Graph Image: https://images.downey.io/max-flow/max-flow-3.png
    let mut graph = StableGraph::<usize, u16>::new();
    let source = graph.add_node(0);
    let _ = graph.add_node(1);
    let _ = graph.add_node(2);
    let node_to_remove = graph.add_node(3);
    let sink = graph.add_node(4);
    graph.extend_with_edges([
        (0, 1, 3),
        (0, 2, 2),
        (1, 2, 5),
        (1, 4, 2),
        (2, 4, 3),
        (0, 3, 1),
        (3, 4, 3),
        (1, 3, 3),
    ]);
    graph.remove_node(node_to_remove);
    let (max_flow, _) = dinics(&graph, source, sink);
    assert_eq!(5, max_flow);
}
