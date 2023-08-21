use criterion::{criterion_group, criterion_main, Criterion};
use petgraph_algorithms::cycles::greedy_feedback_arc_set;

mod common;

fn greedy_tournament(criterion: &mut Criterion) {
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

fn greedy_fan(criterion: &mut Criterion) {
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

criterion_group!(greedy, greedy_tournament, greedy_fan);
criterion_main!(greedy);
