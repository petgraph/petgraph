#![feature(test)]

extern crate petgraph;
extern crate test;

use petgraph::visit::NodeIndexable;
use test::Bencher;

#[allow(dead_code)]
mod common;
use common::*;

use petgraph::algo::{greedy_matching, maximum_bipartite_matching, maximum_matching};
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

/// Generate a bipartite graph instance.
///
/// This function produces an undirected bipartite graph with two partitions:
/// * `partition_1`: nodes with indices divisible by 6 (i.e., `0, 6, 12, ...`);
/// * `partition_2`: nodes with odd indices (i.e., `1, 3, 5, ...`).
///
/// An edge is created between a node `a` in `partition_1` and a node `b` in `partition_b`
/// if `a` index is lower than `j` index.
///
/// # Arguments
/// * `node_count_upper_bound` — The upper bound (exclusive) on the node indices to generate.
///
/// # Returns
/// A tuple with:
/// * `UnGraph<(), ()>` — The undirected bipartite graph;
/// * `Vec<u32>` — The first partition of the bipartite graph;
/// * `Vec<u32>` — The second partition of the bipartite graph.
fn generate_bipartite(node_count_upper_bound: u32) -> (UnGraph<(), ()>, Vec<u32>, Vec<u32>) {
    let mut edges = Vec::new();

    let mut partition_1 = Vec::new();
    let mut partition_2 = Vec::new();
    for i in 0..node_count_upper_bound {
        for j in i..node_count_upper_bound {
            if i % 6 == 0 && j % 2 == 1 {
                edges.push((i, j));
            }
        }
    }

    for i in (0..node_count_upper_bound).step_by(6) {
        partition_1.push(i);
    }

    for i in (1..node_count_upper_bound).step_by(2) {
        partition_2.push(i);
    }

    (UnGraph::from_edges(&edges), partition_1, partition_2)
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

#[bench]
fn maximum_bipartite_matching_generic_100(bench: &mut Bencher) {
    let (g, _, _) = generate_bipartite(100);
    bench.iter(|| maximum_matching(&g));
}

#[bench]
fn maximum_bipartite_matching_generic_1000(bench: &mut Bencher) {
    let (g, _, _) = generate_bipartite(1_000);
    bench.iter(|| maximum_matching(&g));
}

#[bench]
fn maximum_bipartite_matching_specific_100(bench: &mut Bencher) {
    let (g, partition_a, partition_b) = generate_bipartite(100);
    let partition_a_ids: Vec<_> = partition_a
        .iter()
        .map(|&id| NodeIndexable::from_index(&g, id as usize))
        .collect();
    let partition_b_ids: Vec<_> = partition_b
        .iter()
        .map(|&id| NodeIndexable::from_index(&g, id as usize))
        .collect();
    bench.iter(|| maximum_bipartite_matching(&g, &partition_a_ids, &partition_b_ids));
}

#[bench]
fn maximum_bipartite_matching_specific_1000(bench: &mut Bencher) {
    let (g, partition_a, partition_b) = generate_bipartite(1_000);
    let partition_a_ids: Vec<_> = partition_a
        .iter()
        .map(|&id| NodeIndexable::from_index(&g, id as usize))
        .collect();
    let partition_b_ids: Vec<_> = partition_b
        .iter()
        .map(|&id| NodeIndexable::from_index(&g, id as usize))
        .collect();
    bench.iter(|| maximum_bipartite_matching(&g, &partition_a_ids, &partition_b_ids));
}
