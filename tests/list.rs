extern crate itertools;
extern crate petgraph;
#[macro_use]
extern crate defmac;

use itertools::assert_equal;
use petgraph::{
    adj::{AdjacencyList, DefaultIx, IndexType, NodeIndex, UnweightedAdjacencyList},
    algo::tarjan_scc,
    data::{DataMap, DataMapMut},
    dot::Dot,
    prelude::*,
    visit::{
        IntoEdgeReferences, IntoEdges, IntoNeighbors, IntoNodeReferences, NodeCount, NodeIndexable,
    },
};

fn assert_sccs_eq<Ix: IndexType>(
    mut res: Vec<Vec<NodeIndex<Ix>>>,
    normalized: Vec<Vec<NodeIndex<Ix>>>,
) {
    // normalize the result and compare with the answer.
    for scc in &mut res {
        scc.sort();
    }
    // sort by minimum element
    res.sort_by(|v, w| v[0].cmp(&w[0]));
    assert_eq!(res, normalized);
}

fn scc_graph() -> UnweightedAdjacencyList<DefaultIx> {
    let mut gr = AdjacencyList::new();
    for _ in 0..9 {
        gr.add_node();
    }
    for (a, b) in &[
        (6, 0),
        (0, 3),
        (3, 6),
        (8, 6),
        (8, 2),
        (2, 5),
        (5, 8),
        (7, 5),
        (1, 7),
    ] {
        gr.add_edge(n(*a), n(*b), ());
    }
    // make an identical replacement of n(4) and leave a hole
    let x = gr.add_node();
    gr.add_edge(n(7), x, ());
    gr.add_edge(x, n(1), ());
    gr
}

#[test]
fn test_tarjan_scc() {
    let gr = scc_graph();

    let x = n(gr.node_bound() as u32 - 1);
    assert_sccs_eq(tarjan_scc(&gr), vec![
        vec![n(0), n(3), n(6)],
        vec![n(1), n(7), x],
        vec![n(2), n(5), n(8)],
        vec![n(4)],
    ]);
}

fn make_graph() -> AdjacencyList<i32> {
    let mut gr = AdjacencyList::new();
    let mut c = 0..;
    let mut e = || -> i32 { c.next().unwrap() };
    for _ in 0..=9 {
        gr.add_node();
    }
    for &(from, to) in &[
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
    ] {
        gr.add_edge(n(from), n(to), e());
    }
    gr
}
