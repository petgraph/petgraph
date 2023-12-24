use petgraph_dino::{DiDinoGraph, EdgeId, NodeId};
use petgraph_utils::{graph, GraphCollection};

use super::BellmanFord;
use crate::shortest_paths::{bellman_ford::error::BellmanFordError, ShortestPath};

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
    let GraphCollection {
        mut graph, nodes, ..
    } = networkx::create();

    graph.insert_edge(-1, &nodes.a, &nodes.b);
    graph.insert_edge(-1, &nodes.b, &nodes.c);
    graph.insert_edge(-1, &nodes.c, &nodes.d);
    graph.insert_edge(3, &nodes.d, &nodes.a);

    let spfa = BellmanFord::directed();
    assert!(spfa.every_path(&graph).is_ok());

    let da = *graph.insert_edge(1.999, &nodes.d, &nodes.a).id();
    assert!(spfa.every_path(&graph).is_err());

    *graph.edge_mut(&da).unwrap().weight_mut() = 2.0;
    assert!(spfa.every_path(&graph).is_ok());
}
