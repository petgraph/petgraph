#![feature(test)]
extern crate petgraph;
extern crate test;

use test::Bencher;

use petgraph::algo::page_rank;

#[allow(dead_code)]
mod common;

use common::directed_fan;

#[bench]
fn page_rank_bench(bench: &mut Bencher) {
    static NODE_COUNT: usize = 500;
    let g = directed_fan(NODE_COUNT);
    bench.iter(|| {
        let _ranks = page_rank(&g, 0.6_f64, 10);
    });
}
