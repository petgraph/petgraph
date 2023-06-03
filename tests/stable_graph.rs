#![cfg(feature = "stable_graph")]

extern crate itertools;
extern crate petgraph;
#[macro_use]
extern crate defmac;

use std::collections::HashSet;

use itertools::assert_equal;
use petgraph::{
    algo::{kosaraju_scc, min_spanning_tree, tarjan_scc},
    dot::Dot,
    prelude::*,
    stable_graph::{edge_index as e, node_index as n},
    visit::{EdgeIndexable, IntoEdgeReferences, IntoNodeReferences, NodeIndexable},
    EdgeType,
};

fn make_graph<Ty>() -> StableGraph<(), i32, Ty>
where
    Ty: EdgeType,
{
    let mut gr = StableGraph::default();
    let mut c = 0..;
    let mut e = || -> i32 { c.next().unwrap() };
    gr.extend_with_edges(&[
        (6, 0, e()),
        (0, 3, e()),
        (3, 6, e()),
        (8, 6, e()),
        (8, 2, e()),
        (2, 5, e()),
        (5, 8, e()),
        (7, 5, e()),
        (1, 7, e()),
        (7, 4, e()),
        (8, 6, e()), // parallel edge
        (4, 1, e()),
    ]);
    // make an identical replacement of n(4) and leave a hole
    let x = gr.add_node(());
    gr.add_edge(n(7), x, e());
    gr.add_edge(x, n(1), e());
    gr.add_edge(x, x, e()); // make two self loops
    let rm_self_loop = gr.add_edge(x, x, e());
    gr.add_edge(x, x, e());
    gr.remove_node(n(4));
    gr.remove_node(n(6));
    gr.remove_edge(rm_self_loop);
    gr
}

defmac!(edges ref gr, x => gr.edges(x).map(|r| (r.target(), *r.weight())));

defmac!(iter_eq a, b => a.eq(b));
defmac!(nodes_eq ref a, ref b => a.node_references().eq(b.node_references()));
defmac!(edgew_eq ref a, ref b => a.edge_references().eq(b.edge_references()));
defmac!(edges_eq ref a, ref b =>
        iter_eq!(
            a.edge_references().map(|e| (e.source(), e.target())),
            b.edge_references().map(|e| (e.source(), e.target()))));

use petgraph::{data::FromElements, stable_graph::StableGraph};

// TODO: move to algo
#[test]
fn from_min_spanning_tree() {
    let mut g = StableGraph::new();
    let mut nodes = Vec::new();
    for _ in 0..6 {
        nodes.push(g.add_node(()));
    }
    let es = [(4, 5), (3, 4), (3, 5)];
    for &(a, b) in es.iter() {
        g.add_edge(NodeIndex::new(a), NodeIndex::new(b), ());
    }
    for i in 0..3 {
        let _ = g.remove_node(nodes[i]);
    }
    let _ = StableGraph::<(), (), Undirected, usize>::from_elements(min_spanning_tree(&g));
}
