mod common;

use common::graphs::FromDefault;
use petgraph_core::{
    edge::{Direction, Undirected},
    index::DefaultIx,
    visit::{EdgeIndexable, EdgeRef, IntoEdgeReferences, IntoNodeReferences, NodeIndexable},
};
use petgraph_graph::{stable::StableGraph, EdgeIndex, Graph, NodeIndex};

use crate::common::graphs::GraphDoubleLink;

type GraphLink<N, E> = common::graphs::GraphLink<StableGraph<N, E, Undirected>>;

impl GraphLink<(), ()> {
    fn new() -> Self {
        Self::from_default()
    }
}

type GraphDoubleSameDirection<N, E> =
    common::graphs::GraphDoubleSameDirection<StableGraph<N, E, Undirected>>;

impl GraphDoubleSameDirection<(), ()> {
    fn new() -> Self {
        Self::from_default()
    }
}

#[test]
fn node_indices() {
    let mut graph = StableGraph::<(), (), Undirected>::with_capacity(0, 0);

    let a = graph.add_node(());
    let b = graph.add_node(());
    let c = graph.add_node(());

    graph.remove_node(b);

    assert_eq!(graph.node_indices().collect::<Vec<_>>(), vec![a, c]);
}

#[test]
fn node_indices_stable() {
    let mut graph = StableGraph::<i32, (), Undirected>::with_capacity(0, 0);

    let a = graph.add_node(0);
    let b = graph.add_node(1);
    let c = graph.add_node(2);

    graph.remove_node(b);

    // even though we removed the `b` node, the index for `c` should remain the same
    // This is what makes `StableGraph` special, `Graph` would've moved the last index (`c`) to the
    // place of the removed node.
    assert_eq!(graph.node_weight(a), Some(&0));
    assert_eq!(graph.node_weight(b), None);
    assert_eq!(graph.node_weight(c), Some(&2));
}

#[test]
fn node_indices_reclaim() {
    let mut graph = StableGraph::<(), (), Undirected>::with_capacity(0, 0);

    let a = graph.add_node(());
    let b = graph.add_node(());
    let c = graph.add_node(());

    graph.remove_node(b);
    let b2 = graph.add_node(());

    assert_eq!(b, b2);

    // `StableGraph` holds a list of unallocated nodes, if we create a new node and removed one, it
    // should occupy the same index.
    assert_eq!(graph.node_indices().collect::<Vec<_>>(), vec![a, b, c]);
}

#[test]
fn node_bound() {
    let mut graph = StableGraph::<(), (), Undirected>::with_capacity(0, 0);

    assert_eq!(graph.node_bound(), 0);

    graph.add_node(());
    let b = graph.add_node(());

    assert_eq!(graph.node_bound(), 2);

    graph.remove_node(b);

    // `StableGraph` retains a list of unallocated nodes, so the bound should ignore those
    // unallocated nodes.
    assert_eq!(graph.node_bound(), 1);
}

#[test]
fn edge_indices() {
    let mut graph = StableGraph::<(), (), Undirected>::with_capacity(0, 0);

    let a = graph.add_node(());
    let b = graph.add_node(());

    let ab = graph.add_edge(a, b, ());
    let ba = graph.add_edge(b, a, ());

    graph.remove_edge(ba);

    assert_eq!(graph.edge_indices().collect::<Vec<_>>(), vec![ab]);
}

#[test]
fn edge_indices_stable() {
    let mut graph = StableGraph::<(), i32, Undirected>::with_capacity(0, 0);

    let a = graph.add_node(());
    let b = graph.add_node(());

    let ab = graph.add_edge(a, b, 0);
    let ba = graph.add_edge(b, a, 1);

    graph.remove_edge(ab);

    // same reason as `node_indices_stable` test, we remove the edge but the index for `ba` should
    // remain the same
    assert_eq!(graph.edge_weight(ba), Some(&1));
}

#[test]
fn edge_indices_reclaim() {
    let mut graph = StableGraph::<(), (), Undirected>::with_capacity(0, 0);

    let a = graph.add_node(());
    let b = graph.add_node(());

    let ab = graph.add_edge(a, b, ());
    graph.add_edge(b, a, ());

    graph.remove_edge(ab);

    let ab2 = graph.add_edge(a, b, ());

    assert_eq!(ab, ab2);
}

#[test]
fn edge_bound() {
    let mut graph = StableGraph::<(), (), Undirected>::with_capacity(0, 0);

    assert_eq!(graph.edge_bound(), 0);

    let a = graph.add_node(());
    let b = graph.add_node(());

    graph.add_edge(a, b, ());
    let ba = graph.add_edge(b, a, ());

    assert_eq!(graph.edge_bound(), 2);

    graph.remove_edge(ba);

    // same reason as `node_bound` test, we remove the edge but the bound should ignore the
    // unallocated edge
    assert_eq!(graph.edge_bound(), 1);
}

