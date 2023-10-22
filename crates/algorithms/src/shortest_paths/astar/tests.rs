use alloc::{vec, vec::Vec};
use core::sync::atomic::{AtomicUsize, Ordering};

use ordered_float::{NotNan, OrderedFloat};
use petgraph_core::{base::MaybeOwned, edge::marker::Directed, Edge, GraphStorage, Node};
use petgraph_dino::{DiDinoGraph, DinoStorage, EdgeId, NodeId};
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
    ] as NodeId,
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
    ] as EdgeId
);

#[derive(Debug, Copy, Clone, PartialEq)]
struct Point {
    x: f32,
    y: f32,
}

impl Point {
    fn distance(self, other: Self) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
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
    ] as NodeId,
    [
        ab: a -> b: @{ a.distance(*b) },
        ad: a -> d: @{ a.distance(*d) },
        bc: b -> c: @{ b.distance(*c) },
        bf: b -> f: @{ b.distance(*f) },
        ce: c -> e: @{ c.distance(*e) },
        ef: e -> f: @{ e.distance(*f) },
        de: d -> e: @{ d.distance(*e) },
    ] as EdgeId
);

fn no_heuristic<'a, S>(_: Node<'a, S>, _: Node<'a, S>) -> MaybeOwned<'a, usize>
where
    S: GraphStorage,
{
    MaybeOwned::Owned(0)
}

// TODO: multigraph
// TODO: more test cases

#[test]
fn directed_path_between() {
    let GraphCollection {
        graph,
        nodes,
        edges,
    } = networkx::create();

    let astar = AStar::directed().with_heuristic(no_heuristic);
    let route = astar.path_between(&graph, &nodes.s, &nodes.v).unwrap();

    let nodes: Vec<_> = route
        .path
        .to_vec()
        .into_iter()
        .map(|node| *node.weight())
        .collect();

    assert_eq!(nodes, vec!["S", "X", "U", "V"]);
    assert_eq!(route.cost.into_value(), 9);
}

#[test]
fn directed_no_path_between() {
    let GraphCollection {
        graph,
        nodes,
        edges,
    } = networkx::create();

    let astar = AStar::directed().with_heuristic(no_heuristic);
    let route = astar.path_between(&graph, &nodes.s, &nodes.z);

    assert!(route.is_none());
}

#[test]
fn undirected_path_between() {
    let GraphCollection {
        graph,
        nodes,
        edges,
    } = networkx::create();

    let astar = AStar::undirected().with_heuristic(no_heuristic);
    let route = astar.path_between(&graph, &nodes.s, &nodes.v).unwrap();

    let nodes: Vec<_> = route
        .path
        .to_vec()
        .into_iter()
        .map(|node| *node.weight())
        .collect();

    assert_eq!(nodes, vec!["S", "Y", "V"]);
    assert_eq!(route.cost.into_value(), 8);
}

#[test]
fn undirected_no_path_between() {
    let GraphCollection {
        graph,
        nodes,
        edges,
    } = networkx::create();

    let astar = AStar::undirected().with_heuristic(no_heuristic);
    let route = astar.path_between(&graph, &nodes.s, &nodes.z);

    assert!(route.is_none());
}

#[test]
fn directed_distance_between() {
    let GraphCollection {
        graph,
        nodes,
        edges,
    } = networkx::create();

    let astar = AStar::directed().with_heuristic(no_heuristic);
    let cost = astar.distance_between(&graph, &nodes.s, &nodes.v).unwrap();

    assert_eq!(cost.into_value(), 9);
}

#[test]
fn directed_no_distance_between() {
    let GraphCollection {
        graph,
        nodes,
        edges,
    } = networkx::create();

    let astar = AStar::directed().with_heuristic(no_heuristic);
    let cost = astar.distance_between(&graph, &nodes.s, &nodes.z);

    assert!(cost.is_none());
}

#[test]
fn undirected_distance_between() {
    let GraphCollection {
        graph,
        nodes,
        edges,
    } = networkx::create();

    let astar = AStar::undirected().with_heuristic(no_heuristic);
    let cost = astar.distance_between(&graph, &nodes.s, &nodes.v).unwrap();

    assert_eq!(cost.into_value(), 8);
}

#[test]
fn undirected_no_distance_between() {
    let GraphCollection {
        graph,
        nodes,
        edges,
    } = networkx::create();

    let astar = AStar::undirected().with_heuristic(no_heuristic);
    let cost = astar.distance_between(&graph, &nodes.s, &nodes.z);

    assert!(cost.is_none());
}

