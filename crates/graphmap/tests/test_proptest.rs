#![cfg(feature = "proptest")]
extern crate core;

use core::fmt;

use petgraph_core::edge::{Directed, EdgeType, Undirected};
use petgraph_graphmap::{GraphMap, NodeTrait};
use proptest::prelude::*;

fn assert_graphmap_consistent<N, E, Ty>(g: &GraphMap<N, E, Ty>)
where
    Ty: EdgeType,
    N: NodeTrait + fmt::Debug,
{
    for (a, b, _weight) in g.all_edges() {
        assert!(g.contains_edge(a, b), "Edge not in graph! {a:?} to {b:?}",);

        assert!(
            g.neighbors(a).any(|x| x == b),
            "Edge {:?} not in neighbor list for {:?}",
            (a, b),
            a
        );

        if !g.is_directed() {
            assert!(
                g.neighbors(b).any(|x| x == a),
                "Edge {:?} not in neighbor list for {:?}",
                (b, a),
                b
            );
        }
    }
}

fn remove_node<Ty>(graph: &mut GraphMap<i8, (), Ty>, node: i8)
where
    Ty: EdgeType,
{
    assert_graphmap_consistent(graph);
    assert!(graph.contains_node(node));

    graph.remove_node(node);

    assert_graphmap_consistent(graph);
    assert!(!graph.contains_node(node));
}

fn remove_edge<Ty>(graph: &mut GraphMap<i8, (), Ty>, a: i8, b: i8)
where
    Ty: EdgeType,
{
    assert_graphmap_consistent(graph);
    assert!(graph.contains_edge(a, b));
    assert!(graph.neighbors(a).any(|x| x == b));

    graph.remove_edge(a, b);

    assert_graphmap_consistent(graph);
    assert!(!graph.contains_edge(a, b));
    assert!(!graph.neighbors(a).any(|x| x == b));
}

fn add_remove_edge<Ty>(graph: &mut GraphMap<i8, (), Ty>, a: i8, b: i8)
where
    Ty: EdgeType,
{
    assert_graphmap_consistent(graph);
    assert!(!graph.contains_edge(a, b));

    graph.add_edge(a, b, ());
    assert_graphmap_consistent(graph);

    assert!(graph.contains_edge(a, b));
    assert!(graph.neighbors(a).any(|x| x == b));
    if !graph.is_directed() {
        assert!(graph.neighbors(b).any(|x| x == a));
    }

    graph.remove_edge(a, b);
    assert_graphmap_consistent(graph);

    assert!(!graph.contains_edge(a, b));
    assert!(!graph.neighbors(a).any(|x| x == b));
    if !graph.is_directed() {
        assert!(!graph.neighbors(b).any(|x| x == a));
    }
}

fn find_free_edge<Ty>(graph: &GraphMap<i8, (), Ty>) -> (i8, i8)
where
    Ty: EdgeType,
{
    assert_graphmap_consistent(graph);

    let mut a = i8::MIN;
    let mut b = i8::MIN;

    loop {
        if !graph.contains_edge(a, b) {
            break (a, b);
        }

        b += 1;

        if b >= i8::try_from(graph.node_count()).expect("overflow") {
            a = a.checked_add(1).expect("overflow");
            b = 0;
        }
    }
}

fn graph_and_node<Ty>() -> impl Strategy<Value = (GraphMap<i8, (), Ty>, i8)>
where
    Ty: EdgeType + Clone + 'static,
{
    any::<GraphMap<i8, (), Ty>>()
        .prop_filter("graph must have nodes", |graph| graph.node_count() > 0)
        .prop_flat_map(|graph| {
            let nodes = graph.node_count();

            (Just(graph), 0..nodes)
        })
        .prop_map(|(graph, node)| {
            let node = graph.nodes().nth(node).expect("node not in graph");

            (graph, node)
        })
}

fn graph_and_edge<Ty>() -> impl Strategy<Value = (GraphMap<i8, (), Ty>, (i8, i8))>
where
    Ty: EdgeType + Clone + 'static,
{
    any::<GraphMap<i8, (), Ty>>()
        .prop_filter("graph must have edges", |graph| graph.edge_count() > 0)
        .prop_flat_map(|graph| {
            let edges = graph.edge_count();

            (Just(graph), 0..edges)
        })
        .prop_map(|(graph, edge)| {
            let (a, b, _) = graph.all_edges().nth(edge).expect("edge not in graph");

            (graph, (a, b))
        })
}

// The maximum amount of edges we can have is u8::MAX * u8::MAX, if we have more than that, we
// cannot find a free edge (the `find_free_edge` function will panic).
fn at_least_one_free_edge<Ty>() -> impl Strategy<Value = GraphMap<i8, (), Ty>>
where
    Ty: EdgeType + 'static,
{
    any::<GraphMap<i8, (), Ty>>().prop_filter("graph must have at least one free edge", |graph| {
        graph.edge_count() < u8::MAX as usize * u8::MAX as usize
    })
}

#[cfg(not(miri))]
proptest! {
    #[test]
    fn remove_node_directed((mut graph, node) in graph_and_node::<Directed>()) {
        remove_node(&mut graph, node);
    }

    #[test]
    fn remove_node_undirected((mut graph, node) in graph_and_node::<Undirected>()) {
        remove_node(&mut graph, node);
    }

    #[test]
    fn remove_edge_directed((mut graph, edge) in graph_and_edge::<Directed>()) {
        remove_edge(&mut graph, edge.0, edge.1);
    }

    #[test]
    fn remove_edge_undirected((mut graph, edge) in graph_and_edge::<Undirected>()) {
        remove_edge(&mut graph, edge.0, edge.1);
    }

    #[test]
    fn add_remove_edge_directed(mut graph in at_least_one_free_edge::<Directed>()) {
        let edge = find_free_edge(&graph);
        add_remove_edge(&mut graph, edge.0, edge.1);
    }

    #[test]
    fn add_remove_edge_undirected(mut graph in at_least_one_free_edge::<Undirected>()) {
        let edge = find_free_edge(&graph);
        add_remove_edge(&mut graph, edge.0, edge.1);
    }
}
