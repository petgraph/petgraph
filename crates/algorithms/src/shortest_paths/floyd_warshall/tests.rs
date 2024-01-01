use alloc::{vec, vec::Vec};

use hashbrown::HashSet;
use petgraph_core::node::NodeId;
use petgraph_dino::DiDinoGraph;
use petgraph_utils::{graph, GraphCollection};

use crate::shortest_paths::{
    common::tests::{expected, Expect, TestCase},
    floyd_warshall::{error::FloydWarshallError, FloydWarshall},
    ShortestPath,
};

graph!(
    /// Graph:
    ///
    /// ```text
    /// A → B → E → F
    /// ↑   ↓   ↑   ↓
    /// D ← C   H ← G
    /// ```
    factory(uniform) => DiDinoGraph<&'static str, usize>;
    [
        a: "A",
        b: "B",
        c: "C",
        d: "D",
        e: "E",
        f: "F",
        g: "G",
        h: "H",
    ],
    [
        ab: a -> b: 1,
        bc: b -> c: 1,
        cd: c -> d: 1,
        da: d -> a: 1,
        ef: e -> f: 1,
        be: b -> e: 1,
        fg: f -> g: 1,
        gh: g -> h: 1,
        he: h -> e: 1,
    ]
);

fn uniform_expect(nodes: &uniform::NodeCollection) -> Vec<Expect<usize>> {
    expected!(nodes; [
        a -()> a: 0,
        a -()> b: 1,
        a -(b)> c: 2,
        a -(b, c)> d: 3,
        a -(b)> e: 2,
        a -(b, e)> f: 3,
        a -(b, e, f)> g: 4,
        a -(b, e, f, g)> h: 5,

        b -(c, d)> a: 3,
        b -()> b: 0,
        b -()> c: 1,
        b -(c)> d: 2,
        b -()> e: 1,
        b -(e)> f: 2,
        b -(e, f)> g: 3,
        b -(e, f, g)> h: 4,

        c -(d)> a: 2,
        c -(d, a)> b: 3,
        c -()> c: 0,
        c -()> d: 1,
        c -(d, a, b)> e: 4,
        c -(d, a, b, e)> f: 5,
        c -(d, a, b, e, f)> g: 6,
        c -(d, a, b, e, f, g)> h: 7,

        d -()> a: 1,
        d -(a)> b: 2,
        d -(a, b)> c: 3,
        d -()> d: 0,
        d -(a, b)> e: 3,
        d -(a, b, e)> f: 4,
        d -(a, b, e, f)> g: 5,
        d -(a, b, e, f, g)> h: 6,

        e -()> e: 0,
        e -()> f: 1,
        e -(f)> g: 2,
        e -(f, g)> h: 3,

        f -(g, h)> e: 3,
        f -()> f: 0,
        f -()> g: 1,
        f -(g)> h: 2,

        g -(h)> e: 2,
        g -(h, e)> f: 3,
        g -()> g: 0,
        g -()> h: 1,

        h -()> e: 1,
        h -(e)> f: 2,
        h -(e, f)> g: 3,
        h -()> h: 0,
    ])
}

#[test]
fn uniform_directed_path() {
    let GraphCollection { graph, nodes, .. } = uniform::create();
    let expected = uniform_expect(&nodes);

    let floyd_warshall = FloydWarshall::directed();

    TestCase::new(&graph, &floyd_warshall, &expected).assert_every_path();
}

#[test]
fn uniform_directed_distance() {
    let GraphCollection { graph, nodes, .. } = uniform::create();
    let expected = uniform_expect(&nodes);

    let floyd_warshall = FloydWarshall::directed();

    TestCase::new(&graph, &floyd_warshall, &expected).assert_every_distance();
}

graph!(
    /// Graph:
    ///
    /// ```text
    /// A → B
    /// ↓ ⤩ ↓
    /// D ← C
    /// ```
    factory(weighted) => DiDinoGraph<&'static str, usize>;
    [
        a: "A",
        b: "B",
        c: "C",
        d: "D",
    ],
    [
        ab: a -> b: 1,
        ac: a -> c: 4,
        ad: a -> d: 10,
        bc: b -> c: 2,
        bd: b -> d: 2,
        cd: c -> d: 2,
    ]
);

