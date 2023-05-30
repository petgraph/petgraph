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
    index::{DefaultIx, IndexType},
    visit::EdgeRef,
};
use petgraph_graph::{EdgeIndex, Graph, NodeIndex};

use crate::common::{graphs::FromDefault, walk_collect};

type GraphSelfLoop<N, E> = common::graphs::GraphSelfLoop<Graph<N, E>>;

// IntelliJ: false-positive
impl GraphSelfLoop<(), ()> {
    fn new() -> Self {
        Self::from_default()
    }
}

type GraphLink<N, E> = common::graphs::GraphLink<Graph<N, E>>;

// IntelliJ: false-positive
impl GraphLink<(), ()> {
    fn new() -> Self {
        Self::from_default()
    }
}

type GraphDoubleLink<N, E> = common::graphs::GraphDoubleLink<Graph<N, E>>;

// IntelliJ: false-positive
impl GraphDoubleLink<(), ()> {
    fn new() -> Self {
        Self::from_default()
    }
}

type GraphDoubleSameDirection<N, E> = common::graphs::GraphDoubleSameDirection<Graph<N, E>>;

// IntelliJ: false-positive
impl GraphDoubleSameDirection<(), ()> {
    fn new() -> Self {
        Self::from_default()
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
fn neighbours_detach() {
    let GraphDoubleLink { graph, a, b, c, .. } = GraphDoubleLink::new();

    let walk = graph.neighbors(a).detach();
    assert_eq!(walk_collect(walk, &graph), vec![b]);

    let walk = graph.neighbors(b).detach();
    assert_eq!(walk_collect(walk, &graph), vec![c, a]);

    let walk = graph.neighbors(c).detach();
    assert_eq!(walk_collect(walk, &graph), vec![]);
}

#[test]
fn neighbours_directed() {
    let GraphDoubleLink { graph, a, b, c, .. } = GraphDoubleLink::new();

    assert_eq!(
        graph
            .neighbors_directed(a, Direction::Outgoing)
            .collect::<Vec<_>>(),
        vec![b]
    );
    assert_eq!(
        graph
            .neighbors_directed(b, Direction::Outgoing)
            .collect::<Vec<_>>(),
        vec![c, a]
    );
    assert_eq!(
        graph
            .neighbors_directed(c, Direction::Outgoing)
            .collect::<Vec<_>>(),
        vec![]
    );

    assert_eq!(
        graph
            .neighbors_directed(a, Direction::Incoming)
            .collect::<Vec<_>>(),
        vec![b]
    );
    assert_eq!(
        graph
            .neighbors_directed(b, Direction::Incoming)
            .collect::<Vec<_>>(),
        vec![a]
    );
    assert_eq!(
        graph
            .neighbors_directed(c, Direction::Incoming)
            .collect::<Vec<_>>(),
        vec![b]
    );
}

#[test]
fn neighbours_directed_detach() {
    let GraphDoubleLink { graph, a, b, c, .. } = GraphDoubleLink::new();

    let walk = graph.neighbors_directed(a, Direction::Outgoing).detach();
    assert_eq!(walk_collect(walk, &graph), vec![b]);

    let walk = graph.neighbors_directed(b, Direction::Outgoing).detach();
    assert_eq!(walk_collect(walk, &graph), vec![c, a]);

    let walk = graph.neighbors_directed(c, Direction::Outgoing).detach();
    assert_eq!(walk_collect(walk, &graph), vec![]);

    let walk = graph.neighbors_directed(a, Direction::Incoming).detach();
    assert_eq!(walk_collect(walk, &graph), vec![b]);

    let walk = graph.neighbors_directed(b, Direction::Incoming).detach();
    assert_eq!(walk_collect(walk, &graph), vec![a]);

    let walk = graph.neighbors_directed(c, Direction::Incoming).detach();
    assert_eq!(walk_collect(walk, &graph), vec![b]);
}

#[test]
fn neighbours_order() {
    let GraphLink { graph, a, b, .. } = GraphLink::new();

    // neighbours are LIFO
    assert_eq!(graph.neighbors(a).collect::<Vec<_>>(), vec![b]);
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
}

#[test]
fn neighbours_after_removal() {
    let GraphDoubleLink {
        mut graph, a, b, c, ..
    } = GraphDoubleLink::new();

    graph.remove_node(c);

    assert_eq!(graph.neighbors(a).collect::<Vec<_>>(), vec![b]);
    assert_eq!(graph.neighbors(b).collect::<Vec<_>>(), vec![a]);

    assert_graph_consistency(&graph);
}

#[test]
fn neighbours_self_loop() {
    let GraphSelfLoop { graph, a, .. } = GraphSelfLoop::new();

    assert_eq!(graph.neighbors(a).collect::<Vec<_>>(), vec![a]);
    assert_eq!(
        graph
            .neighbors_directed(a, Direction::Outgoing)
            .collect::<Vec<_>>(),
        vec![a]
    );
    assert_eq!(
        graph
            .neighbors_directed(a, Direction::Incoming)
            .collect::<Vec<_>>(),
        vec![a]
    );
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

#[test]
fn externals() {
    let GraphLink { graph, a, b, .. } = GraphLink::<(), u32>::from_default();

    assert_eq!(graph.externals(Direction::Incoming).count(), 1);
    assert_eq!(graph.externals(Direction::Outgoing).count(), 1);

    assert_eq!(graph.externals(Direction::Incoming).next(), Some(a));
    assert_eq!(graph.externals(Direction::Outgoing).next(), Some(b));

    assert_graph_consistency(&graph);
}

#[test]
fn externals_empty() {
    let GraphSelfLoop { graph, .. } = GraphSelfLoop::new();

    assert_eq!(graph.externals(Direction::Incoming).count(), 0);
    assert_eq!(graph.externals(Direction::Outgoing).count(), 0);

    assert_graph_consistency(&graph);
}

// Test different node indices, this will always pass when testing, but fail during compilation.
#[test]
const fn node_indices() {
    const fn graph_index<T>()
    where
        T: IndexType,
    {
    }

    graph_index::<u8>();

    #[cfg(any(
        target_pointer_width = "16",
        target_pointer_width = "32",
        target_pointer_width = "64"
    ))]
    graph_index::<u16>();

