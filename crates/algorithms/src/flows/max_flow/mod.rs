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

pub mod dinics_mod;
pub mod edmonds_karp_mod;

use core::{
    borrow::Borrow,
    error::Error,
    ops::{Add, Sub},
};

use dinics_mod::{Dinics, DinicsOutput};
use petgraph_core::{
    edge::Edge,
    graph::{DirectedGraph, Graph, storable::StorableGraph},
};

use crate::traits::{Bounded, Measure, MinMax, Zero};

/// Errors that can occur in the configuration of max flow algorithms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaxFlowConfigError {
    SourceNodeNotSet,
    DestinationNodeNotSet,
}

impl core::fmt::Display for MaxFlowConfigError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::SourceNodeNotSet => write!(fmt, "Source node is not set"),
            Self::DestinationNodeNotSet => write!(fmt, "Destination node is not set"),
        }
    }
}

impl Error for MaxFlowConfigError {}

/// The return trait that is implemented by Max Flow algorithms
pub trait MaxFlowReturn<G: Graph + StorableGraph, C: Clone> {
    /// Returns the maximum flow value computed by the algorithm.
    fn max_flow(&self) -> &C;

    /// Returns the flow of each edge computed by the algorithm in an `EdgeDataContainer`
    /// corresponding to the respective graph type.
    fn flows(&self) -> &G::EdgeDataContainer<C>;

    /// Consumes the return value and returns the maximum flow value and the flow of each edge in an
    /// `EdgeDataContainer` corresponding to the respective graph type.
    fn into_max_flow_and_flow_data(self) -> (C, G::EdgeDataContainer<C>);
}

/// Wrapper struct for the output of the `max_flow` function, which implements the `MaxFlowReturn`
/// trait.
pub struct MaxFlowOutput<G: Graph + StorableGraph, C: Clone>(DinicsOutput<G, C>);

impl<G: Graph + StorableGraph, C: Clone> MaxFlowReturn<G, C> for MaxFlowOutput<G, C> {
    fn max_flow(&self) -> &C {
        self.0.max_flow()
    }

    fn flows(&self) -> &G::EdgeDataContainer<C> {
        self.0.flows()
    }

    fn into_max_flow_and_flow_data(self) -> (C, G::EdgeDataContainer<C>) {
        self.0.into_max_flow_and_flow_data()
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
    wrapped: Dinics<'graph_ref, G, CapFn, C>,
}

impl<'graph_ref, G: Graph, CapFn: Fn(G::EdgeId) -> C, C> MaxFlow<'graph_ref, G, CapFn, C> {
    /// Creates a new instance of the maximum flow algorithm with the provided graph.
    ///
    /// Data of the graph's edges is interpreted as capacities of edges.
    /// The source and destination nodes can be set using a builder pattern with the
    /// [`with_source`][Self::with_source] and [`with_destination`][Self::with_destination] methods.
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
            wrapped: Dinics::new_with_capacities(network, |edge_id| {
                #[expect(clippy::missing_panics_doc, reason = "infallible")]
                network.edge(edge_id).unwrap().data.borrow().clone()
            }),
        }
    }

    /// Creates a new instance of Dinic's algorithm with the provided graph and capacity function.
    ///
    /// The source and destination nodes can be set using a builder pattern with the
    /// [`with_source`][Self::with_source] and [`with_destination`][Self::with_destination] methods.
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

    /// Sets the destination node for the flow.
    #[must_use]
    pub const fn with_destination(mut self, destination: G::NodeId) -> Self {
        self.wrapped.destination = Some(destination);
        self
    }
}

impl<G, CapFn: Fn(G::EdgeId) -> C, C> MaxFlow<'_, G, CapFn, C>
where
    G: DirectedGraph + StorableGraph,
{
    /// Computes a [Max Flow][maximum_flow] from `source` to `destination`.
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
    /// - `network`: A directed graph with positive edge data which is interpreted as capacities of
    ///   edges.
    /// - `source`: Source node for the flow.
    /// - `destination`: Sink node for the flow.
    ///
    /// # Returns
    /// Returns an implementation of [`MaxFlowReturn`], from which the maximum flow value and the
    /// flow of each edge can be obtained.
    ///
    /// # Complexity
    /// Currently delegates to [`dinics`], so its complexity bounds apply; this may change in future
    /// releases as the underlying algorithm is swapped out. Use specific algorithms directly if you
    /// depend on specific complexity guarantees.
    ///
    /// [maximum_flow]: https://en.wikipedia.org/wiki/Maximum_flow_problem
    ///
    /// # Example
    /// ```rust
    /// // TODO: Make sure this example works
    /// use petgraph_algorithms::flows::maximum_flow::{MaxFlowReturn, max_flow};
    /// use petgraph_core::utils::test_graphs::directed::DirectedTestGraph as Graph;
    ///
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
    /// let result = max_flow::<_, u8>(&graph, source, destination);
    /// assert_eq!(&23, result.max_flow());
    /// ```
    #[cfg(feature = "alloc")]
    pub fn run(&self) -> Result<MaxFlowOutput<G, C>, MaxFlowConfigError>
    where
        G: DirectedGraph + StorableGraph,
        // TODO: Consider switching to num_traits traits.
        C: Sub<Output = C> + Measure + Zero + Bounded + MinMax + Default,
    {
        Ok(MaxFlowOutput(self.wrapped.run()?))
    }
}

/// Returns the residual capacity of given edge.
fn residual_capacity<G, C>(edge: Edge<G::EdgeId, C, G::NodeId>, vertex: G::NodeId, flow: C) -> C
where
    G: DirectedGraph,
    C: Sub<Output = C> + Copy,
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
    C: Sub<Output = C> + Add<Output = C>,
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
