use alloc::collections::VecDeque;
use core::{
    borrow::Borrow,
    error::Error,
    fmt::{Display, Formatter},
    ops::{Add, Sub},
};

use petgraph_core::{
    edge::Edge,
    graph::{
        DirectedGraph, Graph,
        storable::{DataContainer, StorableGraph},
    },
    id::IndexId,
};

use crate::{
    flows::max_flow::{MaxFlowReturn, adjusted_residual_flow, other_endpoint, residual_capacity},
    traits::{Bounded, Measure, Zero},
};

/// Errors that can occur in the configuration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdmondsKarpConfigError {
    SourceNodeNotSet,
    DestinationNodeNotSet,
}

impl Display for EdmondsKarpConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::SourceNodeNotSet => write!(f, "Source node is not set"),
            Self::DestinationNodeNotSet => {
                write!(f, "Destination node is not set")
            }
        }
    }
}

impl Error for EdmondsKarpConfigError {}

/// Struct to run the Edmonds-Karp algorithm.
///
/// Offers more configuration options than [`edmonds_karp`]. For an explanation of the algorithm,
/// see the documentation of [`edmonds_karp`].
pub struct EdmondsKarp<'graph_ref, G: Graph> {
    network: &'graph_ref G,
    source: Option<G::NodeId>,
    destination: Option<G::NodeId>,
}

impl<'graph_ref, G: Graph> EdmondsKarp<'graph_ref, G> {
    /// Creates a new instance of the Edmonds-Karp algorithm with the provided graph.
    ///
    /// The source and destination nodes can be set using a builder pattern with the `with_source`
    /// and `with_destination` methods.
    #[must_use]
    pub const fn new(network: &'graph_ref G) -> Self {
        Self {
            network,
            source: None,
            destination: None,
        }
    }

    /// Sets the source node for the flow.
    #[must_use]
    pub const fn with_source(mut self, source: G::NodeId) -> Self {
        self.source = Some(source);
        self
    }

    /// Sets the destination node for the flow.
    #[must_use]
    pub const fn with_destination(mut self, destination: G::NodeId) -> Self {
        self.destination = Some(destination);
        self
    }
}

impl<'graph_ref, G> EdmondsKarp<'graph_ref, G>
where
    G: DirectedGraph + StorableGraph,
    G::NodeId: IndexId,
    G::EdgeId: IndexId,
{
    /// Runs the Edmonds-Karp algorithm with the current configuration.
    ///
    /// For an explanation of the algorithm, see the documentation of [`edmonds_karp`].
    /// If an invalid configuration is detected, an appropriate error is returned.
    ///
    /// # Errors
    /// Returns an error if the source or destination node is not set.
    pub fn run<C>(&self) -> Result<EdmondsKarpOutput<G, C>, EdmondsKarpConfigError>
    where
        G::EdgeDataRef<'graph_ref>: Borrow<C> + Copy,
        C: Sub<Output = C> + Zero + Measure + Bounded,
    {
        let source = self
            .source
            .ok_or(EdmondsKarpConfigError::SourceNodeNotSet)?;
        let destination = self
            .destination
            .ok_or(EdmondsKarpConfigError::DestinationNodeNotSet)?;
        Ok(edmonds_karp_inner(self.network, source, destination))
    }
}

/// Output of the [`edmonds_karp`] algorithm.
///
/// The wrapped data can be accessed using the provided getter methods, or by consuming the struct
/// with [`EdmondsKarpOutput::into_max_flow_and_flow_data`].
pub struct EdmondsKarpOutput<G: Graph + StorableGraph, C: Default> {
    max_flow: C,
    flows: G::EdgeDataContainer<C>,
}

impl<G: Graph + StorableGraph, C: Default> MaxFlowReturn<G, C> for EdmondsKarpOutput<G, C> {
    fn max_flow(&self) -> &C {
        &self.max_flow
    }

    fn flows(&self) -> &G::EdgeDataContainer<C> {
        &self.flows
    }

    fn into_max_flow_and_flow_data(self) -> (C, G::EdgeDataContainer<C>) {
        (self.max_flow, self.flows)
    }
}

