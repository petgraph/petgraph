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

#[bench]
fn ford_fulkerson_bench_wide(bench: &mut Bencher) {
    let node_count = 1000;
    let mut g: Graph<usize, usize> = Graph::new();
    let source = g.add_node(0);
    let sink = g.add_node(1);

    let mut intermediates = Vec::new();
    for i in 0..(node_count - 2) {
        let n = g.add_node(i + 2);
        intermediates.push(n);
        g.add_edge(source, n, 1);
        g.add_edge(n, sink, 1);
    }

    bench.iter(|| {
        let _flow = ford_fulkerson(&g, source, sink);
    });
}

#[bench]
fn ford_fulkerson_bench_dense_middle(bench: &mut Bencher) {
    let node_count = 500;
    let mut g: Graph<usize, usize> = Graph::new();
    let source = g.add_node(0);
    let sink = g.add_node(1);

    let mut intermediates = Vec::new();
    for i in 0..(node_count - 2) {
        let node = g.add_node(i + 2);
        intermediates.push(node);
    }

    for (i, &node) in (&intermediates).iter().enumerate() {
        if i % 7 == 0 {
            g.add_edge(source, node, 1);
        }
    }

    for (i, &node) in (&intermediates).iter().enumerate() {
        if i % 11 == 0 {
            g.add_edge(node, sink, 1);
        }
    }

    for i in 0..intermediates.len() {
        for j in (i + 1)..intermediates.len() {
            if (i + j) % 13 == 0 {
                g.add_edge(intermediates[i], intermediates[j], 1);
            }
        }
    }

    bench.iter(|| {
        let _flow = ford_fulkerson(&g, source, sink);
    });
}

#[bench]
fn ford_fulkerson_bench_dense_middle_varying_weights(bench: &mut Bencher) {
    let node_count = 500;
    let mut g: Graph<usize, usize> = Graph::new();
    let source = g.add_node(0);
    let sink = g.add_node(1);

    let mut intermediates = Vec::new();
    for i in 0..(node_count - 2) {
        let node = g.add_node(i + 2);
        intermediates.push(node);
    }

    for (i, &node) in (&intermediates).iter().enumerate() {
        if i % 7 == 0 {
            g.add_edge(source, node, 1 + (i % 11));
        }
    }

    for (i, &node) in (&intermediates).iter().enumerate() {
        if i % 11 == 0 {
            g.add_edge(node, sink, 11 - (i % 11));
        }
    }

    for i in 0..intermediates.len() {
        for j in (i + 1)..intermediates.len() {
            if (i + j) % 13 == 0 {
                g.add_edge(intermediates[i], intermediates[j], (i + j) % 13 + 1);
            }
        }
    }

    bench.iter(|| {
        let _flow = ford_fulkerson(&g, source, sink);
    });
}
