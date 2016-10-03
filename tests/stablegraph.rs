#![cfg(feature = "stable_graph")]

extern crate petgraph;

use petgraph::stable_graph::StableGraph;
use petgraph::graph::node_index as n;
use petgraph::graph::NodeIndex;
use petgraph::algo::{scc, tarjan_scc};

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

fn assert_sccs_eq(mut res: Vec<Vec<NodeIndex>>, normalized: Vec<Vec<NodeIndex>>) {
    // normalize the result and compare with the answer.
    for scc in res.iter_mut() {
        scc.sort();
    }
    // sort by minimum element
    res.sort_by(|v, w| v[0].cmp(&w[0]));
    assert_eq!(res, normalized);
}

#[test]
fn test_scc() {
    let mut gr: StableGraph<(), ()> = StableGraph::from_edges(&[
        (6, 0),
        (0, 3),
        (3, 6),
        (8, 6), (8, 2),
        (2, 5), (5, 8), (7, 5),
        (1, 7),
        (7, 4),
        (4, 1)]);
    // make an identical replacement of n(4) and leave a hole
    let x = gr.add_node(());
    gr.add_edge(n(7), x, ());
    gr.add_edge(x, n(1), ());
    gr.remove_node(n(4));
    println!("{:?}", gr);

    assert_sccs_eq(scc(&gr), vec![
        vec![n(0), n(3), n(6)],
        vec![n(1), n(7),   x ],
        vec![n(2), n(5), n(8)],
    ]);
}


#[test]
fn test_tarjan_scc() {
    let mut gr: StableGraph<(), ()> = StableGraph::from_edges(&[
        (6, 0),
        (0, 3),
        (3, 6),
        (8, 6), (8, 2),
        (2, 5), (5, 8), (7, 5),
        (1, 7),
        (7, 4),
        (4, 1)]);
    // make an identical replacement of n(4) and leave a hole
    let x = gr.add_node(());
    gr.add_edge(n(7), x, ());
    gr.add_edge(x, n(1), ());
    gr.remove_node(n(4));

    assert_sccs_eq(tarjan_scc(&gr), vec![
        vec![n(0), n(3), n(6)],
        vec![n(1), n(7),   x ],
        vec![n(2), n(5), n(8)],
    ]);
}

