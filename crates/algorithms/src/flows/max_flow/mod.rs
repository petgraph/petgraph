//! Collection of algorithms for the [Maximum Flow Problem][max_flow_wikipedia].
//!
//! Using the `max_flow` function will select an algorithm. The algorithm(s) it uses
//! are subject to change in the future.
//!
//! Currently, `petgraph-algorithms` provides two algorithms to compute the maximum flow
//! in a flow network:
//! - [Dinic's Algorithm](dinics_mod::Dinics) [(Wikipedia)][dinics_wikipedia]
//! - [Edmonds-Karp Algorithm](edmonds_karp_mod::EdmondsKarp) [(Wikipedia)][edmonds_karp_wikipedia]
//!
//! They can be found in their respective submodules.
//!
//! Dinics and Edmonds-Karp have different time complexities, and
//! their performance can vary significantly depending on the input graph.
//! In general, Dinics is faster, especially on dense graphs, graphs with
//! unit capacities, and bipartite graphs.
//! Edmonds-Karp may be a better choice when working with small or
//! sparse graphs.
//!
//! For more information about each algorithm and their detailed time
//! complexity, check their respective documentation.
//!
//! [dinics_wikipedia]: https://en.wikipedia.org/wiki/Dinic%27s_algorithm
//! [edmonds_karp_wikipedia]: https://en.wikipedia.org/wiki/Edmonds%E2%80%93Karp_algorithm
//! [max_flow_wikipedia]: https://en.wikipedia.org/wiki/Maximum_flow_problem

pub mod dinics_mod;
pub mod edmonds_karp_mod;

use core::{
    error::Error,
    ops::{Add, Sub},
};

use dinics_mod::Dinics;
use petgraph_core::{
    edge::Edge,
    graph::{DirectedGraph, Graph, storable::StorableGraph},
};

use crate::traits::NonNegative;
#[cfg(feature = "alloc")]
use crate::traits::{Bounded, Measure, MinMax, Zero};

/// Errors that can occur in the configuration of max flow algorithms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaxFlowConfigError {
    SourceNodeNotSet,
    SinkNodeNotSet,
}

impl core::fmt::Display for MaxFlowConfigError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::SourceNodeNotSet => write!(fmt, "Source node is not set"),
            Self::SinkNodeNotSet => write!(fmt, "Sink node is not set"),
        }
    }
}

impl Error for MaxFlowConfigError {}

/// Output of max flow algorithms, containing the maximum flow value and the flow of each edge.
///
/// The wrapped data can be accessed using the provided getter methods, or by consuming the struct
/// with [`into_max_flow_and_flow_data`][Self::into_max_flow_and_flow_data].
pub struct MaxFlowOutput<G: Graph + StorableGraph, C: Clone> {
    max_flow: C,
    flows: G::EdgeDataContainer<C>,
}

impl<G: Graph + StorableGraph, C: Clone> MaxFlowOutput<G, C> {
    /// Returns the maximum flow value computed by the algorithm.
    pub const fn max_flow(&self) -> &C {
        &self.max_flow
    }

    /// Returns the flow of each edge computed by the algorithm in an `EdgeDataContainer`
    /// corresponding to the respective graph type.
    pub const fn flow_data(&self) -> &G::EdgeDataContainer<C> {
        &self.flows
    }

    /// Consumes the return value and returns the maximum flow value and the flow of each edge in an
    /// `DataContainer` corresponding to the respective graph type.
    pub fn into_max_flow_and_flow_data(self) -> (C, G::EdgeDataContainer<C>) {
        (self.max_flow, self.flows)
    }
}

