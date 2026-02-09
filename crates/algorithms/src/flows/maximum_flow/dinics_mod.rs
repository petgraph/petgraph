use alloc::{collections::VecDeque, vec, vec::Vec};
use core::ops::Sub;

use petgraph_core::{
    edge::Edge,
    graph::{DirectedGraph, Graph},
    id::IndexId,
};

use super::{other_endpoint, residual_capacity};
use crate::{
    flows::maximum_flow::adjusted_residual_flow,
    traits::{Bounded, Measure, Zero},
};

pub struct Dinics<'graph_ref, G: Graph> {
    network: &'graph_ref G,
    source: Option<G::NodeId>,
    destination: Option<G::NodeId>,
}

impl<'graph_ref, G: Graph> Dinics<'graph_ref, G> {
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

impl<'graph, 'graph_ref, G: 'graph> Dinics<'graph_ref, G>
where
    G: DirectedGraph,
    G::NodeId: IndexId,
    G::EdgeId: IndexId,
    G::EdgeData<'graph>: Sub<Output = G::EdgeData<'graph>> + Measure + Zero + Bounded + Ord,
    G::EdgeDataRef<'graph_ref>: ToOwned<Owned = G::EdgeData<'graph>>,
{
    pub fn run(&self) -> (G::EdgeData<'graph>, Vec<G::EdgeData<'graph>>) {
        let source = self.source.expect("Source node is not set");
        let destination = self.destination.expect("Destination node is not set");
        dinics_inner(self.network, source, destination)
    }
}

/// Find a [maximum flow] from `source` to `destination` using [Dinic's (or Dinitz's)
/// algorithm][dinics], which builds successive level graphs using breadth-first search and finds
/// blocking flows within them through depth-first searches.
///
/// See also [`maximum_flow`][max flow mod] module for other maximum flow algorithms.
///
/// # Arguments
/// * `network` — A directed graph with positive edge weights, namely "flow capacities".
/// * `source` — The source node where flow originates.
/// * `destination` — The destination node where flow terminates.
///
/// # Returns
/// Returns a tuple of two values:
/// * `N::EdgeWeight`: computed maximum flow;
/// * `Vec<N::EdgeWeight>`: the flow of each edge. The vector is indexed by the graph's edge
///   indices.
///
/// # Complexity
/// * Time complexity:
///   * In general: **O(|V|²|E|)**
///   * In networks with only unit capacities: **O(min{|V|²ᐟ³, |E|¹ᐟ²} |E|)**
/// * Auxiliary space: **O(|V| + |E|)**.
///
/// where **|V|** is the number of nodes and **|E|** is the number of edges.
///
/// [dinics]: https://en.wikipedia.org/wiki/Dinic%27s_algorithm
/// [max flow mod]: index.html
///
/// # Example
/// ```rust
/// // use petgraph::{Graph, algo::dinics};
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
/// // let (max_flow, _) = dinics(&graph, source, destination);
/// // assert_eq!(23, max_flow);
/// ```
pub fn dinics<'graph, 'graph_ref, G: 'graph>(
    network: &'graph_ref G,
    source: G::NodeId,
    destination: G::NodeId,
) -> (G::EdgeData<'graph>, Vec<G::EdgeData<'graph>>)
where
    G: DirectedGraph,
    G::NodeId: IndexId,
    G::EdgeId: IndexId,
    G::EdgeData<'graph>: Sub<Output = G::EdgeData<'graph>> + Measure + Zero + Bounded + Ord,
    G::EdgeDataRef<'graph_ref>: ToOwned<Owned = G::EdgeData<'graph>>,
{
    dinics_inner(network, source, destination)
}

