use crate::visit::{EdgeRef, IntoEdges, NodeCount, NodeIndexable};

use super::UnitMeasure;

/// \[Generic\] Page Rank algorithm.
///
/// Computes the ranks of every node in a graph.
///
/// Returns a `Vec` container mapping each node index to its rank.
/// The damping factor should be of type `f32` or `f64`.
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
/// let damping_factor = 0.7_f32;
/// let number_iterations = 10;
/// let output_ranks = page_rank(&g, damping_factor, number_iterations);
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

    for _ in 0..nb_iter {
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
