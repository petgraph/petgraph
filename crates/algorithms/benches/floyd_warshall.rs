mod common;

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use petgraph_algorithms::shortest_paths::floyd_warshall;

use crate::common::nodes;

const SEED: u64 = 0xD44F_C962_9610_EC54;

fn sparse(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("floyd_warshall/sparse");

    for nodes in nodes(Some(256)) {
        group.bench_with_input(
            BenchmarkId::from_parameter(*nodes),
            nodes,
            |bench, &nodes| {
                bench.iter_batched(
                    || common::newman_watts_strogatz_graph::<(), u8>(nodes, 4, 0.1, None),
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
                    || {
                        let connectivity = nodes / 2;

                        common::newman_watts_strogatz_graph::<(), u8>(
                            nodes,
                            connectivity,
                            0.2,
                            Some(SEED),
                        )
                    },
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