fn manhattan_distance<'a, S>(
    source: Node<'a, S>,
    target: Node<'a, S>,
) -> MaybeOwned<'a, NotNan<f32>>
where
    S: GraphStorage<NodeWeight = Point>,
{
    let source = source.weight();
    let target = target.weight();

    let distance =
        NotNan::new(source.manhattan_distance(*target)).expect("distance should be a number");

    MaybeOwned::Owned(distance)
}

fn ensure_not_nan<S>(edge: Edge<S>) -> MaybeOwned<'_, NotNan<f32>>
where
    S: GraphStorage<EdgeWeight = f32>,
{
    let weight = NotNan::new(*edge.weight()).expect("weight should be a number");

    MaybeOwned::Owned(weight)
}

#[test]
fn directed_path_between_manhattan() {
    let GraphCollection {
        graph,
        nodes,
        edges,
    } = planar::create();

    let astar = AStar::directed()
        .with_edge_cost(ensure_not_nan)
        .with_heuristic(manhattan_distance);
    let route = astar.path_between(&graph, &nodes.a, &nodes.f).unwrap();

    let path: Vec<_> = route
        .path
        .to_vec()
        .into_iter()
        .map(|node| *node.id())
        .collect();

    assert_eq!(path, [nodes.a, nodes.b, nodes.f]);

    let a = graph.node(&nodes.a).unwrap();
    let b = graph.node(&nodes.b).unwrap();
    let f = graph.node(&nodes.f).unwrap();

    assert_eq!(
        route.cost.into_value(),
        a.weight().distance(*b.weight()) + b.weight().distance(*f.weight())
    );
}

#[test]
fn directed_distance_between_manhattan() {
    let GraphCollection {
        graph,
        nodes,
        edges,
    } = planar::create();

    let astar = AStar::directed()
        .with_edge_cost(ensure_not_nan)
        .with_heuristic(manhattan_distance);
    let cost = astar.distance_between(&graph, &nodes.a, &nodes.f).unwrap();

    let a = graph.node(&nodes.a).unwrap();
    let b = graph.node(&nodes.b).unwrap();
    let f = graph.node(&nodes.f).unwrap();

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
    ] as NodeId,
    [
        ab: a -> b: 3,
        bc: b -> c: 3,
        cd: c -> d: 3,
        ac: a -> c: 8,
        ad: a -> d: 10,
    ] as EdgeId
);

fn admissible_inconsistent<'a, S>(source: Node<'a, S>, target: Node<'a, S>) -> MaybeOwned<'a, usize>
where
    S: GraphStorage,
    S::NodeWeight: AsRef<str>,
{
    match source.weight().as_ref() {
        "A" => MaybeOwned::Owned(9),
        "B" => MaybeOwned::Owned(6),
        _ => MaybeOwned::Owned(0),
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
    let GraphCollection {
        graph,
        nodes,
        edges,
    } = inconsistent::create();

    let astar = AStar::directed().with_heuristic(admissible_inconsistent);
    let route = astar.path_between(&graph, &nodes.a, &nodes.d).unwrap();

    let path: Vec<_> = route
        .path
        .to_vec()
        .into_iter()
        .map(|node| *node.id())
        .collect();

    assert_eq!(path, [nodes.a, nodes.b, nodes.c, nodes.d]);
    assert_eq!(route.cost.into_value(), 9);
}

graph!(factory(runtime) => DiDinoGraph<char, usize>;
    [
        a: 'A',
        b: 'B',
        c: 'C',
        d: 'D',
        e: 'E',
    ] as NodeId,
    [
        ab: a -> b: 2,
        ac: a -> c: 3,
        bd: b -> d: 3,
        cd: c -> d: 1,
        de: d -> e: 1,
    ] as EdgeId
);

#[test]
fn optimal_runtime() {
    static CALLS: AtomicUsize = AtomicUsize::new(0);

    let GraphCollection {
        graph,
        nodes,
        edges,
    } = runtime::create();

    fn edge_cost<S>(edge: Edge<S>) -> MaybeOwned<usize>
    where
        S: GraphStorage<EdgeWeight = usize>,
    {
        CALLS.fetch_add(1, Ordering::SeqCst);
        MaybeOwned::Borrowed(edge.weight())
    }

    let astar = AStar::directed()
        .with_edge_cost(edge_cost)
        .with_heuristic(no_heuristic);

    astar.path_between(&graph, &nodes.a, &nodes.e).unwrap();

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
