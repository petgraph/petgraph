#![cfg(feature = "serde")]

mod common;

use std::io;

use criterion::{criterion_group, criterion_main, Criterion};
use petgraph_graph::stable::StableGraph;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

use crate::common::nodes;

struct VoidWriter;

impl io::Write for VoidWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn make_stable_graph<N, E>(n: usize) -> StableGraph<N, E>
where
    Standard: Distribution<N> + Distribution<E>,
{
    // ratio is from previous unparameterized version
    let num_nodes = n / 2;
    let num_holes = num_nodes;

    let num_edges = n / 10;

    let mut rng = rand::thread_rng();

    let mut graph = StableGraph::with_capacity(n, num_edges);
    let mut nodes: Vec<_> = (0..n).map(|index| graph.add_node(rng.gen())).collect();

    for _ in 0..num_edges {
        let first = rng.gen_range(0..n);
        let mut second = rng.gen_range(0..(n - 1));

        // ensure that we don't have self-loops
        if first == second {
            second = second.wrapping_add(1);
        }

        let weight = rng.gen();

        graph.add_edge(nodes[first], nodes[second], weight);
    }

    // Remove nodes to make the structure a bit more interesting
    for _ in 0..num_holes {
        let index = rng.gen_range(0..nodes.len());
        graph.remove_node(nodes[index]);
        nodes.remove(index);
    }

    graph
}

fn serialize_stable_graph(bench: &mut Criterion) {
    let mut group = bench.benchmark_group("serialize/stable_graph");

    for size in nodes(None) {
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(*size),
            size,
            |bench, &size| {
                bench.iter_batched(
                    || make_stable_graph::<u32, u32>(size),
                    |graph| {
                        serde_json::to_writer(VoidWriter, &graph)
                            .expect("failed to serialize graph");
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
}

fn deserialize_stable_graph(bench: &mut Criterion) {
    let mut group = bench.benchmark_group("deserialize/stable_graph");

    for size in nodes(None) {
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(*size),
            size,
            |bench, &size| {
                bench.iter_batched(
                    || {
                        let graph = make_stable_graph::<u32, u32>(size);

                        serde_json::to_vec(&graph).expect("failed to serialize graph")
                    },
                    |graph| {
                        let _graph: StableGraph<u32, u32> =
                            serde_json::from_slice(&graph).expect("failed to deserialize graph");
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
}

criterion_group!(benches, serialize_stable_graph, deserialize_stable_graph);
criterion_main!(benches);
