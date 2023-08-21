mod common;

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use petgraph_algorithms::shortest_paths::bellman_ford;

use crate::common::{nodes, Profile};

fn sparse(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("bellman_ford/sparse");

    for nodes in nodes(None) {
        group.bench_with_input(
            BenchmarkId::from_parameter(*nodes),
            nodes,
            |bench, &nodes| {
                bench.iter_batched(
                    || {
                        let graph = Profile::Sparse.newman_watts_strogatz::<(), f32>(nodes);
                        let nodes = graph.node_indices().collect::<Vec<_>>();

                        (graph, nodes)
                    },
                    |(graph, nodes)| {
                        let _scores = bellman_ford(&graph, nodes[0]);
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
}

fn dense(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("bellman_ford/dense");

    for nodes in nodes(Some(128)) {
        group.bench_with_input(
            BenchmarkId::from_parameter(*nodes),
            nodes,
            |bench, &nodes| {
                bench.iter_batched(
                    || {
                        let graph = Profile::Dense.newman_watts_strogatz::<(), f32>(nodes);
                        let nodes = graph.node_indices().collect::<Vec<_>>();

                        (graph, nodes)
                    },
                    |(graph, nodes)| {
                        let _scores = bellman_ford(&graph, nodes[0]);
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
}

criterion_group!(benches, sparse, dense);
criterion_main!(benches);
