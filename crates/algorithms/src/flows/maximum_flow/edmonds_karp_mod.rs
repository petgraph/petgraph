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
        storable::{DataContainer, QueueContainer, Storable},
    },
    id::IndexId,
};

use crate::{
    flows::maximum_flow::{
        MaxFlowReturn, adjusted_residual_flow, other_endpoint, residual_capacity,
    },
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
            EdmondsKarpConfigError::SourceNodeNotSet => write!(f, "Source node is not set"),
            EdmondsKarpConfigError::DestinationNodeNotSet => {
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
    pub fn new(network: &'graph_ref G) -> Self {
        Self {
            network,
            source: None,
            destination: None,
        }
    }

    /// Sets the source node for the flow.
    pub fn with_source(mut self, source: G::NodeId) -> Self {
        self.source = Some(source);
        self
    }

    /// Sets the destination node for the flow.
    pub fn with_destination(mut self, destination: G::NodeId) -> Self {
        self.destination = Some(destination);
        self
    }
}

impl<'graph, 'graph_ref, G: 'graph> EdmondsKarp<'graph_ref, G>
where
    G: DirectedGraph + Storable,
    G::NodeId: IndexId,
    G::EdgeId: IndexId,
    G::EdgeData<'graph>: Sub<Output = G::EdgeData<'graph>> + Zero + Measure + Bounded,
    G::EdgeDataRef<'graph_ref>: Borrow<G::EdgeData<'graph>> + Copy,
{
    /// Runs the Edmonds-Karp algorithm with the current configuration.
    ///
    /// For an explanation of the algorithm, see the documentation of [`edmonds_karp`].
    /// If an invalid configuration is detected, an appropriate error is returned.
    pub fn run(&self) -> Result<EdmondsKarpOutput<'graph, G>, EdmondsKarpConfigError> {
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
/// with [`EdmondsKarpOutput::into_max_flow_and_flows`].
pub struct EdmondsKarpOutput<'graph, G: Graph + Storable + 'graph>
where
    G::EdgeData<'graph>: Default,
{
    max_flow: G::EdgeData<'graph>,
    flows: G::EdgeDataContainer<G::EdgeData<'graph>>,
}

impl<'graph, G: Graph + Storable + 'graph> MaxFlowReturn<'graph, G> for EdmondsKarpOutput<'graph, G>
where
    G::EdgeData<'graph>: Default,
{
    fn max_flow(&self) -> &G::EdgeData<'graph> {
        &self.max_flow
    }

    fn flows(&self) -> &G::EdgeDataContainer<G::EdgeData<'graph>> {
        &self.flows
    }

    fn into_max_flow_and_flow_data(
        self,
    ) -> (
        G::EdgeData<'graph>,
        G::EdgeDataContainer<G::EdgeData<'graph>>,
    ) {
        (self.max_flow, self.flows)
    }
}

/// Find a [maximum_flow_problem] from `source` to `destination` using the
/// [Edmond-Karp][edmonds_karp] implementation of the [Ford-Fulkerson][ford_fulkerson] method. Edge
/// Data of the provided graph is interpreted as capacities of edges.
///
/// See also [`maximum_flow`][maximum_flow_mod] module for other maximum flow algorithms.
///
/// # Arguments
/// - `network`: Directed graph where edge data are capacities of edges.
/// - `source`: Source node for the flow.
/// - `destination`: Sink node for the flow.
///
/// # Returns
/// Returns a struct wrapping the maximum flow value and the flow of each edge.
///
/// # Complexity
/// - Time: **O(|V||E|²)**.
/// - Auxiliary space: **O(|V| + |E|)**.
///
/// where **|V|** is the number of nodes and **|E|** is the number of edges.
///
/// [maximum_flow_problem]: https://en.wikipedia.org/wiki/Maximum_flow_problem
/// [ford_fulkerson]: https://en.wikipedia.org/wiki/Ford%E2%80%93Fulkerson_algorithm
/// [edmonds_karp]: https://en.wikipedia.org/wiki/Edmonds%E2%80%93Karp_algorithm
/// [maximum_flow_mod]: index.html
///
/// # Example
/// ```rust
/// // use petgraph::{Graph, algo::ford_fulkerson};
/// // // Example from CLRS book
/// // let mut graph = Graph::<u8, u8>::new();
/// // let source = graph.add_node(0);
/// // let _ = graph.add_node(1);
/// // let _ = graph.add_node(2);
/// // let _ = graph.add_node(3);
/// // let _ = graph.add_node(4);
/// // let destination = graph.add_node(5);
/// // graph.extend_with_edges(&[
/// //     (0, 1, 16),
/// //     (0, 2, 13),
/// //     (1, 2, 10),
/// //     (1, 3, 12),
/// //     (2, 1, 4),
/// //     (2, 4, 14),
/// //     (3, 2, 9),
/// //     (3, 5, 20),
/// //     (4, 3, 7),
/// //     (4, 5, 4),
/// // ]);
/// // let (max_flow, _) = ford_fulkerson(&graph, source, destination);
/// // assert_eq!(23, max_flow);
/// ```
pub fn edmonds_karp<'graph, 'graph_ref, G: 'graph>(
    network: &'graph_ref G,
    source: G::NodeId,
    destination: G::NodeId,
) -> EdmondsKarpOutput<'graph, G>
where
    G: DirectedGraph + Storable,
    G::NodeId: IndexId,
    G::EdgeId: IndexId,
    G::EdgeData<'graph>: Sub<Output = G::EdgeData<'graph>>
        + Add<Output = G::EdgeData<'graph>>
        + Zero
        + Measure
        + Bounded,
    G::EdgeDataRef<'graph_ref>: Borrow<G::EdgeData<'graph>> + Copy,
{
    edmonds_karp_inner(network, source, destination)
}

