mod common;

use core::{borrow::Borrow, hint::black_box};

use common::{add_edge, add_nodes};
use criterion::{
    BenchmarkGroup, Criterion, criterion_group, criterion_main, measurement::WallTime,
};
use petgraph_algorithms::flows::max_flow::{dinics_mod::Dinics, edmonds_karp_mod::EdmondsKarp};
use petgraph_core::{
    graph::{DirectedGraph, EditableGraph, Graph, storable::StorableGraph},
    utils::test_graphs::directed::DirectedTestGraph,
};

/// A flow network together with its source and sink node.
type FlowNetwork<G> = (G, <G as Graph>::NodeId, <G as Graph>::NodeId);

/// A single path of `node_count` nodes with unit capacities.
fn line<G>(mut graph: G, node_count: usize) -> FlowNetwork<G>
where
    G: EditableGraph<OwnedNodeData = (), OwnedEdgeData = u32>,
{
    let nodes = add_nodes(&mut graph, node_count);
    for window in nodes.windows(2) {
        add_edge(&mut graph, window[0], window[1], 1);
    }
    (graph, nodes[0], nodes[node_count - 1])
}

/// `node_count` nodes with unit capacity edges `i -> (i + 1) * j` for several strides `j`.
///
/// `node_count - 1` should be divisible by every stride.
fn many_edges<G>(mut graph: G, node_count: usize) -> FlowNetwork<G>
where
    G: EditableGraph<OwnedNodeData = (), OwnedEdgeData = u32>,
{
    let nodes = add_nodes(&mut graph, node_count);
    for stride in [1, 2, 4, 5, 10, 20, 25, 50] {
        for i in 0..(node_count - 1) / stride {
            add_edge(&mut graph, nodes[i], nodes[(i + 1) * stride], 1);
        }
    }
    (graph, nodes[0], nodes[node_count - 1])
}

/// Source and sink connected through `node_count - 2` disjoint paths of length two.
fn wide<G>(mut graph: G, node_count: usize) -> FlowNetwork<G>
where
    G: EditableGraph<OwnedNodeData = (), OwnedEdgeData = u32>,
{
    let nodes = add_nodes(&mut graph, node_count);
    let (source, sink) = (nodes[0], nodes[1]);
    for &node in &nodes[2..] {
        add_edge(&mut graph, source, node, 1);
        add_edge(&mut graph, node, sink, 1);
    }
    (graph, source, sink)
}

/// Source and sink connected to a sparse subset of a densely connected set of intermediate nodes.
///
/// If `varying_capacities` is false, all edges have unit capacity.
fn dense_middle<G>(mut graph: G, node_count: usize, varying_capacities: bool) -> FlowNetwork<G>
where
    G: EditableGraph<OwnedNodeData = (), OwnedEdgeData = u32>,
{
    let capacity = |value: usize| {
        if varying_capacities {
            u32::try_from(value).expect("capacity fits into u32")
        } else {
            1
        }
    };

    let nodes = add_nodes(&mut graph, node_count);
    let (source, sink) = (nodes[0], nodes[1]);
    let intermediates = &nodes[2..];

    for (i, &node) in intermediates.iter().enumerate() {
        if i % 7 == 0 {
            add_edge(&mut graph, source, node, capacity(1 + (i % 11)));
        }
    }

    for (i, &node) in intermediates.iter().enumerate() {
        if i % 11 == 0 {
            add_edge(&mut graph, node, sink, capacity(11 - (i % 11)));
        }
    }

    for i in 0..intermediates.len() {
        for j in (i + 1)..intermediates.len() {
            if (i + j) % 13 == 0 {
                add_edge(
                    &mut graph,
                    intermediates[i],
                    intermediates[j],
                    capacity((i + j) % 13 + 1),
                );
            }
        }
    }

    (graph, source, sink)
}

/// Benchmarks all maximum flow algorithms on the given network.
fn bench_algorithms<G>(group: &mut BenchmarkGroup<'_, WallTime>, network: &FlowNetwork<G>)
where
    for<'graph> G: DirectedGraph + StorableGraph + Graph<EdgeData<'graph> = u32> + 'graph,
{
    let (graph, source, destination) = network;
    let capacities = |edge_id| *graph.edge(edge_id).unwrap().data.borrow();

    group.bench_function("dinics", |bencher| {
        bencher.iter(|| {
            Dinics::new_with_capacities(black_box(graph), capacities)
                .with_source(*source)
                .with_destination(*destination)
                .run()
                .unwrap()
        });
    });

    group.bench_function("edmonds_karp", |bencher| {
        bencher.iter(|| {
            EdmondsKarp::new_with_capacities(black_box(graph), capacities)
                .with_source(*source)
                .with_source(*destination)
                .run()
                .unwrap()
        });
    });
}

fn max_flow(criterion: &mut Criterion) {
    let networks = [
        ("max_flow/line", line(DirectedTestGraph::new(), 1_000)),
        (
            "max_flow/many_edges",
            many_edges(DirectedTestGraph::new(), 1_001),
        ),
        ("max_flow/wide", wide(DirectedTestGraph::new(), 1_000)),
        (
            "max_flow/dense_middle",
            dense_middle(DirectedTestGraph::new(), 500, false),
        ),
        (
            "max_flow/dense_middle_varying_capacities",
            dense_middle(DirectedTestGraph::new(), 250, true),
        ),
    ];

    for (name, network) in &networks {
        let mut group = criterion.benchmark_group(*name);
        // Some runs take several seconds, so keep the total runtime manageable.
        group.sample_size(10);
        bench_algorithms(&mut group, network);
        group.finish();
    }
}

criterion_group!(benches, max_flow);
criterion_main!(benches);
