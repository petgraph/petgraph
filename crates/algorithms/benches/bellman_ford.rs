mod common;

use criterion::{criterion_group, criterion_main, Criterion};
use petgraph_algorithms::shortest_paths::bellman_ford;

use crate::common::NODE_COUNTS;

const SEED: u64 = 0x03194CEB7761D0E4;

fn sparse(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("bellman_ford/sparse");

    for nodes in NODE_COUNTS {
        group.bench_with_input(format!("nodes={nodes}"), nodes, |bench, &nodes| {
            let graph = common::newman_watts_strogatz_graph(nodes, 4, 0.1, Some(SEED));
            let nodes = graph.node_indices().collect::<Vec<_>>();

            bench.iter(|| {
                let _scores = bellman_ford(&graph, nodes[0]);
            });
        })
    }
}

fn dense(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("bellman_ford/dense");

    for nodes in NODE_COUNTS {
        group.bench_with_input(format!("nodes={nodes}"), nodes, |bench, &nodes| {
            let connectivity = nodes / 2;

            let mut graph =
                common::newman_watts_strogatz_graph(nodes, connectivity, 0.2, Some(SEED));
            let nodes = graph.node_indices().collect::<Vec<_>>();

            bench.iter(|| {
                let _scores = bellman_ford(&graph, nodes[0]);
            });
        });
    }
}

criterion_group!(benches, sparse, dense);
criterion_main!(benches);
