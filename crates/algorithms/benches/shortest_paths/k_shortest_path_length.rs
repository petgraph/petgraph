use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use petgraph_algorithms::shortest_paths::k_shortest_path_length;

use crate::common::{nodes, Profile};

fn sparse(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("k_shortest_path_length/sparse");

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
                        let _scores = k_shortest_path_length(&graph, nodes[0], None, 2, |edge| {
                            *edge.weight()
                        });
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
}

fn dense(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("k_shortest_path_length/dense");

    for nodes in nodes(Some(512)) {
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
                        let _scores = k_shortest_path_length(&graph, nodes[0], None, 2, |edge| {
                            *edge.weight()
                        });
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
}

criterion_group!(benches, sparse, dense);