/// Struct to run the maximum flow algorithm.
///
/// Uses the builder pattern to set parameters and can be run with the [`run`][Self::run] method.
/// Details of the algorithm can be found in [`run`][Self::run].
///
/// This struct selects a maximum flow algorithm internally and may change which one it uses
/// in future releases as better implementations become available. If you need a specific
/// algorithm's guarantees (e.g. its exact complexity bounds, or reproducible behavior across
/// versions), call that algorithm directly instead. See the [`maximum_flow`](./index.html) module
/// for the full list.
pub struct MaxFlow<'graph_ref, G: Graph, CapFn: Fn(G::EdgeId) -> C, C> {
    // Compare to implementing push relabel and using that as the wrapped max flow alg
    wrapped: Dinics<'graph_ref, G, CapFn, C>,
}

impl<'graph_ref, G>
    MaxFlow<'graph_ref, G, fn(G::EdgeId) -> G::EdgeData<'graph_ref>, G::EdgeData<'graph_ref>>
where
    G: DirectedGraph,
    G::EdgeData<'graph_ref>: Clone,
{
    /// Creates a new instance of the maximum flow algorithm with the provided graph.
    ///
    /// Data of the graph's edges is interpreted as capacities of edges.
    /// The source and sink nodes can be set using a builder pattern with the
    /// [`with_source`][Self::with_source] and [`with_sink`][Self::with_sink] methods.
    ///
    /// To run the algorithm, call the [`run`][Self::run] method on the returned struct.
    #[must_use]
    pub const fn new(
        network: &'graph_ref G,
    ) -> MaxFlow<
        'graph_ref,
        G,
        impl Fn(G::EdgeId) -> G::EdgeData<'graph_ref>,
        G::EdgeData<'graph_ref>,
    >
    where
        G: DirectedGraph,
        G::EdgeData<'graph_ref>: Clone,
    {
        MaxFlow {
            wrapped: Dinics::new(network),
        }
    }
}

