use petgraph::algo::edmonds_karp;
use petgraph::graph::EdgeReference;
use petgraph::Graph;

#[test]
fn edmonds_karp_unweighted() {
    let mut graph = Graph::<_, u32>::new();
    let v0 = graph.add_node(0);
    let v1 = graph.add_node(1);
    let v2 = graph.add_node(2);
    let v3 = graph.add_node(3);
    let v4 = graph.add_node(4);

    graph.extend_with_edges(&[
        (v0, v1, 1),
        (v0, v2, 1),
        (v2, v3, 1),
        (v3, v4, 1),
        (v2, v4, 1),
    ]);
    // 0 ---> 1
    // |
    // v
    // 2 ---> 4
    // |     7
    // v   /
    // 3

    assert_eq!(1, edmonds_karp(&graph, v0, v4, |_| 1));

    graph.add_edge(v1, v4, 1);
    assert_eq!(2, edmonds_karp(&graph, v0, v4, |_| 1));

    graph.add_edge(v0, v3, 1);
    assert_eq!(3, edmonds_karp(&graph, v0, v4, |_| 1));

    graph.clear();
    graph.extend_with_edges(&[
        (v0, v1, 1),
        (v0, v2, 1),
        (v1, v4, 1),
        (v2, v3, 1),
        (v4, v3, 1),
        (v2, v4, 1),
    ]);
    assert_eq!(2, edmonds_karp(&graph, v0, v4, |_| 1));
}

#[test]
fn edmonds_karp_weighted() {
    let edge_weights = |e: EdgeReference<u32>| *e.weight();

    let mut graph = Graph::<_, u32>::new();
    let v0 = graph.add_node(0);
    let v1 = graph.add_node(1);
    let v2 = graph.add_node(2);
    let v3 = graph.add_node(3);
    graph.extend_with_edges(&[
        (v1, v2, 3),
        (v1, v3, 1),
        (v2, v3, 3),
        (v2, v0, 1),
        (v3, v0, 3),
    ]);
    let max_flow = edmonds_karp(&graph, v1, v0, edge_weights);
    assert_eq!(4, max_flow);

    let mut graph = Graph::<_, u32>::new();
    let a1 = graph.add_node(0);
    let b1 = graph.add_node(0);
    let b2 = graph.add_node(0);
    let b3 = graph.add_node(0);
    let c1 = graph.add_node(0);
    let c2 = graph.add_node(0);
    let c3 = graph.add_node(0);
    let d1 = graph.add_node(0);
    graph.extend_with_edges(&[
        (a1, b1, 6),
        (a1, b2, 1),
        (a1, b3, 1),
        (b1, c1, 6),
        (b1, c2, 6),
        (b2, c1, 1),
        (b2, c3, 1),
        (b3, c2, 1),
        (b3, c3, 1),
        (c1, d1, 1),
        (c2, d1, 4),
        (c3, d1, 3),
    ]);
    let max_flow = edmonds_karp(&graph, a1, d1, edge_weights);
    assert_eq!(7, max_flow);

    let mut graph = Graph::<_, u32>::new();
    let a1 = graph.add_node(0);
    let b1 = graph.add_node(0);
    let b2 = graph.add_node(0);
    let b3 = graph.add_node(0);
    let c1 = graph.add_node(0);
    let c2 = graph.add_node(0);
    let d1 = graph.add_node(0);
    graph.extend_with_edges(&[
        (a1, b1, 20),
        (a1, b2, 40),
        (a1, b3, 5),
        (b1, b2, 5),
        (b2, b3, 5),
        (b1, c1, 20),
        (b2, c1, 25),
        (b2, c2, 15),
        (b3, c2, 10),
        (c1, c2, 5),
        (c1, d1, 40),
        (c2, d1, 30),
    ]);
    let max_flow = edmonds_karp(&graph, a1, d1, edge_weights);
    assert_eq!(65, max_flow);
}
