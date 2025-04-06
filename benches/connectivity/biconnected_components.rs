extern crate petgraph;
extern crate test;

use core::hash::Hash;

use test::Bencher;

use petgraph::{
    algo::connectivity::BiconnectedComponentsSearch,
    visit::{IntoNeighbors, IntoNodeIdentifiers},
    Graph, Undirected,
};

#[allow(dead_code)]
#[path = "../common/mod.rs"]
mod common;
use common::{ungraph, ungraph_from_graph6_file};

#[bench]
fn biconnected_components_search_praust(bench: &mut Bencher) {
    let a = ungraph().praust_a();
    let b = ungraph().praust_b();

    bench.iter(|| {
        (
            iterate_biconnected_components_search(&a),
            iterate_biconnected_components_search(&b),
        )
    });
}

#[bench]
fn biconnected_components_search_full(bench: &mut Bencher) {
    let a = ungraph().full_a();
    let b = ungraph().full_b();

    bench.iter(|| {
        (
            iterate_biconnected_components_search(&a),
            iterate_biconnected_components_search(&b),
        )
    });
}

#[bench]
fn biconnected_components_search_petersen(bench: &mut Bencher) {
    let a: Graph<(), (), Undirected> = ungraph().petersen_a();
    let b = ungraph().petersen_b();

    bench.iter(|| {
        (
            iterate_biconnected_components_search(&a),
            iterate_biconnected_components_search(&b),
        )
    });
}

#[bench]
fn biconnected_components_search_2000n(bench: &mut Bencher) {
    let g = ungraph_from_graph6_file("tests/res/graph_2000n.g6");
    bench.iter(|| iterate_biconnected_components_search(&g));
}

fn iterate_biconnected_components_search<G, N>(g: G)
where
    N: Hash + Eq + Copy,
    G: IntoNeighbors<NodeId = N> + IntoNodeIdentifiers,
{
    let mut biconnected_components_search = BiconnectedComponentsSearch::new(g);

    while let Some(node) = biconnected_components_search.next(g) {
        std::hint::black_box(node);
    }
}
