#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{
    collections::BTreeSet,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use petgraph_algorithms::isomorphism::{
    is_isomorphic, is_isomorphic_matching, is_isomorphic_subgraph, subgraph_isomorphisms_iter,
};
use petgraph_core::edge::{Directed, EdgeType, Undirected};
use petgraph_graph::{edge_index, node_index, EdgeIndex, Graph, NodeIndex};

/// Parse a text adjacency matrix format into a directed graph
fn parse_graph<Ty: EdgeType>(s: &str) -> Graph<(), (), Ty> {
    let mut gr = Graph::with_capacity(0, 0);
    let s = s.trim();
    let lines = s.lines().filter(|l| !l.is_empty());
    for (row, line) in lines.enumerate() {
        for (col, word) in line.split(' ').filter(|s| !s.is_empty()).enumerate() {
            let has_edge = word.parse::<i32>().unwrap();
            assert!(has_edge == 0 || has_edge == 1);
            if has_edge == 0 {
                continue;
            }
            while col >= gr.node_count() || row >= gr.node_count() {
                gr.add_node(());
            }
            gr.update_edge(node_index(row), node_index(col), ());
        }
    }
    gr
}

fn str_to_graph(s: &str) -> Graph<(), (), Undirected> {
    parse_graph(s)
}

fn str_to_digraph(s: &str) -> Graph<(), (), Directed> {
    parse_graph(s)
}

macro_rules! test_snapshot {
    // isomorphism (directed)
    ($name:ident <=>) => {
        paste::paste! {
            #[test]
            fn [<$name _directed>]() {
                let a = include_str!(concat!("snapshots/isomorphism/", stringify!([< $name _a >]), ".txt"));
                let b = include_str!(concat!("snapshots/isomorphism/", stringify!([< $name _b >]), ".txt"));

                let graph_a = str_to_digraph(a);
                let graph_b = str_to_digraph(b);

                assert!(is_isomorphic(&graph_a, &graph_b));
            }
        }
    };

    // isomorphism (undirected)
    ($name:ident == =) => {
        paste::paste! {
            #[test]
            fn [<$name _undirected>]() {
                let a = include_str!(concat!("snapshots/isomorphism/", stringify!([< $name _a >]), ".txt"));
                let b = include_str!(concat!("snapshots/isomorphism/", stringify!([< $name _b >]), ".txt"));

                let graph_a = str_to_graph(a);
                let graph_b = str_to_graph(b);

                assert_eq!(graph_a.edge_count(), graph_b.edge_count());
                assert_eq!(graph_a.node_count(), graph_b.node_count());

                assert!(is_isomorphic(&graph_a, &graph_b));
            }
        }
    };

    // no isomorphism (directed)
    ($name:ident <!>) => {
        paste::paste! {
            #[test]
            fn [<$name _directed>]() {
                let a = include_str!(concat!("snapshots/isomorphism/", stringify!([< $name _a >]), ".txt"));
                let b = include_str!(concat!("snapshots/isomorphism/", stringify!([< $name _b >]), ".txt"));

                let graph_a = str_to_digraph(a);
                let graph_b = str_to_digraph(b);

                assert!(!is_isomorphic(&graph_a, &graph_b));
            }
        }
    };

    // no isomorphism (undirected)
    ($name:ident =!=) => {
        paste::paste! {
            #[test]
            fn [<$name _undirected>]() {
                let a = include_str!(concat!("snapshots/isomorphism/", stringify!([< $name _a >]), ".txt"));
                let b = include_str!(concat!("snapshots/isomorphism/", stringify!([< $name _b >]), ".txt"));

                let graph_a = str_to_graph(a);
                let graph_b = str_to_graph(b);

                assert!(!is_isomorphic(&graph_a, &graph_b));
            }
        }
    };
}

/// Petersen A and B are isomorphic
///
/// http://www.dharwadker.org/tevet/isomorphism/
test_snapshot!(petersen <=>);
test_snapshot!(petersen ===);

/// An almost full set, isomorphic
test_snapshot!(full <=>);
test_snapshot!(full ===);

/// Praust A and B are not isomorphic
test_snapshot!(praust<!>);
test_snapshot!(praust =!=);

/// Isomorphic pair
test_snapshot!(coxeter <=>);
test_snapshot!(coxeter ===);

// G8 is not iso
test_snapshot!(g8<!>);
test_snapshot!(g8 =!=);

