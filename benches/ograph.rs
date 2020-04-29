#![feature(test)]

extern crate petgraph;
extern crate test;

use petgraph::graph::IndexType;
use petgraph::prelude::*;
use petgraph::visit::{EdgeRef};
use petgraph::EdgeType;

use petgraph::graph::Graph;
mod common;

#[bench]
fn bench_insert(b: &mut test::Bencher) {
    let mut og = Graph::new();
    let fst = og.add_node(0i32);
    for x in 1..125 {
        let n = og.add_node(x);
        og.add_edge(fst, n, ());
    }
    b.iter(|| og.add_node(1))
}

#[bench]
fn bench_add_edge(b: &mut test::Bencher) {
    let mut og = Graph::new();
    for _ in 0..100 {
        og.add_node(());
    }

    b.iter(|| {
        for (a, b) in og.node_indices().zip(og.node_indices().skip(1)) {
            og.add_edge(a, b, ());
        }
        og.clear_edges();
    })
}

#[bench]
fn bench_remove(b: &mut test::Bencher) {
    // removal is very slow in a big graph.
    // and this one doesn't even have many nodes.
    let mut og = Graph::new();
    let fst = og.add_node(0i32);
    let mut prev = fst;
    for x in 1..1250 {
        let n = og.add_node(x);
        og.add_edge(prev, n, ());
        prev = n;
    }
    //println!("{}", og);
    b.iter(|| {
        for _ in 0..100 {
            og.remove_node(fst);
        }
    })
}

#[bench]
fn bigger_edges_directed(b: &mut test::Bencher) {
    let og: Graph<_, _, petgraph::Directed> = common::graph().bigger();
    bench_edges_directed(b, og);
}

#[bench]
fn bigger_edges_undirected(b: &mut test::Bencher) {
    let og: Graph<_, _, petgraph::Directed> = common::graph().bigger();
    bench_edges_undirected(b, og);
}

#[bench]
fn full_edges_directed(b: &mut test::Bencher) {
    let og: Graph<_, _, petgraph::Directed> = common::graph().full_a();
    bench_edges_directed(b, og);
}

#[bench]
fn full_edges_undirected(b: &mut test::Bencher) {
    let og: Graph<_, _, petgraph::Directed> = common::graph().full_a();
    bench_edges_undirected(b, og);
}

#[bench]
fn praust_edges_directed(b: &mut test::Bencher) {
    let og: Graph<_, _, petgraph::Directed> = common::graph().praust_a();
    bench_edges_directed(b, og);
}

#[bench]
fn praust_edges_undirected(b: &mut test::Bencher) {
    let og: Graph<_, _, petgraph::Directed> = common::graph().praust_a();
    bench_edges_undirected(b, og);
}

#[bench]
fn petersen_edges_directed(b: &mut test::Bencher) {
    let og: Graph<_, _, petgraph::Directed> = common::graph().petersen_a();
    bench_edges_directed(b, og);
}

#[bench]
fn petersen_edges_undirected(b: &mut test::Bencher) {
    let og: Graph<_, _, petgraph::Directed> = common::graph().petersen_a();
    bench_edges_undirected(b, og);
}

fn bench_edges_directed<N, E, Ty, Ix>(b: &mut test::Bencher, g: Graph<N, E, Ty, Ix>)
where
    Ty: EdgeType,
    Ix: IndexType,
{
    b.iter(|| {
        for x in g.edges_directed(NodeIndex::new(0), Outgoing) {
            let _y = x.target();
        }
        for x in g.edges_directed(NodeIndex::new(0), Incoming) {
            let _y = x.target();
        }
    })
}

fn bench_edges_undirected<N, E, Ty, Ix>(b: &mut test::Bencher, g: Graph<N, E, Ty, Ix>)
where
    Ty: EdgeType,
    Ix: IndexType,
{
    b.iter(|| {
        for x in g.edges_undirected(NodeIndex::new(0)) {
            let _y = x.target();
        }
    })
}
