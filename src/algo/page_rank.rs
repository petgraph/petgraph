use crate::visit::{EdgeRef, IntoEdges, NodeCount, NodeIndexable};

pub fn page_rank<G>(graph: G, damping_factor: f32, nb_iter: usize) -> Vec<f32>
where
    G: NodeCount + IntoEdges + NodeIndexable,
{
    let node_count = graph.node_count();
    assert!(node_count > 0, "Graph must have nodes.");
    assert!(
        0.0 <= damping_factor && damping_factor <= 1.0,
        "Damping factor should be between 0 et 1."
    );
    let nb = node_count as f32;
    let mut ranks = vec![1. / nb; node_count];
    let nodeix = |i| graph.from_index(i);
    let out_degrees: Vec<f32> = (0..node_count)
        .map(|i| graph.edges(nodeix(i)).map(|_| 1.).sum::<f32>())
        .collect();

    for iter in 0..nb_iter {
        println!("Iteration: {iter}");
        let mut pi = (0..node_count)
            .enumerate()
            .map(|(v, _)| {
                ranks
                    .iter()
                    .enumerate()
                    .map(|(w, r)| {
                        let mut w_out_edges = graph.edges(nodeix(w));
                        if let Some(_) = w_out_edges.find(|e| e.target() == nodeix(v)) {
                            damping_factor * *r / out_degrees[w]
                        } else if out_degrees[w] == 0. {
                            damping_factor * *r / nb // stochastic matrix condition
                        } else {
                            (1.0 - damping_factor) * *r / nb // random jumps
                        }
                    })
                    .sum::<f32>()
            })
            .collect::<Vec<f32>>();
        let sum = pi.iter().sum::<f32>();
        pi = pi.iter().map(|r| r / sum).collect::<Vec<f32>>();
        ranks = pi;
    }
    ranks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Graph;

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

        let output_ranks = page_rank(&graph, 0.85, 100);
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
}
