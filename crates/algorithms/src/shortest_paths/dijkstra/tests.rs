use alloc::vec::Vec;

use numi::borrow::Moo;
use petgraph_core::{Edge, Graph};
use petgraph_dino::DiDinoGraph;
use petgraph_utils::{GraphCollection, graph};

use crate::shortest_paths::{
    ShortestPath,
    common::tests::{Expect, TestCase, expected},
    dijkstra::Dijkstra,
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
    ], [
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
    ]
);

fn networkx_directed_expect_from(nodes: &networkx::NodeCollection) -> Vec<Expect<i32>> {
    expected!(nodes; [
        a -()> a: 0,
        a -(c)> b: 8,
        a -()> c: 5,
        a -(c, b)> d: 9,
        a -(c)> e: 7,
    ])
}

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
    ], [
        ab: a -> b: "apple",
        bc: b -> c: "cat",
        cd: c -> d: "giraffe",
        de: d -> e: "is",
        ef: e -> f: "banana",
        fa: f -> a: "bear",
        ad: a -> d: "elephant",
    ]
);

// TODO: multigraph

#[test]
fn path_from_directed_default_edge_cost() {
    let GraphCollection { graph, nodes, .. } = networkx::create();
    let expected = networkx_directed_expect_from(&nodes);

    let dijkstra = Dijkstra::directed();

    TestCase::new(&graph, &dijkstra, &expected).assert_path_from(nodes.a);
}

#[test]
fn distance_from_directed_default_edge_cost() {
    let GraphCollection { graph, nodes, .. } = networkx::create();
    let expected = networkx_directed_expect_from(&nodes);

    let dijkstra = Dijkstra::directed();

    TestCase::new(&graph, &dijkstra, &expected).assert_distance_from(nodes.a);
}

fn random_directed_expect_from(nodes: &random::NodeCollection) -> Vec<Expect<usize>> {
    expected!(nodes; [
        a -()> a: 0,
        a -()> b: 5,
        a -(b)> c: 8,
        a -()> d: 8,
        a -(d)> e: 10,
        a -(d, e)> f: 16,
    ])
}

fn edge_cost<S>(edge: Edge<S>) -> Moo<'_, usize>
where
    S: Graph,
    S::EdgeWeight: AsRef<[u8]>,
{
    Moo::Owned(edge.weight().as_ref().len())
}

#[test]
fn path_from_directed_custom_edge_cost() {
    let GraphCollection { graph, nodes, .. } = random::create();

    let dijkstra = Dijkstra::directed().with_edge_cost(edge_cost);
    let expected = random_directed_expect_from(&nodes);

    TestCase::new(&graph, &dijkstra, &expected).assert_path_from(nodes.a);
}

#[test]
fn distance_from_directed_custom_edge_cost() {
    let GraphCollection { graph, nodes, .. } = random::create();

    let dijkstra = Dijkstra::directed().with_edge_cost(edge_cost);
    let expected = random_directed_expect_from(&nodes);

    TestCase::new(&graph, &dijkstra, &expected).assert_distance_from(nodes.a);
}

fn networkx_undirected_expect_from(nodes: &networkx::NodeCollection) -> Vec<Expect<i32>> {
    expected!(nodes; [
        a -()> a: 0,
        a -(c)> b: 7,
        a -()> c: 5,
        a -(e)> d: 8,
        a -()> e: 7,
    ])
}

#[test]
fn path_from_undirected_default_edge_cost() {
    let GraphCollection { graph, nodes, .. } = networkx::create();

    let dijkstra = Dijkstra::undirected();
    let expected = networkx_undirected_expect_from(&nodes);

    TestCase::new(&graph, &dijkstra, &expected).assert_path_from(nodes.a);
}

#[test]
fn distance_from_undirected_default_edge_cost() {
    let GraphCollection { graph, nodes, .. } = networkx::create();

    let dijkstra = Dijkstra::undirected();
    let expected = networkx_undirected_expect_from(&nodes);

    TestCase::new(&graph, &dijkstra, &expected).assert_distance_from(nodes.a);
}

fn random_undirected_expect_from(nodes: &random::NodeCollection) -> Vec<Expect<usize>> {
    expected!(nodes; [
        a -()> a: 0,
        a -()> b: 5,
        a -(b)> c: 8,
        a -()> d: 8,
        a -(f)> e: 10,
        a -()> f: 4,
    ])
}

#[test]
fn path_from_undirected_custom_edge_cost() {
    let GraphCollection { graph, nodes, .. } = random::create();

    let dijkstra = Dijkstra::undirected().with_edge_cost(edge_cost);
    let expected = random_undirected_expect_from(&nodes);

    TestCase::new(&graph, &dijkstra, &expected).assert_path_from(nodes.a);
}

#[test]
fn distance_from_undirected_custom_edge_cost() {
    let GraphCollection { graph, nodes, .. } = random::create();

    let dijkstra = Dijkstra::undirected().with_edge_cost(edge_cost);
    let expected = random_undirected_expect_from(&nodes);

    TestCase::new(&graph, &dijkstra, &expected).assert_distance_from(nodes.a);
}

#[test]
fn lifetime() {
    let GraphCollection { graph, nodes, .. } = networkx::create();

    let dijkstra = Dijkstra::directed();

    let top3: Vec<_> = dijkstra
        .path_from(&graph, nodes.a)
        .unwrap()
        .take(3)
        .collect();

    drop(dijkstra);

    let top3: Vec<_> = top3
        .into_iter()
        .map(|route| route.into_cost().into_value())
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
