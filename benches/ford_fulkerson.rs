#![feature(test)]
extern crate petgraph;
extern crate test;

use petgraph::algo::ford_fulkerson;
use petgraph::prelude::{Graph, NodeIndex};
use test::Bencher;

#[bench]
fn ford_fulkerson_bench(bench: &mut Bencher) {
    static NODE_COUNT: usize = 1_000;
    let mut g: Graph<usize, usize> = Graph::new();
    let nodes: Vec<NodeIndex<_>> = (0..NODE_COUNT).map(|i| g.add_node(i)).collect();
    for i in 0..NODE_COUNT - 1 {
        g.add_edge(nodes[i], nodes[i + 1], 1);
    }
    bench.iter(|| {
        let _flow = ford_fulkerson(
            &g,
            NodeIndex::from(0),
            NodeIndex::from(g.node_count() as u32 - 1),
        );
    });
}

#[bench]
fn ford_fulkerson_bench_many_edges(bench: &mut Bencher) {
    static NODE_COUNT: usize = 1_001;
    let mut g: Graph<usize, usize> = Graph::new();
    let nodes: Vec<NodeIndex<_>> = (0..NODE_COUNT).map(|i| g.add_node(i)).collect();
    for j in [1, 2, 4, 5, 10, 20, 25, 50] {
        for i in 0..(NODE_COUNT - 1) / j {
            g.add_edge(nodes[i], nodes[(i + 1) * j], 1);
        }
    }
    bench.iter(|| {
        let _flow = ford_fulkerson(
            &g,
            NodeIndex::from(0),
            NodeIndex::from(g.node_count() as u32 - 1),
        );
    });
}
