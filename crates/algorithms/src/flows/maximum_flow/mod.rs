//! Collection of algorithms for the [Maximum Flow Problem][max_flow_wikipedia].
//!
//!
//!
//! Currently, `petgraph` provides two algorithms to compute the maximum flow
//! in a flow network:
//! - [Dinic's Algorithm][dinics_wikipedia]
//! - [Edmonds-Karp Algorithm][edmonds_karp_wikipedia]
//! They are implemented in the functions [`dinics`] and [`ford_fulkerson`] and can be found
//! in their respective submodules.
//!
//! [Dinics] and [Edmonds] have different time complexities, and
//! their performance can vary significantly depending on the input graph.
//! In general, [dinics] is faster, especially on dense graphs, graphs with
//! unit capacities, and bipartite graphs.
//! [ford_fulkerson] may be a better choice when working with small or
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
mod ford_fulkerson;

use std::ops::{Deref, Sub};

#[cfg(feature = "alloc")]
pub use dinics::dinics;
#[cfg(feature = "alloc")]
pub use ford_fulkerson::ford_fulkerson;
use petgraph_core::{edge::EdgeRef, graph::DirectedGraph};

use crate::traits::Measure;

/// Returns the residual capacity of given edge.
fn residual_capacity<'graph, G: 'graph>(
    edge: EdgeRef<'graph, G>,
    vertex: G::NodeId,
    flow: G::EdgeData<'graph>,
) -> G::EdgeData<'graph>
where
    G: DirectedGraph,
    G::EdgeData<'graph>: Sub<Output = G::EdgeData<'graph>> + Measure,
    // For Review: Not sure if Deref trait bound makes sense here, or if this should be handled
    // differently
    G::EdgeDataRef<'graph>: Deref<Target = G::EdgeData<'graph>>,
{
    if vertex == edge.source {
        // backward edge
        flow
    } else if vertex == edge.target {
        // forward edge
        *edge.data - flow
    } else {
        panic!("Illegal endpoint {}", vertex);
    }
}

/// Gets the other endpoint of graph edge, if any, otherwise panics.
fn other_endpoint<'graph, G: 'graph>(edge: EdgeRef<'graph, G>, vertex: G::NodeId) -> G::NodeId
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
