mod common;

use criterion::{criterion_group, criterion_main, Criterion};
use petgraph_algorithms::shortest_paths::{bellman_ford, floyd_warshall};

use crate::common::NODE_COUNTS;

const SEED: u64 = 0xD44FC9629610EC54;

fn sparse(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("floyd_warshall/sparse");

    for nodes in NODE_COUNTS {
        group.bench_with_input(format!("nodes={nodes}"), nodes, |bench, &nodes| {
            let graph = common::newman_watts_strogatz_graph(nodes, 4, 0.1, Some(SEED));

            bench.iter(|| {
                let _scores = floyd_warshall(&graph, |edge| edge.weight());
            });
        })
    }
}

fn dense(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("floyd_warshall/dense");

    for nodes in NODE_COUNTS {
        group.bench_with_input(format!("nodes={nodes}"), nodes, |bench, &nodes| {
            let connectivity = nodes / 2;

            let graph = common::newman_watts_strogatz_graph(nodes, connectivity, 0.2, Some(SEED));

            bench.iter(|| {
                let _scores = floyd_warshall(&graph, |edge| edge.weight());
            });
        });
    }
}

criterion_group!(benches, sparse, dense);
criterion_main!(benches);
