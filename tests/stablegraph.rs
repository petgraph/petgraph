#![cfg(feature = "stable_graph")]

extern crate petgraph;

use petgraph::graph::stable::StableGraph;

#[test]
fn node_indices() {
    let mut g = StableGraph::<_, ()>::new();
    let a = g.add_node(0);
    let b = g.add_node(1);
    let c = g.add_node(2);
    g.remove_node(b);
    let mut iter = g.node_indices();
    assert_eq!(iter.next(), Some(a));
    assert_eq!(iter.next(), Some(c));
    assert_eq!(iter.next(), None);
}
