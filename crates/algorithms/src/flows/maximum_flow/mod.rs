//! Collection of algorithms for the [Maximum Flow Problem][max_flow_wikipedia].
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
mod dinics;
#[cfg(feature = "alloc")]
mod edmonds_karp;

use std::ops::Sub;

#[cfg(feature = "alloc")]
pub use dinics::dinics;
#[cfg(feature = "alloc")]
pub use edmonds_karp::edmonds_karp;
use petgraph_core::{edge::Edge, graph::DirectedGraph};

use crate::traits::Measure;

/// Returns the residual capacity of given edge.
fn residual_capacity<'graph, 'graph_ref, G: 'graph>(
    edge: Edge<G::EdgeId, G::EdgeData<'graph>, G::NodeId>,
    vertex: G::NodeId,
    flow: G::EdgeData<'graph>,
) -> G::EdgeData<'graph>
where
    G: DirectedGraph,
    G::EdgeData<'graph>: Sub<Output = G::EdgeData<'graph>> + Measure,
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
