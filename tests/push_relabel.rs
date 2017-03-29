extern crate petgraph;

use petgraph::prelude::*;
use petgraph::algo::{
    has_path_connecting,
    push_relabel_max_flow,
    push_relabel_min_cut,
};
use petgraph::visit::{EdgeFiltered, IntoEdgeReferences};
use std::collections::{HashMap, HashSet};

fn mk_flow(elts: &[(usize, usize, i64)]) -> HashMap<(NodeIndex, NodeIndex), i64> {
    elts.iter().map(|&(u, v, w)| ((NodeIndex::new(u), NodeIndex::new(v)), w)).collect()
}

fn assert_is_flow(
    g: &Graph<(), i64>,
    source: NodeIndex,
    target: NodeIndex,
    flow: &HashMap<(NodeIndex, NodeIndex), i64>)
{
    for (&(u, v), f) in flow.iter() {
        let e = g.find_edge(u, v).unwrap();
        assert!(g.edge_weight(e).unwrap() >= f);
    }

    for u in g.node_indices() {
        if u == source {
            // The source only has outflow.
            for v in g.neighbors_directed(u, Incoming) {
                assert!(flow.get(&(v, u)).is_none());
            }
        } else if u == target {
            // The target only has inflow.
            for v in g.neighbors(u) {
                assert!(flow.get(&(u, v)).is_none());
            }
        } else {
            // Everyone else has inflow = outflow.
            let inflow: i64 = g.neighbors_directed(u, Incoming)
                .filter_map(|v| flow.get(&(v, u)))
                .sum();
            let outflow: i64 = g.neighbors(u)
                .filter_map(|v| flow.get(&(u, v)))
                .sum();
            assert_eq!(inflow, outflow);
        }
    }
}

fn assert_is_cut<'a>(g: &'a Graph<(), i64>, source: NodeIndex, target: NodeIndex, cut: &HashSet<EdgeIndex>) {
    let uncut = |e: <&'a Graph<(), i64> as IntoEdgeReferences>::EdgeRef| !cut.contains(&e.id());
    let cut_g = EdgeFiltered::from_fn(g, uncut);
    assert!(!has_path_connecting(&cut_g, source, target, None));
}

macro_rules! flow_cut_test {
    ($name:ident, $edges:expr, $s:expr, $t:expr, $expected_flow:expr, $expected_cut:expr) => {
        #[test]
        fn $name() {
            let g = Graph::<(), i64>::from_edges($edges);
            let s = NodeIndex::new($s);
            let t = NodeIndex::new($t);
            let flow = push_relabel_max_flow(&g, s, t).unwrap();
            assert_is_flow(&g, s, t, &flow);
            assert_eq!(flow, mk_flow($expected_flow));

            let cut = push_relabel_min_cut(&g, s, t).unwrap();
            let cut_set = cut.iter().map(|e| e.id()).collect::<HashSet<_>>();
            assert_is_cut(&g, s, t, &cut_set);
            let expected_cut = $expected_cut.iter()
                .map(|&(u, v)| g.find_edge(NodeIndex::new(u), NodeIndex::new(v)).unwrap())
                .collect::<HashSet<_>>();
            assert_eq!(cut_set, expected_cut);
        }
    }
}

flow_cut_test!(
    dead_ends,
    &[(0, 1, 1), (0, 2, 1), (0, 3, 1), (3, 4, 1), (4, 5, 1), (5, 6, 1)],
    0, 6,
    &[(0, 3, 1), (3, 4, 1), (4, 5, 1), (5, 6, 1)],
    &[(0, 3)]
);

// The example graph from wikipedia's page on the push-relabel algorithm.
flow_cut_test!(
    wikipedia,
    &[(0, 1, 15), (0, 3, 4), (1, 2, 12), (2, 3, 3), (3, 4, 10), (4, 1, 5), (2, 5, 7), (4, 5, 10)],
    0, 5,
    &[(0, 1, 10), (0, 3, 4), (1, 2, 10), (2, 3, 3), (2, 5, 7), (3, 4, 7), (4, 5, 7)],
    &[(0, 3), (2, 3), (2, 5)]
);

flow_cut_test!(
    negative_weights,
    &[(0, 1, 1), (0, 2, -1), (1, 3, 1), (2, 3, 1)],
    0, 3,
    &[(0, 1, 1), (1, 3, 1)],
    &[(0, 1), (0, 2)]
);

#[test]
fn overflow() {
    let big = 1 << 62;
    let small = 1 << 62 - 1;

    let g_big = Graph::<(), i64>::from_edges(&[
        (0, 2, big), (0, 1, big), (1, 2, big), (2, 3, big)
    ]);
    let g_small = Graph::<(), i64>::from_edges(&[
        (0, 2, small), (0, 1, small), (1, 2, small), (2, 3, small)
    ]);

    let s = NodeIndex::new(0);
    let t = NodeIndex::new(3);

    assert!(push_relabel_max_flow(&g_big, s, t).is_err());
    assert!(push_relabel_max_flow(&g_small, s, t).is_ok());
}