    #[cfg(any(
        target_pointer_width = "32", //
        target_pointer_width = "64"
    ))]
    graph_index::<u32>();

    #[cfg(any(
        target_pointer_width = "64" //
    ))]
    graph_index::<u64>();

    graph_index::<usize>();
}

#[test]
fn node_weight_iterator() {
    let mut graph = Graph::<_, ()>::new();

    graph.add_node(0);
    graph.add_node(1);
    graph.add_node(2);

    assert_eq!(graph.node_weights_mut().count(), 3);
    assert_eq!(
        graph.node_weights_mut().count(),
        graph.node_weights().count()
    );
    assert_eq!(graph.node_weights_mut().count(), graph.node_count());

    assert_eq!(graph.node_weights().collect::<Vec<_>>(), vec![&0, &1, &2]);
    assert_eq!(graph.node_weights_mut().collect::<Vec<_>>(), vec![
        &mut 0, &mut 1, &mut 2
    ]);
}

#[test]
fn edge_weight_iterator() {
    let mut graph = Graph::<(), _>::from_edges([
        (0, 1, 0), //
        (1, 2, 1),
        (2, 0, 2),
    ]);

    assert_eq!(graph.edge_weights_mut().count(), 3);
    assert_eq!(
        graph.edge_weights_mut().count(),
        graph.edge_weights().count()
    );
    assert_eq!(graph.edge_weights_mut().count(), graph.edge_count());

    assert_eq!(graph.edge_weights().collect::<Vec<_>>(), vec![&0, &1, &2]);
    assert_eq!(graph.edge_weights_mut().collect::<Vec<_>>(), vec![
        &mut 0, &mut 1, &mut 2
    ]);
}

