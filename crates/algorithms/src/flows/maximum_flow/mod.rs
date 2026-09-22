//! Collection of algorithms for the [Maximum Flow Problem][max_flow_wikipedia].
//!
//! Using the `max_flow` function will select an algorithm. The algorithm(s) it use
//! are subject to change in the future
//!
//! Currently, `petgraph` provides two algorithms to compute the maximum flow
//! in a flow network:
//! - [Dinic's Algorithm][dinics] [(Wikipedia)][dinics_wikipedia]
//! - [Edmonds-Karp Algorithm][edmonds_karp] [(Wikipedia)][edmonds_karp_wikipedia]
//!
//! They are implemented in the functions [`dinics`] and [`edmonds_karp`] and can be found
//! in their respective submodules.
//!
//! [Dinics][dinics] and [Edmonds][edmonds_karp] have different time complexities, and
//! their performance can vary significantly depending on the input graph.
//! In general, [dinics] is faster, especially on dense graphs, graphs with
//! unit capacities, and bipartite graphs.
//! [Edmonds Karp][edmonds_karp] may be a better choice when working with small or
//! sparse graphs.
//!
//! For more information about each algorithm and their detailed time
//! complexity, check their respective documentation.
//!
//! [dinics_wikipedia]: https://en.wikipedia.org/wiki/Dinic%27s_algorithm
//! [edmonds_karp_wikipedia]: https://en.wikipedia.org/wiki/Edmonds%E2%80%93Karp_algorithm
//! [max_flow_wikipedia]: https://en.wikipedia.org/wiki/Maximum_flow_problem

#[cfg(feature = "alloc")]
pub mod dinics_mod;
#[cfg(feature = "alloc")]
pub mod edmonds_karp_mod;

use std::{
    borrow::Borrow,
    ops::{Add, Sub},
};

#[cfg(feature = "alloc")]
pub use dinics_mod::dinics;
#[cfg(feature = "alloc")]
pub use edmonds_karp_mod::edmonds_karp;
use petgraph_core::{
    edge::Edge,
    graph::{DirectedGraph, Graph},
    id::IndexId,
};

use crate::traits::{Bounded, Measure, Zero};

/// The return trait that is implemented by Max Flow algorithms
pub trait MaxFlowReturn<'graph, G: Graph + 'graph> {
    /// Returns the maximum flow value computed by the algorithm.
    fn max_flow(&self) -> &G::EdgeData<'graph>;

