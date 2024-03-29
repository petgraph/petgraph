use crate::visit::{EdgeRef, IntoEdges, NodeCount, NodeIndexable};

use super::{Measure, UnitMeasure};

/// \[Generic\] Page Rank algorithm.
///
/// Computes the ranks of every node in a graph.
///
/// Returns a `Vec` container mapping each node index to its rank.
/// # Example
/// ```rust
/// use petgraph::Graph;
/// use petgraph::algo::page_rank;
/// let mut g: Graph<(), usize> = Graph::new();
/// let a = g.add_node(());
/// let b = g.add_node(());
/// let c = g.add_node(());
/// let d = g.add_node(());
/// let e = g.add_node(());
/// g.extend_with_edges(&[(0, 1), (0, 3), (1, 2), (1, 3)]);
/// // With the following dot representation.
/// //digraph {
/// //    0 [ label = "()" ]
/// //    1 [ label = "()" ]
/// //    2 [ label = "()" ]
/// //    3 [ label = "()" ]
/// //    4 [ label = "()" ]
/// //    0 -> 1 [ label = "0.0" ]
/// //    0 -> 3 [ label = "0.0" ]
/// //    1 -> 2 [ label = "0.0" ]
/// //    1 -> 3 [ label = "0.0" ]
/// //}
/// let output_ranks = page_rank(&g, 0.7_f32, 10);
/// let expected_ranks = vec![0.14685437, 0.20267677, 0.22389607, 0.27971846, 0.14685437];
/// assert_eq!(expected_ranks, output_ranks);
/// ```
pub fn page_rank<G, D>(graph: G, damping_factor: D, nb_iter: usize) -> Vec<D>
where
    G: NodeCount + IntoEdges + NodeIndexable,
    D: UnitMeasure + Copy,
{
    let node_count = graph.node_count();
    assert!(node_count > 0, "Graph must have nodes.");
    assert!(
        D::zero() <= damping_factor && damping_factor <= D::one(),
        "Damping factor should be between 0 et 1."
    );
    let nb = D::from_usize(node_count);
    let mut ranks = vec![D::one() / nb; node_count];
    let nodeix = |i| graph.from_index(i);
    let out_degrees: Vec<D> = (0..node_count)
        .map(|i| graph.edges(nodeix(i)).map(|_| D::one()).sum::<D>())
        .collect();

    for iter in 0..nb_iter {
        println!("Iteration: {iter}");
        let pi = (0..node_count)
            .enumerate()
            .map(|(v, _)| {
                ranks
                    .iter()
                    .enumerate()
                    .map(|(w, r)| {
                        let mut w_out_edges = graph.edges(nodeix(w));
                        if let Some(_) = w_out_edges.find(|e| e.target() == nodeix(v)) {
                            damping_factor * *r / out_degrees[w]
                        } else if out_degrees[w] == D::zero() {
                            damping_factor * *r / nb // stochastic matrix condition
                        } else {
                            (D::one() - damping_factor) * *r / nb // random jumps
                        }
                    })
                    .sum::<D>()
            })
            .collect::<Vec<D>>();
        let sum = pi.iter().map(|score| *score).sum::<D>();
        ranks = pi.iter().map(|r| *r / sum).collect::<Vec<D>>();
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
}