pub fn dinics_inner<'graph, 'graph_ref, G: 'graph>(
    network: &'graph_ref G,
    source: G::NodeId,
    destination: G::NodeId,
) -> (G::EdgeData<'graph>, Vec<G::EdgeData<'graph>>)
where
    G: DirectedGraph,
    G::NodeId: IndexId,
    G::EdgeId: IndexId,
    G::EdgeData<'graph>: Sub<Output = G::EdgeData<'graph>> + Measure + Zero + Bounded + Ord,
    G::EdgeDataRef<'graph_ref>: ToOwned<Owned = G::EdgeData<'graph>>,
{
    let mut max_flow = G::EdgeData::zero();
    let mut flows = vec![G::EdgeData::zero(); network.edge_count()];
    let mut visited = vec![false; network.node_count()];
    let mut level_edges = vec![Default::default(); network.node_count()];

    while build_level_graph(network, source, destination, &flows, &mut level_edges)
        [destination.as_usize()]
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
    (max_flow, flows)
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
fn build_level_graph<'graph, 'graph_ref, G: 'graph>(
    network: &'graph_ref G,
    source: G::NodeId,
    destination: G::NodeId,
    flows: &[G::EdgeData<'graph>],
    level_edges: &mut [Vec<Edge<G::EdgeId, G::EdgeData<'graph>, G::NodeId>>],
) -> Vec<usize>
where
    G: DirectedGraph,
    G::NodeId: IndexId,
    G::EdgeId: IndexId,
    G::EdgeData<'graph>: Sub<Output = G::EdgeData<'graph>> + Measure + Zero,
    G::EdgeDataRef<'graph_ref>: ToOwned<Owned = G::EdgeData<'graph>>,
{
    let mut level_graph = vec![0; network.node_count()];
    let mut bfs_queue = VecDeque::with_capacity(network.node_count());
    bfs_queue.push_back(source);

    level_graph[source.as_usize()] = 1;
    while let Some(vertex) = bfs_queue.pop_front() {
        let vertex_index = vertex.as_usize();
        let incident_edges = network.incident_edges(vertex);
        level_edges[vertex_index].clear();
        for edge in incident_edges {
            let edge = edge.to_owned_edge();
            let next_vertex = other_endpoint::<G, _>(edge, vertex);
            let residual_cap = residual_capacity::<G>(edge, next_vertex, flows[edge.id.as_usize()]);
            if residual_cap == G::EdgeData::zero() {
                continue;
            }
            let next_vertex_index = next_vertex.as_usize();
            if level_graph[next_vertex_index] == 0 {
                level_graph[next_vertex_index] = level_graph[vertex_index] + 1;
                level_edges[vertex_index].push(edge);
                if next_vertex != destination {
                    bfs_queue.push_back(next_vertex);
                }
            } else if level_graph[next_vertex_index] == level_graph[vertex_index] + 1 {
                level_edges[vertex_index].push(edge);
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
fn find_blocking_flow<'graph, G: 'graph>(
    network: &G,
    source: G::NodeId,
    destination: G::NodeId,
    flows: &mut [G::EdgeData<'graph>],
    level_edges: &mut [Vec<Edge<G::EdgeId, G::EdgeData<'graph>, G::NodeId>>],
    visited: &mut Vec<bool>,
) -> G::EdgeData<'graph>
where
    G: DirectedGraph,
    G::NodeId: IndexId,
    G::EdgeId: IndexId,
    G::EdgeData<'graph>: Sub<Output = G::EdgeData<'graph>> + Measure + Zero + Bounded + Ord,
{
    let mut flow_increase = G::EdgeData::zero();
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
        let mut path_flow = <G::EdgeData<'graph> as Bounded>::max();

        // Find the bottleneck capacity of the path
        let mut vertex = destination;
        while let Some(edge) = edge_to[vertex.as_usize()] {
            let residual_capacity = residual_capacity::<G>(edge, vertex, flows[edge.id.as_usize()]);
            path_flow = path_flow.min(residual_capacity);
            vertex = other_endpoint::<G, _>(edge, vertex);
        }

        // Update the flow of each edge along the discovered path
        let mut vertex = destination;
        while let Some(edge) = edge_to[vertex.as_usize()] {
            let edge_index = edge.id.as_usize();
            flows[edge_index] =
                adjusted_residual_flow::<G, _>(edge, vertex, flows[edge_index], path_flow);
            vertex = other_endpoint::<G, _>(edge, vertex);
        }
        flow_increase = flow_increase + path_flow;
    }
    flow_increase
}

/// Makes a DFS to find an augmenting path from source to destination vertex
/// using previously computed `edge_levels` from level graph.
///
/// Returns a boolean indicating if an augmenting path to destination was found.
fn find_augmenting_path<'graph, G: 'graph>(
    network: &G,
    source: G::NodeId,
    destination: G::NodeId,
    flows: &[G::EdgeData<'graph>],
    level_edges: &mut [Vec<Edge<G::EdgeId, G::EdgeData<'graph>, G::NodeId>>],
    visited: &mut Vec<bool>,
    edge_to: &mut [Option<Edge<G::EdgeId, G::EdgeData<'graph>, G::NodeId>>],
) -> bool
where
    G: DirectedGraph,
    G::NodeId: IndexId,
    G::EdgeId: IndexId,
    G::EdgeData<'graph>: Sub<Output = G::EdgeData<'graph>> + Measure + Zero,
{
    *visited = vec![false; network.node_count()];
    let mut level_edges_i = vec![0; level_edges.len()];

    let mut dfs_stack = Vec::new();
    dfs_stack.push(source);
    visited[source.as_usize()] = true;
    while let Some(&vertex) = dfs_stack.last() {
        let vertex_index = vertex.as_usize();

        let mut found_next = false;
        while level_edges_i[vertex_index] < level_edges[vertex_index].len() {
            let curr_level_edges_i = level_edges_i[vertex_index];
            let edge = level_edges[vertex_index][curr_level_edges_i];
            let next_vertex = other_endpoint::<G, _>(edge, vertex);

            let residual_cap = residual_capacity::<G>(edge, next_vertex, flows[edge.id.as_usize()]);
            if residual_cap == G::EdgeData::zero() {
                level_edges[vertex_index].swap_remove(curr_level_edges_i);
                continue;
            }

            if !visited[next_vertex.as_usize()] {
                edge_to[next_vertex.as_usize()] = Some(edge);
                if destination == next_vertex {
                    return true;
                }
                dfs_stack.push(next_vertex);
                visited[next_vertex.as_usize()] = true;
                found_next = true;
                break;
            }
            level_edges_i[vertex_index] += 1;
        }
        if !found_next {
            dfs_stack.pop();
        }
    }
    false
}
