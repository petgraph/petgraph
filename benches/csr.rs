#![feature(test)]

extern crate petgraph;
extern crate test;

use test::Bencher;
use petgraph::csr::Csr;

#[allow(dead_code)]
mod common;
use common::*;
use petgraph::visit::IntoNeighbors;

#[bench]
fn add_100_nodes(b: &mut test::Bencher) {
    let k = 100;
    b.iter(|| {
        let mut g = Csr::<(), ()>::new();

        for _ in 0..k {
            let _ = g.add_node(());
        }
    });
}


#[bench]
fn add_100_edges_to_self(b: &mut test::Bencher) {
    let mut g = Csr::<(), ()>::new();
    let nodes: Vec<_> = (0..100).map(|_| g.add_node(())).collect();
    let g = g;

    b.iter(|| {
        let mut g = g.clone();

        for &node in nodes.iter() {
            g.add_edge(node, node, ());
        }
    });
}


#[bench]
fn add_5_edges_for_each_of_100_nodes(b: &mut test::Bencher) {
    let mut g = Csr::<(), ()>::new();
    let nodes: Vec<_> = (0..100).map(|_| g.add_node(())).collect();
    let g = g;

    let edges_to_add: Vec<_> = nodes
        .iter()
        .enumerate()
        .map(|(i, &node)| {
            let edges: Vec<_> = (0..5)
                .map(|j| (i + j + 1) % nodes.len())
                .map(|j| (node, nodes[j]))
                .collect();

            edges
        })
        .flatten()
        .collect();

    b.iter(|| {
        let mut g = g.clone();

        for &(source, target) in edges_to_add.iter() {
            g.add_edge(source, target, ());
        }
    });
}

#[bench]
fn add_edges_from_root(bench: &mut test::Bencher) {
    bench.iter(|| {
        let mut g = Csr::<(), ()>::new();
        let a = g.add_node(());

        for _ in 0..100 {
            let b = g.add_node(());
            g.add_edge(a, b, ());
        }
    });
}


#[bench]
fn add_adjacent_edges(bench: &mut test::Bencher) {
    bench.iter(|| {
        let mut g = Csr::<(), ()>::new();
        let mut prev = None;
        for _ in 0..100 {
            let b = g.add_node(());

            if let Some(a) = prev {
                g.add_edge(a, b, ());
            }

            prev = Some(b);
        }
    });
}

#[bench]
fn full_edges(bench: &mut Bencher) {
    let a = csr_graph().full().graph();
    bench.iter(|| a.edges(1))
}


#[bench]
fn full_edges_count(bench: &mut Bencher) {
    let a = csr_graph().full().graph();
    bench.iter(|| a.edge_count())
}

#[bench]
fn full_neighbors(bench: &mut Bencher) {
    let a = csr_graph().full().graph();
    bench.iter(|| a.neighbors(1))
}

#[bench]
fn bigger_edges(bench: &mut Bencher) {
    let a = csr_graph().bigger().graph();
    bench.iter(|| a.edges(1))
}

#[bench]
fn bigger_edges_count(bench: &mut Bencher) {
    let a = csr_graph().bigger().graph();
    bench.iter(|| a.edge_count())
}

#[bench]
fn bigger_neighbors(bench: &mut Bencher) {
    let a = csr_graph().bigger().graph();
    bench.iter(|| a.neighbors(1))
}