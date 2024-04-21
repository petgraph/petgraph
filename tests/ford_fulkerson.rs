use petgraph::{algo::ford_fulkerson, Graph};

#[test]
fn test_ford_fulkerson() {
    // Example from CLRS book
    let mut graph = Graph::<usize, f32>::new();
    let source = graph.add_node(0);
    let _ = graph.add_node(1);
    let _ = graph.add_node(2);
    let _ = graph.add_node(3);
    let _ = graph.add_node(4);
    let destination = graph.add_node(5);
    graph.extend_with_edges(&[
        (0, 1),
        (0, 2),
        (1, 3),
        (2, 1),
        (2, 4),
        (3, 2),
        (3, 5),
        (4, 3),
        (4, 5),
    ]);
    let capacities: Vec<f32> = vec![16., 13., 12., 4., 14., 9., 20., 7., 4.];
    assert_eq!(
        23.0,
        ford_fulkerson(&graph, source, destination, &capacities)
    );
}
