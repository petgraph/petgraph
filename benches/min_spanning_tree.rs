#![feature(test)]

extern crate petgraph;
extern crate test;

use test::Bencher;

#[allow(dead_code)]
mod common;
use common::{digraph, ungraph};

use petgraph::{
    algo::{min_spanning_tree, min_spanning_tree_prim},
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
