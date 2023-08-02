mod common;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use petgraph_algorithms::shortest_paths::bellman_ford;

use crate::common::nodes;

const SEED: u64 = 0x0319_4CEB_7761_D0E4;

fn sparse(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("bellman_ford/sparse");

    for nodes in nodes(None) {
        group.bench_with_input(
            BenchmarkId::from_parameter(*nodes),
            nodes,
            |bench, &nodes| {
                let graph =
                    common::newman_watts_strogatz_graph::<(), f32>(nodes, 4, 0.1, Some(SEED));
                let nodes = graph.node_indices().collect::<Vec<_>>();

                bench.iter(|| {
                    let _scores = bellman_ford(&graph, nodes[0]);
                });
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
                let connectivity = nodes / 2;

                let graph = common::newman_watts_strogatz_graph::<(), f32>(
                    nodes,
                    connectivity,
                    0.2,
                    Some(SEED),
                );
                let nodes = graph.node_indices().collect::<Vec<_>>();

                bench.iter(|| {
                    let _scores = bellman_ford(&graph, nodes[0]);
                });
            },
        );
    }
}

criterion_group!(benches, sparse, dense);
criterion_main!(benches);
