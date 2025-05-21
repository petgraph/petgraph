#![feature(test)]

extern crate petgraph;
extern crate test;

#[allow(dead_code)]
mod common;
use common::*;

use test::Bencher;

use petgraph::algo::floyd_warshall;

#[bench]
fn floyd_warshall_sparse_100_nodes(bench: &mut Bencher) {
    let graph = build_graph(100, false);

    bench.iter(|| {
        let _scores = floyd_warshall(&graph, |e| *e.weight());
    });
}

#[bench]
fn floyd_warshall_dense_100_nodes(bench: &mut Bencher) {
    let graph = build_graph(100, true);

    bench.iter(|| {
        let _scores = floyd_warshall(&graph, |e| *e.weight());
    });
}

#[bench]
fn floyd_warshall_sparse_1000_nodes(bench: &mut Bencher) {
    let graph = build_graph(1000, false);

    bench.iter(|| {
        let _scores = floyd_warshall(&graph, |e| *e.weight());
    });
}

#[bench]
fn floyd_warshall_dense_1000_nodes(bench: &mut Bencher) {
    let graph = build_graph(1000, true);

    bench.iter(|| {
        let _scores = floyd_warshall(&graph, |e| *e.weight());
    });
}
