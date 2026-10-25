#[cfg(feature = "alloc")]
use alloc::collections::VecDeque;
use core::{borrow::Borrow, ops::Sub};

use petgraph_core::{
    edge::Edge,
    graph::{
        DirectedGraph, Graph,
        storable::{DataContainer, QueueContainer, StorableGraph, VisitContainer},
    },
};

use crate::{
    flows::max_flow::{
        MaxFlowConfigError, MaxFlowOutput, adjusted_residual_flow, other_endpoint,
        residual_capacity,
    },
    traits::{Bounded, Measure, NonNegative, Zero},
};

/// Struct to run the Edmonds-Karp algorithm.
///
/// Offers more configuration options than [`edmonds_karp`]. For an explanation of the algorithm,
/// see the documentation of [`edmonds_karp`].
pub struct EdmondsKarp<'graph_ref, G: Graph, CapFn: Fn(G::EdgeId) -> C, C> {
    network: &'graph_ref G,
    source: Option<G::NodeId>,
    sink: Option<G::NodeId>,
    capacities: CapFn,
}

impl<'graph_ref, G>
    EdmondsKarp<'graph_ref, G, fn(G::EdgeId) -> G::EdgeData<'graph_ref>, G::EdgeData<'graph_ref>>
where
    G: DirectedGraph,
    G::EdgeData<'graph_ref>: Clone,
{
    /// Creates a new instance of the Edmonds-Karp algorithm with the provided graph.
    ///
    /// Data of the graph's edges is interpreted as capacities of edges.
    /// The source and sink nodes can be set using a builder pattern with the
    /// [`with_source`][Self::with_source] and [`with_sink`][Self::with_sink] methods.
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
}

impl<'graph_ref, G: Graph, CapFn: Fn(G::EdgeId) -> C, C> EdmondsKarp<'graph_ref, G, CapFn, C> {
    /// Creates a new instance of the Edmonds-Karp algorithm with the provided graph and capacity
    /// function.
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
            network,
            source: None,
            sink: None,
            capacities,
        }
    }

    /// Sets the source node for the flow.
    #[must_use]
    pub const fn with_source(mut self, source: G::NodeId) -> Self {
        self.source = Some(source);
        self
    }

    /// Sets the sink node for the flow.
    #[must_use]
    pub const fn with_sink(mut self, sink: G::NodeId) -> Self {
        self.sink = Some(sink);
        self
    }
}

impl<G, CapFn: Fn(G::EdgeId) -> C, C> EdmondsKarp<'_, G, CapFn, C>
where
    G: DirectedGraph + StorableGraph,
{
    /// Find a [Maximum Flow][maximum_flow] from `source` to `sink` using the
    /// [Edmonds-Karp][edmonds_karp_link] implementation of the [Ford-Fulkerson][ford_fulkerson]
    /// method.
    ///
    /// Edge Data of the provided graph is interpreted as capacities of edges.
    ///
    /// See also [`maximum_flow`](../index.html) module for other maximum flow algorithms.
    ///
    /// # Arguments
    /// The following arguments need to be set in the algorithm struct before calling this method:
    /// - `source`: Source node for the flow.
    /// - `destination`: Sink node for the flow.
    /// - `capacities` (optional): Function that returns the non-negative capacity of an edge given
    ///   its ID. If not set, the edge data of the graph is used as capacity.
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
    /// # Errors
    /// Returns an error if the source or sink node is not set.
    ///
    /// # Example
    /// // TODO: Add example
    ///
    /// [maximum_flow]: https://en.wikipedia.org/wiki/Maximum_flow_problem
    /// [ford_fulkerson]: https://en.wikipedia.org/wiki/Ford%E2%80%93Fulkerson_algorithm
    /// [edmonds_karp_link]: https://en.wikipedia.org/wiki/Edmonds%E2%80%93Karp_algorithm
    #[cfg(feature = "alloc")]
    pub fn run(&self) -> Result<MaxFlowOutput<G, C>, MaxFlowConfigError>
    where
        C: Sub<Output = C> + Zero + Measure + NonNegative + Bounded + Default,
    {
        self.run_with_queue_container(|| VecDeque::new())
    }

    /// Run the [Edmonds-Karp algorithm][edmonds_karp_link] implementation of the
    /// [Ford-Fulkerson][ford_fulkerson] method to find a [Maximum Flow][maximum_flow] from `source`
    /// to `sink` using a custom queue container.
    ///
    /// For further details, see the [`run`](EdmondsKarp::run) method of the
    /// [EdmondsKarp] struct.
    ///
    /// [maximum_flow]: https://en.wikipedia.org/wiki/Maximum_flow_problem
    /// [ford_fulkerson]: https://en.wikipedia.org/wiki/Ford%E2%80%93Fulkerson_algorithm
    /// [edmonds_karp_link]: https://en.wikipedia.org/wiki/Edmonds%E2%80%93Karp_algorithm
    pub fn run_with_queue_container<QContainer>(
        &self,
        queue_container_node_id_constructor: impl Fn() -> QContainer,
    ) -> Result<MaxFlowOutput<G, C>, MaxFlowConfigError>
    where
        C: Sub<Output = C> + Zero + Measure + NonNegative + Bounded + Default,
        QContainer: QueueContainer<G::NodeId>,
    {
        let source = self.source.ok_or(MaxFlowConfigError::SourceNodeNotSet)?;
        let sink = self.sink.ok_or(MaxFlowConfigError::SinkNodeNotSet)?;
        Ok(edmonds_karp_inner(
            self.network,
            source,
            sink,
            &self.capacities,
            queue_container_node_id_constructor,
        ))
    }
}

