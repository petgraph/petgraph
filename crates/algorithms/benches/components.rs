mod common;

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use petgraph_algorithms::components::{connected_components, kosaraju_scc, tarjan_scc};

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

    for size in nodes(Some(1024)) {
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

    for size in nodes(Some(1024)) {
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

criterion_group!(kosaraju, kosaraju_sparse, kosaraju_dense);
criterion_group!(tarjan, tarjan_sparse, tarjan_dense);

criterion_main!(connected, kosaraju, tarjan);
