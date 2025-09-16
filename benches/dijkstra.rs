#![feature(test)]

extern crate petgraph;
extern crate test;

#[allow(dead_code)]
mod common;

use common::*;
use petgraph::algo::{bidirectional_dijkstra, dijkstra};
use petgraph::prelude::*;
use test::Bencher;

const RANDOM_GRAPH_SIZE: usize = 10_000;
const RANDOM_GRAPH_START: usize = 1;
const RANDOM_GRAPH_GOAL: usize = 9_000;

#[bench]
fn dijkstra_bench_random(bench: &mut Bencher) {
    let g = build_graph(RANDOM_GRAPH_SIZE, false);

    bench.iter(|| {
        let _scores = dijkstra(&g, NodeIndex::new(RANDOM_GRAPH_START), None, |e| {
            *e.weight()
        });
    });
}

#[bench]
fn dijkstra_bench_random_with_target(bench: &mut Bencher) {
    let g = build_graph(RANDOM_GRAPH_SIZE, false);

    bench.iter(|| {
        let _scores = dijkstra(
            &g,
            NodeIndex::new(RANDOM_GRAPH_START),
            Some(NodeIndex::new(RANDOM_GRAPH_GOAL)),
            |e| *e.weight(),
        );
    });
}

#[bench]
fn dijkstra_bench_random_bidirectional(bench: &mut Bencher) {
    let g = build_graph(RANDOM_GRAPH_SIZE, false);

    bench.iter(|| {
        let _result = bidirectional_dijkstra(
            &g,
            NodeIndex::new(RANDOM_GRAPH_START),
            NodeIndex::new(RANDOM_GRAPH_GOAL),
            |e| *e.weight(),
        );
    });
}

fn build_grid_graph(side: usize) -> Graph<(), usize, Undirected> {
    let mut graph = Graph::new_undirected();

    let mut node_indices = vec![vec![NodeIndex::end(); side]; side];

    for row in 0..side {
        for col in 0..side {
            let node = graph.add_node(());
            node_indices[row][col] = node;
        }
    }

    for row in 0..side {
        for col in 0..side {
            let node = node_indices[row][col];
            if row > 0 {
                graph.add_edge(node, node_indices[row - 1][col], 1);
            }
            if col > 0 {
                graph.add_edge(node, node_indices[row][col - 1], 1);
            }
        }
    }

    graph
}

const GRID_SIDE: usize = 100;
const GRID_START: usize = 50;
const GRID_GOAL: usize = 399;

#[bench]
fn dijkstra_bench_grid(bench: &mut Bencher) {
    let g = build_grid_graph(GRID_SIDE);

    bench.iter(|| {
        let _scores = dijkstra(&g, NodeIndex::new(GRID_START), None, |e| *e.weight());
    });
}

#[bench]
fn dijkstra_bench_grid_with_target(bench: &mut Bencher) {
    let g = build_grid_graph(GRID_SIDE);

    bench.iter(|| {
        let _scores = dijkstra(
            &g,
            NodeIndex::new(GRID_START),
            Some(NodeIndex::new(GRID_GOAL)),
            |e| *e.weight(),
        );
    });
}

#[bench]
fn dijkstra_bench_grid_bidirectional(bench: &mut Bencher) {
    let g = build_grid_graph(GRID_SIDE);

    bench.iter(|| {
        let _result = bidirectional_dijkstra(
            &g,
            NodeIndex::new(GRID_START),
            NodeIndex::new(GRID_GOAL),
            |e| *e.weight(),
        );
    });
}
