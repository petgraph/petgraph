// TODO: Get rid of alloc here (replace level_edges somehow)
use alloc::{vec, vec::Vec};
use core::{borrow::Borrow, error::Error, ops::Sub};

use petgraph_core::{
    edge::Edge,
    graph::{
        DirectedGraph, Graph,
        storable::{DataContainer, QueueContainer, StackContainer, Storable, VisitContainer},
    },
    id::IndexId,
};

use super::{other_endpoint, residual_capacity};
use crate::{
    flows::max_flow::{MaxFlowReturn, adjusted_residual_flow},
    traits::{Bounded, Measure, Zero},
};

/// Errors that can occur in the configuration of Dinic's algorithm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DinicsConfigError {
    SourceNodeNotSet,
    DestinationNodeNotSet,
}

impl core::fmt::Display for DinicsConfigError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::SourceNodeNotSet => write!(fmt, "Source node is not set"),
            Self::DestinationNodeNotSet => write!(fmt, "Destination node is not set"),
        }
    }
}

impl Error for DinicsConfigError {}

/// Edge with its capacity copied out of the graph.
type FlowEdge<G, C> = Edge<<G as Graph>::EdgeId, C, <G as Graph>::NodeId>;

/// Struct to run Dinic's algorithm.
///
/// Offers more configuration options than [`dinics`]. For an explanation of the algorithm, see the
/// documentation of [`dinics`].
pub struct Dinics<'graph_ref, G: Graph> {
    network: &'graph_ref G,
    source: Option<G::NodeId>,
    destination: Option<G::NodeId>,
}

impl<'graph_ref, G: Graph> Dinics<'graph_ref, G> {
    /// Creates a new instance of Dinic's algorithm with the provided graph.
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

impl<'graph_ref, G> Dinics<'graph_ref, G>
where
    G: DirectedGraph + Storable,
    G::NodeId: IndexId,
    G::EdgeId: IndexId,
{
    /// Runs Dinic's algorithm with the current configuration.
    ///
    /// For an explanation of the algorithm, see the documentation of [`dinics`].
    /// If an invalid configuration is detected, an appropriate error is returned.
    pub fn run<C>(&self) -> Result<DinicsOutput<G, C>, DinicsConfigError>
    where
        G::EdgeDataRef<'graph_ref>: Borrow<C>,
        C: Sub<Output = C> + Measure + Zero + Bounded + Ord,
    {
        let source = self.source.ok_or(DinicsConfigError::SourceNodeNotSet)?;
        let destination = self
            .destination
            .ok_or(DinicsConfigError::DestinationNodeNotSet)?;
        Ok(dinics_inner(self.network, source, destination))
    }
}

/// Output of [`dinics`] algorithm.
///
/// The wrapped data can be accessed using the provided getter methods, or by consuming the struct
/// with [`DinicsOutput::into_max_flow_and_flow_vec`].
pub struct DinicsOutput<G: Graph + Storable, C: Default> {
    max_flow: C,
    flows: G::EdgeDataContainer<C>,
}

impl<G: Graph + Storable, C: Default> MaxFlowReturn<G, C> for DinicsOutput<G, C> {
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

/// Find a [Maximum Flow][maximum_flow] from `source` to `destination` using [Dinic's (or Dinitz's)
/// algorithm][dinics].
///
/// The algorithm works by building successive level graphs using breadth-first search and finds
/// blocking flows within them through depth-first searches.
///
/// Edge Data of the provided graph is interpreted as capacities of edges.
///
/// See also the [`max_flow`](../index.html) module for other maximum flow algorithms.
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
/// - Time complexity:
///   - In general: **O(|V|²|E|)**
///   - In networks with only unit capacities: **O(min{|V|²ᐟ³, |E|¹ᐟ²} |E|)**
/// - Auxiliary space: **O(|V| + |E|)**.
///
/// Where **|V|** is the number of nodes and **|E|** is the number of edges.
///
/// [maximum_flow]: https://en.wikipedia.org/wiki/Maximum_flow_problem
/// [dinics]: https://en.wikipedia.org/wiki/Dinic%27s_algorithm
///
/// # Example
/// ```rust
/// // TODO: Add example
/// ```
pub fn dinics<'graph_ref, G, C>(
    network: &'graph_ref G,
    source: G::NodeId,
    destination: G::NodeId,
) -> DinicsOutput<G, C>
where
    G: DirectedGraph + Storable,
    G::NodeId: IndexId,
    G::EdgeId: IndexId,
    G::EdgeDataRef<'graph_ref>: Borrow<C>,
    C: Sub<Output = C> + Measure + Zero + Bounded + Ord,
{
    dinics_inner(network, source, destination)
}

fn dinics_inner<'graph_ref, G, C>(
    network: &'graph_ref G,
    source: G::NodeId,
    destination: G::NodeId,
) -> DinicsOutput<G, C>
where
    G: DirectedGraph + Storable,
    G::NodeId: IndexId,
    G::EdgeId: IndexId,
    G::EdgeDataRef<'graph_ref>: Borrow<C>,
    C: Sub<Output = C> + Measure + Zero + Bounded + Ord,
{
    let mut max_flow = C::zero();
    let mut flows = G::edge_data_container::<C>(network);
    let mut visited = G::node_visit_container(network);
    let mut level_edges = G::node_data_container::<Vec<FlowEdge<G, C>>>(network);

    while *build_level_graph(network, source, destination, &flows, &mut level_edges)
        .get(destination)
        > 0
    {
        let flow_increase = find_blocking_flow(
            network,
            source,
            destination,
            &mut flows,
            &mut level_edges,
            &mut visited,
        );
        max_flow = max_flow + flow_increase;
    }
    DinicsOutput { max_flow, flows }
}

