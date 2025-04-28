extern crate petgraph;
extern crate test;

use core::hash::Hash;

use test::Bencher;

#[allow(dead_code)]
#[path = "../common/mod.rs"]
mod common;
use common::{ungraph, ungraph_from_graph6_file};

use petgraph::{
    algo::connectivity::CutEdgesSearch,
    visit::{IntoNeighbors, IntoNodeIdentifiers},
    Graph, Undirected,
};

#[bench]
fn cut_edges_search_praust(bench: &mut Bencher) {
    let a = ungraph().praust_a();
    let b = ungraph().praust_b();

    bench.iter(|| (iterate_cut_edges_search(&a), iterate_cut_edges_search(&b)));
}

#[bench]
fn cut_edges_search_full(bench: &mut Bencher) {
    let a = ungraph().full_a();
    let b = ungraph().full_b();

    bench.iter(|| (iterate_cut_edges_search(&a), iterate_cut_edges_search(&b)));
}

#[bench]
fn cut_edges_search_petersen(bench: &mut Bencher) {
    let a: Graph<(), (), Undirected> = ungraph().petersen_a();
    let b = ungraph().petersen_b();

    bench.iter(|| (iterate_cut_edges_search(&a), iterate_cut_edges_search(&b)));
}

#[bench]
fn cut_edges_search_2000n(bench: &mut Bencher) {
    let g = ungraph_from_graph6_file("tests/res/graph_2000n.g6");
    bench.iter(|| iterate_cut_edges_search(&g));
}

fn iterate_cut_edges_search<G, N>(g: G)
where
    N: Hash + Eq + Copy,
    G: IntoNeighbors<NodeId = N> + IntoNodeIdentifiers,
{
    let mut cut_edges_search = CutEdgesSearch::new(g);

    while let Some(edge) = cut_edges_search.next(g) {
        std::hint::black_box(edge);
    }
}
