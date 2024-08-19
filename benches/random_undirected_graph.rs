#![feature(test)]

extern crate petgraph;
extern crate test;

use petgraph::generators::random_undirected_graph;
use test::Bencher;

#[bench]
fn bench_random_undirected_graph_n10_p100(bench: &mut Bencher) {
    bench_random_undirected_graph(bench, 10, 1.);
}

#[bench]
fn bench_random_undirected_graph_n100_p100(bench: &mut Bencher) {
    bench_random_undirected_graph(bench, 100, 1.);
}

#[bench]
fn bench_random_undirected_graph_n1000_p0(bench: &mut Bencher) {
    bench_random_undirected_graph(bench, 1000, 0.);
}

#[bench]
fn bench_random_undirected_graph_n1000_p30(bench: &mut Bencher) {
    bench_random_undirected_graph(bench, 1000, 0.3);
}

#[bench]
fn bench_random_undirected_graph_n1000_p50(bench: &mut Bencher) {
    bench_random_undirected_graph(bench, 1000, 0.5);
}

#[bench]
fn bench_random_undirected_graph_n10000_p20(bench: &mut Bencher) {
    bench_random_undirected_graph(bench, 1000, 0.2);
}

#[bench]
fn bench_random_undirected_graph_n10000_p0(bench: &mut Bencher) {
    bench_random_undirected_graph(bench, 1000, 0.);
}

#[bench]
fn bench_random_undirected_graph_n10000_p100(bench: &mut Bencher) {
    bench_random_undirected_graph(bench, 1000, 1.);
}

fn bench_random_undirected_graph(bench: &mut Bencher, n: usize, p: f64) {
    bench.iter(|| random_undirected_graph::<u32>(n, p));
}