/// Makes a BFS that labels network vertices with levels representing
/// their distance to the source vertex, considering only edges with
/// positive residual capacity.
///
/// The source vertex is labeled as 1, and vertices not reachable are
/// labeled as 0.
///
/// Aggregates in `level_edges` the edges that connects each
/// vertex to its neighbours in the next level.
///
/// Returns the computed level graph.
fn build_level_graph<'graph_ref, G, C>(
    network: &'graph_ref G,
    source: G::NodeId,
    destination: G::NodeId,
    flows: &impl DataContainer<G::EdgeId, C>,
    level_edges: &mut impl DataContainer<G::NodeId, Vec<FlowEdge<G, C>>>,
) -> impl DataContainer<G::NodeId, usize>
where
    G: DirectedGraph + Storable,
    G::NodeId: IndexId,
    G::EdgeId: IndexId,
    G::EdgeDataRef<'graph_ref>: Borrow<C>,
    C: Sub<Output = C> + Measure + Zero,
{
    let mut level_graph = G::node_data_container::<usize>(network);
    let mut bfs_queue = G::queue_container::<G::NodeId>(network);
    bfs_queue.push_back(source);

    *level_graph.get_mut(source) = 1;
    while let Some(vertex) = bfs_queue.pop_front() {
        let incident_edges = network.incident_edges(vertex);
        level_edges.get_mut(vertex).clear();
        for edge in incident_edges {
            let edge: FlowEdge<G, C> = edge.to_owned_edge();
            let next_vertex = other_endpoint::<G, _>(&edge, vertex);
            let residual_cap = residual_capacity::<G, C>(edge, next_vertex, *flows.get(edge.id));
            if residual_cap == C::zero() {
                continue;
            }
            if *level_graph.get(next_vertex) == 0 {
                *level_graph.get_mut(next_vertex) = *level_graph.get(vertex) + 1;
                level_edges.get_mut(vertex).push(edge);
                if next_vertex != destination {
                    bfs_queue.push_back(next_vertex);
                }
            } else if *level_graph.get(next_vertex) == *level_graph.get(vertex) + 1 {
                level_edges.get_mut(vertex).push(edge);
            }
        }
    }

    level_graph
}

