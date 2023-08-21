mod common;

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use petgraph_graph::{DiGraph, EdgeIndex, Graph, NodeIndex};

use crate::common::{cycle_graph, nodes};

fn add_nodes(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("add/nodes");

    for size in nodes(None) {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |bench, &size| {
            bench.iter_batched(
                DiGraph::<(), ()>::new,
                |mut graph| {
                    for _ in 0..size {
                        graph.add_node(());
                    }
                },
                BatchSize::SmallInput,
            );
        });
    }
}

fn add_edges(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("add/edges");

    for size in nodes(None) {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |bench, &size| {
            bench.iter_batched(
                || {
                    let mut graph = DiGraph::<(), ()>::new();

                    let nodes: Vec<_> = (0..size).map(|_| graph.add_node(())).collect();

                    (graph, nodes)
                },
                |(mut graph, nodes)| {
                    for i in 0..size {
                        let start = nodes[i];
                        let target = nodes[(i + 1) % size];

                        graph.add_edge(start, target, ());
                    }
                },
                BatchSize::SmallInput,
            );
        });
    }
}

fn remove_nodes(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("remove/nodes");

    for size in nodes(None) {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |bench, &size| {
            bench.iter_batched(
                || cycle_graph::<(), ()>(size, None),
                |(mut graph, nodes, _)| {
                    // remove ~1/4 of the nodes
                    let remove = nodes.len() / 4;
                    let first = nodes[0];

                    // we can just reuse the first node index, since we're removing nodes and that
                    // will change the indices
                    for _ in 0..remove {
                        graph.remove_node(first);
                    }
                },
                BatchSize::SmallInput,
            );
        });
    }
}

fn remove_edges(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("remove/edges");

    for size in nodes(None) {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |bench, &size| {
            bench.iter_batched(
                || cycle_graph::<(), ()>(size, None),
                |(mut graph, _, edges)| {
                    // remove ~1/4 of the edges
                    let remove = edges.len() / 4;

                    for i in 0..remove {
                        graph.remove_edge(edges[i]);
                    }
                },
                BatchSize::SmallInput,
            );
        });
    }
}

fn map_(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("map");

    for size in nodes(None) {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |bench, &size| {
            bench.iter_batched(
                || cycle_graph::<u8, u8>(size, None),
                |(graph, ..)| {
                    let _mapped = graph.map(
                        |_, weight| weight.saturating_mul(2),
                        |_, weight| weight.saturating_mul(3),
                    );
                },
                BatchSize::SmallInput,
            );
        });
    }
}

criterion_group!(add, add_nodes, add_edges);
criterion_group!(remove, remove_nodes, remove_edges);
criterion_group!(map, map_);

criterion_main!(add, remove, map);
