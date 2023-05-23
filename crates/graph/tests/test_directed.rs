//! Graphs that are used for testing:
//!
//! * G1: a ⤸
//! * G2: a → b
//! * G3: a ⇆ b → c
//! * G4: a ⇉ b

mod common;

use common::assert_graph_consistency;
use petgraph_core::{
    edge::{Directed, Direction},
    visit::EdgeRef,
};
use petgraph_graph::{EdgeIndex, Graph, NodeIndex};

/// Graph: a ⤸
struct GraphSelfLoop {
    graph: Graph<(), (), Directed>,
    a: NodeIndex,
    aa: EdgeIndex,
}

impl GraphSelfLoop {
    fn new() -> Self {
        let mut graph = Graph::new();

        let a = graph.add_node(());
        let aa = graph.add_edge(a, a, ());

        Self { graph, a, aa }
    }
}

/// Graph: a → b
struct GraphLink<N = (), E = ()> {
    graph: Graph<N, E, Directed>,

    a: NodeIndex,
    b: NodeIndex,
    ab: EdgeIndex,
}

impl GraphLink {
    fn new() -> Self {
        Self::from_default()
    }
}

impl<N, E> GraphLink<N, E>
where
    N: Default,
    E: Default,
{
    fn from_default() -> Self {
        let mut graph = Graph::new();

        let a = graph.add_node(N::default());
        let b = graph.add_node(N::default());
        let ab = graph.add_edge(a, b, E::default());

        Self { graph, a, b, ab }
    }
}

struct GraphDoubleLink {
    graph: Graph<(), (), Directed>,

    a: NodeIndex,
    b: NodeIndex,
    c: NodeIndex,
    ab: EdgeIndex,
    ba: EdgeIndex,
    bc: EdgeIndex,
}

impl GraphDoubleLink {
    fn new() -> Self {
        let mut graph = Graph::new();

        let a = graph.add_node(());
        let b = graph.add_node(());
        let c = graph.add_node(());

        let ab = graph.add_edge(a, b, ());
        let ba = graph.add_edge(b, a, ());
        let bc = graph.add_edge(b, c, ());

        Self {
            graph,
            a,
            b,
            c,
            ab,
            ba,
            bc,
        }
    }
}

struct GraphDoubleSameDirection {
    graph: Graph<(), (), Directed>,

    a: NodeIndex,
    b: NodeIndex,

    ab1: EdgeIndex,
    ab2: EdgeIndex,
}

impl GraphDoubleSameDirection {
    fn new() -> Self {
        let mut graph = Graph::new();

        let a = graph.add_node(());
        let b = graph.add_node(());

        let ab1 = graph.add_edge(a, b, ());
        let ab2 = graph.add_edge(a, b, ());

        Self {
            graph,
            a,
            b,
            ab1,
            ab2,
        }
    }
}

// Graph: a ⤸
#[test]
fn self_loop() {
    let GraphSelfLoop { mut graph, a, aa } = GraphSelfLoop::new();

    assert_eq!(graph.node_count(), 1);
    assert_eq!(graph.edge_count(), 1);

    assert_eq!(graph.find_edge(a, a), Some(aa));

    graph.remove_edge(aa);
    assert_eq!(graph.find_edge(a, a), None);
}

#[test]
fn find_directed() {
    let GraphLink { graph, a, b, ab } = GraphLink::new();

    assert_eq!(graph.find_edge(a, b), Some(ab));
    assert_eq!(graph.find_edge(b, a), None);

    assert_graph_consistency(&graph);
}

#[test]
fn find_undirected() {
    let GraphLink { graph, a, b, ab } = GraphLink::new();

    assert_eq!(
        graph.find_edge_undirected(a, b),
        Some((ab, Direction::Outgoing))
    );
    assert_eq!(
        graph.find_edge_undirected(b, a),
        Some((ab, Direction::Incoming))
    );

    assert_graph_consistency(&graph);
}

#[test]
fn find_undirected_self_loop() {
    let GraphSelfLoop { graph, a, aa } = GraphSelfLoop::new();

    assert_eq!(
        graph.find_edge_undirected(a, a),
        Some((aa, Direction::Outgoing))
    );

    assert_graph_consistency(&graph);
}

#[test]
fn find_directed_after_removal() {
    let GraphLink {
        mut graph,
        a,
        b,
        ab,
    } = GraphLink::new();

    graph.remove_edge(ab);

    assert_eq!(graph.find_edge(a, b), None);
    assert_eq!(graph.find_edge(b, a), None);

    assert_graph_consistency(&graph);
}

#[test]
fn find_undirected_after_removal() {
    let GraphLink {
        mut graph,
        a,
        b,
        ab,
    } = GraphLink::new();

    graph.remove_edge(ab);

    assert_eq!(graph.find_edge_undirected(a, b), None);
    assert_eq!(graph.find_edge_undirected(b, a), None);

    assert_graph_consistency(&graph);
}

#[test]
fn neighbours() {
    let GraphDoubleLink { graph, a, b, c, .. } = GraphDoubleLink::new();

    assert_eq!(graph.neighbors(a).collect::<Vec<_>>(), vec![b]);
    assert_eq!(graph.neighbors(b).collect::<Vec<_>>(), vec![c, a]);
    assert_eq!(graph.neighbors(c).collect::<Vec<_>>(), vec![]);

    assert_graph_consistency(&graph);
}

#[test]
fn neighbours_after_removal() {
    let GraphDoubleLink {
        mut graph,
        a,
        b,
        c,
        ab,
        ba,
        bc,
    } = GraphDoubleLink::new();

    graph.remove_node(c);

    assert_eq!(graph.neighbors(a).collect::<Vec<_>>(), vec![b]);
    assert_eq!(graph.neighbors(b).collect::<Vec<_>>(), vec![a]);

    assert_graph_consistency(&graph);
}

#[test]
fn multiple() {
    let GraphDoubleSameDirection {
        graph, a, b, ab2, ..
    } = GraphDoubleSameDirection::new();

    assert_eq!(graph.edge_count(), 2);

    assert_eq!(graph.edges(a).count(), 2);
    assert_eq!(graph.edges(b).count(), 0);

    assert_eq!(graph.neighbors(a).count(), 2);
    assert_eq!(graph.neighbors(b).count(), 0);

    assert_eq!(graph.find_edge(a, b), Some(ab2));
}

#[test]
fn iter_multiple() {
    let GraphDoubleSameDirection {
        graph,
        a,
        b,
        ab1,
        ab2,
    } = GraphDoubleSameDirection::new();

    let expected = vec![ab1, ab2];

    for edge in graph.edges_connecting(a, b) {
        assert!(expected.contains(&edge.id()));
    }
    assert_eq!(graph.edges_connecting(b, a).count(), 0);

    assert_graph_consistency(&graph);
}

#[test]
fn update_edge() {
    let GraphLink {
        mut graph,
        a,
        b,
        ab,
    } = GraphLink::<(), u32>::from_default();

    assert_eq!(graph.edge_weight(ab), Some(&0));
    assert_eq!(graph.find_edge(b, a), None);

    graph.update_edge(a, b, 1);
    assert_eq!(graph.edge_weight(ab), Some(&1));

    assert_graph_consistency(&graph);

    let ba = graph.update_edge(b, a, 2);
    assert_eq!(graph.find_edge(b, a), Some(ba));
    assert_eq!(graph.edge_weight(ba), Some(&2));
}
