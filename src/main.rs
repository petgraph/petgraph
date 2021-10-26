use petgraph::algo::edmonds_karp;
use petgraph::dot::Dot;
use petgraph::Graph;

fn main() {
    let mut graph = Graph::<_, u32>::new();
    let v0 = graph.add_node(0);
    let v1 = graph.add_node(1);
    let v2 = graph.add_node(2);
    let v3 = graph.add_node(3);
    graph.extend_with_edges(&[
        (v1, v2, 3), (v1, v3, 1), (v2, v3, 3),
        (v2, v0, 1), (v3, v0, 3)
    ]);

    println!("{:?}", Dot::with_config(&graph, &[]));
    let max_flow = edmonds_karp(&graph, v1, v0);
    println!("First try {}", max_flow);
}
