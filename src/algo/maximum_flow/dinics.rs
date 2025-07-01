use alloc::{collections::VecDeque, vec, vec::Vec};
use core::ops::Sub;

use crate::{
    algo::{EdgeRef, PositiveMeasure},
    prelude::Direction,
    visit::{
        Data, EdgeCount, EdgeIndexable, IntoEdgeReferences, IntoEdges, IntoEdgesDirected,
        NodeCount, NodeIndexable, VisitMap, Visitable,
    },
};

/// Dinic's (or Dinitz's) algorithm.
///
/// Computes the [maximum flow][ff] of a weighted directed graph.
///
/// Returns the maximum flow and also the computed edge flows.
///
/// [ff]: https://en.wikipedia.org/wiki/Dinic%27s_algorithm
pub fn dinics<N>(
    network: N,
    source: N::NodeId,
    sink: N::NodeId,
) -> (N::EdgeWeight, Vec<N::EdgeWeight>)
where
    N: NodeCount + EdgeCount + IntoEdgesDirected + EdgeIndexable + NodeIndexable + Visitable,
    N::EdgeWeight: Sub<Output = N::EdgeWeight> + PositiveMeasure,
{
    let mut max_flow = N::EdgeWeight::zero();
    let mut flows = vec![N::EdgeWeight::zero(); network.edge_count()];
    let mut visited = network.visit_map();
    let mut level_edges = vec![Default::default(); network.node_count()];

    let sink_index = NodeIndexable::to_index(&network, sink);
    while build_level_graph(&network, source, sink, &flows, &mut level_edges)[sink_index] > 0 {
        let flow_increase = find_blocking_flow(
            network,
            source,
            sink,
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
pub fn build_level_graph<N>(
    network: N,
    source: N::NodeId,
    sink: N::NodeId,
    flows: &[N::EdgeWeight],
    level_edges: &mut [Vec<N::EdgeRef>],
) -> Vec<usize>
where
    N: NodeCount + IntoEdgesDirected + NodeIndexable + EdgeIndexable,
    N::EdgeWeight: Sub<Output = N::EdgeWeight> + PositiveMeasure,
{
    let mut level_graph = vec![0; network.node_count()];
    let mut bfs_queue = VecDeque::with_capacity(network.node_count());
    bfs_queue.push_back(source);

    level_graph[NodeIndexable::to_index(&network, source)] = 1;
    while let Some(vertex) = bfs_queue.pop_front() {
        let vertex_index = NodeIndexable::to_index(&network, vertex);
        let out_edges = network.edges_directed(vertex, Direction::Outgoing);
        let in_edges = network.edges_directed(vertex, Direction::Incoming);
        level_edges[vertex_index].clear();
        for edge in out_edges.chain(in_edges) {
            let next_vertex = other_endpoint(&network, edge, vertex);
            let edge_index = EdgeIndexable::to_index(&network, edge.id());
            let residual_cap = residual_capacity(&network, edge, next_vertex, flows[edge_index]);
            if residual_cap == N::EdgeWeight::zero() {
                continue;
            }
            let next_vertex_index = NodeIndexable::to_index(&network, next_vertex);
            if level_graph[next_vertex_index] == 0 {
                level_graph[next_vertex_index] = level_graph[vertex_index] + 1;
                level_edges[vertex_index].push(edge);
                if next_vertex != sink {
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
fn find_blocking_flow<N>(
    network: N,
    source: N::NodeId,
    sink: N::NodeId,
    flows: &mut [N::EdgeWeight],
    level_edges: &mut [Vec<N::EdgeRef>],
    visited: &mut N::Map,
) -> N::EdgeWeight
where
    N: NodeCount + IntoEdges + NodeIndexable + EdgeIndexable + Visitable,
    N::EdgeWeight: Sub<Output = N::EdgeWeight> + PositiveMeasure,
{
    let mut flow_increase = N::EdgeWeight::zero();
    let mut edge_to = vec![None; network.node_count()];
    let mut level_edges_i = vec![0; level_edges.len()];
    while find_augmenting_path(
        &network,
        source,
        sink,
        flows,
        level_edges,
        visited,
        &mut level_edges_i,
        &mut edge_to,
    ) {
        let mut path_flow = N::EdgeWeight::max();

        // Find the bottleneck capacity of the path
        let mut vertex = sink;
        while let Some(edge) = edge_to[NodeIndexable::to_index(&network, vertex)] {
            let edge_index = EdgeIndexable::to_index(&network, edge.id());
            let residual_capacity = residual_capacity(&network, edge, vertex, flows[edge_index]);
            path_flow = min::<N>(path_flow, residual_capacity);
            vertex = other_endpoint(&network, edge, vertex);
        }

        // Update the flow of each edge along the discovered path
        let mut vertex = sink;
        while let Some(edge) = edge_to[NodeIndexable::to_index(&network, vertex)] {
            let edge_index = EdgeIndexable::to_index(&network, edge.id());
            flows[edge_index] =
                adjusted_residual_flow(&network, edge, vertex, flows[edge_index], path_flow);
            vertex = other_endpoint(&network, edge, vertex);
        }
        flow_increase = flow_increase + path_flow;
    }
    flow_increase
}

/// Makes a DFS to find an augmenting path from source to destination vertex
/// using previously computed `edge_levels` from level graph.
///
/// Returns a boolean indicating if an augmenting path to destination was found.
fn find_augmenting_path<N>(
    network: N,
    source: N::NodeId,
    sink: N::NodeId,
    flows: &[N::EdgeWeight],
    level_edges: &mut [Vec<N::EdgeRef>],
    visited: &mut N::Map,
    level_edges_i: &mut [usize],
    edge_to: &mut [Option<N::EdgeRef>],
) -> bool
where
    N: IntoEdges + NodeIndexable + EdgeIndexable + Visitable,
    N::EdgeWeight: Sub<Output = N::EdgeWeight> + PositiveMeasure,
{
    network.reset_map(visited);
    level_edges_i.fill(0);

    let mut dfs_stack = Vec::new();
    dfs_stack.push(source);
    visited.visit(source);
    while let Some(&vertex) = dfs_stack.last() {
        let vertex_index = NodeIndexable::to_index(&network, vertex);

        let mut found_next = false;
        while level_edges_i[vertex_index] < level_edges[vertex_index].len() {
            let curr_level_edges_i = level_edges_i[vertex_index];
            let edge = level_edges[vertex_index][curr_level_edges_i];
            let next_vertex = other_endpoint(&network, edge, vertex);

            let edge_index: usize = EdgeIndexable::to_index(&network, edge.id());
            let residual_cap = residual_capacity(&network, edge, next_vertex, flows[edge_index]);
            if residual_cap == N::EdgeWeight::zero() {
                level_edges[vertex_index].swap_remove(curr_level_edges_i);
                continue;
            }

            if !visited.is_visited(&next_vertex) {
                let next_vertex_index = NodeIndexable::to_index(&network, next_vertex);
                edge_to[next_vertex_index] = Some(edge);
                if sink == next_vertex {
                    return true;
                }
                dfs_stack.push(next_vertex);
                visited.visit(next_vertex);
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

/// Returns the adjusted residual flow for given edge and flow increase.
fn adjusted_residual_flow<N>(
    network: N,
    edge: N::EdgeRef,
    target_vertex: N::NodeId,
    flow: N::EdgeWeight,
    flow_increase: N::EdgeWeight,
) -> N::EdgeWeight
where
    N: NodeIndexable + IntoEdges,
    N::EdgeWeight: Sub<Output = N::EdgeWeight> + PositiveMeasure,
{
    if target_vertex == edge.source() {
        // backward edge
        flow - flow_increase
    } else if target_vertex == edge.target() {
        // forward edge
        flow + flow_increase
    } else {
        let end_point = NodeIndexable::to_index(&network, target_vertex);
        panic!("Illegal endpoint {}", end_point);
    }
}

/// Returns the residual capacity of given edge.
fn residual_capacity<N>(
    network: N,
    edge: N::EdgeRef,
    target_vertex: N::NodeId,
    flow: N::EdgeWeight,
) -> N::EdgeWeight
where
    N: NodeIndexable + IntoEdges,
    N::EdgeWeight: Sub<Output = N::EdgeWeight> + PositiveMeasure,
{
    if target_vertex == edge.source() {
        // backward edge
        flow
    } else if target_vertex == edge.target() {
        // forward edge
        return *edge.weight() - flow;
    } else {
        let end_point = NodeIndexable::to_index(&network, target_vertex);
        panic!("Illegal endpoint {}", end_point);
    }
}

/// Returns the minimum value between given `a` and `b`.
fn min<N>(a: N::EdgeWeight, b: N::EdgeWeight) -> N::EdgeWeight
where
    N: Data,
    N::EdgeWeight: PartialOrd,
{
    if a < b {
        a
    } else {
        b
    }
}

/// Gets the other endpoint of graph edge, if any, otherwise panics.
fn other_endpoint<N>(network: N, edge: N::EdgeRef, vertex: N::NodeId) -> N::NodeId
where
    N: NodeIndexable + IntoEdgeReferences,
{
    if vertex == edge.source() {
        edge.target()
    } else if vertex == edge.target() {
        edge.source()
    } else {
        let end_point = NodeIndexable::to_index(&network, vertex);
        panic!("Illegal endpoint {}", end_point);
    }
}
