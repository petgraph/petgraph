#![cfg(feature="quickcheck")]
extern crate quickcheck;

extern crate petgraph;

use petgraph::{Graph, Undirected};
use petgraph::algo::{min_spanning_tree, is_cyclic_undirected};

fn prop(g: Graph<(), u32>) -> bool {
    // filter out isolated nodes
    let no_singles = g.filter_map(
        |nx, w| g.neighbors_undirected(nx).next().map(|_| w),
        |_, w| Some(w));
    for i in no_singles.node_indices() {
        assert!(no_singles.neighbors_undirected(i).count() > 0);
    }
    assert_eq!(no_singles.edge_count(), g.edge_count());
    let mst = min_spanning_tree(&no_singles);
    assert!(!is_cyclic_undirected(&mst));
    true
}

fn prop_undir(g: Graph<(), u32, Undirected>) -> bool {
    // filter out isolated nodes
    let no_singles = g.filter_map(
        |nx, w| g.neighbors_undirected(nx).next().map(|_| w),
        |_, w| Some(w));
    for i in no_singles.node_indices() {
        assert!(no_singles.neighbors_undirected(i).count() > 0);
    }
    assert_eq!(no_singles.edge_count(), g.edge_count());
    let mst = min_spanning_tree(&no_singles);
    assert!(!is_cyclic_undirected(&mst));
    true
}

#[test]
fn arbitrary() {
    quickcheck::quickcheck(prop as fn(_) -> bool);
    quickcheck::quickcheck(prop_undir as fn(_) -> bool);
}
