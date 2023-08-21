mod common;

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use petgraph_algorithms::shortest_paths::dijkstra;

use crate::common::{nodes, Profile};

fn sparse(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("dijkstra/sparse");

    for nodes in nodes(None) {
        group.bench_with_input(
            BenchmarkId::from_parameter(*nodes),
            nodes,
            |bench, &nodes| {
                bench.iter_batched(
                    || {
                        let graph = Profile::Sparse.newman_watts_strogatz::<(), u8>(nodes);
                        let nodes = graph.node_indices().collect::<Vec<_>>();

                        (graph, nodes)
                    },
                    |(graph, nodes)| {
                        let _scores = dijkstra(&graph, nodes[0], None, |e| *e.weight());
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
}

fn dense(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("dijkstra/dense");

    for nodes in nodes(Some(1024)) {
        group.bench_with_input(
            BenchmarkId::from_parameter(*nodes),
            nodes,
            |bench, &nodes| {
                bench.iter_batched(
                    || {
                        let graph = Profile::Dense.newman_watts_strogatz::<(), u8>(nodes);
                        let nodes = graph.node_indices().collect::<Vec<_>>();

                        (graph, nodes)
                    },
                    |(graph, nodes)| {
                        let _scores = dijkstra(&graph, nodes[0], None, |e| *e.weight());
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
}

criterion_group!(benches, sparse, dense);
criterion_main!(benches);
