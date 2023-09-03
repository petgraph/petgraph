use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use petgraph_core::{
    edge::Direction,
    index::{FromIndexType, IntoIndexType},
    visit::EdgeRef,
};
use petgraph_matrix_graph::{MatrixGraph, NodeIndex};
use petgraph_test_utils::factories::directed_graph;

use crate::common::nodes;

mod common;

fn add_nodes(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("add/nodes");

    for nodes in nodes(None) {
        group.bench_with_input(
            BenchmarkId::from_parameter(*nodes),
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
            BenchmarkId::from_parameter(*nodes),
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
            BenchmarkId::from_parameter(*nodes),
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
            BenchmarkId::from_parameter(*nodes),
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

fn edges_count(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("edges/count");

    let graph = directed_graph().full_a();
    let graph: MatrixGraph<(), ()> = MatrixGraph::from_edges(graph.edge_references().map(|e| {
        (
            NodeIndex::from_index(e.source().into_index() as u16),
            NodeIndex::from_index(e.target().into_index() as u16),
            (),
        )
    }));

    group.bench_with_input("Full", &graph, |bench, graph| {
        bench.iter(|| {
            let _count = graph.edges(NodeIndex::new(0)).count();
        });
    });
}

fn edges_directed_count(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("edges_directed/count");

    let graph = directed_graph().full_a();
    let graph: MatrixGraph<(), ()> = MatrixGraph::from_edges(graph.edge_references().map(|e| {
        (
            NodeIndex::from_index(e.source().into_index() as u16),
            NodeIndex::from_index(e.target().into_index() as u16),
            (),
        )
    }));

    group.bench_with_input(
        BenchmarkId::new("Outgoing", "Full"),
        &graph,
        |bench, graph| {
            bench.iter(|| {
                let _count = graph
                    .edges_directed(NodeIndex::new(0), Direction::Outgoing)
                    .count();
            });
        },
    );

    group.bench_with_input(
        BenchmarkId::new("Incoming", "Full"),
        &graph,
        |bench, graph| {
            bench.iter(|| {
                let _count = graph
                    .edges_directed(NodeIndex::new(0), Direction::Incoming)
                    .count();
            });
        },
    );

    let graph = directed_graph().bigger();
    let graph: MatrixGraph<(), ()> = MatrixGraph::from_edges(graph.edge_references().map(|e| {
        (
            NodeIndex::from_index(e.source().into_index() as u16),
            NodeIndex::from_index(e.target().into_index() as u16),
            (),
        )
    }));

    group.bench_with_input(
        BenchmarkId::new("Outgoing", "Bigger"),
        &graph,
        |bench, graph| {
            bench.iter(|| {
                let _count = graph
                    .edges_directed(NodeIndex::new(0), Direction::Outgoing)
                    .count();
            });
        },
    );

    group.bench_with_input(
        BenchmarkId::new("Incoming", "Bigger"),
        &graph,
        |bench, graph| {
            bench.iter(|| {
                let _count = graph
                    .edges_directed(NodeIndex::new(0), Direction::Incoming)
                    .count();
            });
        },
    );
}

fn neighbours_count(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("neighbours/count");

    let graph = directed_graph().full_a();
    let graph: MatrixGraph<(), ()> = MatrixGraph::from_edges(graph.edge_references().map(|e| {
        (
            NodeIndex::from_index(e.source().into_index() as u16),
            NodeIndex::from_index(e.target().into_index() as u16),
            (),
        )
    }));

    group.bench_with_input("Full", &graph, |bench, graph| {
        bench.iter(|| {
            let _count = graph.neighbors(NodeIndex::new(0)).count();
        });
    });
}

fn neighbours_directed_count(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("neighbours/count");

    let graph = directed_graph().full_a();
    let graph: MatrixGraph<(), ()> = MatrixGraph::from_edges(graph.edge_references().map(|e| {
        (
            NodeIndex::from_index(e.source().into_index() as u16),
            NodeIndex::from_index(e.target().into_index() as u16),
            (),
        )
    }));

    group.bench_with_input(
        BenchmarkId::new("Outgoing", "Full"),
        &graph,
        |bench, graph| {
            bench.iter(|| {
                let _count = graph
                    .neighbors_directed(NodeIndex::new(0), Direction::Outgoing)
                    .count();
            });
        },
    );

    group.bench_with_input(
        BenchmarkId::new("Incoming", "Full"),
        &graph,
        |bench, graph| {
            bench.iter(|| {
                let _count = graph
                    .neighbors_directed(NodeIndex::new(0), Direction::Incoming)
                    .count();
            });
        },
    );

    let graph = directed_graph().bigger();
    let graph: MatrixGraph<(), ()> = MatrixGraph::from_edges(graph.edge_references().map(|e| {
        (
            NodeIndex::from_index(e.source().into_index() as u16),
            NodeIndex::from_index(e.target().into_index() as u16),
            (),
        )
    }));

    group.bench_with_input(
        BenchmarkId::new("Outgoing", "Bigger"),
        &graph,
        |bench, graph| {
            bench.iter(|| {
                let _count = graph
                    .neighbors_directed(NodeIndex::new(0), Direction::Outgoing)
                    .count();
            });
        },
    );

    group.bench_with_input(
        BenchmarkId::new("Incoming", "Bigger"),
        &graph,
        |bench, graph| {
            bench.iter(|| {
                let _count = graph
                    .neighbors_directed(NodeIndex::new(0), Direction::Incoming)
                    .count();
            });
        },
    );
}

criterion_group!(add, add_nodes, add_edges, add_edges_self, add_edges_flower);
criterion_group!(edges, edges_count, edges_directed_count);
criterion_group!(neighbours, neighbours_count, neighbours_directed_count);

criterion_main!(add, edges, neighbours);
