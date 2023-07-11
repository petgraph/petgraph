mod common;

use criterion::{criterion_group, criterion_main, Criterion};
use petgraph_algorithms::shortest_paths::dijkstra;

fn sparse(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("dijkstra/sparse");

    for nodes in [8, 16, 32, 64, 128, 256, 1024, 2048, 4096, 8192, 16384].iter() {
        group.bench_with_input(format!("nodes={}", nodes), nodes, |bench, &nodes| {
            let graph = common::newman_watts_strogatz_graph(nodes, 4, 0.1, None);
            let nodes = graph.node_indices().collect::<Vec<_>>();

            bench.iter(|| {
                let _scores = dijkstra(&graph, nodes[0], None, |e| *e.weight());
            });
        });
    }
}

fn dense(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("dijkstra/dense");

    for nodes in [8, 16, 32, 64, 128, 256, 1024, 2048, 4096, 8192, 16384].iter() {
        group.bench_with_input(format!("nodes={}", nodes), nodes, |bench, &nodes| {
            let connectivity = nodes / 2;

            let graph = common::newman_watts_strogatz_graph(nodes, connectivity, 0.2, None);
            let nodes = graph.node_indices().collect::<Vec<_>>();

            bench.iter(|| {
                let _scores = dijkstra(&graph, nodes[0], None, |e| *e.weight());
            });
        });
    }
}

criterion_group!(benches, sparse, dense);
criterion_main!(benches);
