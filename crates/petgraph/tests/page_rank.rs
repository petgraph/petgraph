#[cfg(feature = "rayon")]
use petgraph::algo::page_rank::parallel_page_rank;
use petgraph::{Graph, algo::page_rank};

fn graph_example() -> Graph<String, f32> {
    // Taken and adapted from https://github.com/neo4j-labs/graph?tab=readme-ov-file#how-to-run-algorithms
    let mut graph = Graph::<_, f32>::new();
    graph.add_node("A".to_owned());
    graph.add_node("B".to_owned());
    graph.add_node("C".to_owned());
    graph.add_node("D".to_owned());
    graph.add_node("E".to_owned());
    graph.add_node("F".to_owned());
    graph.add_node("G".to_owned());
    graph.add_node("H".to_owned());
    graph.add_node("I".to_owned());
    graph.add_node("J".to_owned());
    graph.add_node("K".to_owned());
    graph.add_node("L".to_owned());
    graph.add_node("M".to_owned());
    graph.extend_with_edges([
        (1, 2),  // B->C
        (2, 1),  // C->B
        (4, 0),  // D->A
        (4, 1),  // D->B
        (5, 4),  // E->D
        (5, 1),  // E->B
        (5, 6),  // E->F
        (6, 1),  // F->B
        (6, 5),  // F->E
        (7, 1),  // G->B
        (7, 5),  // F->E
        (8, 1),  // G->B
        (8, 5),  // G->E
        (9, 1),  // H->B
        (9, 5),  // H->E
        (10, 1), // I->B
        (10, 5), // I->E
        (11, 5), // J->B
        (12, 5), // K->B
    ]);
    graph
}

fn expected_ranks() -> Vec<f32> {
    // Corrected values: standard PageRank with teleportation summed for every
    // node and dangling rank redistributed uniformly (matches networkx
    // `pagerank(graph, alpha=0.85)` to within 1e-7). The previous values were
    // produced by the old implementation whose mutually exclusive if/else
    // branches dropped terms from the recurrence (see issue #960).
    vec![
        0.030145930,
        0.378964203,
        0.336574242,
        0.014454654,
        0.036920650,
        0.079291752,
        0.036920650,
        0.014454654,
        0.014454654,
        0.014454654,
        0.014454654,
        0.014454654,
        0.014454654,
    ]
}

#[test]
fn test_page_rank() {
    let graph = graph_example();
    let output_ranks = page_rank(&graph, 0.85_f32, 100);
    assert!(
        !expected_ranks()
            .iter()
            .zip(output_ranks.iter())
            .any(|(expected, computed)| ((expected - computed).abs() > 1e-6)
                || computed.is_nan()
                || expected.is_nan())
    );
}

#[test]
#[cfg(feature = "rayon")]
fn test_par_page_rank() {
    let graph = graph_example();
    let output_ranks = parallel_page_rank(&graph, 0.85_f32, 100, Some(1e-12));
    assert!(
        !expected_ranks()
            .iter()
            .zip(output_ranks)
            .any(|(expected, computed)| ((expected - computed).abs() > 1e-6)
                || computed.is_nan()
                || expected.is_nan())
    );
}
