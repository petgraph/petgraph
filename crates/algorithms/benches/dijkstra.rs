mod common;

use criterion::{criterion_group, criterion_main, Criterion};
use petgraph_algorithms::shortest_paths::dijkstra;

use crate::common::nodes;

const SEED: u64 = 0xB12F_FF7B_568E_805A;

fn sparse(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("dijkstra/sparse");

    for nodes in nodes(None) {
        group.bench_with_input(format!("nodes={nodes}"), nodes, |bench, &nodes| {
            let graph = common::newman_watts_strogatz_graph::<(), u8>(nodes, 4, 0.1, Some(SEED));
            let nodes = graph.node_indices().collect::<Vec<_>>();

            bench.iter(|| {
                let _scores = dijkstra(&graph, nodes[0], None, |e| *e.weight());
            });
        });
    }
}

fn dense(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("dijkstra/dense");

    for nodes in nodes(Some(1024)) {
        group.bench_with_input(format!("nodes={nodes}"), nodes, |bench, &nodes| {
            let connectivity = nodes / 2;

            let graph =
                common::newman_watts_strogatz_graph::<(), u8>(nodes, connectivity, 0.2, Some(SEED));
            let nodes = graph.node_indices().collect::<Vec<_>>();

            bench.iter(|| {
                let _scores = dijkstra(&graph, nodes[0], None, |e| *e.weight());
            });
        });
    }
}

criterion_group!(benches, sparse, dense);
criterion_main!(benches);
