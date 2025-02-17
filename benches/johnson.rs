#![feature(test)]

extern crate petgraph;
extern crate test;

use petgraph::prelude::*;
use std::cmp::{max, min};
use test::Bencher;

use petgraph::algo::johnson;

#[bench]
#[allow(clippy::needless_range_loop)]
fn johnson_bench(bench: &mut Bencher) {
    static NODE_COUNT: usize = 100;
    let mut g = Graph::new_undirected();
    let nodes: Vec<NodeIndex<_>> = (0..NODE_COUNT).map(|i| g.add_node(i)).collect();
    for i in 0..NODE_COUNT {
        let n1 = nodes[i];
        let neighbour_count = i % 8 + 3;
        let j_from = max(0, i as i32 - neighbour_count as i32 / 2) as usize;
        let j_to = min(NODE_COUNT, j_from + neighbour_count);
        for j in j_from..j_to {
            let n2 = nodes[j];
            let distance = (i + 3) % 10;
            g.add_edge(n1, n2, distance);
        }
    }

    bench.iter(|| {
        let _scores = johnson(&g, |e| *e.weight());
    });
}

#[bench]
fn johnson_sparse_1000_nodes(bench: &mut Bencher) {
    let graph = build_graph(1000, false);

    bench.iter(|| {
        let _scores = johnson(&graph, |e| *e.weight());
    });
}

#[bench]
fn johnson_dense_1000_nodes(bench: &mut Bencher) {
    let graph = build_graph(1000, true);

    bench.iter(|| {
        let _scores = johnson(&graph, |e| *e.weight());
    });
}

#[allow(clippy::needless_range_loop)]
fn build_graph(node_count: usize, dense: bool) -> Graph<usize, i32, Undirected> {
    let mut graph = Graph::new_undirected();
    let nodes: Vec<NodeIndex<_>> = (0..node_count).map(|i| graph.add_node(i)).collect();
    for i in 0..node_count {
        let n1 = nodes[i];
        let neighbour_count = if dense {
            i % (node_count / 3) + 3
        } else {
            i % 8 + 3
        };
        let j_from = max(0, i as i32 - neighbour_count as i32 / 2) as usize;
        let j_to = min(node_count, j_from + neighbour_count);
        for j in j_from..j_to {
            let n2 = nodes[j];
            let distance = (i + 3) % 10;
            graph.add_edge(n1, n2, distance as i32);
        }
    }
    graph
}
