#![feature(test)]

extern crate petgraph;
extern crate test;

#[allow(dead_code)]
mod common;
use common::*;

use petgraph::algo::johnson;
use test::Bencher;

#[cfg(feature = "rayon")]
use petgraph::algo::parallel_johnson;

#[bench]
fn johnson_sparse_100_nodes(bench: &mut Bencher) {
    let graph = build_graph(100, false);
    bench.iter(|| {
        let _scores = johnson(&graph, |e| *e.weight());
    });
}

#[bench]
fn johnson_dense_100_nodes(bench: &mut Bencher) {
    let graph = build_graph(100, true);
    bench.iter(|| {
        let _scores = johnson(&graph, |e| *e.weight());
    });
}

#[bench]
#[cfg(feature = "rayon")]
fn parallel_johnson_sparse_100_nodes(bench: &mut Bencher) {
    let graph = build_graph(100, false);

    bench.iter(|| {
        let _scores = parallel_johnson(&graph, |e| *e.weight());
    });
}

#[bench]
#[cfg(feature = "rayon")]
fn parallel_johnson_dense_100_nodes(bench: &mut Bencher) {
    let graph = build_graph(100, false);

    bench.iter(|| {
        let _scores = parallel_johnson(&graph, |e| *e.weight());
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

#[bench]
#[cfg(feature = "rayon")]
fn parallel_johnson_sparse_1000_nodes(bench: &mut Bencher) {
    let graph = build_graph(1000, false);

    bench.iter(|| {
        let _scores = parallel_johnson(&graph, |e| *e.weight());
    });
}

#[bench]
#[cfg(feature = "rayon")]
fn parallel_johnson_dense_1000_nodes(bench: &mut Bencher) {
    let graph = build_graph(1000, true);

    bench.iter(|| {
        let _scores = parallel_johnson(&graph, |e| *e.weight());
    });
}
