#![feature(test)]

extern crate petgraph;
extern crate test;

use std::{fs::File, hash::Hash, io::Read};

use test::Bencher;

#[allow(dead_code)]
mod common;
use common::ungraph;

use petgraph::{
    algo::connectivity::{CutEdgesSearch, CutVerticesSearch},
    graph6::FromGraph6,
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
    let g = graph_from_graph6_file("tests/res/graph_2000n.g6");
    bench.iter(|| iterate_cut_edges_search(&g));
}

#[bench]
fn cut_vertices_search_praust(bench: &mut Bencher) {
    let a = ungraph().praust_a();
    let b = ungraph().praust_b();

    bench.iter(|| {
        (
            iterate_cut_vertices_search(&a),
            iterate_cut_edges_search(&b),
        )
    });
}

#[bench]
fn cut_vertices_search_full(bench: &mut Bencher) {
    let a = ungraph().full_a();
    let b = ungraph().full_b();

    bench.iter(|| {
        (
            iterate_cut_vertices_search(&a),
            iterate_cut_vertices_search(&b),
        )
    });
}

#[bench]
fn cut_vertices_search_petersen(bench: &mut Bencher) {
    let a: Graph<(), (), Undirected> = ungraph().petersen_a();
    let b = ungraph().petersen_b();

    bench.iter(|| {
        (
            iterate_cut_vertices_search(&a),
            iterate_cut_vertices_search(&b),
        )
    });
}

#[bench]
fn cut_vertices_search_2000n(bench: &mut Bencher) {
    let g = graph_from_graph6_file("tests/res/graph_2000n.g6");
    bench.iter(|| iterate_cut_vertices_search(&g));
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

fn iterate_cut_vertices_search<G, N>(g: G)
where
    N: Hash + Eq + Copy,
    G: IntoNeighbors<NodeId = N> + IntoNodeIdentifiers,
{
    let mut cut_vertices_search = CutVerticesSearch::new(g);

    while let Some(node) = cut_vertices_search.next(g) {
        std::hint::black_box(node);
    }
}

/// Parse a file in graph6 format into an undirected graph
fn graph_from_graph6_file(path: &str) -> Graph<(), (), Undirected, u32> {
    let mut f = File::open(path).expect("file not found");
    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .expect("failed to read from file");
    Graph::from_graph6_string(contents)
}
