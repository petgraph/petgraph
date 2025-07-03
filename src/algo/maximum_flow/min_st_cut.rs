use alloc::{vec, vec::Vec};
use core::ops::{Add, Sub};

use crate::{
    algo::{maximum_flow::dinics::build_level_graph, EdgeRef, PositiveMeasure},
    visit::{EdgeCount, EdgeIndexable, IntoEdgesDirected, NodeCount, NodeIndexable, Visitable},
};

use super::dinics::dinics;

/// Compute the minimum cut that separates `source` and `destination` in a directed graph.
///
/// Implements the [max-flow min-cut theorem][max_flow_min_cut] using [Dinic's algorithm][ff]
/// to compute the maximum flow, then extracts the cut from the final residual graph.
/// The cut corresponds to the set of saturated edges going from the reachable set `S`
/// (from `source`) to the unreachable set `T` (toward `destination`) in the final residual network.
///
/// # Arguments
/// * `network` — A directed graph with positive edge weights, representing flow capacities.
/// * `source` — The source node from which flow originates.
/// * `destination` — The destination node toward which flow terminates.
///
/// # Returns
/// Returns a tuple of two values:
/// * `N::EdgeWeight`: the total capacity of the minimum cut, equal to the maximum flow;
/// * `Vec<N::EdgeRef>`: the edges in the minimum cut.
///
/// # Complexity
/// * Time complexity:
///   * In general: **O(|V|²|E|)** (same as Dinic's algorithm)
///   * In unit-capacity networks: **O(min{|V|²ᐟ³, |E|¹ᐟ²} |E|)**
/// * Auxiliary space: **O(|V| + |E|)**.
///
/// where **|V|** is the number of nodes and **|E|** is the number of edges.
///
/// # Example
/// ```rust
/// use petgraph::Graph;
/// use petgraph::algo::min_st_cut;
/// // Example from CLRS book
/// let mut graph = Graph::<u8, u8>::new();
/// let source = graph.add_node(0);
/// let _ = graph.add_node(1);
/// let _ = graph.add_node(2);
/// let _ = graph.add_node(3);
/// let _ = graph.add_node(4);
/// let destination = graph.add_node(5);
/// graph.extend_with_edges(&[
///    (0, 1, 16),
///    (0, 2, 13),
///    (1, 2, 10),
///    (1, 3, 12),
///    (2, 1, 4),
///    (2, 4, 14),
///    (3, 2, 9),
///    (3, 5, 20),
///    (4, 3, 7),
///    (4, 5, 4),
/// ]);
/// let (cut_capacity, cut_edges) = min_st_cut(&graph, source, destination);
/// assert_eq!(cut_capacity, 23);
/// assert_eq!(cut_edges.len(), 2);
/// ```
///
/// [ff]: https://en.wikipedia.org/wiki/Dinic%27s_algorithm
/// [max_flow_min_cut]: https://en.wikipedia.org/wiki/Max-flow_min-cut_theorem
pub fn min_st_cut<N>(
    network: N,
    source: N::NodeId,
    destination: N::NodeId,
) -> (N::EdgeWeight, Vec<N::EdgeRef>)
where
    N: NodeCount + EdgeCount + IntoEdgesDirected + EdgeIndexable + NodeIndexable + Visitable,
    N::EdgeWeight: Add<Output = N::EdgeWeight> + Sub<Output = N::EdgeWeight> + PositiveMeasure,
{
    let (max_flow, flows) = dinics(network, source, destination);
    let level_edges = &mut vec![Default::default(); network.node_count()];

    let level_graph = build_level_graph(&network, source, destination, &flows, level_edges);
    assert!(
        level_graph[NodeIndexable::to_index(&network, destination)] == 0,
        "destination should be unreachable after Dinic's completion"
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