impl<'graph_ref, G: Graph, CapFn: Fn(G::EdgeId) -> C, C> MaxFlow<'graph_ref, G, CapFn, C> {
    /// Creates a new instance of Dinic's algorithm with the provided graph and capacity function.
    ///
    /// The source and sink nodes can be set using a builder pattern with the
    /// [`with_source`][Self::with_source] and [`with_sink`][Self::with_sink] methods.
    ///
    /// To run the algorithm, call the [`run`][Self::run] method on the returned struct.
    #[must_use]
    pub const fn new_with_capacities(network: &'graph_ref G, capacities: CapFn) -> Self
    where
        G: DirectedGraph,
        CapFn: Fn(G::EdgeId) -> C,
    {
        Self {
            wrapped: Dinics::new_with_capacities(network, capacities),
        }
    }

    /// Sets the source node for the flow.
    #[must_use]
    pub const fn with_source(mut self, source: G::NodeId) -> Self {
        self.wrapped.source = Some(source);
        self
    }

    /// Sets the sink node for the flow.
    #[must_use]
    pub const fn with_sink(mut self, sink: G::NodeId) -> Self {
        self.wrapped.sink = Some(sink);
        self
    }
}

impl<G, CapFn: Fn(G::EdgeId) -> C, C> MaxFlow<'_, G, CapFn, C>
where
    G: DirectedGraph + StorableGraph,
{
    /// Computes a [Max Flow][maximum_flow] from `source` to `sink`.
    ///
    /// This function selects a maximum flow algorithm internally and may change which one it uses
    /// in future releases as better implementations become available. If you need a specific
    /// algorithm's guarantees (e.g. its exact complexity bounds, or reproducible behavior
    /// across versions), call that algorithm directly instead. See the
    /// [`maximum_flow`](./index.html) module for the full list.
    ///
    /// Edge data of the provided graph is interpreted as capacities of edges.
    ///
    /// # Arguments
    /// The following arguments need to be set in the algorithm struct before calling this method:
    /// - `source`: Source node for the flow.
    /// - `destination`: Sink node for the flow.
    /// - `capacities` (optional): Function that returns the non-negative capacity of an edge given
    ///   its ID. If not set, the edge data of the graph is used as capacity.
    ///
    /// # Returns
    /// Returns an instance of [`MaxFlowOutput`], from which the maximum flow value and the
    /// flow of each edge can be obtained.
    ///
    /// # Complexity
    /// Currently delegates to [`Dinics`](dinics_mod::Dinics::run), so its complexity bounds apply;
    /// this may change in future releases as the underlying algorithm is swapped out. Use
    /// specific algorithms directly if you depend on specific complexity guarantees.
    ///
    /// # Errors
    /// Returns an error if the source or sink node is not set.
    ///
    /// [maximum_flow]: https://en.wikipedia.org/wiki/Maximum_flow_problem
    ///
    /// # Example
    /// TODO: Add example
    #[cfg(feature = "alloc")]
    pub fn run(&self) -> Result<MaxFlowOutput<G, C>, MaxFlowConfigError>
    where
        G: DirectedGraph + StorableGraph,
        C: Sub<Output = C> + Measure + NonNegative + Zero + Bounded + MinMax + Default,
    {
        self.wrapped.run()
    }
}

/// Computes a [Max Flow][maximum_flow] from `source` to `sink`.
///
/// This function selects a maximum flow algorithm internally and may change which one it uses
/// in future releases as better implementations become available. If you need a specific
/// algorithm's guarantees (e.g. its exact complexity bounds, or reproducible behavior
/// across versions), call that algorithm directly instead. See the
/// [`maximum_flow`](./index.html) module for the full list.
///
/// For further details, see the [`run`](MaxFlow::run) method of the [`MaxFlow`] struct.
///
/// [maximum_flow]: https://en.wikipedia.org/wiki/Maximum_flow_problem
#[cfg(feature = "alloc")]
#[expect(clippy::missing_panics_doc, reason = "source and sink are always set")]
pub fn max_flow<G, C>(
    network: &G,
    source: G::NodeId,
    sink: G::NodeId,
    capacities: impl Fn(G::EdgeId) -> C,
) -> MaxFlowOutput<G, C>
where
    G: DirectedGraph + StorableGraph,
    C: Sub<Output = C> + Measure + NonNegative + Zero + Bounded + MinMax + Default,
{
    MaxFlow::new_with_capacities(network, capacities)
        .with_source(source)
        .with_sink(sink)
        .run()
        .expect("All required parameters should be set.")
}

/// Returns the residual capacity of given edge.
fn residual_capacity<G, C>(edge: Edge<G::EdgeId, C, G::NodeId>, vertex: G::NodeId, flow: C) -> C
where
    G: DirectedGraph,
    C: Sub<Output = C> + Copy + NonNegative,
{
    if vertex == edge.source {
        // backward edge
        flow
    } else if vertex == edge.target {
        // forward edge
        edge.data - flow
    } else {
        panic!("Illegal endpoint {vertex}");
    }
}

/// Gets the other endpoint of graph edge, if any, otherwise panics.
fn other_endpoint<G, D>(edge: &Edge<G::EdgeId, D, G::NodeId>, vertex: G::NodeId) -> G::NodeId
where
    G: DirectedGraph,
{
    if vertex == edge.source {
        edge.target
    } else if vertex == edge.target {
        edge.source
    } else {
        panic!("Illegal endpoint {vertex}");
    }
}

/// Returns the adjusted residual flow for given edge and flow increase.
fn adjusted_residual_flow<G, D, C>(
    edge: &Edge<G::EdgeId, D, G::NodeId>,
    target_vertex: G::NodeId,
    flow: C,
    flow_increase: C,
) -> C
where
    G: DirectedGraph,
    C: Sub<Output = C> + Add<Output = C> + NonNegative,
{
    if target_vertex == edge.source {
        // backward edge
        flow - flow_increase
    } else if target_vertex == edge.target {
        // forward edge
        flow + flow_increase
    } else {
        panic!("Illegal endpoint {target_vertex}");
    }
}
