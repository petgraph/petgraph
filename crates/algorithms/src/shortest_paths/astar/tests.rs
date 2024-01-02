use alloc::{vec, vec::Vec};
use core::sync::atomic::{AtomicUsize, Ordering};

use numi::borrow::Moo;
use ordered_float::NotNan;
use petgraph_core::{edge::marker::Directed, Edge, GraphStorage, Node};
use petgraph_dino::{DiDinoGraph, DinoStorage};
use petgraph_utils::{graph, GraphCollection};

use crate::shortest_paths::{AStar, ShortestDistance, ShortestPath};

graph!(
    /// Uses the graph from networkx
    ///
    /// <https://github.com/networkx/networkx/blob/ce237b7d63920ddcf8eb749f6be4db42cf3a5f85/networkx/algorithms/shortest_paths/tests/test_astar.py#L22>
    factory(networkx) => DiDinoGraph<&'static str, usize>;
    [
        s: "S",
        u: "U",
        v: "V",
        x: "X",
        y: "Y",
        z: "Z",
    ],
    [
        su: s -> u: 10,
        sx: s -> x: 5,
        uv: u -> v: 1,
        ux: u -> x: 2,
        vy: v -> y: 1,
        xu: x -> u: 3,
        xv: x -> v: 5,
        xy: x -> y: 2,
        ys: y -> s: 7,
        yv: y -> v: 6,
    ]
);

#[derive(Debug, Copy, Clone, PartialEq)]
struct Point {
    x: f32,
    y: f32,
}

impl Point {
    fn distance(self, other: Self) -> f32 {
        (self.x - other.x).hypot(self.y - other.y)
    }

    fn manhattan_distance(self, other: Self) -> f32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

graph!(
    factory(planar) => DiDinoGraph<Point, f32>;
    [
        a: Point { x: 0.0, y: 0.0 },
        b: Point { x: 2.0, y: 0.0 },
        c: Point { x: 1.0, y: 1.0 },
        d: Point { x: 0.0, y: 2.0 },
        e: Point { x: 3.0, y: 3.0 },
        f: Point { x: 4.0, y: 2.0 },
        g: Point { x: 5.0, y: 5.0 },
    ],
    [
        ab: a -> b: @{ a.distance(*b) },
        ad: a -> d: @{ a.distance(*d) },
        bc: b -> c: @{ b.distance(*c) },
        bf: b -> f: @{ b.distance(*f) },
        ce: c -> e: @{ c.distance(*e) },
        ef: e -> f: @{ e.distance(*f) },
        de: d -> e: @{ d.distance(*e) },
    ]
);

const fn no_heuristic<'a, S>(_: Node<'a, S>, _: Node<'a, S>) -> Moo<'a, usize>
where
    S: GraphStorage,
{
    Moo::Owned(0)
}

// TODO: multigraph

#[test]
fn directed_path_between() {
    let GraphCollection { graph, nodes, .. } = networkx::create();

    let astar = AStar::directed().with_heuristic(no_heuristic);
    let route = astar.path_between(&graph, nodes.s, nodes.v).unwrap();
    let (path, cost) = route.into_parts();

    let nodes: Vec<_> = path
        .to_vec()
        .into_iter()
        .map(|node| *node.weight())
        .collect();

    assert_eq!(nodes, vec!["S", "X", "U", "V"]);
    assert_eq!(cost.into_value(), 9);
}

#[test]
fn directed_no_path_between() {
    let GraphCollection { graph, nodes, .. } = networkx::create();

    let astar = AStar::directed().with_heuristic(no_heuristic);
    let route = astar.path_between(&graph, nodes.s, nodes.z);

    assert!(route.is_none());
}

#[test]
fn undirected_path_between() {
    let GraphCollection { graph, nodes, .. } = networkx::create();

    let astar = AStar::undirected().with_heuristic(no_heuristic);
    let route = astar.path_between(&graph, nodes.s, nodes.v).unwrap();
    let (path, cost) = route.into_parts();

    let nodes: Vec<_> = path
        .to_vec()
        .into_iter()
        .map(|node| *node.weight())
        .collect();

    assert_eq!(nodes, vec!["S", "Y", "V"]);
    assert_eq!(cost.into_value(), 8);
}

#[test]
fn undirected_no_path_between() {
    let GraphCollection { graph, nodes, .. } = networkx::create();

    let astar = AStar::undirected().with_heuristic(no_heuristic);
    let route = astar.path_between(&graph, nodes.s, nodes.z);

    assert!(route.is_none());
}

#[test]
fn directed_distance_between() {
    let GraphCollection { graph, nodes, .. } = networkx::create();

    let astar = AStar::directed().with_heuristic(no_heuristic);
    let cost = astar.distance_between(&graph, nodes.s, nodes.v).unwrap();

    assert_eq!(cost.into_value(), 9);
}

#[test]
fn directed_no_distance_between() {
    let GraphCollection { graph, nodes, .. } = networkx::create();

    let astar = AStar::directed().with_heuristic(no_heuristic);
    let cost = astar.distance_between(&graph, nodes.s, nodes.z);

    assert!(cost.is_none());
}

#[test]
fn undirected_distance_between() {
    let GraphCollection { graph, nodes, .. } = networkx::create();

    let astar = AStar::undirected().with_heuristic(no_heuristic);
    let cost = astar.distance_between(&graph, nodes.s, nodes.v).unwrap();

    assert_eq!(cost.into_value(), 8);
}

#[test]
fn undirected_no_distance_between() {
    let GraphCollection { graph, nodes, .. } = networkx::create();

    let astar = AStar::undirected().with_heuristic(no_heuristic);
    let cost = astar.distance_between(&graph, nodes.s, nodes.z);

    assert!(cost.is_none());
}

fn manhattan_distance<'graph, S>(
    source: Node<'graph, S>,
    target: Node<'graph, S>,
) -> Moo<'graph, NotNan<f32>>
where
    S: GraphStorage<NodeWeight = Point>,
{
    let source = source.weight();
    let target = target.weight();

    let distance =
        NotNan::new(source.manhattan_distance(*target)).expect("distance should be a number");

    Moo::Owned(distance)
}

