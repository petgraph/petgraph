//! Because most methods tested here are not generic, we re-implement the same tests from the normal
//! graph.
#![cfg(feature = "proptest")]

mod common;

use common::proptest::{assert_edges_eq, assert_edges_without_weight_eq, assert_nodes_eq};
use petgraph_core::{
    edge::{Directed, EdgeType, Undirected},
    visit::{EdgeRef, IntoEdgeReferences, IntoNodeReferences, NodeRef},
};
use petgraph_graph::{stable::StableGraph, EdgeIndex, NodeIndex};
use proptest::prelude::*;

fn retain_nodes<Ty>(graph: StableGraph<i32, (), Ty, u8>)
where
    Ty: EdgeType,
{
    let mut graph = graph;
    let node_count = graph.node_count();
    let nodes_with_negative_weights = graph.node_weights().filter(|&&weight| weight < 0).count();

    let mut removed = 0;
    graph.retain_nodes(|graph, index| {
        if graph[index] < 0 {
            removed += 1;
            false
        } else {
            true
        }
    });

    let has_negative_weights = graph.node_weights().any(|&weight| weight < 0);
    assert!(!has_negative_weights);

    let nodes_with_positive_weights = graph.node_weights().filter(|&&weight| weight >= 0).count();

    assert_eq!(graph.node_count(), node_count - nodes_with_negative_weights);
    assert_eq!(graph.node_count(), nodes_with_positive_weights);
}

#[allow(clippy::needless_pass_by_value)]
fn filter_map_nodes<Ty>(graph: StableGraph<i32, (), Ty, u8>)
where
    Ty: EdgeType,
{
    let node_count = graph.node_count();
    let nodes_with_negative_weights = graph.node_weights().filter(|&&weight| weight < 0).count();

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

    let has_negative_weights = graph.node_weights().any(|&weight| weight < 0);
    assert!(!has_negative_weights);
    let nodes_with_positive_weights = graph.node_weights().filter(|&&weight| weight >= 0).count();

    assert_eq!(graph.node_count(), node_count - nodes_with_negative_weights);
    assert_eq!(graph.node_count(), nodes_with_positive_weights);
}

fn retain_edges<Ty>(mut graph: StableGraph<(), i32, Ty, u8>)
where
    Ty: EdgeType,
{
    let edge_count = graph.edge_count();
    let edges_with_negative_weights = graph.edge_weights().filter(|&&weight| weight < 0).count();

    let mut removed = 0;
    graph.retain_edges(|graph, index| {
        if graph[index] < 0 {
            removed += 1;
            false
        } else {
            true
        }
    });

    let has_negative_weights = graph.edge_weights().any(|&weight| weight < 0);
    assert!(!has_negative_weights);

    let edges_with_positive_weights = graph.edge_weights().filter(|&&weight| weight >= 0).count();

    assert_eq!(graph.edge_count(), edge_count - edges_with_negative_weights);
    assert_eq!(graph.edge_count(), edges_with_positive_weights);
}

#[allow(clippy::needless_pass_by_value)]
fn filter_map_edges<Ty>(graph: StableGraph<(), i32, Ty, u8>)
where
    Ty: EdgeType,
{
    let edge_count = graph.edge_count();
    let edges_with_negative_weights = graph.edge_weights().filter(|&&weight| weight < 0).count();

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

    let has_negative_weights = graph.edge_weights().any(|&weight| weight < 0);
    assert!(!has_negative_weights);
    let edges_with_positive_weights = graph.edge_weights().filter(|&&weight| weight >= 0).count();

    assert_eq!(graph.edge_count(), edge_count - edges_with_negative_weights);
    assert_eq!(graph.edge_count(), edges_with_positive_weights);
}

fn remove_edge<Ty>(graph: &mut StableGraph<(), (), Ty, u8>, edge: EdgeIndex<u8>)
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

    assert_eq!(graph.edge_weight(edge), None);
    assert!(!graph.neighbors(a).any(|node| node == b));
}

fn remove_edge_add_edge<Ty>(
    graph: &mut StableGraph<(), (), Ty, u8>,
    remove: EdgeIndex<u8>,
    create: (NodeIndex<u8>, NodeIndex<u8>),
) where
    Ty: EdgeType,
{
    assert_eq!(graph.edge_weight(remove).copied(), Some(()));

    let (a, b) = graph.edge_endpoints(remove).expect("edge should exist");
    assert!(graph.find_edge(a, b).is_some());

    graph.remove_edge(remove);

    assert_eq!(graph.edge_weight(remove), None);

    let add = graph.add_edge(create.0, create.1, ());

    assert_eq!(graph.edge_weight(add).copied(), Some(()));
    assert!(graph.find_edge(create.0, create.1).is_some());
    // We reuse the edge index, as we keep track of a free list of removed edges.
    assert_eq!(add, remove);
}

