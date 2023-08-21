use criterion::{criterion_group, criterion_main, Criterion};
use petgraph_algorithms::cycles::{is_cyclic_directed, is_cyclic_undirected};

use crate::common::factories::{directed_graph, undirected_graph};

mod common;

fn cyclic_directed(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("is_cyclic/directed");

    for (id, graph) in [
        ("praust A", directed_graph().praust_a()),
        ("praust B", directed_graph().praust_b()),
        ("full A", directed_graph().full_a()),
        ("full B", directed_graph().full_b()),
        ("petersen A", directed_graph().petersen_a()),
        ("petersen B", directed_graph().petersen_b()),
    ] {
        group.bench_with_input(id, &graph, |bench, graph| {
            bench.iter(|| is_cyclic_directed(graph));
        });
    }
}

fn cyclic_undirected(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("is_cyclic/undirected");

    for (id, graph) in [
        ("praust A", undirected_graph().praust_a()),
        ("praust B", undirected_graph().praust_b()),
        ("full A", undirected_graph().full_a()),
        ("full B", undirected_graph().full_b()),
        ("petersen A", undirected_graph().petersen_a()),
        ("petersen B", undirected_graph().petersen_b()),
    ] {
        group.bench_with_input(id, &graph, |bench, graph| {
            bench.iter(|| is_cyclic_undirected(graph));
        });
    }
}

criterion_group!(cyclic, cyclic_directed, cyclic_undirected);
criterion_main!(cyclic);
