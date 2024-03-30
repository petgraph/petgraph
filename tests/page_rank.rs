use petgraph::{algo::page_rank, Graph};

#[test]
fn test_page_rank() {
    // Taken and adapted from https://github.com/neo4j-labs/graph?tab=readme-ov-file#how-to-run-algorithms
    let mut graph = Graph::<_, f32>::new();
    graph.add_node("A");
    graph.add_node("B");
    graph.add_node("C");
    graph.add_node("D");
    graph.add_node("E");
    graph.add_node("F");
    graph.add_node("G");
    graph.add_node("H");
    graph.add_node("I");
    graph.add_node("J");
    graph.add_node("K");
    graph.add_node("L");
    graph.add_node("M");
    graph.extend_with_edges(&[
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

    let output_ranks = page_rank(&graph, 0.85_f32, 100);
    let expected_ranks = vec![
        0.029228685,
        0.38176042,
        0.3410649,
        0.014170233,
        0.035662483,
        0.077429585,
        0.035662483,
        0.014170233,
        0.014170233,
        0.014170233,
        0.014170233,
        0.014170233,
        0.014170233,
    ];
    assert_eq!(expected_ranks, output_ranks);
}