/// Find blocking flow for current level graph by repeatingly finding
/// augmenting paths in it.
///
/// Attach computed flows to `flows` and returns the total flow increase from
/// edges available in `level_edges` at this iteration.
fn find_blocking_flow<G, C>(
    network: &G,
    source: G::NodeId,
    destination: G::NodeId,
    flows: &mut impl DataContainer<G::EdgeId, C>,
    level_edges: &mut impl DataContainer<G::NodeId, Vec<FlowEdge<G, C>>>,
    visited: &mut impl VisitContainer<G::NodeId>,
) -> C
where
    G: DirectedGraph + Storable,
    G::NodeId: IndexId,
    G::EdgeId: IndexId,
    C: Sub<Output = C> + Measure + Zero + Bounded + Ord,
{
    let mut flow_increase = C::zero();
    let mut edge_to = vec![None; network.node_count()];
    while find_augmenting_path(
        network,
        source,
        destination,
        flows,
        level_edges,
        visited,
        &mut edge_to,
    ) {
        let mut path_flow = <C as Bounded>::max();

        // Find the bottleneck capacity of the path
        let mut vertex = destination;
        while let Some(edge) = edge_to[vertex.as_usize()] {
            let residual_capacity = residual_capacity::<G, C>(edge, vertex, *flows.get(edge.id));
            path_flow = path_flow.min(residual_capacity);
            vertex = other_endpoint::<G, _>(&edge, vertex);
        }

        // Update the flow of each edge along the discovered path
        let mut vertex = destination;
        while let Some(edge) = edge_to[vertex.as_usize()] {
            *flows.get_mut(edge.id) =
                adjusted_residual_flow::<G, _, C>(&edge, vertex, *flows.get(edge.id), path_flow);
            vertex = other_endpoint::<G, _>(&edge, vertex);
        }
        flow_increase = flow_increase + path_flow;
    }
    flow_increase
}

/// Makes a DFS to find an augmenting path from source to destination vertex
/// using previously computed `edge_levels` from level graph.
///
/// Returns a boolean indicating if an augmenting path to destination was found.
fn find_augmenting_path<G, C>(
    network: &G,
    source: G::NodeId,
    destination: G::NodeId,
    flows: &impl DataContainer<G::EdgeId, C>,
    level_edges: &mut impl DataContainer<G::NodeId, Vec<FlowEdge<G, C>>>,
    visited: &mut impl VisitContainer<G::NodeId>,
    edge_to: &mut [Option<FlowEdge<G, C>>],
) -> bool
where
    G: DirectedGraph + Storable,
    G::NodeId: IndexId,
    G::EdgeId: IndexId,
    C: Sub<Output = C> + Measure + Zero,
{
    visited.clear();
    let mut level_edges_i = G::node_data_container::<usize>(network);

    let mut dfs_stack = G::stack_container::<G::NodeId>(network);
    dfs_stack.push(source);
    visited.mark_visited(source);
    while let Some(&vertex) = dfs_stack.peek_last() {
        let mut found_next = false;
        while *level_edges_i.get(vertex) < level_edges.get(vertex).len() {
            let curr_level_edges_i = level_edges_i.get(vertex);
            let edge = level_edges.get(vertex)[*curr_level_edges_i];
            let next_vertex = other_endpoint::<G, _>(&edge, vertex);

            let residual_cap = residual_capacity::<G, C>(edge, next_vertex, *flows.get(edge.id));
            if residual_cap == C::zero() {
                level_edges.get_mut(vertex).swap_remove(*curr_level_edges_i);
                continue;
            }

            if !visited.is_visited(next_vertex) {
                edge_to[next_vertex.as_usize()] = Some(edge);
                if destination == next_vertex {
                    return true;
                }
                dfs_stack.push(next_vertex);
                visited.mark_visited(next_vertex);
                found_next = true;
                break;
            }
            *level_edges_i.get_mut(vertex) += 1;
        }
        if !found_next {
            dfs_stack.pop();
        }
    }
    false
}
