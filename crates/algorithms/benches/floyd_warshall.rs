mod common;

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use petgraph_algorithms::shortest_paths::floyd_warshall;

use crate::common::{nodes, Profile};

fn sparse(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("floyd_warshall/sparse");

    for nodes in nodes(Some(256)) {
        group.bench_with_input(
            BenchmarkId::from_parameter(*nodes),
            nodes,
            |bench, &nodes| {
                bench.iter_batched(
                    || Profile::Sparse.newman_watts_strogatz::<(), u8>(nodes),
                    |graph| {
                        let _scores = floyd_warshall(&graph, |edge| *edge.weight());
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
}

fn dense(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("floyd_warshall/dense");

    for nodes in nodes(Some(128)) {
        group.bench_with_input(
            BenchmarkId::from_parameter(*nodes),
            nodes,
            |bench, &nodes| {
                bench.iter_batched(
                    || Profile::Dense.newman_watts_strogatz::<(), u8>(nodes),
                    |graph| {
                        let _scores = floyd_warshall(&graph, |edge| *edge.weight());
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
}

criterion_group!(benches, sparse, dense);
criterion_main!(benches);
