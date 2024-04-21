use std::{collections::VecDeque, ops::Sub};

use crate::{
    data::DataMap,
    visit::{EdgeCount, EdgeIndexable, IntoEdges, NodeCount, NodeIndexable},
};

use super::{EdgeRef, FloatMeasure};

fn has_augmented_path<N>(
    network: N,
    source: N::NodeId,
    destination: N::NodeId,
    edge_to: &mut [Option<N::NodeId>],
    capacities: &[N::EdgeWeight],
    forward_flows: &[N::EdgeWeight],
) -> bool
where
    N: NodeCount + IntoEdges + NodeIndexable + EdgeIndexable,
    N::EdgeWeight: Sub<Output = N::EdgeWeight> + FloatMeasure,
{
    let mut marked = vec![false; network.node_count()];
    let mut queue = VecDeque::new();

    marked[NodeIndexable::to_index(&network, source)] = true;
    queue.push_back(source);

    while let Some(vertex) = queue.pop_front() {
        let edges = network.edges(vertex);
        for edge in edges {
            let next = edge.target();
            let index_next = NodeIndexable::to_index(&network, next);
            let edge_index = EdgeIndexable::to_index(&network, edge.id());
            let residual_capacity = capacities[edge_index] - forward_flows[edge_index];
            if !marked[index_next] && (residual_capacity > N::EdgeWeight::zero()) {
                marked[index_next] = true;
                edge_to[index_next] = Some(vertex);
                if next == destination {
                    return true;
                }
                queue.push_back(next);
            }
        }
    }
    false
}

pub fn ford_fulkerson<N>(
    network: N,
    source: N::NodeId,
    destination: N::NodeId,
    capacities: &[N::EdgeWeight],
) -> N::EdgeWeight
where
    N: NodeCount + EdgeCount + IntoEdges + EdgeIndexable + NodeIndexable + DataMap, // + Visitable,
    N::EdgeWeight: Sub<Output = N::EdgeWeight> + FloatMeasure,
{
    let mut edge_to = vec![None; network.node_count()];
    let mut forward_flows = vec![N::EdgeWeight::zero(); network.edge_count()];
    let mut max_flow = N::EdgeWeight::zero();
    while has_augmented_path(
        network,
        source,
        destination,
        &mut edge_to,
        capacities,
        &forward_flows,
    ) {
        let mut path_flow = N::EdgeWeight::infinite();

        // Find the bottleneck capacity of the path
        let mut vertex = destination;
        while let Some(parent_vertex) = edge_to[NodeIndexable::to_index(&network, vertex)] {
            let edge = network
                .edges(parent_vertex)
                .find(|e| e.target() == vertex)
                .unwrap();
            let edge_index = EdgeIndexable::to_index(&network, edge.id());
            let res_cap = capacities[edge_index] - forward_flows[edge_index];
            // Minimum between the path flow and the residual capacity.
            path_flow = if path_flow > res_cap {
                res_cap
            } else {
                path_flow
            };
            vertex = parent_vertex;
        }

        // Update the flow of each edge along the path
        vertex = destination;
        while let Some(parent_vertex) = edge_to[NodeIndexable::to_index(&network, vertex)] {
            let forward_edge = network
                .edges(parent_vertex)
                .find(|e| e.target() == vertex)
                .unwrap();
            let fwd_index = EdgeIndexable::to_index(&network, forward_edge.id());

            forward_flows[fwd_index] = add_residual_flow_to(
                network,
                forward_edge,
                vertex,
                forward_flows[fwd_index],
                path_flow,
            );
            vertex = parent_vertex;
        }
        max_flow = max_flow + path_flow;
    }
    max_flow
}

fn add_residual_flow_to<N>(
    network: N,
    edge: N::EdgeRef,
    vertex: N::NodeId,
    flow: N::EdgeWeight,
    delta: N::EdgeWeight,
) -> N::EdgeWeight
where
    N: NodeIndexable + IntoEdges,
    N::EdgeWeight: Sub<Output = N::EdgeWeight> + FloatMeasure,
{
    if vertex == edge.source() {
        flow - delta
    } else if vertex == edge.target() {
        flow + delta
    } else {
        let end_point = NodeIndexable::to_index(&network, vertex);
        panic!("Illegal endpoint {}", end_point);
    }
}
