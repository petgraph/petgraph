use alloc::vec::Vec;
use core::{fmt::Debug, hash::Hash};

use hashbrown::HashMap;
use petgraph_core::{base::MaybeOwned, Edge, Graph, GraphStorage, Node};
use petgraph_dino::{DiDinoGraph, EdgeId, NodeId};
use petgraph_utils::{graph, GraphCollection};

use crate::shortest_paths::{
    common::tests::{assert_distance, assert_path, distance_from, path_from},
    dijkstra::Dijkstra,
    ShortestDistance, ShortestPath,
};

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

graph!(
    /// Uses a randomly generated graph
    factory(random) => DiDinoGraph<&'static str, &'static str>;
    [
        a: "A",
        b: "B",
        c: "C",
        d: "D",
        e: "E",
        f: "F",
    ] as NodeId, [
        ab: a -> b: "apple",
        bc: b -> c: "cat",
        cd: c -> d: "giraffe",
        de: d -> e: "is",
        ef: e -> f: "banana",
        fa: f -> a: "bear",
        ad: a -> d: "elephant",
    ] as EdgeId
);

// TODO: multigraph
// TODO: more test cases

#[test]
fn path_from_directed_default_edge_cost() {
    let GraphCollection {
        graph,
        nodes,
        edges,
    } = networkx::create();

    let dijkstra = Dijkstra::directed();

    let received = path_from(&graph, &nodes.a, &dijkstra);

    let expected = [
        (nodes.a, 0, &[nodes.a, nodes.a] as &[_]),
        (nodes.b, 8, &[nodes.a, nodes.c, nodes.b]),
        (nodes.c, 5, &[nodes.a, nodes.c]),
        (nodes.d, 9, &[nodes.a, nodes.c, nodes.b, nodes.d]),
        (nodes.e, 7, &[nodes.a, nodes.c, nodes.e]),
    ];

    assert_path(received, &expected);
}

#[test]
fn distance_from_directed_default_edge_cost() {
    let GraphCollection {
        graph,
        nodes,
        edges,
    } = networkx::create();

    let dijkstra = Dijkstra::directed();

    let received = distance_from(&graph, &nodes.a, &dijkstra);

    let expected = [
        (nodes.a, 0),
        (nodes.b, 8),
        (nodes.c, 5),
        (nodes.d, 9),
        (nodes.e, 7),
    ];

    assert_distance(received, &expected)
}

fn edge_cost<S>(edge: Edge<S>) -> MaybeOwned<'_, usize>
where
    S: GraphStorage,
    S::EdgeWeight: AsRef<[u8]>,
{
    MaybeOwned::Owned(edge.weight().as_ref().len())
}

#[test]
fn path_from_directed_custom_edge_cost() {
    let GraphCollection {
        graph,
        nodes,
        edges,
    } = random::create();

    let dijkstra = Dijkstra::directed().with_edge_cost(edge_cost);

    let received = path_from(&graph, &nodes.a, &dijkstra);

    let expected = [
        (nodes.a, 0, &[nodes.a, nodes.a] as &[_]),
        (nodes.b, 5, &[nodes.a, nodes.b]),
        (nodes.c, 8, &[nodes.a, nodes.b, nodes.c]),
        (nodes.d, 8, &[nodes.a, nodes.d]),
        (nodes.e, 10, &[nodes.a, nodes.d, nodes.e]),
        (nodes.f, 16, &[nodes.a, nodes.d, nodes.e, nodes.f]),
    ];

    assert_path(received, &expected);
}

#[test]
fn distance_from_directed_custom_edge_cost() {
    let GraphCollection {
        graph,
        nodes,
        edges,
    } = random::create();

    let dijkstra = Dijkstra::directed().with_edge_cost(edge_cost);

    let received = distance_from(&graph, &nodes.a, &dijkstra);

    let expected = [
        (nodes.a, 0),
        (nodes.b, 5),
        (nodes.c, 8),
        (nodes.d, 8),
        (nodes.e, 10),
        (nodes.f, 16),
    ];

    assert_distance(received, &expected)
}

