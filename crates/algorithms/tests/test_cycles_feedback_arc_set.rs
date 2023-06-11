use petgraph::{graph::stable::StableDiGraph, visit::EdgeRef};
use petgraph_algorithms::cycles::{greedy_feedback_arc_set, is_cyclic_directed};
use proptest::prelude::*;

proptest! {
    #[test]
    fn remaining_graph_is_acyclic(mut graph in any::<StableDiGraph<(), (), u8>>()) {
        let feedback_arc_set = greedy_feedback_arc_set(&graph).map(|edge| edge.id()).collect::<Vec<_>>();

        for edge in feedback_arc_set {
            graph.remove_edge(edge);
        }

        prop_assert!(!is_cyclic_directed(&graph));
    }
}
