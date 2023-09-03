use core::hash::Hash;
use std::vec::Vec;

use indexmap::IndexSet;
use petgraph_algorithms::heuristics::{greedy_matching, maximum_matching, Matching};
use petgraph_core::visit::{
    EdgeRef, IntoEdges, IntoNodeIdentifiers, NodeCount, NodeIndexable, VisitMap, Visitable,
};
use petgraph_graph::{stable::StableUnGraph, NodeIndex, UnGraph};
use proptest::prelude::*;

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

fn is_valid_matching<G>(matching: &Matching<G>) -> bool
where
    G: NodeIndexable,
{
    // A set of edges is a matching if no two edges from the matching share an
    // endpoint.
    for (s1, t1) in matching.edges() {
        for (s2, t2) in matching.edges() {
            if s1 == s2 && t1 == t2 {
                continue;
            }

            if s1 == s2 || s1 == t2 || t1 == s2 || t1 == t2 {
                // Two edges share an endpoint.
                return false;
            }
        }
    }

    true
}

fn is_maximum_matching<G>(graph: G, matching: &Matching<G>) -> bool
where
    G: NodeIndexable + IntoEdges + IntoNodeIdentifiers + Visitable,
{
    // Berge's lemma: a matching is maximum iff there is no augmenting path (a
    // path that starts and ends in unmatched vertices, and alternates between
    // matched and unmatched edges). Thus if we find an augmenting path, the
    // matching is not maximum.
    //
    // Start with an unmatched node and traverse the graph alternating matched
    // and unmatched edges. If an unmatched node is found, then an augmenting
    // path was found.
    for unmatched in graph
        .node_identifiers()
        .filter(|u| !matching.contains_node(*u))
    {
        let visited = &mut graph.visit_map();
        let mut stack = Vec::new();

        stack.push((unmatched, false));
        while let Some((u, do_matched_edges)) = stack.pop() {
            if visited.visit(u) {
                for e in graph.edges(u) {
                    if e.source() == e.target() {
                        // Ignore self-loops.
                        continue;
                    }

                    let is_matched = matching.contains_edge(e.source(), e.target());

                    if do_matched_edges && is_matched || !do_matched_edges && !is_matched {
                        stack.push((e.target(), !do_matched_edges));

                        // Found another free node (other than the starting one)
                        // that is unmatched - an augmenting path.
                        if !is_matched
                            && !matching.contains_node(e.target())
                            && e.target() != unmatched
                        {
                            return false;
                        }
                    }
                }
            }
        }
    }

    true
}

fn is_perfect_matching<G>(g: G, m: &Matching<G>) -> bool
where
    G: NodeCount + NodeIndexable,
{
    // By definition.
    g.node_count() % 2 == 0 && m.edges().count() == g.node_count() / 2
}

#[cfg(not(miri))]
proptest! {
    #[test]
    fn greedy_matching_is_valid(graph: StableUnGraph<(), (), u8>) {
        let matching = greedy_matching(&graph);

        prop_assert!(is_valid_matching(&matching));
    }

    #[test]
    fn maximum_matching_is_valid(graph: StableUnGraph<(), (), u8>) {
        let matching = maximum_matching(&graph);

        prop_assert!(is_valid_matching(&matching));
    }

    #[test]
    fn maximum_matching_is_maximum(graph: StableUnGraph<(), (), u8>) {
        let matching = maximum_matching(&graph);

        prop_assert!(is_maximum_matching(&graph, &matching));
    }

    #[test]
    fn greedy_matching_is_maximum(graph: StableUnGraph<(), (), u8>) {
        let matching = greedy_matching(&graph);

        prop_assert_eq!(matching.is_perfect(), is_perfect_matching(&graph, &matching));
    }

    #[test]
    fn maximum_matching_is_perfect(graph: StableUnGraph<(), (), u8>) {
        let matching = maximum_matching(&graph);

        prop_assert_eq!(matching.is_perfect(), is_perfect_matching(&graph, &matching));
    }
}
