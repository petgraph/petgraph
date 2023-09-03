//! # Reasoning
//!
//! Why are these specific tests property based tests? Why are these not just unit tests?
//!
//! These specific functions manipulate the graph directly and are quite complex in nature. We use
//! property based tests to ensure that the functions are correct and do not break any invariants.
//!
//! These do not substitute unit tests, but rather complement them.
#![cfg(feature = "proptest")]

mod common;

use common::assert_graph_consistency;
use petgraph_core::{
    edge::{Directed, Direction, EdgeType, Undirected},
    index::IndexType,
    visit::EdgeCount,
};
use petgraph_graph::{EdgeIndex, Graph};
use proptest::prelude::*;

fn retain_nodes<Ty>(graph: Graph<i32, (), Ty, u8>)
where
    Ty: EdgeType,
{
    let mut graph = graph;
    let node_count = graph.node_count();
    let nodes_with_negative_weights = graph
        .raw_nodes()
        .iter()
        .filter(|node| node.weight < 0)
        .count();

    let mut removed = 0;
    graph.retain_nodes(|graph, index| {
        if graph[index] < 0 {
            removed += 1;
            false
        } else {
            true
        }
    });

    let has_negative_weights = graph.raw_nodes().iter().any(|node| node.weight < 0);
    assert!(!has_negative_weights);
    let nodes_with_positive_weights = graph
        .raw_nodes()
        .iter()
        .filter(|node| node.weight >= 0)
        .count();

    assert_eq!(graph.node_count(), node_count - nodes_with_negative_weights);
    assert_eq!(graph.node_count(), nodes_with_positive_weights);
}

#[allow(clippy::needless_pass_by_value)]
fn filter_map_nodes<Ty>(graph: Graph<i32, (), Ty, u8>)
where
    Ty: EdgeType,
{
    let node_count = graph.node_count();
    let nodes_with_negative_weights = graph
        .raw_nodes()
        .iter()
        .filter(|node| node.weight < 0)
        .count();

    let mut removed = 0;
    let graph = graph.filter_map(
        |_, weight| {
            if *weight < 0 {
                removed += 1;
                None
            } else {
                Some(*weight)
            }
        },
        |_, _| Some(()),
    );

    let has_negative_weights = graph.raw_nodes().iter().any(|node| node.weight < 0);
    assert!(!has_negative_weights);
    let nodes_with_positive_weights = graph
        .raw_nodes()
        .iter()
        .filter(|node| node.weight >= 0)
        .count();

    assert_eq!(graph.node_count(), node_count - nodes_with_negative_weights);
    assert_eq!(graph.node_count(), nodes_with_positive_weights);
}

fn retain_edges<Ty>(mut graph: Graph<(), i32, Ty, u8>)
where
    Ty: EdgeType,
{
    let edge_count = graph.edge_count();
    let edges_with_negative_weights = graph
        .raw_edges()
        .iter()
        .filter(|node| node.weight < 0)
        .count();

    let mut removed = 0;
    graph.retain_edges(|graph, index| {
        if graph[index] < 0 {
            removed += 1;
            false
        } else {
            true
        }
    });

    let has_negative_weights = graph.raw_edges().iter().any(|node| node.weight < 0);
    assert!(!has_negative_weights);
    let edges_with_positive_weights = graph
        .raw_edges()
        .iter()
        .filter(|node| node.weight >= 0)
        .count();

    assert_eq!(graph.edge_count(), edge_count - edges_with_negative_weights);
    assert_eq!(graph.edge_count(), edges_with_positive_weights);
}

#[allow(clippy::needless_pass_by_value)]
fn filter_map_edges<Ty>(graph: Graph<(), i32, Ty, u8>)
where
    Ty: EdgeType,
{
    let edge_count = graph.edge_count();
    let edges_with_negative_weights = graph
        .raw_edges()
        .iter()
        .filter(|node| node.weight < 0)
        .count();

    let mut removed = 0;
    let graph = graph.filter_map(
        |_, _| Some(()),
        |_, weight| {
            if *weight < 0 {
                removed += 1;
                None
            } else {
                Some(*weight)
            }
        },
    );

    let has_negative_weights = graph.raw_edges().iter().any(|node| node.weight < 0);
    assert!(!has_negative_weights);
    let edges_with_positive_weights = graph
        .raw_edges()
        .iter()
        .filter(|node| node.weight >= 0)
        .count();

    assert_eq!(graph.edge_count(), edge_count - edges_with_negative_weights);
    assert_eq!(graph.edge_count(), edges_with_positive_weights);
}

fn reverse<Ty>(mut graph: Graph<(), (), Ty, u8>)
where
    Ty: EdgeType,
{
    let externals_outgoing: Vec<_> = graph.externals(Direction::Outgoing).collect();
    let externals_incoming: Vec<_> = graph.externals(Direction::Incoming).collect();

    let out_degress = graph
        .node_indices()
        .map(|index| graph.neighbors_directed(index, Direction::Outgoing).count())
        .collect::<Vec<_>>();

    let in_degrees = graph
        .node_indices()
        .map(|index| graph.neighbors_directed(index, Direction::Incoming).count())
        .collect::<Vec<_>>();

    graph.reverse();

    let reversed_externals_outgoing: Vec<_> = graph.externals(Direction::Outgoing).collect();
    let reversed_externals_incoming: Vec<_> = graph.externals(Direction::Incoming).collect();

    let reversed_out_degress = graph
        .node_indices()
        .map(|index| graph.neighbors_directed(index, Direction::Outgoing).count())
        .collect::<Vec<_>>();

    let reversed_in_degrees = graph
        .node_indices()
        .map(|index| graph.neighbors_directed(index, Direction::Incoming).count())
        .collect::<Vec<_>>();

    assert_eq!(externals_outgoing, reversed_externals_incoming);
    assert_eq!(externals_incoming, reversed_externals_outgoing);
    assert_eq!(out_degress, reversed_in_degrees);
    assert_eq!(in_degrees, reversed_out_degress);

    // additional test if we're isomorphic by simply eq out and in
    if !Ty::is_directed() {
        assert_eq!(externals_outgoing, externals_incoming);
        assert_eq!(reversed_externals_outgoing, reversed_externals_incoming);

        assert_eq!(out_degress, in_degrees);
        assert_eq!(reversed_out_degress, reversed_in_degrees);
    }
}

