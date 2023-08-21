mod common;

use criterion::{criterion_group, criterion_main, Criterion};
use petgraph_algorithms::components::connected_components;

use crate::common::factories::undirected_graph;

fn connected_directed(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("connected/directed");

    for (id, graph) in [
        ("praust A", undirected_graph().praust_a()),
        ("praust B", undirected_graph().praust_b()),
        ("full A", undirected_graph().full_a()),
        ("full B", undirected_graph().full_b()),
        ("petersen A", undirected_graph().petersen_a()),
        ("petersen B", undirected_graph().petersen_b()),
    ] {
        group.bench_with_input(id, &graph, |bench, graph| {
            bench.iter(|| connected_components(graph));
        });
    }
}

fn connected_undirected(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("connected/undirected");

    for (id, graph) in [
        ("praust A", undirected_graph().praust_a()),
        ("praust B", undirected_graph().praust_b()),
        ("full A", undirected_graph().full_a()),
        ("full B", undirected_graph().full_b()),
        ("petersen A", undirected_graph().petersen_a()),
        ("petersen B", undirected_graph().petersen_b()),
    ] {
        group.bench_with_input(id, &graph, |bench, graph| {
            bench.iter(|| connected_components(graph));
        });
    }
}

criterion_group!(connected, connected_directed, connected_undirected);
criterion_main!(connected);
