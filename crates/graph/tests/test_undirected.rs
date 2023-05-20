//! Graphs that are tested:
//!
//! * G1: a ⤸
//! * G2: a - b
//! * G3: a = b - c
//! * G4: a = b
//! * G5: a - b   c
use petgraph_core::{
    edge::{Direction, EdgeType, Undirected},
    index::IndexType,
    visit::{EdgeRef, IntoNeighborsDirected},
};
use petgraph_graph::{EdgeIndex, Graph, NodeIndex};

fn assert_graph_consistency<N, E, Ty, Ix>(graph: &Graph<N, E, Ty, Ix>)
where
    Ty: EdgeType,
    Ix: IndexType,
{
    assert_eq!(graph.node_count(), graph.node_indices().count());
    assert_eq!(graph.edge_count(), graph.edge_indices().count());

    for edge in graph.raw_edges() {
        assert!(
            graph.find_edge(edge.source(), edge.target()).is_some(),
            "Edge not in graph! {:?} to {:?}",
            edge.source(),
            edge.target()
        );
    }
}

/// Graph: a ⤸
struct GraphSelfLoop {
    graph: Graph<(), (), Undirected>,
    a: NodeIndex,
    aa: EdgeIndex,
}

impl GraphSelfLoop {
    fn new() -> Self {
        let mut graph = Graph::new_undirected();

        let a = graph.add_node(());
        let aa = graph.add_edge(a, a, ());

        Self { graph, a, aa }
    }
}

/// Graph: a - b
struct GraphLink<N = (), E = ()> {
    graph: Graph<N, E, Undirected>,
    a: NodeIndex,
    b: NodeIndex,

    ab: EdgeIndex,
}

impl<N, E> GraphLink<N, E>
where
    N: Default,
    E: Default,
{
    fn new() -> Self {
        let mut graph = Graph::new_undirected();

        let a = graph.add_node(N::default());
        let b = graph.add_node(N::default());

        let ab = graph.add_edge(a, b, E::default());

        Self { graph, a, b, ab }
    }
}

/// Graph: a = b - c
struct GraphDoubleLink {
    graph: Graph<(), (), Undirected>,
    a: NodeIndex,
    b: NodeIndex,
    c: NodeIndex,

    ab: EdgeIndex,
    ba: EdgeIndex,
    bc: EdgeIndex,
}

impl GraphDoubleLink {
    fn new() -> Self {
        let mut graph = Graph::new_undirected();

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

/// Graph: a = b
struct GraphDoubleSameDirection {
    graph: Graph<(), (), Undirected>,
    a: NodeIndex,
    b: NodeIndex,

    ab1: EdgeIndex,
    ab2: EdgeIndex,
}

impl GraphDoubleSameDirection {
    fn new() -> Self {
        let mut graph = Graph::new_undirected();

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

struct GraphLoner {
    graph: Graph<(), (), Undirected>,
    a: NodeIndex,
    b: NodeIndex,
    c: NodeIndex,

    ab: EdgeIndex,
}

impl GraphLoner {
    fn new() -> Self {
        let mut graph = Graph::new_undirected();

        let a = graph.add_node(());
        let b = graph.add_node(());
        let c = graph.add_node(());

        let ab = graph.add_edge(a, b, ());

        Self { graph, a, b, c, ab }
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
    assert_eq!(graph.neighbors(b).collect::<Vec<_>>(), vec![a, a, c]);
    assert_eq!(graph.neighbors(c).collect::<Vec<_>>(), vec![b]);
}

// Graph: a = b - c
#[test]
fn neighbours_after_removal() {
    let GraphDoubleLink {
        mut graph, a, b, c, ..
    } = GraphDoubleLink::new();

    assert_eq!(graph.neighbors(a).count(), 2);
    assert_eq!(graph.neighbors(b).count(), 3);
    assert_eq!(graph.neighbors(c).count(), 1);

    graph.remove_node(a);

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
        graph, a, b, ab1, ..
    } = GraphDoubleSameDirection::new();

    assert_eq!(graph.neighbors(a).count(), 2);
    assert_eq!(graph.neighbors(b).count(), 2);
    assert_eq!(graph.find_edge(a, b), Some(ab1));
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
    graph.edges_connecting(a, b).for_each(|edge| {
        assert!(expected.contains(&edge.id()));
    });

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
    graph.edges_connecting(a, b).for_each(|edge| {
        assert!(expected.contains(&edge.id()));
    });

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
        mut graph,
        a,
        b,
        ab,
    } = GraphLink::<u32, ()>::new();

    graph.remove_node(a);

    let result = std::panic::catch_unwind(|| {
        // ensure that our access is not optimized away
        let access = graph[a];
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

// #[test]
// fn find_directed() {
//     let mut graph = Graph::new();
//
//     let a = graph.add_node(());
//     let b = graph.add_node(());
//
//     let edge = graph.add_edge(a, b, ());
//
//     assert_eq!(graph.neighbors(a).count(), 1);
//     assert_eq!(graph.neighbors(b).count(), 0);
//
//     assert_eq!(graph.find_edge(a, b), Some(edge));
//     assert_eq!(graph.find_edge(b, a), None);
// }
