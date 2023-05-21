#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
use alloc::{format, vec::Vec};
use core::hash::Hash;

use indexmap::IndexSet;
use petgraph_algorithms::heuristics::{greedy_matching, maximum_matching};
use petgraph_graph::{stable::StableUnGraph, NodeIndex, UnGraph};

macro_rules! assert_one_of {
    ($actual:expr, [$($expected:expr),+]) => {
        let expected = &[$($expected),+];
        if !expected.iter().any(|expected| expected == &$actual) {
            let expected = expected.iter().map(|e| format!("\n{:?}", e)).collect::<Vec<_>>();
            let comma_separated = expected.join(", ");

            panic!("assertion failed: `actual does not equal to any of expected`\nactual:\n{:?}\nexpected:{}", $actual, comma_separated);
        }
    };
}

macro_rules! set {
    () => {
        IndexSet::new()
    };
    ($(($source:expr, $target:expr)),+) => {
        {
            let mut set = IndexSet::new();
            $(
                set.insert(($source.into(), $target.into()));
            )*
            set
        }
    };
    ($($elem:expr),+) => {
        {
            let mut set = IndexSet::new();
            $(
                set.insert($elem.into());
            )*
            set
        }
    };
}

// So we don't have to type `.collect::<HashSet<_>>`.
fn collect<'a, T: Copy + Eq + Hash + 'a>(iter: impl Iterator<Item = T>) -> IndexSet<T> {
    iter.collect()
}

#[test]
fn greedy_empty() {
    let graph: UnGraph<(), ()> = UnGraph::default();
    let matching = greedy_matching(&graph);

    assert_eq!(collect(matching.edges()), set![]);
    assert_eq!(collect(matching.nodes()), set![]);
}

#[test]
fn greedy_disjoint() {
    let graph: UnGraph<(), ()> = UnGraph::from_edges([
        (0, 1), //
        (2, 3),
    ]);

    let matching = greedy_matching(&graph);

    assert_eq!(collect(matching.edges()), set![(0, 1), (2, 3)]);
    assert_eq!(collect(matching.nodes()), set![0, 1, 2, 3]);
}

#[test]
fn greedy_odd_path() {
    let graph: UnGraph<(), ()> = UnGraph::from_edges([
        (0, 1), //
        (1, 2),
        (2, 3),
    ]);

    let matching = greedy_matching(&graph);

    assert_one_of!(collect(matching.edges()), [
        set![(0, 1), (2, 3)], //
        set![(1, 2)]
    ]);

    assert_one_of!(collect(matching.nodes()), [set![0, 1, 2, 3], set![1, 2]]);
}

#[test]
fn greedy_star() {
    let graph: UnGraph<(), ()> = UnGraph::from_edges([
        (0, 1), //
        (0, 2),
        (0, 3),
    ]);

    let matching = greedy_matching(&graph);

    assert_one_of!(collect(matching.edges()), [
        set![(0, 1)],
        set![(0, 2)],
        set![(0, 3)]
    ]);
    assert_one_of!(collect(matching.nodes()), [set![0, 1], set![0, 2], set![
        0, 3
    ]]);
}

#[test]
fn maximum_empty() {
    let graph: UnGraph<(), ()> = UnGraph::default();
    let matching = maximum_matching(&graph);

    assert_eq!(collect(matching.edges()), set![]);
    assert_eq!(collect(matching.nodes()), set![]);
}

#[test]
fn maximum_disjoint() {
    let graph: UnGraph<(), ()> = UnGraph::from_edges([
        (0, 1), //
        (2, 3),
    ]);

    let matching = maximum_matching(&graph);

    assert_eq!(collect(matching.edges()), set![(0, 1), (2, 3)]);
    assert_eq!(collect(matching.nodes()), set![0, 1, 2, 3]);
}

#[test]
fn maximum_odd_path() {
    let graph: UnGraph<(), ()> = UnGraph::from_edges([
        (0, 1), //
        (1, 2),
        (2, 3),
    ]);

    let matching = maximum_matching(&graph);

    assert_eq!(collect(matching.edges()), set![(0, 1), (2, 3)]);
    assert_eq!(collect(matching.nodes()), set![0, 1, 2, 3]);
}

#[test]
fn maximum_in_stable_graph() {
    let mut graph: StableUnGraph<(), ()> = StableUnGraph::from_edges([
        (0, 1), //
        (0, 2),
        (1, 2),
        (1, 3),
        (2, 4),
        (3, 4),
        (3, 5),
    ]);

    // Create a hole by removing node that would otherwise belong to the maximum
    // matching.
    graph.remove_node(NodeIndex::new(4));

    let matching = maximum_matching(&graph);

    assert_one_of!(collect(matching.edges()), [
        set![(0, 1), (3, 5)],
        set![(0, 2), (1, 3)],
        set![(0, 2), (3, 5)]
    ]);
    assert_one_of!(collect(matching.nodes()), [
        set![0, 1, 3, 5],
        set![0, 2, 1, 3],
        set![0, 2, 3, 5]
    ]);
}

#[test]
fn is_perfect_in_stable_graph() {
    let mut graph: StableUnGraph<(), ()> = StableUnGraph::from_edges([
        (0, 1), //
        (1, 2),
        (2, 3),
    ]);

    graph.remove_node(NodeIndex::new(0));
    graph.remove_node(NodeIndex::new(1));

    let matching = maximum_matching(&graph);
    assert_eq!(matching.len(), 1);
    assert!(matching.is_perfect());
}
