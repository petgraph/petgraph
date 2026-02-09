use core::ops::Sub;
use std::{
    error::Error,
    fmt::{Display, Formatter},
    ops::Add,
};

use petgraph_core::{
    edge::Edge,
    graph::{DirectedGraph, Graph},
    id::IndexId,
};

use crate::{
    alloc::collections::VecDeque,
    flows::maximum_flow::{other_endpoint, residual_capacity},
    traits::{Bounded, Measure, Zero},
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum EdmondsKarpError {
    SourceNodeNotSet,
    DestinationNodeNotSet,
}

impl Display for EdmondsKarpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            EdmondsKarpError::SourceNodeNotSet => write!(f, "Source node is not set"),
            EdmondsKarpError::DestinationNodeNotSet => write!(f, "Destination node is not set"),
        }
    }
}

impl Error for EdmondsKarpError {}

struct EdmondsKarp<'graph_ref, G: Graph> {
    network: &'graph_ref G,
    source: Option<G::NodeId>,
    destination: Option<G::NodeId>,
}

impl<'graph_ref, G: Graph> EdmondsKarp<'graph_ref, G> {
    pub fn new(network: &'graph_ref G) -> Self {
        Self {
            network,
            source: None,
            destination: None,
        }
    }

    pub fn with_source(mut self, source: G::NodeId) -> Self {
        self.source = Some(source);
        self
    }

    pub fn with_destination(mut self, destination: G::NodeId) -> Self {
        self.destination = Some(destination);
        self
    }
}

impl<'graph, 'graph_ref, G: 'graph> EdmondsKarp<'graph_ref, G>
where
    G: DirectedGraph,
    G::NodeId: IndexId,
    G::EdgeId: IndexId,
    G::EdgeData<'graph>: Sub<Output = G::EdgeData<'graph>>
        + Add<Output = G::EdgeData<'graph>>
        + Zero
        + Measure
        + Bounded,
    G::EdgeDataRef<'graph_ref>: ToOwned<Owned = G::EdgeData<'graph>> + Copy,
{
    pub fn run(&self) -> Result<(G::EdgeData<'graph>, Vec<G::EdgeData<'graph>>), EdmondsKarpError> {
        let source = self.source.ok_or(EdmondsKarpError::SourceNodeNotSet)?;
        let destination = self
            .destination
            .ok_or(EdmondsKarpError::DestinationNodeNotSet)?;
        Ok(inner_edmonds_karp(self.network, source, destination))
    }
}

/// Find a [maximum flow] from `source` to `destination` using [Edmond-Karp][ek] implementation of
/// the [Ford-Fulkerson][ff] method. Weights of the provided graph are interpreted as capacities of
/// edges.
///
/// See also [`maximum_flow`][maximum_flow] module for other maximum flow algorithms.
///
/// # Arguments
/// - `network`: Directed graph where edge weights are capacities of edges.
/// - `source`: Source node for the flow.
/// - `destination`: Sink node for the flow.
///
/// # Returns
/// Returns a tuple of two values:
/// - `N::EdgeWeight`: computed maximum flow;
/// - `Vec<N::EdgeWeight>`: the flow of each edge. The vector is indexed by the graph's edge
///   indices.
///
/// # Complexity
/// - Time: **O(|V||E|²)**.
/// - Auxiliary space: **O(|V| + |E|)**.
///
/// where **|V|** is the number of nodes and **|E|** is the number of edges.
///
/// [maximum flow]: https://en.wikipedia.org/wiki/Maximum_flow_problem
/// [ff]: https://en.wikipedia.org/wiki/Ford%E2%80%93Fulkerson_algorithm
/// [ek]: https://en.wikipedia.org/wiki/Edmonds%E2%80%93Karp_algorithm
/// [maximum_flow]: index.html
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
) -> (G::EdgeData<'graph>, Vec<G::EdgeData<'graph>>)
where
    G: DirectedGraph,
    G::NodeId: IndexId,
    G::EdgeId: IndexId,
    G::EdgeData<'graph>: Sub<Output = G::EdgeData<'graph>>
        + Add<Output = G::EdgeData<'graph>>
        + Zero
        + Measure
        + Bounded,
    G::EdgeDataRef<'graph_ref>: ToOwned<Owned = G::EdgeData<'graph>> + Copy,
{
    EdmondsKarp::new(network)
        .with_source(source)
        .with_destination(destination)
        .run()
        .expect("Source and destination nodes should be set")
}

