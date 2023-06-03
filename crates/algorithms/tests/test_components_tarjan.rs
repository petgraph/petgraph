//! Integration tests for Tarjan's algorithm.
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

use petgraph_adjacency_matrix::{AdjacencyList, UnweightedAdjacencyList};
use petgraph_algorithms::components::tarjan_scc;
use petgraph_core::index::{DefaultIx, SafeCast};

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

#[test]
fn adjacency_list() {
    let mut graph = UnweightedAdjacencyList::<DefaultIx>::new();

    for _ in 0..10 {
        graph.add_node();
    }

    for &(from, to) in EDGES {
        graph.add_edge(
            petgraph_adjacency_matrix::NodeIndex::new(from),
            petgraph_adjacency_matrix::NodeIndex::new(to),
            (),
        );
    }

    let scc = tarjan_scc(&graph);

    assert_scc(scc, &[
        vec![0, 3, 6], //
        vec![1, 7, 9],
        vec![2, 5, 8],
        vec![4],
    ]);
}
