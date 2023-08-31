mod common;

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use petgraph::{
    graph::{stable::StableGraph, Graph},
    matrix_graph::MatrixGraph,
};
use petgraph_algorithms::components::{connected_components, kosaraju_scc, tarjan_scc};
use petgraph_core::{
    edge::Directed,
    id::{FromIndexType, IntoIndexType},
};

use crate::common::{factories::undirected_graph, nodes, Profile};

fn connected_directed(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("connected_components/directed");

    for (id, graph) in [
        ("praust A", undirected_graph().praust_a()),
        ("praust B", undirected_graph().praust_b()),
        ("full A", undirected_graph().full_a()),
        ("full B", undirected_graph().full_b()),
        ("petersen A", undirected_graph().petersen_a()),
        ("petersen B", undirected_graph().petersen_b()),
    ] {
        group.bench_with_input(id, &graph, |bench, graph| {
            bench.iter(|| connected_components(graph));
        });
    }
}

fn connected_undirected(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("connected_components/undirected");

    for (id, graph) in [
        ("praust A", undirected_graph().praust_a()),
        ("praust B", undirected_graph().praust_b()),
        ("full A", undirected_graph().full_a()),
        ("full B", undirected_graph().full_b()),
        ("petersen A", undirected_graph().petersen_a()),
        ("petersen B", undirected_graph().petersen_b()),
    ] {
        group.bench_with_input(id, &graph, |bench, graph| {
            bench.iter(|| connected_components(graph));
        });
    }
}

macro_rules! scc_compare {
    (@bench $graph:ident, $index:ty, $default:ty, $group:ident, $size:ident) => {
        $group.bench_with_input(
            BenchmarkId::new(stringify!($graph), $size),
            $size,
            |bench, &size| {
                bench.iter_batched(
                    || {
                        let (_, edges) = Profile::Sparse
                            .newman_watts_strogatz_directed::<(), ()>(size)
                            .into_nodes_edges();

                        let graph: $graph<(), (), Directed> =
                            $graph::from_edges(edges.into_iter().map(|edge| {
                                (
                                    <$index>::from_index(edge.source().into_index() as $default),
                                    <$index>::from_index(edge.target().into_index() as $default),
                                    edge.weight,
                                )
                            }));

                        graph
                    },
                    |graph| {
                        let _scc = kosaraju_scc(&graph);
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    };

    ($func_name:ident, $name:ident, $func:ident,[$($graph:ident, $index:ty, $default:ty);*]) => {
        fn $func_name(criterion: &mut Criterion) {
            let mut group = criterion.benchmark_group(concat!(stringify!($name), "/compare"));

            for size in nodes(Some(1024)) {
                $(
                    scc_compare!(@bench $graph, $index, $default, group, size);
                )*
            }
        }
    };
}

scc_compare!(
    kosaraju_compare,
    kosaraju,
    kosaraju_scc,
    [
        Graph, petgraph::graph::NodeIndex, u32;
        StableGraph, petgraph::stable_graph::NodeIndex, u32;
        MatrixGraph, petgraph::matrix_graph::NodeIndex, u16
    ]
);

fn kosaraju_sparse(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("kosaraju_scc/sparse");

    for size in nodes(None) {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |bench, &size| {
            bench.iter_batched(
                || Profile::Sparse.newman_watts_strogatz_directed::<(), ()>(size),
                |graph| {
                    let _scc = kosaraju_scc(&graph);
                },
                BatchSize::SmallInput,
            );
        });
    }
}

fn kosaraju_dense(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("kosaraju_scc/dense");

    for size in nodes(Some(64)) {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |bench, &size| {
            bench.iter_batched(
                || Profile::Dense.newman_watts_strogatz_directed::<(), ()>(size),
                |graph| {
                    let _scc = kosaraju_scc(&graph);
                },
                BatchSize::SmallInput,
            );
        });
    }
}

scc_compare!(
    tarjan_compare,
    tarjan,
    tarjan_scc,
    [
        Graph, petgraph::graph::NodeIndex, u32;
        StableGraph, petgraph::stable_graph::NodeIndex, u32;
        MatrixGraph, petgraph::matrix_graph::NodeIndex, u16
    ]
);

fn tarjan_sparse(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("tarjan_scc/sparse");

    for size in nodes(None) {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |bench, &size| {
            bench.iter_batched(
                || Profile::Sparse.newman_watts_strogatz_directed::<(), ()>(size),
                |graph| {
                    let _scc = tarjan_scc(&graph);
                },
                BatchSize::SmallInput,
            );
        });
    }
}

fn tarjan_dense(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("tarjan_scc/dense");

    for size in nodes(Some(64)) {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |bench, &size| {
            bench.iter_batched(
                || Profile::Dense.newman_watts_strogatz_directed::<(), ()>(size),
                |graph| {
                    let _scc = tarjan_scc(&graph);
                },
                BatchSize::SmallInput,
            );
        });
    }
}

criterion_group!(connected, connected_directed, connected_undirected);

criterion_group!(kosaraju, kosaraju_sparse, kosaraju_dense, kosaraju_compare);
criterion_group!(tarjan, tarjan_sparse, tarjan_dense, tarjan_compare);

criterion_main!(connected, kosaraju, tarjan);