proptest! {
    /// Integration test, which tests the `retain_nodes` method.
    ///
    /// This is done by generating a random graph, and then removing all nodes with a negative weight.
    ///
    /// The index is `u8`, as that way we do not explore the entire range of `usize`, which would take a long time.
    ///
    /// With this the maximum is 255 nodes, and the maximum number of edges is `(current maximum nodes)^2`
    #[test]
    fn retain_nodes_directed(graph in any::<StableGraph::<i32, (), Directed, u8>>()) {
        retain_nodes(graph);
    }

    #[test]
    fn retain_nodes_undirected(graph in any::<StableGraph::<i32, (), Undirected, u8>>()) {
        retain_nodes(graph);
    }

    /// Virtually the same thing as `retain_nodes`, but with `filter_map`.
    #[test]
    fn filter_map_nodes_directed(graph in any::<StableGraph::<i32, (), Directed, u8>>()) {
        filter_map_nodes(graph);
    }

    #[test]
    fn filter_map_nodes_undirected(graph in any::<StableGraph::<i32, (), Undirected, u8>>()) {
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
    fn retain_edges_directed(graph in any::<StableGraph::<(), i32, Directed, u8>>()) {
        retain_edges(graph);
    }

    #[test]
    fn retain_edges_undirected(graph in any::<StableGraph::<(), i32, Undirected, u8>>()) {
        retain_edges(graph);
    }

    /// Virtually the same thing as `retain_edges`, but with `filter_map`.
    #[test]
    fn filter_map_edges_directed(graph in any::<StableGraph::<(), i32, Directed, u8>>()) {
        filter_map_edges(graph);
    }

    #[test]
    fn filter_map_edges_undirected(graph in any::<StableGraph::<(), i32, Undirected, u8>>()) {
        filter_map_edges(graph);
    }

}

// because we handle a `StableGraph` that has holes we need to make sure that we pick edges that
// have been removed.
fn graph_and_edge<Ty>() -> impl Strategy<Value = (StableGraph<(), (), Ty, u8>, EdgeIndex<u8>)>
where
    Ty: EdgeType + Send + Sync + 'static,
{
    any::<StableGraph<(), (), Ty, u8>>()
        .prop_filter(
            "an edge needs to exist in the graph, to be able to index into it",
            |graph| graph.edge_count() > 0,
        )
        .prop_flat_map(move |graph| {
            let edge_count = graph.edge_count();

            (Just(graph), (0..edge_count))
        })
        .prop_map(|(graph, edge)| {
            let edge = graph.edge_references().nth(edge).unwrap().id();
            (graph, edge)
        })
}
fn graph_and_edge_and_nodes<Ty>() -> impl Strategy<
    Value = (
        StableGraph<(), (), Ty, u8>,
        EdgeIndex<u8>,
        (NodeIndex<u8>, NodeIndex<u8>),
    ),
>
where
    Ty: EdgeType + Send + Sync + 'static,
{
    graph_and_edge()
        .prop_flat_map(|(graph, edge)| {
            let node_count = graph.node_count();

            let create = (0..node_count, 0..node_count);

            (Just(graph), Just(edge), create)
        })
        .prop_map(|(graph, edge, (start, target))| {
            let start = graph.node_references().nth(start).unwrap().id();
            let target = graph.node_references().nth(target).unwrap().id();

            (graph, edge, (start, target))
        })
}

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

    #[test]
    fn remove_edge_add_edge_directed((mut graph, edge, create) in graph_and_edge_and_nodes::<Directed>()) {
        remove_edge_add_edge(&mut graph, edge, create);
    }

    #[test]
    fn remove_edge_add_edge_undirected((mut graph, edge, create) in graph_and_edge_and_nodes::<Undirected>()) {
        remove_edge_add_edge(&mut graph, edge, create);
    }
}

proptest! {
    #[test]
    fn map_identity_directed(graph in any::<StableGraph<u8, u8, Directed, u8>>()) {
        let graph2 = graph.map(
            |_, &weight| weight,
            |_, &weight| weight,
        );

        assert_nodes_eq(&graph, &graph2)?;
        assert_edges_eq(&graph, &graph2)?;
        assert_edges_without_weight_eq(&graph, &graph2)?;
    }


    #[test]
    fn map_identity_undirected(graph in any::<StableGraph<u8, u8, Undirected, u8>>()) {
        let graph2 = graph.map(
            |_, &weight| weight,
            |_, &weight| weight,
        );

        assert_nodes_eq(&graph, &graph2)?;
        assert_edges_eq(&graph, &graph2)?;
        assert_edges_without_weight_eq(&graph, &graph2)?;
    }

    #[test]
    fn filter_map_identity_directed(graph in any::<StableGraph<u8, u8, Directed, u8>>()) {
        let graph2 = graph.filter_map(
            |_, &weight| Some(weight),
            |_, &weight| Some(weight),
        );

        assert_nodes_eq(&graph, &graph2)?;
        assert_edges_eq(&graph, &graph2)?;
        assert_edges_without_weight_eq(&graph, &graph2)?;
    }

    #[test]
    fn filter_map_identity_undirected(graph in any::<StableGraph<u8, u8, Undirected, u8>>()) {
        let graph2 = graph.filter_map(
            |_, &weight| Some(weight),
            |_, &weight| Some(weight),
        );

        assert_nodes_eq(&graph, &graph2)?;
        assert_edges_eq(&graph, &graph2)?;
        assert_edges_without_weight_eq(&graph, &graph2)?;
    }
}
