#![feature(test)]

extern crate petgraph;
extern crate test;

use core::cmp::{max, min};
use petgraph::prelude::*;
use test::Bencher;

mod common;
use petgraph::algo::dijkstra;

#[bench]
#[allow(clippy::needless_range_loop)]
fn dijkstra_bench(bench: &mut Bencher) {
    static NODE_COUNT: usize = 10_000;
    let mut g = Graph::new_undirected();
    let nodes: Vec<NodeIndex<_>> = (0..NODE_COUNT).map(|i| g.add_node(i)).collect();
    for i in 0..NODE_COUNT {
        let n1 = nodes[i];
        let neighbour_count = i % 8 + 3;
        let j_from = max(0, i as i32 - neighbour_count as i32 / 2) as usize;
        let j_to = min(NODE_COUNT, j_from + neighbour_count);
        for j in j_from..j_to {
            let n2 = nodes[j];
            let distance = (i + 3) % 10;
            g.add_edge(n1, n2, distance);
        }
    }

    bench.iter(|| {
        let _scores = dijkstra(&g, nodes[0], None, |e| *e.weight());
    });
}

#[bench]
fn dijkstra_2d_grid_line_bench(bench: &mut Bencher) {
    // Same line grid as ./astar.rs
    const N: usize = 500;
    const M: usize = 5;
    let g = common::build_2d_grid(N, M);

    let start_i = 0;
    let goal_i = N * M - 1;
    let start = NodeIndex::new(start_i);
    let goal = NodeIndex::new(goal_i);

    let costs = dijkstra(&g, start, None, |e| *e.weight());
    assert_eq!(costs[&goal], N + M - 2);

    bench.iter(|| {
        let _ = dijkstra(&g, start, None, |e| *e.weight());
    });
}

#[bench]
fn dijkstra_2d_grid_bench(bench: &mut Bencher) {
    // Same 2D grid as ./astar.rs
    const N: usize = 500;
    let g = common::build_2d_grid(N, N);

    let start_i = 0;
    let goal_i = N * N - 1;
    let start = NodeIndex::new(start_i);
    let goal = NodeIndex::new(goal_i);

    let costs = dijkstra(&g, start, None, |e| *e.weight());
    assert_eq!(costs[&goal], 2 * (N - 1));

    bench.iter(|| {
        let _ = dijkstra(&g, start, None, |e| *e.weight());
    });
}
