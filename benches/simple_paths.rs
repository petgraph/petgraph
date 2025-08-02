#![feature(test)]

extern crate petgraph;
extern crate test;

use std::collections::hash_map::RandomState;

use hashbrown::HashSet;
use petgraph::algo::{all_simple_paths, all_simple_paths_multi};
use petgraph::prelude::*;
use test::Bencher;

#[bench]
fn simple_paths_bench(bench: &mut Bencher) {
    static NODE_COUNT: usize = 5;
    let mut g = Graph::new_undirected();
    let nodes: Vec<NodeIndex> = (0..NODE_COUNT).map(|_| g.add_node(())).collect();

    // Create a complete undirected graph - every pair of nodes has an edge
    for i in 0..NODE_COUNT {
        for j in i + 1..NODE_COUNT {
            g.add_edge(nodes[i], nodes[j], ());
        }
    }

    let from = nodes[0];
    let to = nodes[NODE_COUNT - 1];

    bench.iter(|| {
        let paths = all_simple_paths::<Vec<_>, _, RandomState>(&g, from, to, 0, None);
        paths.count()
    });
}

#[bench]
fn simple_paths_multi_bench(bench: &mut Bencher) {
    static NODE_COUNT: usize = 5;
    let mut g = Graph::new_undirected();
    let nodes: Vec<NodeIndex> = (0..NODE_COUNT).map(|_| g.add_node(())).collect();

    // Create a complete undirected graph - every pair of nodes has an edge
    for i in 0..NODE_COUNT {
        for j in i + 1..NODE_COUNT {
            g.add_edge(nodes[i], nodes[j], ());
        }
    }

    let from = nodes[0];
    let to: HashSet<_, RandomState> = (1..NODE_COUNT).map(|i| nodes[i]).collect();

    bench.iter(|| {
        let paths = all_simple_paths_multi::<Vec<_>, _, RandomState>(&g, from, &to, 0, None);
        paths.count()
    });
}

#[bench]
fn simple_paths_single_target_equivalent_bench(bench: &mut Bencher) {
    static NODE_COUNT: usize = 5;
    let mut g = Graph::new_undirected();
    let nodes: Vec<NodeIndex> = (0..NODE_COUNT).map(|_| g.add_node(())).collect();

    // Create a complete undirected graph - every pair of nodes has an edge
    for i in 0..NODE_COUNT {
        for j in i + 1..NODE_COUNT {
            g.add_edge(nodes[i], nodes[j], ());
        }
    }

    let from = nodes[0];
    let targets: Vec<_> = (1..NODE_COUNT).map(|i| nodes[i]).collect();

    bench.iter(|| {
        let mut total_paths = 0;
        for &to in &targets {
            let paths = all_simple_paths::<Vec<_>, _, RandomState>(&g, from, to, 0, None);
            total_paths += paths.count();
        }
        total_paths
    });
}
