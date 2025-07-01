use alloc::{vec, vec::Vec};
use core::ops::{Add, Sub};

use crate::{
    algo::{maximum_flow::dinics::build_level_graph, EdgeRef, PositiveMeasure},
    visit::{EdgeCount, EdgeIndexable, IntoEdgesDirected, NodeCount, NodeIndexable, Visitable},
};

use super::dinics::dinics;

/// Computes the [minimum cut][min_cut] of a weighted directed graph that
/// separates the vertices `source` and `sink` in different partitions,
/// using the [max-flow min-cut theorem][max_flow_min_cut].
///
/// Underneath, uses Dinic's algorithm to solve the maximum flow
/// problem in the network and then builds the minimum cut from the
/// last level graph built in Dinic's.
///
/// Returns the edges present in minimum cut and the computed min cut capacity
/// (which is equivalent to the maximum flow in the network).
///
/// [min_cut]: https://en.wikipedia.org/wiki/Minimum_cut
/// [max_flow_min_cut]: https://en.wikipedia.org/wiki/Max-flow_min-cut_theorem
pub fn min_st_cut<N>(
    network: N,
    source: N::NodeId,
    sink: N::NodeId,
) -> (N::EdgeWeight, Vec<N::EdgeRef>)
where
    N: NodeCount + EdgeCount + IntoEdgesDirected + EdgeIndexable + NodeIndexable + Visitable,
    N::EdgeWeight: Add<Output = N::EdgeWeight> + Sub<Output = N::EdgeWeight> + PositiveMeasure,
{
    let (max_flow, flows) = dinics(network, source, sink);
    let level_edges = &mut vec![Default::default(); network.node_count()];

    let (sink_reachable, level_graph) =
        build_level_graph(&network, source, sink, &flows, level_edges);

    assert!(
        !sink_reachable,
        "Sink should be unreachable after Dinic's completion"
    );

    let cut_edges: Vec<N::EdgeRef> = network
        .edge_references()
        .filter(|edge| is_edge_in_st_cut(network, &flows, &level_graph, edge))
        .collect();

    let cut_capacity = cut_edges
        .iter()
        .map(|edge| *edge.weight())
        .fold(N::EdgeWeight::zero(), |a, b| a + b);

    assert_eq!(
        max_flow, cut_capacity,
        "Min-cut capacity should equal to the network's maximum flow"
    );

    (cut_capacity, cut_edges)
}

// Checks if edge is part of network's st cut.
fn is_edge_in_st_cut<N>(
    network: N,
    flows: &[N::EdgeWeight],
    level_graph: &[usize],
    edge: &N::EdgeRef,
) -> bool
where
    N: NodeCount + EdgeCount + IntoEdgesDirected + EdgeIndexable + NodeIndexable + Visitable,
    N::EdgeWeight: PartialEq,
{
    let source_index = NodeIndexable::to_index(&network, edge.source());
    let target_index = NodeIndexable::to_index(&network, edge.target());

    // Source is in `s` partition if it is reachable in last level graph
    let source_in_s = level_graph[source_index] > 0;

    // Target is in `t` partition if it is not reachable in last level graph
    let target_in_t = level_graph[target_index] == 0;

    let is_cut = source_in_s && target_in_t;

    assert!(
        !is_cut || flows[EdgeIndexable::to_index(&network, edge.id())] == *edge.weight(),
        "Cut edge should be saturated"
    );

    is_cut
}