#[test]
fn clear_edges() {
    let mut graph = StableGraph::<(), (), Undirected>::with_capacity(0, 0);

    let a = graph.add_node(());
    let b = graph.add_node(());

    graph.add_edge(a, b, ());
    graph.add_edge(b, a, ());

    graph.clear_edges();

    assert_eq!(graph.edge_count(), 0);
    assert_eq!(graph.edge_indices().count(), 0);

    assert_eq!(graph.edge_references().count(), 0);
    assert!(
        graph
            .node_indices()
            .all(|index| graph.neighbors(index).count() == 0)
    );
}

#[test]
fn edges() {
    let GraphLink { graph, a, b, ab } = GraphLink::new();

    assert_eq!(graph.edges(a).count(), 1);
    assert_eq!(graph.edges(b).count(), 1);

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
        vec![ab]
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
    assert_eq!(graph.edges(b).count(), 2);

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
        vec![ab2, ab1]
    );
}

#[test]
fn edges_directed() {
    let GraphLink { graph, a, b, ab } = GraphLink::new();

    assert_eq!(graph.edges_directed(a, Direction::Outgoing).count(), 1);
    assert_eq!(graph.edges_directed(a, Direction::Incoming).count(), 1);

    assert_eq!(graph.edges_directed(b, Direction::Outgoing).count(), 1);
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
        vec![ab]
    );

    assert_eq!(
        graph
            .edges_directed(b, Direction::Outgoing)
            .map(|reference| reference.id())
            .collect::<Vec<_>>(),
        vec![ab]
    );
    assert_eq!(
        graph
            .edges_directed(b, Direction::Incoming)
            .map(|reference| reference.id())
            .collect::<Vec<_>>(),
        vec![ab]
    );
}

#[cfg(feature = "std")]
#[test]
fn access_removed_node() {
    let GraphLink {
        mut graph, a, b, ..
    } = GraphLink::new();

    // we don't need to do the move shenanigans here because StableGraph does not move indices when
    // a node is removed.
    graph.remove_node(a);

    let result = std::panic::catch_unwind(|| {
        let access = graph[a];
        core::hint::black_box(&access);
    });

    result.expect_err("Accessing removed node should panic");
}

#[cfg(feature = "std")]
#[test]
fn add_node_out_of_bounds() {
    let mut graph = StableGraph::<(), (), Undirected, u8>::with_capacity(0, 0);

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
    let mut graph = StableGraph::<(), (), Undirected, u8>::with_capacity(0, 0);

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

#[test]
fn from() {
    let mut stable =
        GraphDoubleLink::<StableGraph<i32, i32, Undirected, DefaultIx>, DefaultIx>::from_default();

    let mut normal =
        GraphDoubleLink::<Graph<i32, i32, Undirected, DefaultIx>, DefaultIx>::from_default();

    // permute both the same way, nodes are tagged with their index (starting from 1)
    // and edges are tagged with their index, but negative (starting from -1)
    for (index, weight) in stable.graph.node_weights_mut().enumerate() {
        *weight = i32::try_from(index + 1).expect("too many nodes");
    }

    for (index, weight) in normal.graph.node_weights_mut().enumerate() {
        *weight = i32::try_from(index + 1).expect("too many nodes");
    }

    for (index, weight) in stable.graph.edge_weights_mut().enumerate() {
        *weight = -i32::try_from(index + 1).expect("too many edges");
    }

    for (index, weight) in normal.graph.edge_weights_mut().enumerate() {
        *weight = -i32::try_from(index + 1).expect("too many edges");
    }

    // now remove the middle node (b) from both
    stable.graph.remove_node(stable.b);
    normal.graph.remove_node(normal.b);

    // convert the stable graph to a normal graph
    let converted: Graph<_, _, _, _> = Graph::from(stable.graph);

    // assert that all edges and nodes are the same
    assert!(
        converted
            .node_references()
            .eq(normal.graph.node_references())
    );
    assert!(
        converted
            .edge_references()
            .eq(normal.graph.edge_references())
    );
}

#[test]
fn into() {
    let stable =
        GraphDoubleLink::<StableGraph<i32, i32, Undirected, DefaultIx>, DefaultIx>::from_default();

    let normal =
        GraphDoubleLink::<Graph<i32, i32, Undirected, DefaultIx>, DefaultIx>::from_default();

    // here we don't permute, because the conversion from graph to stable cannot handle holes that
    // we created, as it shuffles indices in that case

    let converted: StableGraph<_, _, _, _> = normal.graph.into();

    // assert that all edges and nodes are the same
    assert!(
        converted
            .node_references()
            .eq(stable.graph.node_references())
    );

    assert!(
        converted
            .edge_references()
            .eq(stable.graph.edge_references())
    );
}
