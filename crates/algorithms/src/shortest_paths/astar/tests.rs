use alloc::{vec, vec::Vec};

use ordered_float::OrderedFloat;
use petgraph_core::{base::MaybeOwned, Edge, GraphStorage, Node};
use petgraph_dino::{DiDinoGraph, EdgeId, NodeId};
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
) -> MaybeOwned<'a, OrderedFloat<f32>>
where
    S: GraphStorage,
    S::NodeWeight: AsRef<Point>,
{
    let source = source.weight().as_ref();
    let target = target.weight().as_ref();

    MaybeOwned::Owned(OrderedFloat(source.manhattan_distance(*target)))
}

fn into_ordered_float<S>(edge: Edge<S>) -> MaybeOwned<'_, OrderedFloat<f32>>
where
    S: GraphStorage,
    S::EdgeWeight: AsRef<f32>,
{
    MaybeOwned::Owned(OrderedFloat(*edge.weight().as_ref()))
}

#[test]
fn directed_path_between_manhattan() {
    let GraphCollection {
        graph,
        nodes,
        edges,
    } = planar::create();

    let astar = AStar::directed()
        .with_edge_cost(into_ordered_float)
        .with_heuristic(manhattan_distance);
    let route = astar.path_between(&graph, &nodes.a, &nodes.f).unwrap();

    let path: Vec<_> = route
        .path
        .to_vec()
        .into_iter()
        .map(|node| *node.id())
        .collect();

    assert_eq!(path, [nodes.a, nodes.b, nodes.f]);
    // TODO: distance
}

// TODO: admissible