// G3 is not iso
test_snapshot!(g3<!>);
test_snapshot!(g3 =!=);

// S is not iso due to selfloop difference
test_snapshot!(s<!>);
test_snapshot!(s =!=);

test_snapshot!(g4d<!>);
test_snapshot!(g4u =!=);

test_snapshot!(g2u <=>);
test_snapshot!(g2u ===);

#[cfg(feature = "std")]
/// Parse a file in adjacency matrix format into a directed graph
fn graph_from_file(path: &str) -> Graph<(), (), Directed> {
    let mut f = std::fs::File::open(path).expect("file not found");
    let mut contents = String::new();

    std::io::Read::read_to_string(&mut f, &mut contents).expect("failed to read from file");

    parse_graph(&contents)
}

#[test]
fn empty() {
    let graph0 = Graph::<(), ()>::new();
    let graph1 = Graph::<(), ()>::new();

    assert!(is_isomorphic(&graph0, &graph1));
}

#[test]
fn one_node() {
    let mut graph0 = Graph::<_, ()>::new();
    let mut graph1 = Graph::<_, ()>::new();

    graph0.add_node(0);
    graph1.add_node(0);

    assert!(is_isomorphic(&graph0, &graph1));
}

#[test]
fn two_nodes() {
    let mut graph0 = Graph::<_, ()>::new();
    let mut graph1 = Graph::<_, ()>::new();

    graph0.add_node(0);
    graph1.add_node(0);

    graph0.add_node(1);
    graph1.add_node(1);

    assert!(is_isomorphic(&graph0, &graph1));
}

#[test]
fn three_nodes() {
    let mut graph0 = Graph::<_, ()>::new();
    let mut graph1 = Graph::<_, ()>::new();

    graph0.add_node(0);
    graph1.add_node(0);

    graph0.add_node(1);
    graph1.add_node(1);

    graph0.add_node(2);
    assert!(!is_isomorphic(&graph0, &graph1));

    graph1.add_node(2);
    assert!(is_isomorphic(&graph0, &graph1));
}

#[test]
fn identical_edge() {
    let mut g0 = Graph::<_, ()>::new();
    let mut g1 = Graph::<_, ()>::new();

    let a0 = g0.add_node(0);
    let a1 = g1.add_node(0);

    let b0 = g0.add_node(1);
    let b1 = g1.add_node(1);

    g0.add_edge(a0, b0, ());
    assert!(!is_isomorphic(&g0, &g1));

    g1.add_edge(a1, b1, ());
    assert!(is_isomorphic(&g0, &g1));
}

struct DisjointGraph {
    graph: Graph<i32, ()>,

    a: NodeIndex,
    b: NodeIndex,
    c: NodeIndex,
}

impl DisjointGraph {
    fn new() -> Self {
        let mut graph = Graph::<_, ()>::new();

        let a = graph.add_node(0);
        let b = graph.add_node(1);
        let c = graph.add_node(2);

        Self { graph, a, b, c }
    }
}

/// Graph A:
///
/// ```text
/// A → B
///
/// C
/// ```
///
/// Graph B:
///
/// ```text
/// A   B
///   ↗
/// C
/// ```
#[test]
fn one_edge() {
    let DisjointGraph {
        graph: mut graph_a,
        a: a0,
        b: b0,
        ..
    } = DisjointGraph::new();

    let DisjointGraph {
        graph: mut graph_b,
        b: b1,
        c: c1,
        ..
    } = DisjointGraph::new();

    graph_a.add_edge(a0, b0, ());
    graph_b.add_edge(c1, b1, ());

    assert!(is_isomorphic(&graph_a, &graph_b));
}

/// Graph A:
///
/// ```text
/// A → B
/// ↓
/// C
/// ```
///
/// Graph B:
///
/// ```text
/// A   B
/// ↑ ↗
/// C
/// ```
#[test]
fn two_edges() {
    let DisjointGraph {
        graph: mut graph_a,
        a: a0,
        b: b0,
        c: c0,
    } = DisjointGraph::new();

    let DisjointGraph {
        graph: mut graph_b,
        a: a1,
        b: b1,
        c: c1,
    } = DisjointGraph::new();

    graph_a.add_edge(a0, b0, ());
    graph_b.add_edge(c1, b1, ());

    graph_a.add_edge(a0, c0, ());
    graph_b.add_edge(c1, a1, ());
    assert!(is_isomorphic(&graph_a, &graph_b));
}

