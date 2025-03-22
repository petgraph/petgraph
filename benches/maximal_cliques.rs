#![feature(test)]

extern crate petgraph;
extern crate test;

use test::Bencher;

use petgraph::algo::maximal_cliques;

#[allow(dead_code)]
mod common;

use common::directed_fan;

#[bench]
fn maximal_cliques_fan_10_bench(bench: &mut Bencher) {
    let g = directed_fan(10);
    bench.iter(|| maximal_cliques(&g));
}

#[bench]
fn maximal_cliques_fan_50_bench(bench: &mut Bencher) {
    let g = directed_fan(50);
    bench.iter(|| maximal_cliques(&g));
}

#[bench]
fn maximal_cliques_fan_100_bench(bench: &mut Bencher) {
    let g = directed_fan(100);
    bench.iter(|| maximal_cliques(&g));
}

#[bench]
fn maximal_cliques_fan_200_bench(bench: &mut Bencher) {
    let g = directed_fan(200);
    bench.iter(|| maximal_cliques(&g));
}
