use petgraph_dino::{DiDinoGraph, EdgeId, NodeId};
use petgraph_utils::{graph, GraphCollection};

use super::ShortestPathFaster;
use crate::shortest_paths::common::tests::{assert_path, path_from};

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
fn path_from_directed_default_edge_cost() {
    let GraphCollection {
        graph,
        nodes,
        edges,
    } = networkx::create();

    let spfa = ShortestPathFaster::directed();
    let received = path_from(&graph, &nodes.a, &spfa);

    dbg!(&received);
    let expected = [
        (nodes.a, 0, &[nodes.a, nodes.a] as &[_]),
        (nodes.b, 8, &[nodes.a, nodes.c, nodes.b]),
        (nodes.c, 5, &[nodes.a, nodes.c]),
        (nodes.d, 9, &[nodes.a, nodes.c, nodes.b, nodes.d]),
        (nodes.e, 7, &[nodes.a, nodes.c, nodes.e]),
    ];

    assert_path(received, &expected);
}
