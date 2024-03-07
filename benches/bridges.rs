#![feature(test)]

extern crate petgraph;
extern crate test;

use petgraph::prelude::*;
use test::Bencher;

use petgraph::algo::bridges;

#[bench]
fn bridges_bench(bench: &mut Bencher) {
    static NODE_COUNT: usize = 1000;
    let mut g = Graph::new_undirected();
    let nodes: Vec<NodeIndex<_>> = (0..NODE_COUNT).into_iter().map(|i| g.add_node(i)).collect();
    for i in 0..NODE_COUNT {
        let n1 = nodes[i];
        let neighbour_count = i % 8 + 1;

        for j in (i % 117)..(i % 117) + neighbour_count {
            let n2 = nodes[j];
            g.add_edge(n1, n2, ());
        }
    }

    bench.iter(|| bridges(&g).collect::<Vec<_>>());
}
