#![cfg(feature = "proptest")]
extern crate core;

use core::fmt;

use petgraph_core::{
    edge::{Directed, EdgeType, Undirected},
    visit::IntoNodeIdentifiers,
};
use petgraph_graphmap::{EntryStorage, NodeTrait};
use proptest::prelude::*;

fn assert_graphmap_consistent<N, E, Ty>(g: &EntryStorage<N, E, Ty>)
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

fn remove_node<Ty>(graph: &mut EntryStorage<i8, (), Ty>, node: i8)
where
    Ty: EdgeType,
{
    assert_graphmap_consistent(graph);
    assert!(graph.contains_node(node));

    graph.remove_node(node);

    assert_graphmap_consistent(graph);
    assert!(!graph.contains_node(node));
}

fn remove_edge<Ty>(graph: &mut EntryStorage<i8, (), Ty>, a: i8, b: i8)
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

fn add_remove_edge<Ty>(graph: &mut EntryStorage<i8, (), Ty>, a: i8, b: i8)
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

fn find_free_edge<Ty>(graph: &EntryStorage<i8, (), Ty>) -> (i8, i8)
where
    Ty: EdgeType,
{
    assert_graphmap_consistent(graph);

    for a in graph.node_identifiers() {
        for b in graph.node_identifiers() {
            if !graph.contains_edge(a, b) {
                return (a, b);
            }
        }
    }

    panic!("no free edge found");
}

fn graph_and_node<Ty>() -> impl Strategy<Value = (EntryStorage<i8, (), Ty>, i8)>
where
    Ty: EdgeType + Clone + 'static,
{
    any::<EntryStorage<i8, (), Ty>>()
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

fn graph_and_edge<Ty>() -> impl Strategy<Value = (EntryStorage<i8, (), Ty>, (i8, i8))>
where
    Ty: EdgeType + Clone + 'static,
{
    any::<EntryStorage<i8, (), Ty>>()
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

fn at_least_one_free_edge<Ty>() -> impl Strategy<Value = EntryStorage<i8, (), Ty>>
where
    Ty: EdgeType + 'static,
{
    any::<EntryStorage<i8, (), Ty>>()
        .prop_filter("graph must have at least one node", |graph| {
            graph.node_count() > 0
        })
        .prop_filter("graph must have at least one free edge", |graph| {
            // generate the maximum amount of edges possible
            let edges = graph.node_count() * graph.node_count();

            // if we have the same amount of edges as possible combinations, we have no free edges
            graph.edge_count() < edges
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
