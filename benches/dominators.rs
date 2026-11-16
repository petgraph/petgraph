#![feature(test)]

extern crate test;

use petgraph::algo::dominators::simple_fast;
use petgraph::prelude::*;
use test::{black_box, Bencher};

fn graph(nodes: usize, width: usize, cyclic: bool) -> DiGraph<(), ()> {
    let mut graph = DiGraph::new();
    let nodes: Vec<_> = (0..nodes).map(|_| graph.add_node(())).collect();
    for i in 1..nodes.len() {
        graph.add_edge(nodes[i - 1], nodes[i], ());
        for distance in 2..=width.min(i) {
            graph.add_edge(nodes[i - distance], nodes[i], ());
        }
        if cyclic && i % 8 == 0 {
            graph.add_edge(nodes[i], nodes[i - 7], ());
        }
    }
    graph
}

#[bench]
fn dominators_chain(bench: &mut Bencher) {
    let graph = graph(2048, 1, false);
    bench.iter(|| black_box(simple_fast(&graph, NodeIndex::new(0))));
}

#[bench]
fn dominators_joins(bench: &mut Bencher) {
    let graph = graph(1024, 8, false);
    bench.iter(|| black_box(simple_fast(&graph, NodeIndex::new(0))));
}

#[bench]
fn dominators_cycles(bench: &mut Bencher) {
    let graph = graph(1024, 4, true);
    bench.iter(|| black_box(simple_fast(&graph, NodeIndex::new(0))));
}
