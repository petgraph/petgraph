use petgraph::algo::{hits, HitsNorm};
use petgraph::prelude::Graph;

#[cfg(feature = "rayon")]
use petgraph::algo::hits::parallel_hits;

static EXPECTED_AUTH: [f32; 9] = [
    0.0, 0.42644632, 0.32187292, 0.64638627, 0.0, 0.2364649, 0.10200262, 0.4265718, 0.22009642,
];

static EXPECTED_HUB: [f32; 9] = [
    0.51476306, 0.35736868, 0.23857062, 0.0, 0.640681, 0.2763222, 0.23867469, 0.08123399, 0.0,
];

#[test]
fn test_hits() {
    // Example taken from https://neo4j.com/docs/graph-data-science/current/algorithms/hits/
    let mut graph = Graph::<usize, ()>::new();
    graph.add_node(0);
    graph.add_node(1);
    graph.add_node(2);
    graph.add_node(3);
    graph.add_node(4);
    graph.add_node(5);
    graph.add_node(6);
    graph.add_node(7);
    graph.add_node(8);
    graph.extend_with_edges(&[
        (0, 1),
        (0, 2),
        (0, 3),
        (1, 2),
        (1, 3),
        (2, 3),
        (4, 1),
        (4, 3),
        (4, 5),
        (4, 7),
        (5, 6),
        (5, 8),
        (5, 7),
        (6, 7),
        (6, 8),
        (7, 8),
    ]);
    let (auth, hub) = hits::<_, f32>(&graph, None, 20, HitsNorm::Two);
    assert_eq!(Vec::from(EXPECTED_AUTH), auth);
    assert_eq!(Vec::from(EXPECTED_HUB), hub);

    let (auth, hub) = hits(&graph, Some(0.000001f32), 20, HitsNorm::Two);
    assert_eq!(Vec::from(EXPECTED_AUTH), auth);
    assert_eq!(Vec::from(EXPECTED_HUB), hub);
}

#[cfg(feature = "rayon")]
#[test]
fn test_parallel_hits() {
    // Example taken from https://neo4j.com/docs/graph-data-science/current/algorithms/hits/
    let mut graph = Graph::<usize, ()>::new();
    graph.add_node(0);
    graph.add_node(1);
    graph.add_node(2);
    graph.add_node(3);
    graph.add_node(4);
    graph.add_node(5);
    graph.add_node(6);
    graph.add_node(7);
    graph.add_node(8);
    graph.extend_with_edges(&[
        (0, 1),
        (0, 2),
        (0, 3),
        (1, 2),
        (1, 3),
        (2, 3),
        (4, 1),
        (4, 3),
        (4, 5),
        (4, 7),
        (5, 6),
        (5, 8),
        (5, 7),
        (6, 7),
        (6, 8),
        (7, 8),
    ]);
    let (auth, hub) = parallel_hits::<_, f32>(&graph, None, 20, HitsNorm::Two);
    assert!(EXPECTED_AUTH
        .iter()
        .zip(auth)
        .all(|(expected, computed)| ((expected - computed).abs() <= 1e-6)));
    assert!(EXPECTED_HUB
        .iter()
        .zip(hub)
        .all(|(expected, computed)| ((expected - computed).abs() <= 1e-6)));
}
