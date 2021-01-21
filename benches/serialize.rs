#![feature(test)]

extern crate petgraph;
extern crate test;

use test::Bencher;
use petgraph::prelude::*;

fn make_stable_graph() -> StableGraph<usize, usize> {
    let mut g = StableGraph::default();
    let indices: Vec<_> = (0 .. 10000).map(|i| g.add_node(i)).collect();
    for i in 0 .. 1000 {
        g.extend_with_edges((1 .. 1000).map(|j| (indices[j], indices[(j + i) % 10000], i)));
    }

    // Remove nodes to make the structure a bit more interesting
    for i in (0 .. 10000).step_by(2) {
        g.remove_node(indices[i]);
    }
    g
}


#[bench]
fn serialize_bench(bench: &mut Bencher) {
    let graph = make_stable_graph();
    bench.iter(|| {
        bincode::serialize(&graph).unwrap()
    });
}

#[bench]
fn deserialize_bench(bench: &mut Bencher) {
    let graph = make_stable_graph();
    let data = bincode::serialize(&graph).unwrap();
    bench.iter(|| {
        let graph2: StableGraph<usize, usize> = bincode::deserialize(&data).unwrap();
        graph2
    });
}
