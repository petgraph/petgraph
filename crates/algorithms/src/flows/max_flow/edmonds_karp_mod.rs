use alloc::collections::VecDeque;
use core::{borrow::Borrow, ops::Sub};

use petgraph_core::{
    edge::Edge,
    graph::{
        DirectedGraph, Graph,
        storable::{DataContainer, StorableGraph, VisitContainer},
    },
};

use crate::{
    flows::max_flow::{
        MaxFlowConfigError, MaxFlowReturn, adjusted_residual_flow, other_endpoint,
        residual_capacity,
    },
    traits::{Bounded, Measure, Zero},
};

/// Struct to run the Edmonds-Karp algorithm.
///
/// Offers more configuration options than [`edmonds_karp`]. For an explanation of the algorithm,
/// see the documentation of [`edmonds_karp`].
pub struct EdmondsKarp<'graph_ref, G: Graph, CapFn: Fn(G::EdgeId) -> C, C> {
    network: &'graph_ref G,
    source: Option<G::NodeId>,
    destination: Option<G::NodeId>,
    capacities: CapFn,
}

impl<'graph_ref, G: Graph, CapFn: Fn(G::EdgeId) -> C, C> EdmondsKarp<'graph_ref, G, CapFn, C> {
    /// Creates a new instance of the Edmonds-Karp algorithm with the provided graph.
    ///
    /// Data of the graph's edges is interpreted as capacities of edges.
    /// The source and destination nodes can be set using a builder pattern with the
    /// [`with_source`][Self::with_source] and [`with_destination`][Self::with_destination] methods.
    ///
    /// To run the algorithm, call the [`run`][Self::run] method on the returned struct.
    #[must_use]
    pub const fn new(
        network: &'graph_ref G,
    ) -> EdmondsKarp<
        'graph_ref,
        G,
        impl Fn(G::EdgeId) -> G::EdgeData<'graph_ref>,
        G::EdgeData<'graph_ref>,
    >
    where
        G: DirectedGraph,
        G::EdgeData<'graph_ref>: Clone,
    {
        EdmondsKarp::new_with_capacities(network, |edge_id| {
            #[expect(clippy::missing_panics_doc, reason = "infallible")]
            network.edge(edge_id).unwrap().data.borrow().clone()
        })
    }

    /// Creates a new instance of the Edmonds-Karp algorithm with the provided graph and capacity
    /// function.
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
            network,
            source: None,
            destination: None,
            capacities,
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

impl<G, CapFn: Fn(G::EdgeId) -> C, C> EdmondsKarp<'_, G, CapFn, C>
where
    G: DirectedGraph + StorableGraph,
{
    /// Runs the Edmonds-Karp algorithm with the current configuration.
    ///
    /// For an explanation of the algorithm, see the documentation of [`edmonds_karp`].
    /// If an invalid configuration is detected, an appropriate error is returned.
    ///
    /// # Errors
    /// Returns an error if the source or destination node is not set.
    pub fn run(&self) -> Result<EdmondsKarpOutput<G, C>, MaxFlowConfigError>
    where
        C: Sub<Output = C> + Zero + Measure + Bounded + Default,
    {
        let source = self.source.ok_or(MaxFlowConfigError::SourceNodeNotSet)?;
        let destination = self
            .destination
            .ok_or(MaxFlowConfigError::DestinationNodeNotSet)?;
        Ok(edmonds_karp_inner(
            self.network,
            source,
            destination,
            &self.capacities,
        ))
    }
}

/// Output of the [`edmonds_karp`] algorithm.
///
/// The wrapped data can be accessed using the provided getter methods, or by consuming the struct
/// with [`EdmondsKarpOutput::into_max_flow_and_flow_data`].
pub struct EdmondsKarpOutput<G: Graph + StorableGraph, C: Clone> {
    max_flow: C,
    flows: G::EdgeDataContainer<C>,
}

impl<G: Graph + StorableGraph, C: Clone> MaxFlowReturn<G, C> for EdmondsKarpOutput<G, C> {
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
pub fn edmonds_karp<G, C>(
    network: &G,
    source: G::NodeId,
    destination: G::NodeId,
    capacities: impl Fn(G::EdgeId) -> C,
) -> EdmondsKarpOutput<G, C>
where
    G: DirectedGraph + StorableGraph,
    C: Sub<Output = C> + Measure + Zero + Bounded + Default,
{
    edmonds_karp_inner(network, source, destination, &capacities)
}

fn edmonds_karp_inner<G, C>(
    network: &G,
    source: G::NodeId,
    destination: G::NodeId,
    capacities: impl Fn(G::EdgeId) -> C,
) -> EdmondsKarpOutput<G, C>
where
    G: DirectedGraph + StorableGraph,
    C: Sub<Output = C> + Zero + Measure + Bounded + Default,
{
    let mut edge_to = network.node_data_container::<Option<Edge<G::EdgeId, C, G::NodeId>>>(None);
    let mut flows = network.edge_data_container::<C>(C::zero());
    let mut max_flow = C::zero();
    while has_augmented_path(
        network,
        source,
        destination,
        &capacities,
        &mut edge_to,
        &flows,
    ) {
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
fn has_augmented_path<G, C>(
    network: &G,
    source: G::NodeId,
    destination: G::NodeId,
    capacities: impl Fn(G::EdgeId) -> C,
    edge_to: &mut G::NodeDataContainer<Option<Edge<G::EdgeId, C, G::NodeId>>>,
    flows: &G::EdgeDataContainer<C>,
) -> bool
where
    G: DirectedGraph + StorableGraph,
    C: Sub<Output = C> + Zero + Measure + Default,
{
    let mut visited = network.node_visit_container();
    let mut queue = VecDeque::new();
    visited.mark_visited(source);
    queue.push_back(source);

    while let Some(vertex) = queue.pop_front() {
        for edge in network.incident_edges(vertex) {
            let next = other_endpoint::<G, _>(&edge, vertex);
            let edge_index = edge.id;
            let capacity = capacities(edge_index);
            let edge = Edge {
                id: edge_index,
                source: edge.source,
                target: edge.target,
                data: capacity,
            };
            let residual_cap = residual_capacity::<G, C>(edge, next, *flows.get(edge_index));
            if !visited.is_visited(next) && (residual_cap > C::zero()) {
                visited.mark_visited(next);
                edge_to.get_mut(next).replace(edge);
                if destination == next {
                    return true;
                }
                queue.push_back(next);
            }
        }
    }
    false
}
