mod common;

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use petgraph_core::{edge::Direction, visit::IntoEdges};
use petgraph_graph::NodeIndex;

use crate::common::{cycle_stable_graph, nodes};

fn edges_count(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("edges/count");

    for size in nodes(None) {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |bench, &size| {
            bench.iter_batched(
                || cycle_stable_graph::<(), ()>(size, None),
                |(graph, ..)| {
                    let _count = graph.edges(NodeIndex::new(0)).count();
                },
                BatchSize::SmallInput,
            );
        });
    }
}

fn edges_directed_count(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("edges_directed/outgoing/count");

    for size in nodes(None) {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |bench, &size| {
            bench.iter_batched(
                || cycle_stable_graph::<(), ()>(size, None),
                |(graph, ..)| {
                    let _count = graph
                        .edges_directed(NodeIndex::new(0), Direction::Outgoing)
                        .count();
                },
                BatchSize::SmallInput,
            );
        });
    }

    let mut group = criterion.benchmark_group("edges_directed/incoming/count");

    for size in nodes(None) {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |bench, &size| {
            bench.iter_batched(
                || cycle_stable_graph::<(), ()>(size, None),
                |(graph, ..)| {
                    let _count = graph
                        .edges_directed(NodeIndex::new(0), Direction::Incoming)
                        .count();
                },
                BatchSize::SmallInput,
            );
        });
    }
}

fn neighbours_count(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("neighbors/count");

    for size in nodes(None) {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |bench, &size| {
            bench.iter_batched(
                || cycle_stable_graph::<(), ()>(size, None),
                |(graph, ..)| {
                    let _count = graph.neighbors(NodeIndex::new(0)).count();
                },
                BatchSize::SmallInput,
            );
        });
    }
}

fn neighbours_directed_count(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("neighbors_directed/outgoing/count");

    for size in nodes(None) {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |bench, &size| {
            bench.iter_batched(
                || cycle_stable_graph::<(), ()>(size, None),
                |(graph, ..)| {
                    let _count = graph
                        .neighbors_directed(NodeIndex::new(0), Direction::Outgoing)
                        .count();
                },
                BatchSize::SmallInput,
            );
        });
    }

    let mut group = criterion.benchmark_group("neighbors_directed/incoming/count");

    for size in nodes(None) {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |bench, &size| {
            bench.iter_batched(
                || cycle_stable_graph::<(), ()>(size, None),
                |(graph, ..)| {
                    let _count = graph
                        .neighbors_directed(NodeIndex::new(0), Direction::Incoming)
                        .count();
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
                || cycle_stable_graph::<u8, u8>(size, None),
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

fn retain_nodes(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("retain/nodes");

    for size in nodes(None) {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |bench, &size| {
            bench.iter_batched(
                || {
                    let (graph, ..) = cycle_stable_graph::<(), ()>(size, None);
                    let remove = graph.node_count() / 2;

                    (graph, remove)
                },
                |(mut graph, remove)| {
                    graph.retain_nodes(|_, index| index.index() > remove);
                },
                BatchSize::SmallInput,
            );
        });
    }
}

fn retain_edges(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("retain/edges");

    for size in nodes(None) {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |bench, &size| {
            bench.iter_batched(
                || {
                    let (graph, ..) = cycle_stable_graph::<(), ()>(size, None);
                    let remove = graph.edge_count() / 2;

                    (graph, remove)
                },
                |(mut graph, remove)| {
                    graph.retain_edges(|_, index| index.index() > remove);
                },
                BatchSize::SmallInput,
            );
        });
    }
}

criterion_group!(edges, edges_count, edges_directed_count);
criterion_group!(neighbours, neighbours_count, neighbours_directed_count);
criterion_group!(retain, retain_nodes, retain_edges);
criterion_group!(map, map_);

criterion_main!(edges, neighbours, retain, map);
