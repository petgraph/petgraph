mod common;

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use petgraph_algorithms::shortest_paths::bellman_ford;

use crate::common::nodes;

fn sparse(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("bellman_ford/sparse");

    for nodes in nodes(None) {
        group.bench_with_input(
            BenchmarkId::from_parameter(*nodes),
            nodes,
            |bench, &nodes| {
                bench.iter_batched(
                    || {
                        let graph =
                            common::newman_watts_strogatz_graph::<(), f32>(nodes, 4, 0.1, None);
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
                        let connectivity = nodes / 2;

                        let graph = common::newman_watts_strogatz_graph::<(), f32>(
                            nodes,
                            connectivity,
                            0.2,
                            None,
                        );
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
