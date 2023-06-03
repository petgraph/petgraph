#![cfg(feature = "stable")]

use common::graphs::FromDefault;
use petgraph_core::{
    edge::{Directed, Direction},
    visit::{EdgeRef, IntoEdgeReferences, IntoNodeReferences},
};
use petgraph_graph::stable::StableGraph;

mod common;

type GraphLink<N, E> = common::graphs::GraphLink<StableGraph<N, E, Directed>>;

impl GraphLink<(), ()> {
    fn new() -> Self {
        Self::from_default()
    }
}

type GraphDoubleSameDirection<N, E> =
    common::graphs::GraphDoubleSameDirection<StableGraph<N, E, Directed>>;

impl GraphDoubleSameDirection<(), ()> {
    fn new() -> Self {
        Self::from_default()
    }
}

#[test]
fn edges() {
    let GraphLink { graph, a, b, ab } = GraphLink::new();

    assert_eq!(graph.edges(a).count(), 1);
    assert_eq!(graph.edges(b).count(), 0);

    assert_eq!(
        graph
            .edges(a)
            .map(|reference| reference.id())
            .collect::<Vec<_>>(),
        vec![ab]
    );
    assert_eq!(
        graph
            .edges(b)
            .map(|reference| reference.id())
            .collect::<Vec<_>>(),
        vec![]
    );
}

#[test]
fn edges_multi() {
    let GraphDoubleSameDirection {
        graph,
        a,
        b,
        ab1,
        ab2,
    } = GraphDoubleSameDirection::new();

    assert_eq!(graph.edges(a).count(), 2);
    assert_eq!(graph.edges(b).count(), 0);

    assert_eq!(
        graph
            .edges(a)
            .map(|reference| reference.id())
            .collect::<Vec<_>>(),
        vec![ab2, ab1]
    );
    assert_eq!(
        graph
            .edges(b)
            .map(|reference| reference.id())
            .collect::<Vec<_>>(),
        vec![]
    );
}

#[test]
fn edges_multi_connecting() {
    let GraphDoubleSameDirection {
        graph,
        a,
        b,
        ab1,
        ab2,
    } = GraphDoubleSameDirection::new();

    assert_eq!(graph.edges_connecting(a, b).count(), 2);

    assert_eq!(
        graph
            .edges_connecting(a, b)
            .map(|reference| reference.id())
            .collect::<Vec<_>>(),
        vec![ab2, ab1]
    );

    assert_eq!(graph.edges_connecting(b, a).count(), 0);

    assert_eq!(
        graph
            .edges_connecting(b, a)
            .map(|reference| reference.id())
            .collect::<Vec<_>>(),
        vec![]
    );
}

#[test]
fn edges_directed() {
    let GraphLink { graph, a, b, ab } = GraphLink::new();

    assert_eq!(graph.edges_directed(a, Direction::Outgoing).count(), 1);
    assert_eq!(graph.edges_directed(a, Direction::Incoming).count(), 0);

    assert_eq!(graph.edges_directed(b, Direction::Outgoing).count(), 0);
    assert_eq!(graph.edges_directed(b, Direction::Incoming).count(), 1);

    assert_eq!(
        graph
            .edges_directed(a, Direction::Outgoing)
            .map(|reference| reference.id())
            .collect::<Vec<_>>(),
        vec![ab]
    );
    assert_eq!(
        graph
            .edges_directed(a, Direction::Incoming)
            .map(|reference| reference.id())
            .collect::<Vec<_>>(),
        vec![]
    );

    assert_eq!(
        graph
            .edges_directed(b, Direction::Outgoing)
            .map(|reference| reference.id())
            .collect::<Vec<_>>(),
        vec![]
    );
    assert_eq!(
        graph
            .edges_directed(b, Direction::Incoming)
            .map(|reference| reference.id())
            .collect::<Vec<_>>(),
        vec![ab]
    );
}

#[test]
fn neighbours() {
    let GraphLink { graph, a, b, .. } = GraphLink::new();

    assert_eq!(graph.neighbors(a).count(), 1);
    assert_eq!(graph.neighbors(b).count(), 0);

    assert_eq!(graph.neighbors(a).collect::<Vec<_>>(), vec![b]);
    assert_eq!(graph.neighbors(b).collect::<Vec<_>>(), vec![]);
}

#[test]
fn neighbours_directed() {
    let GraphLink { graph, a, b, .. } = GraphLink::new();

    assert_eq!(graph.neighbors_directed(a, Direction::Outgoing).count(), 1);
    assert_eq!(graph.neighbors_directed(a, Direction::Incoming).count(), 0);

    assert_eq!(graph.neighbors_directed(b, Direction::Outgoing).count(), 0);
    assert_eq!(graph.neighbors_directed(b, Direction::Incoming).count(), 1);

    assert_eq!(
        graph
            .neighbors_directed(a, Direction::Outgoing)
            .collect::<Vec<_>>(),
        vec![b]
    );
    assert_eq!(
        graph
            .neighbors_directed(a, Direction::Incoming)
            .collect::<Vec<_>>(),
        vec![]
    );

    assert_eq!(
        graph
            .neighbors_directed(b, Direction::Outgoing)
            .collect::<Vec<_>>(),
        vec![]
    );
    assert_eq!(
        graph
            .neighbors_directed(b, Direction::Incoming)
            .collect::<Vec<_>>(),
        vec![a]
    );
}

#[test]
fn node_references() {
    let GraphLink { graph, a, b, .. } = GraphLink::new();

    assert_eq!(graph.node_references().count(), 2);

    assert_eq!(
        graph
            .node_references()
            .map(|(index, _)| index)
            .collect::<Vec<_>>(),
        vec![a, b]
    );
}

#[test]
fn edge_references() {
    let GraphLink { graph, a, b, ab } = GraphLink::new();

    assert_eq!(graph.edge_references().count(), 1);

    assert_eq!(
        graph
            .edge_references()
            .map(|edge| (edge.source(), edge.target(), edge.id()))
            .collect::<Vec<_>>(),
        vec![(a, b, ab)]
    );
}

#[test]
fn nodes_mut() {
    let GraphLink {
        mut graph, a, b, ..
    } = GraphLink::<i32, ()>::from_default();

    assert_eq!(graph.node_weights_mut().count(), 2);

    for (index, weight) in graph.node_weights_mut().enumerate() {
        *weight = i32::try_from(index).expect("should be able to convert") + 1;
    }

    assert_eq!(graph.node_weight(a), Some(&1));
    assert_eq!(graph.node_weight(b), Some(&2));
}

#[test]
fn edges_mut() {
    let GraphLink { mut graph, ab, .. } = GraphLink::<(), i32>::from_default();

    assert_eq!(graph.edge_weights_mut().count(), 1);

    for (index, weight) in graph.edge_weights_mut().enumerate() {
        *weight = i32::try_from(index).expect("should be able to convert") + 1;
    }

    assert_eq!(graph.edge_weight(ab), Some(&1));
}
