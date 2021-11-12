use petgraph::algo::edmonds_karp;
use petgraph::Graph;

fn main() {
    let mut graph = Graph::new();
    let v0 = graph.add_node(0);
    let v1 = graph.add_node(1);
    let v2 = graph.add_node(2);
    let v3 = graph.add_node(3);
    graph.extend_with_edges(&[
        (v1, v2, 3), (v1, v3, 1), (v2, v3, 3),
        (v2, v0, 1), (v3, v0, 3)
    ]);

    // Graph represented with edge weights
    //
    //       3
    //   v1 --> v2
    //   |    / |
    // 1 |  3/  | 1 
    //   v  L   V
    //   v3 --> v0
    //       3
    
    let max_flow = edmonds_karp(&graph, v1, v0, |e| *e.weight());
    assert_eq!(max_flow, 4);

    println!("First try {}", max_flow);
    println!("Correct answer: 4");
}
