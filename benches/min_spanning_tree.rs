#![feature(test)]

extern crate petgraph;
extern crate test;

use std::{fs::File, io::Read};

use test::Bencher;

#[allow(dead_code)]
mod common;
use common::{digraph, ungraph};

use petgraph::{
    algo::{min_spanning_tree, min_spanning_tree_prim},
    graph6_decoder::FromGraph6,
    visit::{Data, IntoEdgeReferences, IntoEdges, IntoNodeReferences, NodeIndexable},
    Graph, Undirected,
};

#[bench]
fn min_spanning_tree_kruskal_praust_undir_bench(bench: &mut Bencher) {
    let a = ungraph().praust_a();
    let b = ungraph().praust_b();

    bench.iter(|| (iterate_mst_kruskal(&a), iterate_mst_kruskal(&b)));
}

#[bench]
fn min_spanning_tree_kruskal_praust_dir_bench(bench: &mut Bencher) {
    let a = digraph().praust_a();
    let b = digraph().praust_b();

    bench.iter(|| (iterate_mst_kruskal(&a), iterate_mst_kruskal(&b)));
}

#[bench]
fn min_spanning_tree_kruskal_full_undir_bench(bench: &mut Bencher) {
    let a = ungraph().full_a();
    let b = ungraph().full_b();

    bench.iter(|| (iterate_mst_kruskal(&a), iterate_mst_kruskal(&b)));
}

#[bench]
fn min_spanning_tree_kruskal_full_dir_bench(bench: &mut Bencher) {
    let a = digraph().full_a();
    let b = digraph().full_b();

    bench.iter(|| (iterate_mst_kruskal(&a), iterate_mst_kruskal(&b)));
}

#[bench]
fn min_spanning_tree_kruskal_petersen_undir_bench(bench: &mut Bencher) {
    let a = ungraph().petersen_a();
    let b = ungraph().petersen_b();

    bench.iter(|| (iterate_mst_kruskal(&a), iterate_mst_kruskal(&b)));
}

#[bench]
fn min_spanning_tree_kruskal_petersen_dir_bench(bench: &mut Bencher) {
    let a = digraph().petersen_a();
    let b = digraph().petersen_b();

    bench.iter(|| (iterate_mst_kruskal(&a), iterate_mst_kruskal(&b)));
}

#[bench]
fn min_spanning_tree_kruskal_2_000n(bench: &mut Bencher) {
    let g = graph_from_graph6_file("tests/res/graph_2000n.g6");
    bench.iter(|| iterate_mst_kruskal(&g));
}

#[bench]
fn min_spanning_tree_kruskal_6_000n(bench: &mut Bencher) {
    let g = graph_from_graph6_file("tests/res/graph_6000n.g6");
    bench.iter(|| iterate_mst_kruskal(&g));
}

#[bench]
fn min_spanning_tree_prim_praust_undir_bench(bench: &mut Bencher) {
    let a = ungraph().praust_a();
    let b = ungraph().praust_b();

    bench.iter(|| (iterate_mst_prim(&a), iterate_mst_prim(&b)));
}

#[bench]
fn min_spanning_tree_prim_full_undir_bench(bench: &mut Bencher) {
    let a = ungraph().full_a();
    let b = ungraph().full_b();

    bench.iter(|| (iterate_mst_prim(&a), iterate_mst_prim(&b)));
}

#[bench]
fn min_spanning_tree_prim_petersen_undir_bench(bench: &mut Bencher) {
    let a: Graph<(), (), Undirected> = ungraph().petersen_a();
    let b = ungraph().petersen_b();

    bench.iter(|| (iterate_mst_prim(&a), iterate_mst_prim(&b)));
}

#[bench]
fn min_spanning_tree_prim_2_000n(bench: &mut Bencher) {
    let g = graph_from_graph6_file("tests/res/graph_2000n.g6");
    bench.iter(|| iterate_mst_prim(&g));
}

#[bench]
fn min_spanning_tree_prim_6_000n(bench: &mut Bencher) {
    let g = graph_from_graph6_file("tests/res/graph_6000n.g6");
    bench.iter(|| iterate_mst_prim(&g));
}

fn iterate_mst_kruskal<G>(g: G)
where
    G: Data + IntoEdges + IntoNodeReferences + IntoEdgeReferences + NodeIndexable,
    G::NodeWeight: Clone,
    G::EdgeWeight: Clone + PartialOrd,
{
    for e in min_spanning_tree(g) {
        std::hint::black_box(e);
    }
}
fn iterate_mst_prim<G>(g: G)
where
    G: Data + IntoEdges + IntoNodeReferences + IntoEdgeReferences + NodeIndexable,
    G::NodeWeight: Clone,
    G::EdgeWeight: Clone + PartialOrd,
{
    for e in min_spanning_tree_prim(g) {
        std::hint::black_box(e);
    }
}

/// Parse a file in adjacency matrix format into a directed graph
fn graph_from_graph6_file(path: &str) -> Graph<(), (), Undirected, u32> {
    let mut f = File::open(path).expect("file not found");
    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .expect("failed to read from file");
    Graph::from_graph6_string(contents)
}