fn directed_weighted_expect(nodes: &weighted::NodeCollection) -> Vec<Expect<usize>> {
    expected!(nodes; [
        a -()> a: 0,
        a -()> b: 1,
        a -(b)> c: 3,
        a -(b)> d: 3,

        b -()> b: 0,
        b -()> c: 2,
        b -()> d: 2,

        c -()> c: 0,
        c -()> d: 2,

        d -()> d: 0
    ])
}

#[test]
fn weighted_directed_path() {
    let GraphCollection { graph, nodes, .. } = weighted::create();
    let expected = directed_weighted_expect(&nodes);

    let floyd_warshall = FloydWarshall::directed();

    TestCase::new(&graph, &floyd_warshall, &expected).assert_every_path();
}

#[test]
fn weighted_directed_path_between() {
    let GraphCollection { graph, nodes, .. } = weighted::create();

    let floyd_warshall = FloydWarshall::directed();

    let route = floyd_warshall
        .path_between(&graph, nodes.a, nodes.c)
        .unwrap();
    let (path, cost) = route.into_parts();

    assert_eq!(cost.into_value(), 3);
    assert_eq!(
        path.to_vec()
            .into_iter()
            .map(|node| node.id())
            .collect::<Vec<_>>(),
        vec![nodes.a, nodes.b, nodes.c]
    );
}

#[test]
fn weighted_directed_path_from() {
    let GraphCollection { graph, nodes, .. } = weighted::create();

    let floyd_warshall = FloydWarshall::directed();

    let paths = floyd_warshall.path_from(&graph, nodes.a).unwrap();

    let mut expected: HashSet<_> = [nodes.a, nodes.b, nodes.c, nodes.d].into_iter().collect();
    // cost is tested above, this just checks that the filter is correct.

    for route in paths {
        assert_eq!(route.path().source().id(), nodes.a);
        assert!(expected.remove(&route.path().target().id()));
    }
}

#[test]
fn weighted_directed_path_to() {
    let GraphCollection { graph, nodes, .. } = weighted::create();

    let floyd_warshall = FloydWarshall::directed();

    let paths = floyd_warshall.path_to(&graph, nodes.c).unwrap();

    let mut expected: HashSet<_> = [nodes.a, nodes.b, nodes.c].into_iter().collect();
    // cost is tested above, this just checks that the filter is correct.

    for route in paths {
        assert!(expected.remove(&route.path().source().id()));
        assert_eq!(route.path().target().id(), nodes.c);
    }
}

#[test]
fn weighted_directed_distance() {
    let GraphCollection { graph, nodes, .. } = weighted::create();
    let expected = directed_weighted_expect(&nodes);

    let floyd_warshall = FloydWarshall::directed();

    TestCase::new(&graph, &floyd_warshall, &expected).assert_every_distance();
}

fn undirected_weighted_expect(nodes: &weighted::NodeCollection) -> Vec<Expect<usize>> {
    expected!(nodes; [
        a -()> a: 0,
        a -()> b: 1,
        a -(b)> c: 3,
        a -(b)> d: 3,

        b -()> a: 1,
        b -()> b: 0,
        b -()> c: 2,
        b -()> d: 2,

        c -(b)> a: 3,
        c -()> b: 2,
        c -()> c: 0,
        c -()> d: 2,

        d -(b)> a: 3,
        d -()> b: 2,
        d -()> c: 2,
        d -()> d: 0,
    ])
}

#[test]
fn weighted_undirected_path() {
    let GraphCollection { graph, nodes, .. } = weighted::create();
    let expected = undirected_weighted_expect(&nodes);

    let floyd_warshall = FloydWarshall::undirected();

    TestCase::new(&graph, &floyd_warshall, &expected).assert_every_path();
}

