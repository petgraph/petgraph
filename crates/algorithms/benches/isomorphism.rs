mod common;

use criterion::{criterion_group, criterion_main, Criterion};
use petgraph_algorithms::isomorphism::is_isomorphic;

use crate::common::factories::{directed_graph, undirected_graph};

fn directed(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("directed");

    for (id, a, b, expected) in [
        (
            "petersen",
            directed_graph().petersen_a(),
            directed_graph().petersen_b(),
            true,
        ),
        (
            "full",
            directed_graph().full_a(),
            directed_graph().full_b(),
            true,
        ),
        (
            "praust",
            directed_graph().praust_a(),
            directed_graph().praust_b(),
            false,
        ),
    ] {
        group.bench_with_input(id, &(a, b, expected), |bench, (a, b, expected)| {
            bench.iter(|| is_isomorphic(a, b));
            assert_eq!(is_isomorphic(a, b), *expected);
        });
    }
}

fn undirected(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("undirected");

    for (id, a, b, expected) in [
        (
            "petersen",
            undirected_graph().petersen_a(),
            undirected_graph().petersen_b(),
            true,
        ),
        (
            "full",
            undirected_graph().full_a(),
            undirected_graph().full_b(),
            true,
        ),
        (
            "praust",
            undirected_graph().praust_a(),
            undirected_graph().praust_b(),
            false,
        ),
    ] {
        group.bench_with_input(id, &(a, b, expected), |bench, (a, b, expected)| {
            bench.iter(|| is_isomorphic(a, b));
            assert_eq!(is_isomorphic(a, b), *expected);
        });
    }
}

criterion_group!(check, directed, undirected);
criterion_main!(check);