/// Find a [Maximum Flow][maximum_flow] from `source` to `destination` using the
/// [Edmond-Karp][edmonds_karp] implementation of the [Ford-Fulkerson][ford_fulkerson] method.
///
/// Edge Data of the provided graph is interpreted as capacities of edges.
///
/// See also [`maximum_flow`](../index.html) module for other maximum flow algorithms.
///
/// # Arguments
/// - `network`: A directed graph with positive edge data which is interpreted as capacities of
///   edges.
/// - `source`: Source node for the flow.
/// - `destination`: Sink node for the flow.
///
/// # Returns
/// Returns a struct wrapping the maximum flow value and the flow of each edge.
///
/// # Complexity
/// - Time complexity: **O(|V||E|²)**
/// - Auxiliary space: **O(|V| + |E|)**
///
/// Where **|V|** is the number of nodes and **|E|** is the number of edges.
///
/// [maximum_flow]: https://en.wikipedia.org/wiki/Maximum_flow_problem
/// [ford_fulkerson]: https://en.wikipedia.org/wiki/Ford%E2%80%93Fulkerson_algorithm
/// [edmonds_karp]: https://en.wikipedia.org/wiki/Edmonds%E2%80%93Karp_algorithm
///
/// # Example
/// // TODO
pub fn edmonds_karp<'graph_ref, G, C>(
    network: &'graph_ref G,
    source: G::NodeId,
    destination: G::NodeId,
) -> EdmondsKarpOutput<G, C>
where
    G: DirectedGraph + StorableGraph,
    G::NodeId: IndexId,
    G::EdgeId: IndexId,
    G::EdgeDataRef<'graph_ref>: Borrow<C> + Copy,
    C: Sub<Output = C> + Add<Output = C> + Zero + Measure + Bounded,
{
    edmonds_karp_inner(network, source, destination)
}

fn edmonds_karp_inner<'graph_ref, G, C>(
    network: &'graph_ref G,
    source: G::NodeId,
    destination: G::NodeId,
) -> EdmondsKarpOutput<G, C>
where
    G: DirectedGraph + StorableGraph,
    G::NodeId: IndexId,
    G::EdgeId: IndexId,
    G::EdgeDataRef<'graph_ref>: Borrow<C> + Copy,
    C: Sub<Output = C> + Zero + Measure + Bounded,
{
    let mut edge_to = G::node_data_container::<Option<Edge<G::EdgeId, C, G::NodeId>>>(network);
    let mut flows = G::edge_data_container::<C>(network);
    let mut max_flow = C::zero();
    while has_augmented_path(network, source, destination, &mut edge_to, &flows) {
        let mut path_flow = <C as Bounded>::max();

        // Find the bottleneck capacity of the path
        let mut vertex = destination;
        while let Some(edge) = edge_to.get(vertex) {
            let residual_capacity =
                residual_capacity::<G, C>(edge.to_owned_edge(), vertex, *flows.get(edge.id));
            // Minimum between the current path flow and the residual capacity.
            path_flow = if path_flow > residual_capacity {
                residual_capacity
            } else {
                path_flow
            };
            vertex = other_endpoint::<G, _>(edge, vertex);
        }

        // Update the flow of each edge along the path
        let mut vertex = destination;
        while let Some(edge) = edge_to.get(vertex) {
            *flows.get_mut(edge.id) =
                adjusted_residual_flow::<G, _, C>(edge, vertex, *flows.get(edge.id), path_flow);
            vertex = other_endpoint::<G, _>(edge, vertex);
        }
        max_flow = max_flow + path_flow;
    }
    EdmondsKarpOutput { max_flow, flows }
}

/// Returns whether there is an augmenting path in the graph
#[allow(clippy::type_complexity)]
fn has_augmented_path<'graph_ref, G, C>(
    network: &'graph_ref G,
    source: G::NodeId,
    destination: G::NodeId,
    edge_to: &mut G::NodeDataContainer<Option<Edge<G::EdgeId, C, G::NodeId>>>,
    flows: &G::EdgeDataContainer<C>,
) -> bool
where
    G: DirectedGraph + StorableGraph,
    G::NodeId: IndexId,
    G::EdgeId: IndexId,
    G::EdgeDataRef<'graph_ref>: Borrow<C> + Copy,
    C: Sub<Output = C> + Zero + Measure,
{
    // TODO(next): Replace by proper visit map
    let mut visited = vec![false; network.node_count()];
    let mut queue = VecDeque::new();
    visited[source.as_usize()] = true;
    queue.push_back(source);

    while let Some(vertex) = queue.pop_front() {
        for edge in network.incident_edges(vertex) {
            let next = other_endpoint::<G, _>(&edge, vertex);
            let edge_index = edge.id;
            let residual_cap =
                residual_capacity::<G, C>(edge.to_owned_edge(), next, *flows.get(edge_index));
            if !visited[next.as_usize()] && (residual_cap > C::zero()) {
                visited[next.as_usize()] = true;
                *edge_to.get_mut(next) = Some(edge.to_owned_edge());
                if destination == next {
                    return true;
                }
                queue.push_back(next);
            }
        }
    }
    false
}
