use alloc::{collections::VecDeque, vec, vec::Vec};
use core::{
    iter::{Chain, Peekable},
    ops::Sub,
};
use std::collections::LinkedList;

use crate::{
    algo::{EdgeRef, PositiveMeasure},
    prelude::Direction,
    visit::{
        Data, EdgeCount, EdgeIndexable, IntoEdges, IntoEdgesDirected, NodeCount, NodeIndexable,
        VisitMap, Visitable,
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
    let mut level_graph = vec![0; network.node_count()];
    while build_level_graph(&network, source, sink, &mut level_graph, &flows) {
        let flow_increase = find_blocking_flow(network, source, sink, &level_graph, &mut flows);
        max_flow = max_flow + flow_increase;
        // Resets level graph for next iteration
        level_graph.fill(0);
    }
    (max_flow, flows)
}

/// Makes a BFS to label network vertices with levels representing
/// their distance to the source vertex.
///
/// Returns a boolean indicating if sink vertex is reachable.
fn build_level_graph<N>(
    network: N,
    source: N::NodeId,
    sink: N::NodeId,
    level_graph: &mut [usize],
    flows: &[N::EdgeWeight],
    // allowed_edges: &mut [LinkedList<N::EdgeRef>],
) -> bool
where
    N: IntoEdgesDirected + NodeIndexable + EdgeIndexable,
    N::EdgeWeight: Sub<Output = N::EdgeWeight> + PositiveMeasure,
{
    let mut queue = VecDeque::new();
    queue.push_back(source);

    //    println!("\n----level-graph-----\n");
    level_graph[NodeIndexable::to_index(&network, source)] = 1;
    while let Some(vertex) = queue.pop_front() {
        let vertex_level = level_graph[NodeIndexable::to_index(&network, vertex)];
        let out_edges = network.edges_directed(vertex, Direction::Outgoing);
        for edge in out_edges {
            let next_vertex = other_endpoint(&network, edge, vertex);
            let next_vertex_level = NodeIndexable::to_index(&network, next_vertex);
            let edge_index = EdgeIndexable::to_index(&network, edge.id());
            let residual_cap = residual_capacity(&network, edge, next_vertex, flows[edge_index]);
            if level_graph[next_vertex_level] == 0 && residual_cap > N::EdgeWeight::zero() {
                level_graph[next_vertex_level] = vertex_level + 1;
                queue.push_back(next_vertex);
            }
        }
    }

    let sink_level = level_graph[NodeIndexable::to_index(&network, sink)];
    //    println!("sink level {:?}", sink_level);
    sink_level > 0
}

/// Find blocking flow for current level graph by repeatingly finding
/// augmenting paths in it using DFS.
///
/// Attach computed flows to given `flows` parameter and returns the
/// flow increase of current level graph.
fn find_blocking_flow<N>(
    network: N,
    source: N::NodeId,
    sink: N::NodeId,
    level_graph: &[usize],
    flows: &mut [N::EdgeWeight],
) -> N::EdgeWeight
where
    N: NodeCount + IntoEdgesDirected + NodeIndexable + EdgeIndexable + Visitable,
    N::EdgeWeight: Sub<Output = N::EdgeWeight> + PositiveMeasure,
{
    let mut flow_increase = N::EdgeWeight::zero();
    let mut edge_to = vec![None; network.node_count()];
    let mut virtual_edges = Vec::new();
    virtual_edges.resize_with(network.node_count(), || None);
    //    println!("\n----blocking-flow-----\n");
    while find_augmenting_path(
        &network,
        source,
        sink,
        level_graph,
        flows,
        &mut edge_to,
        &mut virtual_edges,
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
    //    println!("flow increase {:?}", flow_increase);
    flow_increase
}

/// Makes a DFS to build an augmenting path to destination vertex using
/// previously found vertex levels.
///
/// Returns a boolean indicating if an augmenting path to destination was found.
fn find_augmenting_path<N>(
    network: N,
    source: N::NodeId,
    destination: N::NodeId,
    level_graph: &[usize],
    flows: &[N::EdgeWeight],
    edge_to: &mut [Option<N::EdgeRef>],
    virtual_edges: &mut [Option<Peekable<N::EdgesDirected>>],
) -> bool
where
    N: NodeCount + IntoEdgesDirected + NodeIndexable + EdgeIndexable + Visitable,
    N::EdgeWeight: Sub<Output = N::EdgeWeight> + PositiveMeasure,
{
    let mut visited = network.visit_map();
    let mut stack = Vec::new();
    visited.visit(source);
    stack.push(source);

    //    println!("\n----augmenting-path-----\n");
    while let Some(&vertex) = stack.last() {
        let vertex_index = NodeIndexable::to_index(&network, vertex);
        //        println!("vertex: {:?}", vertex_index);

        let edges = virtual_edges[vertex_index].get_or_insert(
            network
                .edges_directed(vertex, Direction::Outgoing)
                .peekable(),
        );

        let mut found_next = false;
        while let Some(&edge) = edges.peek() {
            let next_vertex = other_endpoint(&network, edge, vertex);
            let next_vertex_index = NodeIndexable::to_index(&network, next_vertex);
            let edge_index: usize = EdgeIndexable::to_index(&network, edge.id());
            let residual_cap = residual_capacity(&network, edge, next_vertex, flows[edge_index]);
            if level_graph[next_vertex_index] == level_graph[vertex_index] + 1
                && !visited.is_visited(&next_vertex)
                && (residual_cap > N::EdgeWeight::zero())
            {
                visited.visit(next_vertex);
                edge_to[next_vertex_index] = Some(edge);
                if destination == next_vertex {
                    return true;
                }
                stack.push(next_vertex);
                found_next = true;
                break;
            } else {
                (*edges).next();
            }
        }
        if !found_next {
            stack.pop();
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
    N: NodeIndexable + IntoEdges,
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