/// Graph A:
///
/// ```text
/// A → B
/// ↓ ↙
/// C
/// ```
///
/// Graph B:
///
/// ```text
/// A ← B
/// ↑ ↗
/// C
/// ```
#[test]
fn three_edges() {
    let DisjointGraph {
        graph: mut graph_a,
        a: a0,
        b: b0,
        c: c0,
    } = DisjointGraph::new();

    let DisjointGraph {
        graph: mut graph_b,
        a: a1,
        b: b1,
        c: c1,
    } = DisjointGraph::new();

    graph_a.add_edge(a0, b0, ());
    graph_b.add_edge(c1, b1, ());

    graph_a.add_edge(a0, c0, ());
    graph_b.add_edge(c1, a1, ());

    graph_a.add_edge(b0, c0, ());
    graph_b.add_edge(b1, a1, ());
    assert!(is_isomorphic(&graph_a, &graph_b));
}

/// Graph A:
///
/// ```text
/// A → B
/// ↓ ↙   ↘
/// C   D ← E
/// ```
///
/// Graph B:
///
/// ```text
/// A ← B
/// ↑ ↗ ↓
/// C   D → E
/// ```
#[test]
fn five_edges() {
    let DisjointGraph {
        graph: mut graph_a,
        a: a0,
        b: b0,
        c: c0,
    } = DisjointGraph::new();

    let DisjointGraph {
        graph: mut graph_b,
        a: a1,
        b: b1,
        c: c1,
    } = DisjointGraph::new();

    graph_a.add_edge(a0, b0, ());
    graph_b.add_edge(c1, b1, ());

    graph_a.add_edge(a0, c0, ());
    graph_b.add_edge(c1, a1, ());

    graph_a.add_edge(b0, c0, ());
    graph_b.add_edge(b1, a1, ());

    let d0 = graph_a.add_node(3);
    let d1 = graph_b.add_node(3);
    let e0 = graph_a.add_node(4);
    let e1 = graph_b.add_node(4);

    assert!(is_isomorphic(&graph_a, &graph_b));

    graph_a.add_edge(b0, e0, ());
    graph_a.add_edge(e0, d0, ());
    graph_b.add_edge(b1, d1, ());
    graph_b.add_edge(d1, e1, ());

    assert!(is_isomorphic(&graph_a, &graph_b));
}

/// Graph:
///
/// ```text
/// ⤿0 → 1
///  ↓ ↙
///  2
/// ```
#[test]
fn isomorphic_matching() {
    let graph_a = Graph::<(), _>::from_edges([
        (0, 0, 1), //
        (0, 1, 2),
        (0, 2, 3),
        (1, 2, 4),
    ]);
    let mut graph_b = graph_a.clone();

    assert!(is_isomorphic_matching(
        &graph_a,
        &graph_b,
        |x, y| x == y,
        |x, y| x == y
    ));

    graph_b[EdgeIndex::new(0)] = 0;
    assert!(!is_isomorphic_matching(
        &graph_a,
        &graph_b,
        |x, y| x == y,
        |x, y| x == y
    ));

    let mut graph_c = graph_a.clone();
    graph_c[EdgeIndex::new(1)] = 0;
    assert!(!is_isomorphic_matching(
        &graph_a,
        &graph_c,
        |x, y| x == y,
        |x, y| x == y
    ));
}

#[test]
fn integration_test_large() {
    let graph_a = str_to_digraph(include_str!("snapshots/isomorphism/large_a.txt"));
    let graph_b = str_to_digraph(include_str!("snapshots/isomorphism/large_b.txt"));

    assert!(is_isomorphic(&graph_a, &graph_b));
}

// TODO: potentially too slow (exclude from hack)
#[test]
fn integration_test_huge() {
    let graph_a = graph_from_file("tests/snapshots/isomorphism/huge_a.txt");
    let graph_b = graph_from_file("tests/snapshots/isomorphism/huge_b.txt");

    assert!(is_isomorphic(&graph_a, &graph_b));
}

// isomorphism isn't correct for multigraphs.
// Keep this testcase to document how
#[should_panic]
#[test]
fn panic_on_multigraph() {
    let graph_a = Graph::<(), ()>::from_edges([
        (0, 0), //
        (0, 0),
        (0, 1),
        (1, 1),
        (1, 1),
        (1, 0),
    ]);

    let graph_b = Graph::<(), ()>::from_edges([
        (0, 0), //
        (0, 1),
        (0, 1),
        (1, 1),
        (1, 0),
        (1, 0),
    ]);

    assert!(!is_isomorphic(&graph_a, &graph_b));
}

