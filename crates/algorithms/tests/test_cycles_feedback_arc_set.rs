use petgraph::{graph::stable::StableDiGraph, visit::EdgeRef};
use petgraph_algorithms::cycles::{greedy_feedback_arc_set, is_cyclic_directed};
use petgraph_graph::DiGraph;
use petgraph_proptest::tournament::graph_tournament_strategy;
use proptest::prelude::*;

#[cfg(not(miri))]
proptest! {
    #[test]
    fn remaining_graph_is_acyclic(mut graph in any::<StableDiGraph<(), (), u8>>()) {
        let feedback_arc_set = greedy_feedback_arc_set(&graph).map(|edge| edge.id()).collect::<Vec<_>>();

        for edge in feedback_arc_set {
            graph.remove_edge(edge);
        }

        prop_assert!(!is_cyclic_directed(&graph));
    }

    /// Assert that the size of the feedback arc set of a tournament does not exceed
    /// **|E| / 2 - |V| / 6**
    #[test]
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    fn performance_within_bound(graph in graph_tournament_strategy::<DiGraph<(), ()>>(0..128)) {
        let expected_bound = if graph.node_count() < 2 {
            0
        } else {
            ((graph.edge_count() as f64) / 2.0 - (graph.node_count() as f64) / 6.0) as usize
        };

        let size = greedy_feedback_arc_set(&graph).count();

        prop_assert!(size <= expected_bound);
    }
}
