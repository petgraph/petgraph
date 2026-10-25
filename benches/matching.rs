#![feature(test)]

extern crate petgraph;
extern crate test;

use test::Bencher;

#[allow(dead_code)]
mod common;
use common::*;

use petgraph::algo::{greedy_matching, maximum_matching, maximum_weight_matching};
use petgraph::graph::UnGraph;

fn huge() -> UnGraph<(), ()> {
    static NODE_COUNT: u32 = 1_000;

    let mut edges = Vec::new();

    for i in 0..NODE_COUNT {
        for j in i..NODE_COUNT {
            if i % 3 == 0 && j % 2 == 0 {
                edges.push((i, j));
            }
        }
    }

    // 999 nodes, 83500 edges
    UnGraph::from_edges(&edges)
}

#[bench]
fn greedy_matching_bipartite(bench: &mut Bencher) {
    let g = ungraph().bipartite();
    bench.iter(|| greedy_matching(&g));
}

#[bench]
fn greedy_matching_full(bench: &mut Bencher) {
    let g = ungraph().full_a();
    bench.iter(|| greedy_matching(&g));
}

#[bench]
fn greedy_matching_bigger(bench: &mut Bencher) {
    let g = ungraph().bigger();
    bench.iter(|| greedy_matching(&g));
}

#[bench]
fn greedy_matching_huge(bench: &mut Bencher) {
    let g = huge();
    bench.iter(|| greedy_matching(&g));
}

#[bench]
fn maximum_matching_bipartite(bench: &mut Bencher) {
    let g = ungraph().bipartite();
    bench.iter(|| maximum_matching(&g));
}

#[bench]
fn maximum_matching_full(bench: &mut Bencher) {
    let g = ungraph().full_a();
    bench.iter(|| maximum_matching(&g));
}

#[bench]
fn maximum_matching_bigger(bench: &mut Bencher) {
    let g = ungraph().bigger();
    bench.iter(|| maximum_matching(&g));
}

#[bench]
fn maximum_matching_huge(bench: &mut Bencher) {
    let g = huge();
    bench.iter(|| maximum_matching(&g));
}

fn weighted_graph(n: usize, dense: bool) -> UnGraph<(), i64> {
    let mut graph = UnGraph::default();
    let nodes: Vec<_> = (0..n).map(|_| graph.add_node(())).collect();
    let mut state = 0xA789_302C_8F62_51BDu64;
    for u in 0..n {
        for v in u + 1..n {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            if dense || v - u <= 6 {
                graph.add_edge(nodes[u], nodes[v], (state % 100_000) as i64 - 1_000);
            }
        }
    }
    graph
}

#[bench]
fn maximum_weight_matching_dense_128(bench: &mut Bencher) {
    let graph = weighted_graph(128, true);
    bench.iter(|| maximum_weight_matching(&graph, |e| *e.weight(), false));
}

#[bench]
fn maximum_weight_matching_dense_512(bench: &mut Bencher) {
    let graph = weighted_graph(512, true);
    bench.iter(|| maximum_weight_matching(&graph, |e| *e.weight(), false));
}

#[bench]
fn maximum_weight_matching_sparse_1000(bench: &mut Bencher) {
    let graph = weighted_graph(1_000, false);
    bench.iter(|| maximum_weight_matching(&graph, |e| *e.weight(), true));
}