    /// Returns the flow of each edge computed by the algorithm. The slice is indexed by the
    /// graph's edge indices.
    fn flows(&self) -> &[G::EdgeData<'graph>];

    /// Consumes the struct and returns a tuple of the maximum flow value and a vector of the flow
    /// of each edge. The vector is indexed by the graph's edge indices.
    fn into_max_flow_and_flow_vec(self) -> (G::EdgeData<'graph>, Vec<G::EdgeData<'graph>>);
}

/// Solves the [Max Flow Problem] from `source` to `destination`.
///
/// This function selects a maximum flow algorithm internally and may change which one it uses in future
/// releases as better implementations become available. If you need a specific algorithm's guarantees
/// (e.g. its exact complexity bounds, or reproducible behavior across versions), call that algorithm
/// directly instead. See the [`maximum_flow`][maximum_flow_mod] module for the full list.
///
/// Edge data of the provided graph is interpreted as capacities of edges.
///
/// # Arguments
/// - `network`: A directed graph with positive edge data which is interpreted as capacities of
///   edges.
/// - `source`: Source node for the flow.
/// - `destination`: Sink node for the flow.
///
/// # Returns
/// Returns an implementation of [`MaxFlowReturn`], from which the maximum flow value and the
/// flow of each edge can be obtained, either by reference (via [`max_flow`][MaxFlowReturn::max_flow]
/// / [`flows`][MaxFlowReturn::flows]) or by consuming it (via
/// [`into_max_flow_and_flow_vec`][MaxFlowReturn::into_max_flow_and_flow_vec]).
///
/// # Complexity
/// Currently delegates to [`dinics`], so its complexity bounds apply; this may change in future
/// releases as the underlying algorithm is swapped out. Use specific algorithms directly if you
/// depend on specific complexity guarantees.
///
/// [Max Flow Problem]: https://en.wikipedia.org/wiki/Maximum_flow_problem
/// [maximum_flow_mod]: index.html
///
/// # Example
/// ```rust
/// use petgraph::{Graph, algo::max_flow};
/// // Example from CLRS book
/// let mut graph = Graph::<u8, u8>::new();
/// let source = graph.add_node(0);
/// let _ = graph.add_node(1);
/// let _ = graph.add_node(2);
/// let _ = graph.add_node(3);
/// let _ = graph.add_node(4);
/// let destination = graph.add_node(5);
/// graph.extend_with_edges(&[
///     (0, 1, 16),
///     (0, 2, 13),
///     (1, 2, 10),
///     (1, 3, 12),
///     (2, 1, 4),
///     (2, 4, 14),
///     (3, 2, 9),
///     (3, 5, 20),
///     (4, 3, 7),
///     (4, 5, 4),
/// ]);
///
/// let result = max_flow(&graph, source, destination);
/// assert_eq!(&23, result.max_flow());
/// ```
pub fn max_flow<'graph, 'graph_ref, G>(
    network: &'graph_ref G,
    source: G::NodeId,
    destination: G::NodeId,
) -> impl MaxFlowReturn<'graph, G>
where
    G: Graph + DirectedGraph + 'graph,
    G::NodeId: IndexId,
    G::EdgeId: IndexId,
    G::EdgeData<'graph>: Sub<Output = G::EdgeData<'graph>> + Measure + Zero + Bounded + Ord,
    G::EdgeDataRef<'graph_ref>: Borrow<G::EdgeData<'graph>>,
{
    dinics(network, source, destination)
}

/// Returns the residual capacity of given edge.
fn residual_capacity<'graph, 'graph_ref, G: 'graph>(
    edge: Edge<G::EdgeId, G::EdgeData<'graph>, G::NodeId>,
    vertex: G::NodeId,
    flow: G::EdgeData<'graph>,
) -> G::EdgeData<'graph>
where
    G: DirectedGraph,
    G::EdgeData<'graph>: Sub<Output = G::EdgeData<'graph>>,
{
    if vertex == edge.source {
        // backward edge
        flow
    } else if vertex == edge.target {
        // forward edge
        edge.data - flow
    } else {
        panic!("Illegal endpoint {}", vertex);
    }
}

/// Gets the other endpoint of graph edge, if any, otherwise panics.
fn other_endpoint<G, D>(edge: Edge<G::EdgeId, D, G::NodeId>, vertex: G::NodeId) -> G::NodeId
where
    G: DirectedGraph,
{
    if vertex == edge.source {
        edge.target
    } else if vertex == edge.target {
        edge.source
    } else {
        panic!("Illegal endpoint {}", vertex);
    }
}

/// Returns the adjusted residual flow for given edge and flow increase.
fn adjusted_residual_flow<'graph, G: 'graph, D>(
    edge: Edge<G::EdgeId, D, G::NodeId>,
    target_vertex: G::NodeId,
    flow: G::EdgeData<'graph>,
    flow_increase: G::EdgeData<'graph>,
) -> G::EdgeData<'graph>
where
    G: DirectedGraph,
    G::EdgeData<'graph>: Sub<Output = G::EdgeData<'graph>> + Add<Output = G::EdgeData<'graph>>,
{
    if target_vertex == edge.source {
        // backward edge
        flow - flow_increase
    } else if target_vertex == edge.target {
        // forward edge
        flow + flow_increase
    } else {
        panic!("Illegal endpoint {}", target_vertex);
    }
}
