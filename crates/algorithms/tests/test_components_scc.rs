//! Integration tests for Tarjan's and Kosaraju's strongly connected components algorithms.
//!
//! As both algorithms are very similar, we test them together.
//!
//! Uses the various graph representations that are consolidated in the `petgraph` crate to test
//! against.
//!
//! All tests are run against the same graph, which is a directed graph with 9 nodes and 9 edges.
//!
//! The graph is as follows:
//!
//! ```text
//! 6 → 0   4
//! ↑ ↖ ↓
//! 8   3
//! ↓ ↖
//! 2 → 5 ← 7 → 1
//!         ↓ ↗
//!         9
//! ```
//!
//! The strongly connected components are:
//! * `0, 3, 6`
//! * `1, 7, 9`
//! * `2, 5, 8`
//! * `4`

use petgraph::graphmap::GraphMap;
use petgraph_adjacency_matrix::{AdjacencyList, UnweightedAdjacencyList};
use petgraph_algorithms::components::{kosaraju_scc, tarjan_scc};
use petgraph_core::{
    edge::Directed,
    index::{DefaultIx, SafeCast},
    visit::GraphBase,
};
use petgraph_graph::Graph;

const EDGES: &[(DefaultIx, DefaultIx)] = &[
    (6, 0),
    (0, 3),
    (3, 6),
    (8, 6),
    (8, 2),
    (2, 5),
    (5, 8),
    (7, 5),
    (1, 7),
    (7, 9),
    (9, 1),
];

fn assert_scc<T: SafeCast<usize>>(received: Vec<Vec<T>>, expected: &[Vec<usize>]) {
    assert_eq!(received.len(), expected.len());

    // We first convert both representations to usize, so that we can sort them.
    let mut received: Vec<_> = received
        .into_iter()
        .map(|component| {
            component
                .into_iter()
                .map(SafeCast::cast)
                .collect::<Vec<_>>()
        })
        .collect();

    // The order of the components is not guaranteed, so we first order all components by their
    // index.
    for component in &mut received {
        component.sort_unstable();
    }

    // we now have a ordered components, but the order of the components is still not guaranteed,
    // therefore we sort them by their first element.
    // We know that each component has at least one element, so we can safely use indexed access
    // here.
    received.sort_by_key(|component| component[0]);

    assert_eq!(received, expected);
}

fn adjacency_list(
    scc: impl FnOnce(
        AdjacencyList<(), DefaultIx>,
    ) -> Vec<Vec<petgraph::adjacency_matrix::NodeIndex<DefaultIx>>>,
) {
    let mut graph = UnweightedAdjacencyList::<DefaultIx>::new();

    for _ in 0..10 {
        graph.add_node();
    }

    for &(from, to) in EDGES {
        graph.add_edge(
            petgraph::adjacency_matrix::NodeIndex::new(from),
            petgraph::adjacency_matrix::NodeIndex::new(to),
            (),
        );
    }

    let scc = scc(graph);

    assert_scc(scc, &[
        vec![0, 3, 6], //
        vec![1, 7, 9],
        vec![2, 5, 8],
        vec![4],
    ]);
}

#[test]
fn adjacency_list_tarjan() {
    adjacency_list(|graph| tarjan_scc(&graph));
}

fn graph_map(scc: impl FnOnce(GraphMap<DefaultIx, (), Directed>) -> Vec<Vec<DefaultIx>>) {
    let mut graph = GraphMap::<DefaultIx, (), Directed>::new();

    for index in 0..10 {
        graph.add_node(index);
    }

    for &(from, to) in EDGES {
        graph.add_edge(from, to, ());
    }

    let scc = scc(graph);

    assert_scc(scc, &[
        vec![0, 3, 6], //
        vec![1, 7, 9],
        vec![2, 5, 8],
        vec![4],
    ]);
}

#[test]
fn graph_map_tarjan() {
    graph_map(|graph| tarjan_scc(&graph));
}

#[test]
fn graph_map_kosaraju() {
    graph_map(|graph| kosaraju_scc(&graph));
}

fn graph(
    scc: impl FnOnce(Graph<(), (), Directed, DefaultIx>) -> Vec<Vec<petgraph::graph::NodeIndex>>,
) {
    let graph = Graph::from_edges(EDGES);

    let scc = scc(graph);

    assert_scc(scc, &[
        vec![0, 3, 6], //
        vec![1, 7, 9],
        vec![2, 5, 8],
        vec![4],
    ]);
}

#[test]
fn graph_tarjan() {
    graph(|graph| tarjan_scc(&graph));
}

#[test]
fn graph_kosaraju() {
    graph(|graph| kosaraju_scc(&graph));
}