#[test]
fn path_from_undirected_default_edge_cost() {
    let GraphCollection {
        graph,
        nodes,
        edges,
    } = networkx::create();

    let dijkstra = Dijkstra::undirected();

    let received = path_from(&graph, &nodes.a, &dijkstra);

    let expected = [
        (nodes.a, 0, &[nodes.a, nodes.a] as &[_]),
        (nodes.b, 7, &[nodes.a, nodes.c, nodes.b]),
        (nodes.c, 5, &[nodes.a, nodes.c]),
        (nodes.d, 8, &[nodes.a, nodes.e, nodes.d]),
        (nodes.e, 7, &[nodes.a, nodes.e]),
    ];

    assert_path(received, &expected)
}

#[test]
fn distance_from_undirected_default_edge_cost() {
    let GraphCollection {
        graph,
        nodes,
        edges,
    } = networkx::create();

    let dijkstra = Dijkstra::undirected();

    let received = distance_from(&graph, &nodes.a, &dijkstra);

    let expected = [
        (nodes.a, 0),
        (nodes.b, 7),
        (nodes.c, 5),
        (nodes.d, 8),
        (nodes.e, 7),
    ];

    assert_distance(received, &expected)
}

#[test]
fn path_from_undirected_custom_edge_cost() {
    let GraphCollection {
        graph,
        nodes,
        edges,
    } = random::create();

    let dijkstra = Dijkstra::undirected().with_edge_cost(edge_cost);

    let received = path_from(&graph, &nodes.a, &dijkstra);

    let expected = [
        (nodes.a, 0, &[nodes.a, nodes.a] as &[_]),
        (nodes.b, 5, &[nodes.a, nodes.b]),
        (nodes.c, 8, &[nodes.a, nodes.b, nodes.c]),
        (nodes.d, 8, &[nodes.a, nodes.d]),
        (nodes.e, 10, &[nodes.a, nodes.f, nodes.e]),
        (nodes.f, 4, &[nodes.a, nodes.f]),
    ];

    assert_path(received, &expected);
}

#[test]
fn distance_from_undirected_custom_edge_cost() {
    let GraphCollection {
        graph,
        nodes,
        edges,
    } = random::create();

    let dijkstra = Dijkstra::undirected().with_edge_cost(edge_cost);

    let received = distance_from(&graph, &nodes.a, &dijkstra);

    let expected = [
        (nodes.a, 0),
        (nodes.b, 5),
        (nodes.c, 8),
        (nodes.d, 8),
        (nodes.e, 10),
        (nodes.f, 4),
    ];

    assert_distance(received, &expected)
}

#[test]
fn lifetime() {
    let GraphCollection {
        graph,
        nodes,
        edges,
    } = networkx::create();

    let dijkstra = Dijkstra::directed();

    let top3: Vec<_> = dijkstra
        .path_from(&graph, &nodes.a)
        .unwrap()
        .take(3)
        .collect();

    drop(dijkstra);

    let top3: Vec<_> = top3
        .into_iter()
        .map(|route| route.cost.into_value())
        .collect();

    assert_eq!(top3, [0, 5, 7]);
}

// fn non_empty_graph() -> impl Strategy<Value = Graph<(), u8, Directed, u8>> {
//     any::<Graph<(), u8, Directed, u8>>()
//         .prop_filter("graph is empty", |graph| graph.node_count() > 0)
// }

// #[cfg(not(miri))]
// proptest! {
//     #[test]
//     fn triangle_inequality(
//         graph in non_empty_graph(),
//         node in any::<Index>()
//     ) { let node = NodeIndex::new(node.index(graph.node_count())); let result = dijkstra(&graph,
//       node, None, |edge| *edge.weight() as u32);
//
//         // triangle inequality:
//         // d(v,u) <= d(v,v2) + d(v2,u)
//         for (node, weight) in &result {
//             for edge in graph.edges(*node) {
//                 let next = edge.target();
//                 let next_weight = *edge.weight() as u32;
//
//                 if result.contains_key(&next) {
//                     assert!(result[&next] <= *weight + next_weight);
//                 }
//             }
//         }
//     }
// }