#[test]
fn index_twice_mut() {
    let GraphLink {
        mut graph, a, b, ..
    } = GraphLink::<i32, ()>::from_default();

    let (a_weight, b_weight) = graph.index_twice_mut(a, b);
    assert_eq!(*a_weight, 0);
    assert_eq!(*b_weight, 0);

    *a_weight = 1;
    *b_weight = 2;

    assert_eq!(graph[a], 1);
    assert_eq!(graph[b], 2);
}

#[cfg(feature = "std")]
#[test]
fn index_twice_mut_same_index() {
    let GraphLink { mut graph, a, .. } = GraphLink::<i32, ()>::from_default();

    let result = std::panic::catch_unwind(move || {
        graph.index_twice_mut(a, a);
    });

    result.expect_err("index_twice_mut should panic when given the same index twice");
}

#[cfg(feature = "std")]
#[test]
fn index_twice_mut_out_of_range() {
    let GraphLink {
        mut graph, a, b, ..
    } = GraphLink::<i32, ()>::from_default();

    let c = NodeIndex::new(b.index() + 1);
    let result = std::panic::catch_unwind(move || {
        graph.index_twice_mut(a, c);
    });

    result.expect_err("index_twice_mut should panic when given an out of range index");
}

#[cfg(feature = "std")]
#[test]
fn index_twice_mut_out_of_range_same_index() {
    let GraphLink { mut graph, a, .. } = GraphLink::<i32, ()>::from_default();

    let c = NodeIndex::new(a.index() + 1);
    let result = std::panic::catch_unwind(move || {
        graph.index_twice_mut(c, c);
    });

    result.expect_err("index_twice_mut should panic when given an out of range index");
}

#[test]
fn index_twice_mut_node_and_edge() {
    let GraphLink {
        mut graph, a, ab, ..
    } = GraphLink::<i32, i32>::from_default();

    let (node_weight, edge_weight) = graph.index_twice_mut(a, ab);

    assert_eq!(*node_weight, 0);
    assert_eq!(*edge_weight, 0);

    *node_weight = 1;
    *edge_weight = 2;

    assert_eq!(graph[a], 1);
    assert_eq!(graph[ab], 2);
}

#[test]
fn from_edges() {
    let graph = Graph::<(), ()>::from_edges([
        (0, 1), //
        (1, 2),
        (2, 0),
    ]);

    let a = NodeIndex::new(0);
    let b = NodeIndex::new(1);
    let c = NodeIndex::new(2);

    assert_eq!(graph.node_count(), 3);
    assert_eq!(graph.edge_count(), 3);

    assert_eq!(graph.neighbors(a).collect::<Vec<_>>(), vec![b]);
    assert_eq!(graph.neighbors(b).collect::<Vec<_>>(), vec![c]);
    assert_eq!(graph.neighbors(c).collect::<Vec<_>>(), vec![a]);

    assert_graph_consistency(&graph);
}

#[test]
fn from_edges_weighted() {
    let graph = Graph::<(), i32>::from_edges([
        (0, 1, 0), //
        (1, 2, 1),
        (2, 0, 2),
    ]);

    let a = NodeIndex::new(0);
    let b = NodeIndex::new(1);
    let c = NodeIndex::new(2);

    let ab = EdgeIndex::new(0);
    let bc = EdgeIndex::new(1);
    let ca = EdgeIndex::new(2);

    assert_eq!(graph.node_count(), 3);
    assert_eq!(graph.edge_count(), 3);

    assert_eq!(
        graph
            .edge_references()
            .map(|reference| (
                reference.id(),
                reference.source(),
                reference.target(),
                reference.weight()
            ))
            .collect::<Vec<_>>(),
        vec![(ab, a, b, &0), (bc, b, c, &1), (ca, c, a, &2),]
    );
}