fn inner_edmonds_karp<'graph, 'graph_ref, G: 'graph>(
    network: &'graph_ref G,
    source: G::NodeId,
    destination: G::NodeId,
) -> (G::EdgeData<'graph>, Vec<G::EdgeData<'graph>>)
where
    G: DirectedGraph,
    G::NodeId: IndexId,
    G::EdgeId: IndexId,
    G::EdgeData<'graph>: Sub<Output = G::EdgeData<'graph>>
        + Add<Output = G::EdgeData<'graph>>
        + Zero
        + Measure
        + Bounded,
    G::EdgeDataRef<'graph_ref>: ToOwned<Owned = G::EdgeData<'graph>> + Copy,
{
    let mut edge_to = vec![None; network.node_count()];
    let mut flows = vec![G::EdgeData::zero(); network.edge_count()];
    let mut max_flow = G::EdgeData::zero();
    while has_augmented_path(network, source, destination, &mut edge_to, &flows) {
        let mut path_flow = G::EdgeData::max();

        // Find the bottleneck capacity of the path
        let mut vertex = destination;
        while let Some(edge) = edge_to[vertex.as_usize()] {
            let residual_capacity = residual_capacity::<G>(edge, vertex, flows[edge.id.as_usize()]);
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
        while let Some(edge) = edge_to[vertex.as_usize()] {
            flows[edge.id.as_usize()] =
                adjust_residual_flow::<G, _>(edge, vertex, flows[edge.id.as_usize()], path_flow);
            vertex = other_endpoint::<G, _>(edge, vertex);
        }
        max_flow = max_flow + path_flow;
    }
    (max_flow, flows)
}

/// Returns whether there is an augmenting path in the graph
fn has_augmented_path<'graph, 'graph_ref, G: 'graph>(
    network: &'graph_ref G,
    source: G::NodeId,
    destination: G::NodeId,
    edge_to: &mut [Option<Edge<G::EdgeId, G::EdgeData<'graph>, G::NodeId>>],
    flows: &[G::EdgeData<'graph>],
) -> bool
where
    G: DirectedGraph,
    G::NodeId: IndexId,
    G::EdgeId: IndexId,
    G::EdgeData<'graph>: Sub<Output = G::EdgeData<'graph>> + Zero + Measure,
    G::EdgeDataRef<'graph_ref>: ToOwned<Owned = G::EdgeData<'graph>> + Copy,
{
    // TODO(next): Replace by proper visit map
    let mut visited = vec![false; network.node_count()];
    let mut queue = VecDeque::new();
    visited[source.as_usize()] = true;
    queue.push_back(source);

    while let Some(vertex) = queue.pop_front() {
        let incident_edges = network.incident_edges(vertex);
        for edge in incident_edges {
            let next = other_endpoint::<G, _>(edge, vertex);
            let edge_index: usize = edge.id.as_usize();
            let residual_cap =
                residual_capacity::<G>(edge.to_owned_edge(), next, flows[edge_index]);
            if !visited[next.as_usize()] && (residual_cap > <G::EdgeData<'graph>>::zero()) {
                visited[next.as_usize()] = true;
                edge_to[next.as_usize()] = Some(edge.to_owned_edge());
                if destination == next {
                    return true;
                }
                queue.push_back(next);
            }
        }
    }
    false
}

fn adjust_residual_flow<'graph, 'graph_ref, G, D>(
    edge: Edge<G::EdgeId, D, G::NodeId>,
    vertex: G::NodeId,
    flow: G::EdgeData<'graph>,
    delta: G::EdgeData<'graph>,
) -> G::EdgeData<'graph>
where
    G: DirectedGraph,
    G::EdgeData<'graph>: Sub<Output = G::EdgeData<'graph>> + Add<Output = G::EdgeData<'graph>>,
{
    if vertex == edge.source {
        // backward edge
        flow - delta
    } else if vertex == edge.target {
        // forward edge
        flow + delta
    } else {
        panic!("Illegal endpoint {}", vertex);
    }
}
