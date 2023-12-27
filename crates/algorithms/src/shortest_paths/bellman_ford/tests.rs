use core::array;

use petgraph_core::{
    edge::marker::{Directed, Undirected},
    Graph, GraphStorage, ManagedGraphId,
};
use petgraph_dino::{DiDinoGraph, DinoStorage, EdgeId, NodeId};
use petgraph_utils::{graph, GraphCollection};

use super::BellmanFord;
use crate::shortest_paths::{
    bellman_ford::error::BellmanFordError,
    common::tests::{expected, TestCase},
    ShortestPath,
};

graph!(
    /// Uses the graph from networkx
    ///
    /// <https://github.com/networkx/networkx/blob/main/networkx/algorithms/shortest_paths/tests/test_weighted.py>
    factory(networkx) => DiDinoGraph<&'static str, f32>;
    [
        a: "A",
        b: "B",
        c: "C",
        d: "D",
        e: "E",
    ] as NodeId, [
        ab: a -> b: 10f32,
        ac: a -> c: 5f32,
        bd: b -> d: 1f32,
        bc: b -> c: 2f32,
        de: d -> e: 1f32,
        cb: c -> b: 3f32,
        cd: c -> d: 5f32,
        ce: c -> e: 2f32,
        ea: e -> a: 7f32,
        ed: e -> d: 6f32,
    ] as EdgeId
);

#[test]
fn source_does_not_exist() {
    let GraphCollection { mut graph, .. } = networkx::create();

    let f = *graph.insert_node("F").id();
    graph.remove_node(&f);

    let spfa = BellmanFord::directed();
    let Err(received) = spfa.path_from(&graph, &f) else {
        panic!("Expected an error");
    };
    let error = received.current_context();

    assert_eq!(error, &BellmanFordError::NodeNotFound);
}

#[test]
fn negative_cycle_heuristic() {
    let mut graph = DiDinoGraph::new();

    let a = *graph.insert_node("A").id();
    let b = *graph.insert_node("B").id();
    let c = *graph.insert_node("C").id();
    let d = *graph.insert_node("D").id();

    graph.insert_edge(-1f32, &a, &b);
    graph.insert_edge(-1f32, &b, &c);
    graph.insert_edge(-1f32, &c, &d);
    graph.insert_edge(3f32, &d, &a);

    let spfa = BellmanFord::directed();
    assert!(spfa.every_path(&graph).is_ok());

    let ca = *graph.insert_edge(1.999, &c, &a).id();
    assert!(spfa.every_path(&graph).is_err());

    *graph.edge_mut(&ca).unwrap().weight_mut() = 2.0;
    assert!(spfa.every_path(&graph).is_ok());
}

fn cycle_graph<const N: usize, S>() -> (Graph<S>, [S::NodeId; N], [S::EdgeId; N])
where
    S: GraphStorage<NodeWeight = usize, EdgeWeight = f32>,
    S::NodeId: ManagedGraphId + Copy,
    S::EdgeId: ManagedGraphId + Copy,
{
    let mut graph = Graph::new();

    let nodes: [_; N] = array::from_fn(|index| *graph.insert_node(index).id());

    let edges: [_; N] = array::from_fn(|index| {
        *graph
            .insert_edge(1f32, &nodes[index], &nodes[(index + 1) % N])
            .id()
    });

    (graph, nodes, edges)
}

#[test]
fn negative_cycle_directed() {
    let (mut graph, nodes, _) = cycle_graph::<5, DinoStorage<_, _>>();

    let spfa = BellmanFord::directed();
    assert!(spfa.every_path(&graph).is_ok());

    // add negative cycle between nodes 1 and 2
    *graph.insert_edge(-7f32, &nodes[1], &nodes[2]).id();
    assert!(spfa.every_path(&graph).is_err());
}

#[test]
fn negative_cyle_undirected() {
    let (mut graph, nodes, _) = cycle_graph::<5, DinoStorage<_, _, Undirected>>();

    let spfa = BellmanFord::undirected();
    assert!(spfa.every_path(&graph).is_ok());

    // add negative cycle between nodes 1 and 2
    *graph.insert_edge(-3f32, &nodes[1], &nodes[2]).id();
    assert!(spfa.every_path(&graph).is_err());
}

#[test]
fn negative_self_cycle() {
    let mut graph = DiDinoGraph::new();

    let a = *graph.insert_node("A").id();

    graph.insert_edge(-1f32, &a, &a);

    let spfa = BellmanFord::directed();
    assert!(spfa.every_path(&graph).is_err());
}

#[test]
fn zero_cycle() {
    let (mut graph, nodes, _) = cycle_graph::<5, DinoStorage<_, _>>();

    let spfa = BellmanFord::directed();
    assert!(spfa.every_path(&graph).is_ok());

    // add zero cycle between nodes 2 and 3
    let edge = *graph.insert_edge(-4f32, &nodes[2], &nodes[3]).id();
    assert!(spfa.every_path(&graph).is_ok());

    // increase that cycle to a negative cycle
    *graph.edge_mut(&edge).unwrap().weight_mut() = -4.0001f32;
    assert!(spfa.every_path(&graph).is_err());
}

#[test]
fn negative_weight() {
    let (mut graph, nodes, _) = cycle_graph::<5, DinoStorage<_, _, Directed>>();

    let spfa = BellmanFord::directed();
    assert!(spfa.every_path(&graph).is_ok());

    // add negative weight to edge between nodes 1 and 2
    graph.insert_edge(-3f32, &nodes[1], &nodes[2]);

    let expected = expected!(nodes; [
        0 -()> 0: 0f32,
        0 -()> 1: 1f32,
        0 -(1)> 2: -2f32,
        0 -(1, 2)> 3: -1f32,
        0 -(1, 2, 3)> 4: 0f32,
    ]);

    TestCase::new(&graph, &spfa, &expected).assert_path_from(&nodes[0]);
}

#[test]
fn negative_cycle_not_connected() {
    unimplemented!()
}

#[test]
fn path_graph() {
    unimplemented!()
}

#[test]
fn multigraph() {
    unimplemented!()
}
