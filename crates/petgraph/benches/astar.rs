#![feature(test)]

extern crate petgraph;
extern crate test;

use petgraph::prelude::*;
use test::Bencher;

mod common;
use petgraph::algo::astar;

#[bench]
fn astar_2d_grid_line_bench(bench: &mut Bencher) {
    // Same line grid as ./dijkstra.rs
    const N: usize = 500;
    const M: usize = 5;
    let g = common::build_2d_grid(N, M);

    let heuristic_for = |goal: NodeIndex| {
        let g = &g;
        move |node: NodeIndex| -> usize {
            let (x1, y1): (usize, usize) = g[node];
            let (x2, y2): (usize, usize) = g[goal];

            x2.abs_diff(x1) + y2.abs_diff(y1)
        }
    };

    let start_i = 0;
    let goal_i = N * M - 1;
    let start = NodeIndex::new(start_i);
    let goal = NodeIndex::new(goal_i);

    if let Some((cost, path)) = astar(
        &g,
        start,
        |n| n == goal,
        |e| *e.weight(),
        heuristic_for(goal),
    ) {
        assert_eq!(cost, N + M - 2);

        assert_eq!(path.first().unwrap().index(), start_i);
        assert_eq!(path.last().unwrap().index(), goal_i);
        assert_eq!(path.len(), cost + 1);
    }

    bench.iter(|| {
        let _ = astar(
            &g,
            start,
            |n| n == goal,
            |e| *e.weight(),
            heuristic_for(goal),
        );
    });
}

#[bench]
fn astar_2d_grid_bench(bench: &mut Bencher) {
    // Same 2D grid as ./dijkstra.rs
    const N: usize = 500;
    let g = common::build_2d_grid(N, N);

    let heuristic_for = |goal: NodeIndex| {
        let g = &g;
        move |node: NodeIndex| -> usize {
            let (x1, y1): (usize, usize) = g[node];
            let (x2, y2): (usize, usize) = g[goal];

            x2.abs_diff(x1) + y2.abs_diff(y1)
        }
    };

    let start_i = 0;
    let goal_i = N * N - 1;
    let start = NodeIndex::new(start_i);
    let goal = NodeIndex::new(goal_i);

    if let Some((cost, path)) = astar(
        &g,
        start,
        |n| n == goal,
        |e| *e.weight(),
        heuristic_for(goal),
    ) {
        assert_eq!(cost, 2 * (N - 1));

        assert_eq!(path.first().unwrap().index(), start_i);
        assert_eq!(path.last().unwrap().index(), goal_i);
        assert_eq!(path.len(), cost + 1);
    }

    bench.iter(|| {
        let _ = astar(
            &g,
            start,
            |n| n == goal,
            |e| *e.weight(),
            heuristic_for(goal),
        );
    });
}
