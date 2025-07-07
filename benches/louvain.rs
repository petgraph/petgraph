#![feature(test)]

extern crate petgraph;
extern crate test;

use petgraph::algo::community::louvain_communities;
use test::Bencher;

#[allow(dead_code)]
mod common;
use common::directed_fan;

use crate::common::build_graph;

#[bench]
fn louvain_fan_010_bench(bench: &mut Bencher) {
    let g = build_graph(10, true);
    bench.iter(|| louvain_communities(&g, 1.0, 0.001, None, None));
}

#[bench]
fn louvain_fan_050_bench(bench: &mut Bencher) {
    let g = build_graph(50, true);
    bench.iter(|| louvain_communities(&g, 1.0, 0.001, None, None));
}

#[bench]
fn louvain_fan_100_bench(bench: &mut Bencher) {
    let g = build_graph(100, true);
    bench.iter(|| louvain_communities(&g, 1.0, 0.001, None, None));
}

#[bench]
fn louvain_fan_200_bench(bench: &mut Bencher) {
    let g = build_graph(200, true);
    bench.iter(|| louvain_communities(&g, 1.0, 0.001, None, None));
}