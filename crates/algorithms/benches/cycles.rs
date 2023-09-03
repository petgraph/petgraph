use criterion::{criterion_group, criterion_main, Criterion};
use petgraph_algorithms::cycles::{
    greedy_feedback_arc_set, is_cyclic_directed, is_cyclic_undirected,
};

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

fn greedy_feedback_arc_set_tournament(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("tournament");

    for nodes in common::nodes(Some(256)) {
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(*nodes),
            nodes,
            |bench, &nodes| {
                bench.iter_batched(
                    || common::tournament::<(), ()>(nodes, None),
                    |graph| {
                        let _scores = greedy_feedback_arc_set(&graph);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
}

fn greedy_feedback_arc_set_fan(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("fan");

    for nodes in common::nodes(Some(1024)) {
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(*nodes),
            nodes,
            |bench, &nodes| {
                bench.iter_batched(
                    || common::directed_fan::<(), ()>(nodes, None),
                    |graph| {
                        let _scores = greedy_feedback_arc_set(&graph);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
}

criterion_group!(is_cyclic_benches, cyclic_directed, cyclic_undirected);
criterion_group!(
    greedy_feedback_arc_set_benches,
    greedy_feedback_arc_set_tournament,
    greedy_feedback_arc_set_fan
);

criterion_main!(is_cyclic_benches, greedy_feedback_arc_set_benches);