fn ensure_not_nan<S>(edge: Edge<S>) -> Moo<'_, NotNan<f32>>
where
    S: GraphStorage<EdgeWeight = f32>,
{
    let weight = NotNan::new(*edge.weight()).expect("weight should be a number");

    Moo::Owned(weight)
}

#[test]
fn directed_path_between_manhattan() {
    let GraphCollection { graph, nodes, .. } = planar::create();

    let astar = AStar::directed()
        .with_edge_cost(ensure_not_nan)
        .with_heuristic(manhattan_distance);

    let route = astar.path_between(&graph, nodes.a, nodes.f).unwrap();

    let (path, cost) = route.into_parts();

    let path: Vec<_> = path.to_vec().into_iter().map(|node| node.id()).collect();

    assert_eq!(path, [nodes.a, nodes.b, nodes.f]);

    let a = graph.node(nodes.a).unwrap();
    let b = graph.node(nodes.b).unwrap();
    let f = graph.node(nodes.f).unwrap();

    assert_eq!(
        cost.into_value(),
        a.weight().distance(*b.weight()) + b.weight().distance(*f.weight())
    );
}

#[test]
fn directed_distance_between_manhattan() {
    let GraphCollection { graph, nodes, .. } = planar::create();

    let astar = AStar::directed()
        .with_edge_cost(ensure_not_nan)
        .with_heuristic(manhattan_distance);
    let cost = astar.distance_between(&graph, nodes.a, nodes.f).unwrap();

    let a = graph.node(nodes.a).unwrap();
    let b = graph.node(nodes.b).unwrap();
    let f = graph.node(nodes.f).unwrap();

    assert_eq!(
        cost.into_value(),
        a.weight().distance(*b.weight()) + b.weight().distance(*f.weight())
    );
}

graph!(factory(inconsistent) => DiDinoGraph<&'static str, usize>;
    [
        a: "A",
        b: "B",
        c: "C",
        d: "D",
    ],
    [
        ab: a -> b: 3,
        bc: b -> c: 3,
        cd: c -> d: 3,
        ac: a -> c: 8,
        ad: a -> d: 10,
    ]
);

fn admissible_inconsistent<'a, S>(source: Node<'a, S>, _target: Node<'a, S>) -> Moo<'a, usize>
where
    S: GraphStorage,
    S::NodeWeight: AsRef<str>,
{
    match source.weight().as_ref() {
        "A" => Moo::Owned(9),
        "B" => Moo::Owned(6),
        _ => Moo::Owned(0),
    }
}

