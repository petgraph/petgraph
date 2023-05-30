//! Graphs that are tested:
//!
//! * G1: a ⤸
//! * G2: a - b
//! * G3: a = b - c
//! * G4: a = b
//! * G5: a - b   c

mod common;

use common::assert_graph_consistency;
use petgraph_core::{
    edge::{Direction, EdgeType, Undirected},
    index::IndexType,
    visit::{EdgeRef, IntoNeighborsDirected},
};
use petgraph_graph::{EdgeIndex, Graph, NodeIndex};

use crate::common::{graphs::FromDefault, walk_collect};

type GraphSelfLoop<N, E> = common::graphs::GraphSelfLoop<Graph<N, E, Undirected>>;

impl GraphSelfLoop<(), ()> {
    fn new() -> Self {
        Self::from_default()
    }
}

type GraphLink<N, E> = common::graphs::GraphLink<Graph<N, E, Undirected>>;

impl<N, E> GraphLink<N, E>
where
    N: Default,
    E: Default,
{
    fn new() -> Self {
        Self::from_default()
    }
}

type GraphDoubleLink<N, E> = common::graphs::GraphDoubleLink<Graph<N, E, Undirected>>;

impl GraphDoubleLink<(), ()> {
    fn new() -> Self {
        Self::from_default()
    }
}

type GraphDoubleSameDirection<N, E> =
    common::graphs::GraphDoubleSameDirection<Graph<N, E, Undirected>>;

impl GraphDoubleSameDirection<(), ()> {
    fn new() -> Self {
        Self::from_default()
    }
}

type GraphLoner<N, E> = common::graphs::GraphLoner<Graph<N, E, Undirected>>;

impl GraphLoner<(), ()> {
    fn new() -> Self {
        Self::from_default()
    }
}

// Graph: a ⤸
#[test]
fn self_loop() {
    let GraphSelfLoop { graph, a, aa } = GraphSelfLoop::new();

    assert_eq!(graph.node_count(), 1);
    assert_eq!(graph.edge_count(), 1);

    assert_eq!(graph.find_edge(a, a), Some(aa));
}

// Graph: a - b
#[test]
fn find_undirected() {
    let GraphLink { graph, a, b, ab } = GraphLink::<(), ()>::new();

    assert_eq!(graph.neighbors(a).count(), 1);
    assert_eq!(graph.neighbors(b).count(), 1);

    assert_eq!(graph.find_edge(a, b), Some(ab));
    assert_eq!(graph.find_edge(b, a), Some(ab));

    assert_graph_consistency(&graph);
}

// Graph: a - b
#[test]
fn find_undirected_after_removal() {
    let GraphLink {
        mut graph,
        a,
        b,
        ab,
    } = GraphLink::<(), ()>::new();

    assert_eq!(graph.find_edge(a, b), Some(ab));
    assert_eq!(graph.find_edge(b, a), Some(ab));

    graph.remove_node(a);

    assert_eq!(graph.find_edge(a, b), None);
    assert_eq!(graph.find_edge(b, a), None);

    assert_graph_consistency(&graph);
}

// Graph: a = b - c
#[test]
fn neighbours() {
    let GraphDoubleLink { graph, a, b, c, .. } = GraphDoubleLink::new();

    assert_eq!(graph.neighbors(a).count(), 2);
    assert_eq!(graph.neighbors(b).count(), 3);
    assert_eq!(graph.neighbors(c).count(), 1);

    assert_eq!(graph.neighbors(a).collect::<Vec<_>>(), vec![b, b]);
    assert_eq!(graph.neighbors(b).collect::<Vec<_>>(), vec![c, a, a]);
    assert_eq!(graph.neighbors(c).collect::<Vec<_>>(), vec![b]);
}

#[test]
fn neighbours_detach() {
    let GraphDoubleLink { graph, a, b, c, .. } = GraphDoubleLink::<u32, ()>::from_default();

    let walk = graph.neighbors(a).detach();
    assert_eq!(walk_collect(walk, &graph), vec![b, b]);

    let walk = graph.neighbors(b).detach();
    assert_eq!(walk_collect(walk, &graph), vec![c, a, a]);

    let walk = graph.neighbors(c).detach();
    assert_eq!(walk_collect(walk, &graph), vec![b]);
}

// Graph: a = b - c
#[test]
fn neighbours_after_removal() {
    let GraphDoubleLink {
        mut graph, a, b, c, ..
    } = GraphDoubleLink::<u32, ()>::from_default();

    // mark note c, we need this later when we remove, to ensure the swap was correct
    graph[c] = 2;

    assert_eq!(graph.neighbors(a).count(), 2);
    assert_eq!(graph.neighbors(b).count(), 3);
    assert_eq!(graph.neighbors(c).count(), 1);

    graph.remove_node(a);

    // The last index is switched with the first index, therefore we need to rebind
    let c = a;
    // ensure that we still point to `c`
    assert_eq!(graph[c], 2);

    assert_eq!(graph.neighbors(b).collect::<Vec<_>>(), vec![c]);
    assert_eq!(graph.neighbors(c).collect::<Vec<_>>(), vec![b]);
}

// Graph: a = b - c
#[test]
fn neighbours_directed_same_count() {
    let GraphDoubleLink { graph, a, b, c, .. } = GraphDoubleLink::new();

    assert_eq!(graph.neighbors_directed(a, Direction::Incoming).count(), 2);
    assert_eq!(graph.neighbors_directed(a, Direction::Outgoing).count(), 2);

    assert_eq!(graph.neighbors_directed(b, Direction::Incoming).count(), 3);
    assert_eq!(graph.neighbors_directed(b, Direction::Outgoing).count(), 3);

    assert_eq!(graph.neighbors_directed(c, Direction::Incoming).count(), 1);
    assert_eq!(graph.neighbors_directed(c, Direction::Outgoing).count(), 1);

    assert_graph_consistency(&graph);
}