fn remove_edge<Ty>(graph: &mut Graph<(), (), Ty, u8>, edge: EdgeIndex<u8>)
where
    Ty: EdgeType,
{
    assert_eq!(graph.edge_weight(edge).copied(), Some(()));
    // we don't generate any parallel edges, therefore if we remove an edge, `find_node` should
    // return `None`
    let (a, b) = graph.edge_endpoints(edge).expect("edge should exist");
    // we cannot check if `.is_some()` because there might be parallel edges
    assert!(graph.find_edge(a, b).is_some());

    graph.remove_edge(edge);

    assert_graph_consistency(graph);

    // we cannot test for the weight, as another edge has likely taken its place
    assert!(!graph.neighbors(a).any(|node| node == b));
    // we cannot assert that the edge does not exist, as there might be parallel edges
    // (via `find_edge()`)
}

#[cfg(not(miri))]
proptest! {
    /// Integration test, which tests the `retain_nodes` method.
    ///
    /// This is done by generating a random graph, and then removing all nodes with a negative weight.
    ///
    /// The index is `u8`, as that way we do not explore the entire range of `usize`, which would take a long time.
    ///
    /// With this the maximum is 255 nodes, and the maximum number of edges is `(current maximum nodes)^2`
    #[test]
    fn retain_nodes_directed(graph in any::<Graph::<i32, (), Directed, u8>>()) {
        retain_nodes(graph);
    }

    #[test]
    fn retain_nodes_undirected(graph in any::<Graph::<i32, (), Undirected, u8>>()) {
        retain_nodes(graph);
    }

    /// Virtually the same thing as `retain_nodes`, but with `filter_map`.
    #[test]
    fn filter_map_nodes_directed(graph in any::<Graph::<i32, (), Directed, u8>>()) {
        filter_map_nodes(graph);
    }

    #[test]
    fn filter_map_nodes_undirected(graph in any::<Graph::<i32, (), Undirected, u8>>()) {
        filter_map_nodes(graph);
    }

    /// Integration test, which tests the `retain_edges` method.
    ///
    /// This is done by generating a random graph, and then removing all edges with a negative weight.
    ///
    /// The index is `u8`, as that way we do not explore the entire range of `usize`, which would take a long time.
    ///
    /// With this the maximum is 255 nodes, and the maximum number of edges is `(current maximum nodes)^2`
    #[test]
    fn retain_edges_directed(graph in any::<Graph::<(), i32, Directed, u8>>()) {
        retain_edges(graph);
    }

    #[test]
    fn retain_edges_undirected(graph in any::<Graph::<(), i32, Undirected, u8>>()) {
        retain_edges(graph);
    }

    /// Virtually the same thing as `retain_edges`, but with `filter_map`.
    #[test]
    fn filter_map_edges_directed(graph in any::<Graph::<(), i32, Directed, u8>>()) {
        filter_map_edges(graph);
    }

    #[test]
    fn filter_map_edges_undirected(graph in any::<Graph::<(), i32, Undirected, u8>>()) {
        filter_map_edges(graph);
    }

    /// Integration test, which tests the `reverse` method.
    ///
    /// If we reverse a graph, the externals should be reversed as well,
    /// not only that, but the amount of neighbours should be the same, but only switched.
    #[test]
    fn reverse_directed(graph in any::<Graph::<(), (), Directed, u8>>()) {
        reverse(graph);
    }

    #[test]
    fn reverse_undirected(graph in any::<Graph::<(), (), Undirected, u8>>()) {
        reverse(graph);
    }
}

fn graph_and_edge<Ty>() -> impl Strategy<Value = (Graph<(), (), Ty, u8>, EdgeIndex<u8>)>
where
    Ty: EdgeType + Send + Sync + 'static,
{
    any::<Graph<(), (), Ty, u8>>()
        .prop_filter(
            "an edge needs to exist in the graph, to be able to index into it",
            |graph| graph.edge_count() > 0,
        )
        .prop_flat_map(move |graph| {
            let edge_count = graph.edge_count();

            (Just(graph), (0..edge_count))
        })
        .prop_map(|(graph, edge)| (graph, EdgeIndex::new(edge)))
}

#[cfg(not(miri))]
proptest! {
    #[test]
    fn remove_edge_directed((mut graph, edge) in graph_and_edge::<Directed>()) {
        remove_edge(&mut graph, edge);
    }

    // there are some additional checks for undirected graphs, as we need to check both directions
    #[test]
    fn remove_edge_undirected((mut graph, edge) in graph_and_edge::<Undirected>()) {
        let (a, b) = graph.edge_endpoints(edge).expect("edge should exist");
        prop_assert_eq!(graph.find_edge(b, a), Some(edge));

        remove_edge(&mut graph, edge);

        prop_assert_eq!(graph.find_edge(b, a), None);
        prop_assert!(!graph.neighbors(b).any(|node| node == a));
    }
}
