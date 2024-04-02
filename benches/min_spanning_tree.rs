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
fn min_spanning_tree_praust_undir_bench(bench: &mut Bencher) {
    let a = ungraph().praust_a();
    let b = ungraph().praust_b();

    bench.iter(|| (min_spanning_tree(&a), min_spanning_tree(&b)));
}

#[bench]
fn min_spanning_tree_praust_dir_bench(bench: &mut Bencher) {
    let a = digraph().praust_a();
    let b = digraph().praust_b();

    bench.iter(|| (min_spanning_tree(&a), min_spanning_tree(&b)));
}

#[bench]
fn min_spanning_tree_full_undir_bench(bench: &mut Bencher) {
    let a = ungraph().full_a();
    let b = ungraph().full_b();

    bench.iter(|| (min_spanning_tree(&a), min_spanning_tree(&b)));
}

#[bench]
fn min_spanning_tree_full_dir_bench(bench: &mut Bencher) {
    let a = digraph().full_a();
    let b = digraph().full_b();

    bench.iter(|| (min_spanning_tree(&a), min_spanning_tree(&b)));
}

#[bench]
fn min_spanning_tree_petersen_undir_bench(bench: &mut Bencher) {
    let a = ungraph().petersen_a();
    let b = ungraph().petersen_b();

    bench.iter(|| (min_spanning_tree(&a), min_spanning_tree(&b)));
}

#[bench]
fn min_spanning_tree_petersen_dir_bench(bench: &mut Bencher) {
    let a = digraph().petersen_a();
    let b = digraph().petersen_b();

    bench.iter(|| (min_spanning_tree(&a), min_spanning_tree(&b)));
}

#[bench]
fn min_spanning_tree_prim_praust_undir_bench(bench: &mut Bencher) {
    let a = ungraph().praust_a();
    let b = ungraph().praust_b();

    bench.iter(|| (iterate_mst(&a), iterate_mst(&b)));
}

#[bench]
fn min_spanning_tree_prim_full_undir_bench(bench: &mut Bencher) {
    let a = ungraph().full_a();
    let b = ungraph().full_b();

    bench.iter(|| (iterate_mst(&a), iterate_mst(&b)));
}

#[bench]
fn min_spanning_tree_prim_petersen_undir_bench(bench: &mut Bencher) {
    let a: Graph<(), (), Undirected> = ungraph().petersen_a();
    let b = ungraph().petersen_b();

    bench.iter(|| (iterate_mst(&a), iterate_mst(&b)));
}

fn iterate_mst<G>(g: G) -> bool
where
    G: Data + IntoEdges + IntoNodeReferences + IntoEdgeReferences + NodeIndexable,
    G::NodeWeight: Clone,
    G::EdgeWeight: Clone + PartialOrd,
{
    let mst = min_spanning_tree_prim(g);
    mst.into_iter().all(|_| true)
}