fn edmonds_karp_inner<'graph, 'graph_ref, G: 'graph>(
    network: &'graph_ref G,
    source: G::NodeId,
    destination: G::NodeId,
) -> EdmondsKarpOutput<'graph, G>
where
    G: DirectedGraph + Storable,
    G::NodeId: IndexId,
    G::EdgeId: IndexId,
    G::EdgeData<'graph>: Sub<Output = G::EdgeData<'graph>> + Zero + Measure + Bounded,
    G::EdgeDataRef<'graph_ref>: Borrow<G::EdgeData<'graph>> + Copy,
{
    let mut edge_to =
        G::node_data_container::<Option<Edge<G::EdgeId, G::EdgeData<'graph>, G::NodeId>>>(network);
    let mut flows = G::edge_data_container::<G::EdgeData<'graph>>(network);
    let mut max_flow = G::EdgeData::zero();
    while has_augmented_path(network, source, destination, &mut edge_to, &flows) {
        let mut path_flow = G::EdgeData::max();

        // Find the bottleneck capacity of the path
        let mut vertex = destination;
        while let Some(edge) = edge_to.get(vertex) {
            let residual_capacity =
                residual_capacity::<G>(edge.to_owned_edge(), vertex, *flows.get(edge.id));
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
                adjusted_residual_flow::<G, _>(edge, vertex, *flows.get(edge.id), path_flow);
            vertex = other_endpoint::<G, _>(edge, vertex);
        }
        max_flow = max_flow + path_flow;
    }
    EdmondsKarpOutput { max_flow, flows }
}

/// Returns whether there is an augmenting path in the graph
fn has_augmented_path<'graph, 'graph_ref, G: 'graph>(
    network: &'graph_ref G,
    source: G::NodeId,
    destination: G::NodeId,
    edge_to: &mut G::NodeDataContainer<Option<Edge<G::EdgeId, G::EdgeData<'graph>, G::NodeId>>>,
    flows: &G::EdgeDataContainer<G::EdgeData<'graph>>,
) -> bool
where
    G: DirectedGraph + Storable,
    G::NodeId: IndexId,
    G::EdgeId: IndexId,
    G::EdgeData<'graph>: Sub<Output = G::EdgeData<'graph>> + Zero + Measure,
    G::EdgeDataRef<'graph_ref>: Borrow<G::EdgeData<'graph>> + Copy,
{
    // TODO(next): Replace by proper visit map
    let mut visited = vec![false; network.node_count()];
    let mut queue = G::queue_container::<G::NodeId>(network);
    visited[source.as_usize()] = true;
    queue.push_back(source);

    while let Some(vertex) = queue.pop_front() {
        for edge in network.incident_edges(vertex) {
            let next = other_endpoint::<G, _>(&edge, vertex);
            let edge_index = edge.id;
            let residual_cap =
                residual_capacity::<G>(edge.to_owned_edge(), next, *flows.get(edge_index));
            if !visited[next.as_usize()] && (residual_cap > <G::EdgeData<'graph>>::zero()) {
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
