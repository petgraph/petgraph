#![feature(test)]
extern crate petgraph;
extern crate test;

use petgraph::{algo::label_propagation, Graph};
use test::Bencher;

#[bench]
fn label_propagation_bench(bench: &mut Bencher) {
    let mut g = Graph::<Option<&str>, ()>::with_capacity(10000, 1000);
    let labels = [Some("A"), Some("B"), Some("C"), Some("D")];
    let n = labels.len();
    for i in 0..1000 {
        let _ = g.add_node(labels[i % n]);
    }
    for _ in 0..9000 {
        let _ = g.add_node(None);
    }
    for i in 0..1000 {
        let _ = g.add_edge(i.into(), (10000 - i - 1).into(), ());
    }
    bench.iter(|| {
        let _labels = label_propagation(&g, &labels, 2, 10);
    });
}
