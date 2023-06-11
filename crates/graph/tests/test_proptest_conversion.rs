#![cfg(feature = "proptest")]

mod common;

use common::proptest::{assert_edges_eq, assert_edges_without_weight_eq, assert_nodes_eq};
use petgraph_core::visit::{EdgeIndexable, EdgeRef, IntoEdgeReferences, NodeIndexable};
use petgraph_graph::{
    stable::{StableDiGraph, StableGraph},
    DiGraph, EdgeIndex, Graph, NodeIndex, UnGraph,
};
use proptest::prelude::*;

proptest! {
    #[test]
    fn graph_to_stable_to_graph_directed(graph in any::<DiGraph<i32, i32, u8>>()) {
        let stable = StableGraph::from(graph.clone());
        let back_to_graph = Graph::from(stable);

        assert_nodes_eq(&graph, &back_to_graph)?;
        assert_edges_eq(&graph, &back_to_graph)?;
        assert_edges_without_weight_eq(&graph, &back_to_graph)?;
    }

    #[test]
    fn graph_to_stable_to_graph_undirected(graph in any::<UnGraph<i32, i32, u8>>()) {
        let stable = StableGraph::from(graph.clone());
        let back_to_graph = Graph::from(stable);

        assert_nodes_eq(&graph, &back_to_graph)?;
        assert_edges_eq(&graph, &back_to_graph)?;
        assert_edges_without_weight_eq(&graph, &back_to_graph)?;
    }

    #[test]
    fn stable_to_graph_directed(mut graph in any::<StableDiGraph<usize, usize, u8>>()) {
        let back_to_graph = Graph::from(graph.clone());

        prop_assert_eq!(graph.node_count(), back_to_graph.node_count());
        prop_assert_eq!(graph.edge_count(), back_to_graph.edge_count());

        // we cannot directly compare the indices, as stable graphs have holes
        prop_assert_eq!(graph.node_weights().collect::<Vec<_>>(), back_to_graph.node_weights().collect::<Vec<_>>());
        prop_assert_eq!(graph.edge_weights().collect::<Vec<_>>(), back_to_graph.edge_weights().collect::<Vec<_>>());

        // we cannot compare graphs and stable graphs directly
        // (at least when looking at the edge source and target),
        // because stable graphs contain holes.

        // What we do is a bit sneaky: we use the weight and map it to a compact index.
        // Then we compare the graphs with the compact index as weight.
        let mut compact = 0;
        for index in 0..graph.node_bound() {
            let index = NodeIndex::new(index);
            if graph.contains_node(index) {
                graph[index] = compact;
                compact += 1;
            }
        }

        let mut compact = 0;
        for index in 0..graph.edge_bound() {
            let index = EdgeIndex::new(index);
            if graph.edge_weight(index).is_some() {
                graph[index] = compact;
                compact += 1;
            }
        }

        // now that we have the compact index mapped to the weight, we can compare the graphs edges
        // source and destinations.
        prop_assert_eq!(
            graph.edge_references()
                .map(|edge| (EdgeIndex::new(*edge.weight()), graph[edge.source()], graph[edge.target()]))
                .collect::<Vec<_>>(),
            back_to_graph
                .edge_references()
                .map(|edge| (edge.id(), edge.source().index(), edge.target().index()))
                .collect::<Vec<_>>()
        );
    }
}
