mod common;

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use petgraph_algorithms::shortest_paths::k_shortest_path_length;

use crate::common::nodes;

fn sparse(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("k_shortest_path_length/sparse");

    for nodes in nodes(None) {
        group.bench_with_input(
            BenchmarkId::from_parameter(*nodes),
            nodes,
            |bench, &nodes| {
                bench.iter_batched(
                    || {
                        let graph =
                            common::newman_watts_strogatz_graph::<(), u8>(nodes, 4, 0.1, None);
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
                        let connectivity = nodes / 2;

                        let graph = common::newman_watts_strogatz_graph::<(), u8>(
                            nodes,
                            connectivity,
                            0.2,
                            None,
                        );
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
criterion_main!(benches);