/// Excerpt from https://en.wikipedia.org/wiki/A*_search_algorithm#Admissibility_and_optimality
///
/// > If the heuristic function is admissible – meaning that it never overestimates the actual
/// > cost to get to the goal – A* is guaranteed to return a least-cost path from start to goal.
///
/// If a heuristic is admissible, but inconsistent, A* will still find the optimal path, but it
/// may expand more nodes than needed.
///
/// Papers:
/// * <https://www.sciencedirect.com/science/article/pii/S0004370211000221>
/// * <https://citeseerx.ist.psu.edu/document?repid=rep1&type=pdf&doi=1f81b34c3729709e5d81e4d2dc33fa609b335473>
// TODO: move to algorithm docs
#[test]
fn directed_path_between_admissible_inconsistent() {
    let GraphCollection { graph, nodes, .. } = inconsistent::create();

    let astar = AStar::directed().with_heuristic(admissible_inconsistent);
    let route = astar.path_between(&graph, nodes.a, nodes.d).unwrap();

    let (path, cost) = route.into_parts();

    let path: Vec<_> = path.to_vec().into_iter().map(|node| node.id()).collect();

    assert_eq!(path, [nodes.a, nodes.b, nodes.c, nodes.d]);
    assert_eq!(cost.into_value(), 9);
}

graph!(factory(runtime) => DiDinoGraph<char, usize>;
    [
        a: 'A',
        b: 'B',
        c: 'C',
        d: 'D',
        e: 'E',
    ],
    [
        ab: a -> b: 2,
        ac: a -> c: 3,
        bd: b -> d: 3,
        cd: c -> d: 1,
        de: d -> e: 1,
    ]
);

#[test]
fn optimal_runtime() {
    // Needed to bind the lifetime of the closure, does some compiler magic.
    fn bind<S, T>(closure: impl Fn(Edge<S>) -> Moo<T>) -> impl Fn(Edge<S>) -> Moo<T> {
        closure
    }

    static CALLS: AtomicUsize = AtomicUsize::new(0);

    let GraphCollection { graph, nodes, .. } = runtime::create();

    let astar = AStar::directed()
        .with_edge_cost(bind(|edge: Edge<DinoStorage<char, usize, Directed>>| {
            CALLS.fetch_add(1, Ordering::SeqCst);
            Moo::Borrowed(edge.weight())
        }))
        .with_heuristic(no_heuristic);

    astar.path_between(&graph, nodes.a, nodes.e).unwrap();

    // A* is runtime optimal in the sense it won't expand more nodes than needed, for the given
    // heuristic. Here, A* should expand, in order: A, B, C, D, E. This should should ask for
    // the costs of edges (A, B), (A, C), (B, D), (C, D), (D, E). Node D will be added
    // to `visit_next` twice, but should only be expanded once. If it is erroneously
    // expanded twice, it will call for (D, E) again and `times_called` will be 6.
    assert_eq!(CALLS.load(Ordering::SeqCst), 5);
}

// fn expand_graph_value_space(graph: &DiGraph<(), u8, u8>) -> Graph<(), u64, Directed, u8> {
//     graph.map(|_, _| (), |_, weight| u64::from(*weight))
// }
//
// prop_compose! {
//     // we allow selecting the same node as start and end, because it's a valid use case.
//     // we also expand the value space from the initial `u8` to `u64` to avoid overflows.
//     fn graph_with_two_nodes()
//        (graph in any::<DiGraph::<(), u8, u8>>().prop_filter("graph must have at least one node",
// |graph| graph.node_count() >= 1))        (start in 0..graph.node_count(), end in
// 0..graph.node_count(), graph in Just(graph))         -> (DiGraph<(), u64, u8>, NodeIndex<u8>,
// NodeIndex<u8>) {         (expand_graph_value_space(&graph), NodeIndex::new(start),
// NodeIndex::new(end))     }
// }
//
// #[cfg(not(miri))]
// proptest! {
//     #[test]
//     fn null_heuristic_is_dijkstra(
//         (graph, start, end) in graph_with_two_nodes()
//     ) { let astar_path = astar(&graph, start, |node| node == end, |edge| *edge.weight(), |_| 0);
//       let dijkstra_path = dijkstra(&graph, start, Some(end), |edge| *edge.weight());
//
//
//         match astar_path {
//             None => {
//                 prop_assert_eq!(dijkstra_path.get(&end), None);
//             }
//             Some((distance, _)) => {
//                 prop_assert_eq!(dijkstra_path.get(&end), Some(&distance));
//             }
//         }
//     }
// }
