mod common;

use criterion::{criterion_group, criterion_main, Criterion};
use petgraph_algorithms::heuristics::{greedy_matching, maximum_matching};
use petgraph_graph::UnGraph;

use crate::common::factories::undirected_graph;

fn huge() -> UnGraph<(), ()> {
    static NODE_COUNT: u32 = 1_000;

    let mut edges = Vec::new();

    for i in 0..NODE_COUNT {
        for j in i..NODE_COUNT {
            if i % 3 == 0 && j % 2 == 0 {
                edges.push((i, j));
            }
        }
    }

    // 999 nodes, 83500 edges
    UnGraph::from_edges(&edges)
}

fn greedy(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("greedy");

    for (id, graph) in [
        ("bipartite", undirected_graph().bipartite()),
        ("full", undirected_graph().full_a()),
        ("bigger", undirected_graph().bigger()),
        ("huge", huge()),
    ] {
        group.bench_with_input(id, &graph, |bench, graph| {
            bench.iter(|| {
                let _scores = greedy_matching(graph);
            });
        });
    }
}

fn maximum(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("maximum");

    for (id, graph) in [
        ("bipartite", undirected_graph().bipartite()),
        ("full", undirected_graph().full_a()),
        ("bigger", undirected_graph().bigger()),
        ("huge", huge()),
    ] {
        group.bench_with_input(id, &graph, |bench, graph| {
            bench.iter(|| {
                let _scores = maximum_matching(graph);
            });
        });
    }
}

criterion_group!(matching, greedy, maximum);
criterion_main!(matching);