#[test]
fn neighbours_directed_equivalent() {
    let GraphDoubleLink { graph, a, b, c, .. } = GraphDoubleLink::new();

    assert_eq!(
        graph
            .neighbors_directed(a, Direction::Incoming)
            .collect::<Vec<_>>(),
        graph
            .neighbors_directed(a, Direction::Outgoing)
            .collect::<Vec<_>>()
    );

    assert_eq!(
        graph
            .neighbors_directed(b, Direction::Incoming)
            .collect::<Vec<_>>(),
        graph
            .neighbors_directed(b, Direction::Outgoing)
            .collect::<Vec<_>>()
    );

    assert_eq!(
        graph
            .neighbors_directed(c, Direction::Incoming)
            .collect::<Vec<_>>(),
        graph
            .neighbors_directed(c, Direction::Outgoing)
            .collect::<Vec<_>>()
    );

    assert_graph_consistency(&graph);
}

#[test]
fn multiple() {
    let GraphDoubleSameDirection {
        graph, a, b, ab2, ..
    } = GraphDoubleSameDirection::new();

    assert_eq!(graph.neighbors(a).count(), 2);
    assert_eq!(graph.neighbors(b).count(), 2);
    assert_eq!(graph.find_edge(a, b), Some(ab2));
    assert_eq!(graph.edge_count(), 2);

    assert_graph_consistency(&graph);
}

#[test]
fn iter_multiple() {
    let GraphDoubleLink {
        graph,
        a,
        b,
        ab,
        ba,
        ..
    } = GraphDoubleLink::new();

    let expected = vec![ab, ba];

    for edge in graph.edges_connecting(a, b) {
        assert!(expected.contains(&edge.id()));
    }

    assert_graph_consistency(&graph);
}

#[test]
fn iter_multiple_same_direction() {
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
    for edge in graph.edges_connecting(b, a) {
        assert!(expected.contains(&edge.id()));
    }

    assert_graph_consistency(&graph);
}

#[test]
fn update_edge() {
    let GraphLink {
        mut graph,
        a,
        b,
        ab,
    } = GraphLink::<(), u32>::new();

    assert_eq!(graph.edge_weight(ab), Some(&0));
    let ab2 = graph.update_edge(a, b, 1);
    assert_eq!(graph.edge_weight(ab), Some(&1));

    assert_eq!(ab, ab2);
    assert_eq!(graph.edge_count(), 1);

    let ab2 = graph.update_edge(a, b, 2);
    assert_eq!(graph.edge_weight(ab), Some(&2));

    assert_eq!(ab, ab2);
    assert_eq!(graph.edge_count(), 1);

    assert_graph_consistency(&graph);
}

#[test]
fn externals() {
    let GraphLoner { graph, c, .. } = GraphLoner::new();

    assert_eq!(graph.externals(Direction::Incoming).count(), 1);
    assert_eq!(graph.externals(Direction::Outgoing).count(), 1);

    assert_eq!(graph.externals(Direction::Incoming).next(), Some(c));
    assert_eq!(graph.externals(Direction::Outgoing).next(), Some(c));

    assert_graph_consistency(&graph);
}

#[test]
fn externals_empty() {
    let graph = Graph::<(), ()>::new();

    assert_eq!(graph.externals(Direction::Incoming).count(), 0);
    assert_eq!(graph.externals(Direction::Outgoing).count(), 0);

    assert_graph_consistency(&graph);
}

#[cfg(feature = "std")]
#[test]
fn access_removed_node() {
    let GraphLink {
        mut graph, a, b, ..
    } = GraphLink::<u32, ()>::new();

    // mark b with a weight of 1
    // we will check this later
    graph[b] = 1;

    graph.remove_node(a);

    // This is a bit unintuitive, but to preserve indices, the last index is swapped with the
    // first index. Therefore we actually have to check for b here.

    // To verify that `b` is actually at the `a` index now, we check the weight, we previously set.
    // Remember: `a` would have a weight of 0, because we never set it.
    assert_eq!(graph[a], 1);

    let result = std::panic::catch_unwind(|| {
        // now we access `b`, which should panic because we swapped it with `a`, when we removed it.
        let access = graph[b];
        // ensure that our access is not optimized away
        core::hint::black_box(&access);
    });

    result.expect_err("Accessing removed node should panic");
}

#[cfg(feature = "std")]
#[test]
fn add_node_out_of_bounds() {
    let mut graph = Graph::<(), (), Undirected, u8>::with_capacity(0, 0);

    // fill up the graph
    for _ in 0..255 {
        graph.add_node(());
    }

    let result = std::panic::catch_unwind(move || {
        graph.add_node(());
    });

    result.expect_err("Creating more than Ix::MAX nodes should panic");
}

#[cfg(feature = "std")]
#[test]
fn add_edge_out_of_bounds() {
    let mut graph = Graph::<(), (), Undirected, u8>::with_capacity(0, 0);

    let a = graph.add_node(());
    let b = graph.add_node(());

    // fill up the graph
    for _ in 0..255 {
        graph.add_edge(a, b, ());
    }

    let result = std::panic::catch_unwind(move || {
        graph.add_edge(a, b, ());
    });

    result.expect_err("Creating more than Ix::MAX edges should panic");
}