#[test]
fn weighted_undirected_path_between() {
    let GraphCollection { graph, nodes, .. } = weighted::create();

    let floyd_warshall = FloydWarshall::undirected();

    let route = floyd_warshall
        .path_between(&graph, nodes.a, nodes.c)
        .unwrap();

    let (path, cost) = route.into_parts();
    assert_eq!(cost.into_value(), 3);
    assert_eq!(
        path.to_vec()
            .into_iter()
            .map(|node| node.id())
            .collect::<Vec<_>>(),
        vec![nodes.a, nodes.b, nodes.c]
    );
}

#[test]
fn weighted_undirected_path_from() {
    let GraphCollection { graph, nodes, .. } = weighted::create();

    let floyd_warshall = FloydWarshall::undirected();

    let paths = floyd_warshall.path_from(&graph, nodes.a).unwrap();

    let mut expected: HashSet<_> = [nodes.a, nodes.b, nodes.c, nodes.d].into_iter().collect();
    // cost is tested above, this just checks that the filter is correct.

    for route in paths {
        assert_eq!(route.path().source().id(), nodes.a);
        assert!(expected.remove(&route.path().target().id()));
    }
}

#[test]
fn weighted_undirected_path_to() {
    let GraphCollection { graph, nodes, .. } = weighted::create();

    let floyd_warshall = FloydWarshall::undirected();

    let paths = floyd_warshall.path_to(&graph, nodes.c).unwrap();

    let mut expected: HashSet<_> = [nodes.a, nodes.b, nodes.c, nodes.d].into_iter().collect();
    // cost is tested above, this just checks that the filter is correct.

    for route in paths {
        assert!(expected.remove(&route.path().source().id()));
        assert_eq!(route.path().target().id(), nodes.c);
    }
}

#[test]
fn weighted_undirected_distance() {
    let GraphCollection { graph, nodes, .. } = weighted::create();
    let expected = undirected_weighted_expect(&nodes);

    let floyd_warshall = FloydWarshall::undirected();

    TestCase::new(&graph, &floyd_warshall, &expected).assert_every_distance();
}

graph!(factory(negative_cycle) => DiDinoGraph<&'static str, isize>;
    [
        a: "A",
        b: "B",
        c: "C",
    ],
    [
        ab: a -> b: 1,
        bc: b -> c: -3,
        ca: c -> a: 1,
    ]
);

#[test]
fn directed_negative_cycle() {
    let GraphCollection { graph, nodes, .. } = negative_cycle::create();

    let floyd_warshall = FloydWarshall::directed();

    let Err(error) = floyd_warshall.every_path(&graph) else {
        panic!("expected error");
    };

    assert_eq!(error.current_context(), &FloydWarshallError::NegativeCycle);
    let participants: HashSet<_> = error.request_ref::<NodeId>().copied().collect();

    assert_eq!(
        participants,
        [nodes.a, nodes.b, nodes.c]
            .into_iter()
            .collect::<HashSet<_>>()
    );
}

// #[cfg(not(miri))]
// proptest! {
//     // because floyd-warshall is O(n^3) we limit the size of the graph to 32 nodes and 32 edges
//     #[test]
//     fn verify_dijkstra(
//         graph in graph_strategy::<Graph::<(), u8, Directed, u8>>(
//             false,
//             false,
//             0..32,
//             Some(Arc::new(|max| {
//                 SizeRange::new(0..=32)
//             }))
//         ),
//     ) { let received = floyd_warshall(&graph, |edge| *edge.weight() as u32) .expect("expected
//       floyd-warshall to succeed");
//
//         for node in graph.node_identifiers() {
//             let expected = dijkstra(&graph, node, None, |edge| *edge.weight() as u32);
//
//             for target in graph.node_identifiers() {
//                 if let Some(expected) = expected.get(&target) {
//                     let received = received.get(&(node, target)).unwrap();
//
//                     prop_assert_eq!(received, expected);
//                 } else {
//                     // if there are no path between two nodes then floyd_warshall will return
// maximum value possible                     // TODO: no that's just bad design
//                     prop_assert_eq!(received.get(&(node, target)), Some(&u32::MAX));
//                 }
//             }
//         }
//     }
// }