#[test]
fn subgraph() {
    let edges = [
        (0, 1), //
        (1, 2),
        (2, 0),
        (2, 3),
        (0, 4),
    ];

    let graph_a = Graph::<(), ()>::from_edges(&edges[..3]);
    let graph_b = Graph::<(), ()>::from_edges(edges);

    assert!(!is_isomorphic(&graph_a, &graph_b));
    assert!(is_isomorphic_subgraph(&graph_a, &graph_b));
}

#[test]
fn subgraph_iter() {
    let edges = [
        (0, 1), //
        (1, 2),
        (2, 0),
        (2, 3),
        (0, 4),
    ];

    let graph_a = Graph::<(), ()>::from_edges(&edges[..3]);
    let graph_b = Graph::<(), ()>::from_edges(edges);

    let graph_a_ref = &graph_a;
    let graph_b_ref = &graph_b;

    let mut node_match = PartialEq::eq;
    let mut edge_match = PartialEq::eq;

    let mappings =
        subgraph_isomorphisms_iter(&graph_a_ref, &graph_b_ref, &mut node_match, &mut edge_match)
            .unwrap();

    // Verify the iterator returns the expected mappings
    let expected_mappings: Vec<Vec<usize>> = vec![vec![0, 1, 2], vec![1, 2, 0], vec![2, 0, 1]];
    for mapping in mappings {
        assert!(expected_mappings.contains(&mapping));
    }
}

// TODO: potentially too slow (exclude from hack)
#[test]
fn subgraph_iter_coxeter() {
    // Verify all the mappings from the iterator are different
    let graph_a = str_to_digraph(include_str!("snapshots/isomorphism/coxeter_a.txt"));
    let graph_b = str_to_digraph(include_str!("snapshots/isomorphism/coxeter_b.txt"));

    let graph_a_ref = &graph_a;
    let graph_b_ref = &graph_b;

    let mut node_match = PartialEq::eq;
    let mut edge_match = PartialEq::eq;

    let mut unique = BTreeSet::new();

    let mappings =
        subgraph_isomorphisms_iter(&graph_a_ref, &graph_b_ref, &mut node_match, &mut edge_match)
            .unwrap();

    for mapping in mappings {
        let inserted = unique.insert(mapping);
        assert!(inserted);
    }
}

#[test]
fn subgraph_iter_non_isomorphic() {
    // The iterator should return None for graphs that are not isomorphic
    let graph_a = str_to_digraph(include_str!("snapshots/isomorphism/g8_a.txt"));
    let graph_b = str_to_digraph(include_str!("snapshots/isomorphism/g8_b.txt"));

    let graph_a_ref = &graph_a;
    let graph_b_ref = &graph_b;

    let mut node_match = PartialEq::eq;
    let mut edge_match = PartialEq::eq;

    let iter =
        subgraph_isomorphisms_iter(&graph_a_ref, &graph_b_ref, &mut node_match, &mut edge_match)
            .unwrap();

    assert_eq!(iter.count(), 0);
}

#[test]
fn subgraph_iter_regression_534() {
    // https://github.com/petgraph/petgraph/issues/534

    let mut graph = Graph::<String, ()>::new();
    let l1 = graph.add_node("l1".to_owned());
    let l2 = graph.add_node("l2".to_owned());
    graph.add_edge(l1, l2, ());

    let l3 = graph.add_node("l3".to_owned());
    graph.add_edge(l2, l3, ());

    let l4 = graph.add_node("l4".to_owned());
    graph.add_edge(l3, l4, ());

    let mut subgraph = Graph::<String, ()>::new();
    let l3 = subgraph.add_node("l3".to_owned());
    let l4 = subgraph.add_node("l4".to_owned());
    subgraph.add_edge(l3, l4, ());

    let mut node_match = PartialEq::eq;
    let mut edge_match = PartialEq::eq;

    let mappings =
        subgraph_isomorphisms_iter(&&subgraph, &&graph, &mut node_match, &mut edge_match)
            .unwrap()
            .collect::<Vec<_>>();

    assert_eq!(mappings, vec![vec![2, 3]]);
}
