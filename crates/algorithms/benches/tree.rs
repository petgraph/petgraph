mod common;

use criterion::{criterion_group, criterion_main, Criterion};
use petgraph_algorithms::tree::min_spanning_tree;

use crate::common::factories::directed_graph;

fn min_spanning_tree_directed(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("min_spanning/tree_directed");

    for (id, graph) in [
        ("praust A", directed_graph().praust_a()),
        ("praust B", directed_graph().praust_b()),
        ("full A", directed_graph().full_a()),
        ("full B", directed_graph().full_b()),
        ("petersen A", directed_graph().petersen_a()),
        ("petersen B", directed_graph().petersen_b()),
    ] {
        group.bench_with_input(id, &graph, |bench, graph| {
            bench.iter(|| min_spanning_tree(graph));
        });
    }
}

fn min_spanning_tree_undirected(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("min_spanning_tree/undirected");

    for (id, graph) in [
        ("praust A", directed_graph().praust_a()),
        ("praust B", directed_graph().praust_b()),
        ("full A", directed_graph().full_a()),
        ("full B", directed_graph().full_b()),
        ("petersen A", directed_graph().petersen_a()),
        ("petersen B", directed_graph().petersen_b()),
    ] {
        group.bench_with_input(id, &graph, |bench, graph| {
            bench.iter(|| min_spanning_tree(graph));
        });
    }
}

criterion_group!(
    min_spanning,
    min_spanning_tree_directed,
    min_spanning_tree_undirected
);

criterion_main!(min_spanning);
