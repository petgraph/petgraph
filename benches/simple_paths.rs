#![feature(test)]

extern crate petgraph;
extern crate test;

mod common;

use common::ungraph;
use hashbrown::HashSet;
use petgraph::algo::{all_simple_paths, all_simple_paths_multi};
use petgraph::prelude::*;
use test::Bencher;

#[bench]
fn complete_graph_single_bench(bench: &mut Bencher) {
    static NODE_COUNT: usize = 6;
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
            let paths = all_simple_paths::<Vec<_>, _, fxhash::FxBuildHasher>(&g, from, to, 0, None);
            total_paths += paths.count();
        }
        total_paths
    });
}

#[bench]
fn complete_graph_multi_bench(bench: &mut Bencher) {
    static NODE_COUNT: usize = 6;
    let mut g = Graph::new_undirected();
    let nodes: Vec<NodeIndex> = (0..NODE_COUNT).map(|_| g.add_node(())).collect();

    // Create a complete undirected graph - every pair of nodes has an edge
    for i in 0..NODE_COUNT {
        for j in i + 1..NODE_COUNT {
            g.add_edge(nodes[i], nodes[j], ());
        }
    }

    let from = nodes[0];
    let to: HashSet<_, fxhash::FxBuildHasher> = (1..NODE_COUNT).map(|i| nodes[i]).collect();

    bench.iter(|| {
        let paths =
            all_simple_paths_multi::<Vec<_>, _, fxhash::FxBuildHasher>(&g, from, &to, 0, None);
        paths.count()
    });
}

#[bench]
fn petersen_single_bench(bench: &mut Bencher) {
    let g = ungraph().petersen_a();
    let from = NodeIndex::new(0);
    let targets: Vec<_> = (1..g.node_count()).map(NodeIndex::new).collect();

    bench.iter(|| {
        let mut total_paths = 0;
        for &to in &targets {
            let paths = all_simple_paths::<Vec<_>, _, fxhash::FxBuildHasher>(&g, from, to, 0, None);
            total_paths += paths.count();
        }
        total_paths
    });
}

#[bench]
fn petersen_multi_bench(bench: &mut Bencher) {
    let g = ungraph().petersen_a();
    let from = NodeIndex::new(0);
    let to: HashSet<_, fxhash::FxBuildHasher> = (1..g.node_count()).map(NodeIndex::new).collect();

    bench.iter(|| {
        let paths =
            all_simple_paths_multi::<Vec<_>, _, fxhash::FxBuildHasher>(&g, from, &to, 0, None);
        paths.count()
    });
}

#[bench]
fn bipartite_single_bench(bench: &mut Bencher) {
    let g = ungraph().bipartite();
    let from = NodeIndex::new(0);
    let targets: Vec<_> = (1..g.node_count()).map(NodeIndex::new).collect();

    bench.iter(|| {
        let mut total_paths = 0;
        for &to in &targets {
            let paths = all_simple_paths::<Vec<_>, _, fxhash::FxBuildHasher>(&g, from, to, 0, None);
            total_paths += paths.count();
        }
        total_paths
    });
}

#[bench]
fn bipartite_multi_bench(bench: &mut Bencher) {
    let g = ungraph().bipartite();
    let from = NodeIndex::new(0);
    let to: HashSet<_, fxhash::FxBuildHasher> = (1..g.node_count()).map(NodeIndex::new).collect();

    bench.iter(|| {
        let paths =
            all_simple_paths_multi::<Vec<_>, _, fxhash::FxBuildHasher>(&g, from, &to, 0, None);
        paths.count()
    });
}

#[bench]
fn full_single_bench(bench: &mut Bencher) {
    let g = ungraph().full_a();
    let from = NodeIndex::new(0);
    let targets: Vec<_> = (1..g.node_count()).map(NodeIndex::new).collect();

    bench.iter(|| {
        let mut total_paths = 0;
        for &to in &targets {
            let paths = all_simple_paths::<Vec<_>, _, fxhash::FxBuildHasher>(&g, from, to, 0, None);
            total_paths += paths.count();
        }
        total_paths
    });
}

#[bench]
fn full_multi_bench(bench: &mut Bencher) {
    let g = ungraph().full_a();
    let from = NodeIndex::new(0);
    let to: HashSet<_, fxhash::FxBuildHasher> = (1..g.node_count()).map(NodeIndex::new).collect();

    bench.iter(|| {
        let paths =
            all_simple_paths_multi::<Vec<_>, _, fxhash::FxBuildHasher>(&g, from, &to, 0, None);
        paths.count()
    });
}
