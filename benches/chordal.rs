#![feature(test)]

extern crate petgraph;
extern crate test;

#[allow(dead_code)]
mod common;
use common::*;

use petgraph::algo::johnson;
use test::Bencher;

#[bench]
fn is_chordal_sparse_100_nodes(bench: &mut Bencher) {
    let graph = build_graph(100, false);
    bench.iter(|| {
        let _scores = johnson(&graph, |e| *e.weight());
    });
}

#[bench]
fn is_chordal_dense_100_nodes(bench: &mut Bencher) {
    let graph = build_graph(100, true);
    bench.iter(|| {
        let _scores = johnson(&graph, |e| *e.weight());
    });
}

#[bench]
fn is_chordal_sparse_500_nodes(bench: &mut Bencher) {
    let graph = build_graph(500, false);
    bench.iter(|| {
        let _scores = johnson(&graph, |e| *e.weight());
    });
}

#[bench]
fn is_chordal_dense_500_nodes(bench: &mut Bencher) {
    let graph = build_graph(500, true);
    bench.iter(|| {
        let _scores = johnson(&graph, |e| *e.weight());
    });
}