/// Run the [Edmonds-Karp algorithm][edmonds_karp_link] implementation of the
/// [Ford-Fulkerson][ford_fulkerson] method to find a [Maximum Flow][maximum_flow] from `source` to
/// `sink`.
///
/// For further details, see the [`run`](EdmondsKarp::run) method of the [EdmondsKarp]
/// struct.
///
/// [maximum_flow]: https://en.wikipedia.org/wiki/Maximum_flow_problem
/// [ford_fulkerson]: https://en.wikipedia.org/wiki/Ford%E2%80%93Fulkerson_algorithm
/// [edmonds_karp_link]: https://en.wikipedia.org/wiki/Edmonds%E2%80%93Karp_algorithm
#[cfg(feature = "alloc")]
pub fn edmonds_karp<G, C>(
    network: &G,
    source: G::NodeId,
    sink: G::NodeId,
    capacities: impl Fn(G::EdgeId) -> C,
) -> MaxFlowOutput<G, C>
where
    G: DirectedGraph + StorableGraph,
    C: Sub<Output = C> + Measure + NonNegative + Zero + Bounded + Default,
{
    EdmondsKarp::new_with_capacities(network, capacities)
        .with_source(source)
        .with_sink(sink)
        .run()
        .unwrap()
}

fn edmonds_karp_inner<G, C, QContainer>(
    network: &G,
    source: G::NodeId,
    sink: G::NodeId,
    capacities: impl Fn(G::EdgeId) -> C,
    queue_container_node_id_constructor: impl Fn() -> QContainer,
) -> MaxFlowOutput<G, C>
where
    G: DirectedGraph + StorableGraph,
    C: Sub<Output = C> + Zero + Measure + NonNegative + Bounded + Default,
    QContainer: QueueContainer<G::NodeId>,
{
    let mut edge_to = network.node_data_container::<Option<Edge<G::EdgeId, C, G::NodeId>>>(None);
    let mut flows = network.edge_data_container::<C>(C::zero());
    let mut max_flow = C::zero();
    while has_augmented_path(
        network,
        source,
        sink,
        &capacities,
        &mut edge_to,
        &flows,
        &queue_container_node_id_constructor,
    ) {
        let mut path_flow = <C as Bounded>::max();

        // Find the bottleneck capacity of the path
        let mut vertex = sink;
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
        let mut vertex = sink;
        while let Some(edge) = edge_to.get(vertex) {
            *flows.get_mut(edge.id) =
                adjusted_residual_flow::<G, _, C>(edge, vertex, *flows.get(edge.id), path_flow);
            vertex = other_endpoint::<G, _>(edge, vertex);
        }
        max_flow = max_flow + path_flow;
    }
    MaxFlowOutput { max_flow, flows }
}

/// Returns whether there is an augmenting path in the graph
#[allow(clippy::type_complexity)]
fn has_augmented_path<G, C, QContainer>(
    network: &G,
    source: G::NodeId,
    sink: G::NodeId,
    capacities: impl Fn(G::EdgeId) -> C,
    edge_to: &mut G::NodeDataContainer<Option<Edge<G::EdgeId, C, G::NodeId>>>,
    flows: &G::EdgeDataContainer<C>,
    queue_container_node_id_constructor: impl Fn() -> QContainer,
) -> bool
where
    G: DirectedGraph + StorableGraph,
    C: Sub<Output = C> + NonNegative + Zero + Measure + Default,
    QContainer: QueueContainer<G::NodeId>,
{
    let mut visited = network.node_visit_container();
    let mut queue = queue_container_node_id_constructor();
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
                if sink == next {
                    return true;
                }
                queue.push_back(next);
            }
        }
    }
    false
}