#[test]
fn retain_nodes() {
    let mut graph = Graph::<(), ()>::from_edges([
        (0, 1), //
        (1, 2),
        (2, 0),
    ]);

    // remove the last node, this way we do not switch indices
    graph.retain_nodes(|_, node| node != NodeIndex::new(2));

    let a = NodeIndex::new(0);
    let b = NodeIndex::new(1);

    assert_eq!(graph.node_count(), 2);
    assert_eq!(graph.edge_count(), 1);

    assert_eq!(graph.neighbors(a).collect::<Vec<_>>(), vec![b]);
    assert_eq!(graph.neighbors(b).collect::<Vec<_>>(), vec![]);

    assert_graph_consistency(&graph);
}

#[test]
fn retain_edges() {
    let mut graph = Graph::<(), ()>::from_edges([
        (0, 1), //
        (1, 2),
        (2, 0),
    ]);

    // remove the last edge, this way we do not switch indices
    graph.retain_edges(|_, edge| edge != EdgeIndex::new(2));

    let a = NodeIndex::new(0);
    let b = NodeIndex::new(1);
    let c = NodeIndex::new(2);

    assert_eq!(graph.node_count(), 3);
    assert_eq!(graph.edge_count(), 2);

    assert_eq!(graph.neighbors(a).collect::<Vec<_>>(), vec![b]);
    assert_eq!(graph.neighbors(b).collect::<Vec<_>>(), vec![c]);
    assert_eq!(graph.neighbors(c).collect::<Vec<_>>(), vec![]);

    assert_graph_consistency(&graph);
}

#[test]
fn map() {
    let GraphLink { graph, .. } = GraphLink::<i32, i32>::from_default();

    assert_eq!(graph.node_weights().collect::<Vec<_>>(), vec![&0, &0]);
    assert_eq!(graph.edge_weights().collect::<Vec<_>>(), vec![&0]);

    let graph = graph.map(|_, _| 1, |_, _| 2);

    assert_eq!(graph.node_weights().collect::<Vec<_>>(), vec![&1, &1]);
    assert_eq!(graph.edge_weights().collect::<Vec<_>>(), vec![&2]);
}

#[test]
fn filter_map() {
    let GraphDoubleLink { mut graph, b, .. } = GraphDoubleLink::<i32, i32>::from_default();

    let d = graph.add_node(0);
    graph.add_edge(b, d, 0);
    graph.add_edge(d, b, 0);

    assert_eq!(graph.node_weights().collect::<Vec<_>>(), vec![
        &0, &0, &0, &0
    ]);
    assert_eq!(graph.edge_weights().collect::<Vec<_>>(), vec![
        &0, &0, &0, &0, &0
    ]);

    // Important to note is that `filter_map` is not perfect, it removes all nodes first and then
    // only edges to nodes that passed the filter are passed to the `edge_map` function.
    // The node filter removes `a` and `c`, only leaving `b` and `d` behind (which is why we added d
    // in the first place)
    // This means that the only edge that pass through the edge_map is `bd` and `db`, from those
    // we only select `bd`.
    // During the filtering the edge index is not updated, so the edge index of `bd` is still 3 even
    // though `ab`, `ba` and `cd` were removed in the node filter.
    let graph = graph.filter_map(
        |index, _| ((index.index() % 2) == 1).then_some(index.index()),
        // to ensure that edges are not run through the node filter we increment the edge weight
        |index, _| ((index.index() % 2) == 1).then_some(index.index() + 1),
    );

    assert_eq!(graph.node_weights().collect::<Vec<_>>(), vec![&1, &3]);
    assert_eq!(graph.edge_weights().collect::<Vec<_>>(), vec![&4]);

    assert_graph_consistency(&graph);
}
