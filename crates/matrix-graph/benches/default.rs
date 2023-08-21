use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use petgraph_matrix_graph::MatrixGraph;

use crate::common::nodes;

mod common;

fn add_nodes(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("add/nodes");

    for nodes in nodes(None) {
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(*nodes),
            nodes,
            |bench, &nodes| {
                bench.iter_batched(
                    || MatrixGraph::<(), ()>::with_capacity(nodes),
                    |mut graph| {
                        for _ in 0..nodes {
                            graph.add_node(());
                        }
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
}

fn add_edges(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("add/edges");

    for nodes in nodes(None) {
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(*nodes),
            nodes,
            |bench, &nodes| {
                bench.iter_batched(
                    || {
                        let mut graph = MatrixGraph::<(), ()>::with_capacity(nodes);
                        let nodes: Vec<_> = (0..nodes).map(|_| graph.add_node(())).collect();

                        (graph, nodes)
                    },
                    |(mut graph, nodes)| {
                        for (source, &node) in nodes.iter().enumerate() {
                            let target = (source + 1) % nodes.len();

                            graph.add_edge(node, nodes[target], ());
                        }
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
}

fn add_edges_self(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("add/edges/self");

    for nodes in nodes(None) {
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(*nodes),
            nodes,
            |bench, &nodes| {
                bench.iter_batched(
                    || {
                        let mut graph = MatrixGraph::<(), ()>::with_capacity(nodes);
                        let nodes: Vec<_> = (0..nodes).map(|_| graph.add_node(())).collect();

                        (graph, nodes)
                    },
                    |(mut graph, nodes)| {
                        for node in nodes {
                            graph.add_edge(node, node, ());
                        }
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
}

fn add_edges_flower(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("add/edges/flower");

    for nodes in nodes(None) {
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(*nodes),
            nodes,
            |bench, &nodes| {
                bench.iter_batched(
                    || {
                        let mut graph = MatrixGraph::<(), ()>::with_capacity(nodes);
                        let nodes: Vec<_> = (0..nodes).map(|_| graph.add_node(())).collect();

                        (graph, nodes)
                    },
                    |(mut graph, nodes)| {
                        let source = nodes[0];

                        for &target in &nodes[1..] {
                            graph.add_edge(source, target, ());
                        }
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
}

criterion_group!(add, add_nodes, add_edges, add_edges_self, add_edges_flower);
criterion_main!(add);
