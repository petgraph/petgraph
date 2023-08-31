//! Tests are to be included as unit tests in the same file as the code they are testing.
//!
//! They are currently not included in that file, because the old code (with traits) as a file it
//! way too huge.
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{vec, vec::Vec};
use core::fmt;

use petgraph_adjacency_matrix::{AdjacencyList, NodeIndex};
use petgraph_core::{
    data::DataMap,
    id::DefaultIx,
    visit::{
        EdgeRef, IntoEdgeReferences, IntoEdges, IntoNeighbors, IntoNodeReferences, NodeCount,
        NodeIndexable,
    },
};

#[test]
fn node_indices() {
    let mut graph = AdjacencyList::<()>::new();
    let a = graph.add_node();
    let b = graph.add_node();
    let c = graph.add_node();

    let mut iter = graph.node_indices();

    assert_eq!(iter.next(), Some(a));
    assert_eq!(iter.next(), Some(b));
    assert_eq!(iter.next(), Some(c));
    assert_eq!(iter.next(), None);
}

fn assert_length<E>(graph: &AdjacencyList<E>, len: usize) {
    assert_eq!(graph.node_count(), len);
    assert_eq!(graph.node_bound(), len);
    assert_eq!(graph.node_indices().count(), len);
    assert_eq!(graph.node_indices().len(), len);
    assert_eq!(graph.node_references().count(), len);
    assert_eq!(graph.node_references().len(), len);
}

#[test]
fn node_bound() {
    let mut graph = AdjacencyList::<()>::new();
    assert_length(&graph, 0);

    for index in 1..=16 {
        graph.add_node();
        assert_length(&graph, index);
    }

    graph.clear();
    assert_length(&graph, 0);
}

fn create_graph() -> AdjacencyList<i32> {
    let mut graph = AdjacencyList::new();
    let mut c = 0..;

    for _ in 0..10 {
        graph.add_node();
    }

    let edges = [
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
        (8, 6), // parallel edge
        (9, 1),
        (9, 9),
        (9, 9),
    ];

    for (weight, (from, to)) in edges.into_iter().enumerate() {
        graph.add_edge(
            NodeIndex::new(from),
            NodeIndex::new(to),
            i32::try_from(weight).expect("too many edges"),
        );
    }

    graph
}

fn assert_iterator<T>(a: impl IntoIterator<Item = T>, b: impl IntoIterator<Item = T>)
where
    T: PartialEq + fmt::Debug,
{
    let a: Vec<_> = a.into_iter().collect();
    let b: Vec<_> = b.into_iter().collect();

    assert_eq!(a, b);
}

#[test]
fn edges_directed() {
    let graph = create_graph();

    assert_iterator(
        graph
            .edges(NodeIndex::new(9))
            .map(|r| (r.target(), *r.weight())),
        vec![
            (NodeIndex::new(1), 11), //
            (NodeIndex::new(9), 12),
            (NodeIndex::new(9), 13),
        ],
    );

    assert_iterator(
        graph
            .edges(NodeIndex::new(0))
            .map(|r| (r.target(), *r.weight())),
        vec![
            (NodeIndex::new(3), 1), //
        ],
    );
}

#[test]
fn edge_references() {
    let mut graph = create_graph();
    assert_eq!(graph.edge_count(), graph.edge_references().count());

    for reference in graph.edge_references() {
        assert_eq!(
            graph.edge_endpoints(reference.id()),
            Some((reference.source(), reference.target()))
        );

        assert_eq!(graph.edge_weight(reference.id()), Some(reference.weight()));
    }

    let edge_indices = graph.edge_indices_from(NodeIndex::new(9));

    for (index, edge) in edge_indices.enumerate() {
        assert_eq!(
            graph.edge_weight(edge).copied(),
            Some(i32::try_from(index + 11).expect("too many edges"))
        );
    }
}

#[test]
fn edge_iterators() {
    let graph = create_graph();

    for index in graph.node_indices() {
        assert_iterator(
            graph.neighbors(index),
            graph.edges(index).map(|reference| {
                assert_eq!(reference.source(), index);

                reference.target()
            }),
        );
    }
}

#[test]
fn node_references_eq_indices() {
    let graph = create_graph();

    assert_iterator(graph.node_references(), graph.node_indices());
}

#[test]
fn neighbours_undirected() {
    let mut graph: AdjacencyList<_, DefaultIx> = AdjacencyList::new();

    let a = graph.add_node();
    let b = graph.add_node();
    let c = graph.add_node();
    let d = graph.add_node();

    let edges = [
        (a, b), //
        (a, c),
        (b, c),
        (c, c),
        (a, d),
    ];

    for (weight, (from, to)) in edges.into_iter().enumerate() {
        graph.add_edge(from, to, weight);
    }

    assert_iterator(graph.neighbors(a), vec![b, c, d]);
    assert_iterator(graph.neighbors(b), vec![c]);
    assert_iterator(graph.neighbors(c), vec![c]);
    assert_iterator(graph.neighbors(d), vec![]);
}

#[cfg(feature = "std")]
#[test]
fn add_node_out_of_bounds() {
    let mut graph = AdjacencyList::<(), u8>::new();

    for _ in 0..=u8::MAX {
        graph.add_node();
    }

    let result = std::panic::catch_unwind(move || {
        graph.add_node();
    });

    result.expect_err("adding node out of bounds should panic");
}

#[cfg(feature = "std")]
#[test]
fn add_edge_vacant() {
    let mut graph: AdjacencyList<_, DefaultIx> = AdjacencyList::new();

    let a = graph.add_node();
    let b = graph.add_node();
    graph.add_node();

    graph.clear();

    let result = std::panic::catch_unwind(move || {
        graph.add_edge(a, b, 0);
    });

    result.expect_err("adding edge to vacant node should panic");
}

#[cfg(feature = "std")]
#[test]
fn add_edge_out_of_bounds_target() {
    let mut graph: AdjacencyList<_, DefaultIx> = AdjacencyList::new();

    let a = graph.add_node();
    graph.add_node();
    graph.add_node();

    let result = std::panic::catch_unwind(move || {
        graph.add_edge(a, NodeIndex::new(3), 0);
    });

    result.expect_err("adding edge to out of bounds node should panic");
}

#[cfg(feature = "std")]
#[test]
fn add_edge_out_of_bounds_source() {
    let mut graph: AdjacencyList<_, DefaultIx> = AdjacencyList::new();

    let a = graph.add_node();
    graph.add_node();
    graph.add_node();

    let result = std::panic::catch_unwind(move || {
        graph.add_edge(NodeIndex::new(3), a, 0);
    });

    result.expect_err("adding edge to out of bounds node should panic");
}
