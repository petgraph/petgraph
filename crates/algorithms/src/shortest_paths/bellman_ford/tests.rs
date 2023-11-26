use petgraph_dino::{DiDinoGraph, EdgeId, NodeId};
use petgraph_utils::{graph, GraphCollection};

use super::BellmanFord;
use crate::shortest_paths::{bellman_ford::error::BellmanFordError, ShortestPath};

graph!(
    /// Uses the graph from networkx
    ///
    /// <https://github.com/networkx/networkx/blob/main/networkx/algorithms/shortest_paths/tests/test_weighted.py>
    factory(networkx) => DiDinoGraph<&'static str, i32>;
    [
        a: "A",
        b: "B",
        c: "C",
        d: "D",
        e: "E",
    ] as NodeId, [
        ab: a -> b: 10,
        ac: a -> c: 5,
        bd: b -> d: 1,
        bc: b -> c: 2,
        de: d -> e: 1,
        cb: c -> b: 3,
        cd: c -> d: 5,
        ce: c -> e: 2,
        ea: e -> a: 7,
        ed: e -> d: 6,
    ] as EdgeId
);

#[test]
fn source_does_not_exist() {
    let GraphCollection {
        mut graph,
        nodes,
        edges,
    } = networkx::create();

    let f = *graph.insert_node("F").id();
    graph.remove_node(&f);

    let spfa = BellmanFord::directed();
    let received = spfa.path_from(&graph, &f).expect_err("node not found");
    let error = received.current_context();

    assert_eq!(error, &BellmanFordError::NodeNotFound);
}

#[test]
fn path_from_directed_default_edge_cost() {
    let GraphCollection {
        graph,
        nodes,
        edges,
    } = networkx::create();

    let spfa = BellmanFord::directed();
    // let received = path_from(&graph, &nodes.a, &spfa);

    let res = spfa
        .path_from(&graph, &nodes.a)
        .unwrap()
        .map(|v| {
            v.path
                .to_vec()
                .into_iter()
                .map(|q| q.weight())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    dbg!(&res);
    let expected = [
        (nodes.a, 0, &[nodes.a, nodes.a] as &[_]),
        (nodes.b, 8, &[nodes.a, nodes.c, nodes.b]),
        (nodes.c, 5, &[nodes.a, nodes.c]),
        (nodes.d, 9, &[nodes.a, nodes.c, nodes.b, nodes.d]),
        (nodes.e, 7, &[nodes.a, nodes.c, nodes.e]),
    ];

    // assert_path(received, &expected);
    assert!(false)
}
