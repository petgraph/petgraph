use petgraph_algorithms::flows::max_flow::{
    MaxFlowReturn, dinics_mod::dinics, edmonds_karp_mod::edmonds_karp,
};
use petgraph_core::{
    graph::{DirectedGraph, EditableGraph, storable::StorableGraph},
    utils::test_graphs::directed::DirectedTestGraph,
};

use crate::run_macro_for_all_dir_graphs;

macro_rules! test_max_flow_on_graph {
    ($graph_constructor:expr) => {
        mod edmonds_karp {
            use super::*;
            test_max_flow_all_examples!($graph_constructor, |graph, source, destination| {
                edmonds_karp(graph, source, destination, |edge_id| {
                    graph.edge(edge_id).unwrap().data.clone()
                })
                .into_max_flow_and_flow_data()
            });
        }
        mod dinics {
            use super::*;
            test_max_flow_all_examples!($graph_constructor, |graph, source, destination| dinics(
                graph,
                source,
                destination,
                |edge_id| graph.edge(edge_id).unwrap().data.clone()
            )
            .into_max_flow_and_flow_data());
        }
    };
}

macro_rules! test_max_flow_all_examples {
    ($graph_constructor:expr, $max_flow_algorithm:expr) => {
        #[test]
        fn test_one() {
            crate::flows::max_flow::test_max_flow_one($graph_constructor, $max_flow_algorithm);
        }

        #[test]
        fn test_two() {
            crate::flows::max_flow::test_max_flow_two($graph_constructor, $max_flow_algorithm);
        }

        #[test]
        fn test_three() {
            crate::flows::max_flow::test_max_flow_three($graph_constructor, $max_flow_algorithm);
        }

        #[test]
        fn test_four() {
            crate::flows::max_flow::test_max_flow_four($graph_constructor, $max_flow_algorithm);
        }

        #[test]
        fn test_five() {
            crate::flows::max_flow::test_max_flow_five($graph_constructor, $max_flow_algorithm);
        }

        #[test]
        fn test_six() {
            crate::flows::max_flow::test_max_flow_six($graph_constructor, $max_flow_algorithm);
        }

        #[test]
        fn test_seven() {
            crate::flows::max_flow::test_max_flow_seven($graph_constructor, $max_flow_algorithm);
        }

        // TODO Add missing test functions once proper IndexId etc handling in Algorithms is done.
    };
}

run_macro_for_all_dir_graphs!(test_max_flow_on_graph);

fn test_max_flow_one<
    G: DirectedGraph + StorableGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = u32>,
>(
    graph_constructor: impl Fn() -> G,
    flow_algorithm: impl Fn(&G, G::NodeId, G::NodeId) -> (u32, G::EdgeDataContainer<u32>),
) {
    // Example from https://downey.io/blog/max-flow-ford-fulkerson-algorithm-explanation/
    let mut graph = graph_constructor();
    let nodes = [
        graph.add_node(()).unwrap(),
        graph.add_node(()).unwrap(),
        graph.add_node(()).unwrap(),
        graph.add_node(()).unwrap(),
    ];
    for (source, sink, weight) in [(0, 1, 3), (0, 2, 2), (1, 2, 5), (1, 3, 2), (2, 3, 3)] {
        graph.add_edge(nodes[source], nodes[sink], weight).unwrap();
    }
    let (max_flow, _) = flow_algorithm(&graph, nodes[0], nodes[3]);
    assert_eq!(5, max_flow);
}

fn test_max_flow_two<
    G: DirectedGraph + StorableGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = u32>,
>(
    graph_constructor: impl Fn() -> G,
    flow_algorithm: impl Fn(&G, G::NodeId, G::NodeId) -> (u32, G::EdgeDataContainer<u32>),
) {
    // Example from https://brilliant.org/wiki/ford-fulkerson-algorithm/
    let mut graph = graph_constructor();
    let mut nodes = Vec::new();
    for _ in 0..6 {
        nodes.push(graph.add_node(()).unwrap());
    }
    for (source, sink, weight) in [
        (0, 1, 4),
        (0, 2, 3),
        (1, 3, 4),
        (2, 4, 6),
        (3, 2, 3),
        (3, 5, 2),
        (4, 5, 6),
    ] {
        graph.add_edge(nodes[source], nodes[sink], weight).unwrap();
    }
    let (max_flow, _) = flow_algorithm(&graph, nodes[0], nodes[5]);
    assert_eq!(7, max_flow);
}

fn test_max_flow_three<
    G: DirectedGraph + StorableGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = u32>,
>(
    graph_constructor: impl Fn() -> G,
    flow_algorithm: impl Fn(&G, G::NodeId, G::NodeId) -> (u32, G::EdgeDataContainer<u32>),
) {
    // Example from https://cp-algorithms.com/graph/edmonds_karp.html
    let mut graph = graph_constructor();
    let mut nodes = Vec::new();
    for _ in 0..6 {
        nodes.push(graph.add_node(()).unwrap());
    }
    for (source, sink, weight) in [
        (0, 1, 7),
        (0, 2, 4),
        (1, 3, 5),
        (1, 4, 3),
        (2, 1, 3),
        (2, 4, 2),
        (3, 5, 8),
        (4, 3, 3),
        (4, 5, 5),
    ] {
        graph.add_edge(nodes[source], nodes[sink], weight).unwrap();
    }
    let (max_flow, _) = flow_algorithm(&graph, nodes[0], nodes[5]);
    assert_eq!(10, max_flow);
}

fn test_max_flow_four<
    G: DirectedGraph + StorableGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = u32>,
>(
    graph_constructor: impl Fn() -> G,
    flow_algorithm: impl Fn(&G, G::NodeId, G::NodeId) -> (u32, G::EdgeDataContainer<u32>),
) {
    // Example from https://www.programiz.com/dsa/ford-fulkerson-algorithm (corrected: result not 6 but 5)
    let mut graph = graph_constructor();
    let mut nodes = Vec::new();
    for _ in 0..6 {
        nodes.push(graph.add_node(()).unwrap());
    }
    for (source, sink, weight) in [
        (0, 1, 8),
        (0, 2, 3),
        (1, 3, 9),
        (2, 3, 7),
        (2, 4, 4),
        (3, 5, 2),
        (4, 5, 5),
    ] {
        graph.add_edge(nodes[source], nodes[sink], weight).unwrap();
    }
    let (max_flow, _) = flow_algorithm(&graph, nodes[0], nodes[5]);
    assert_eq!(5, max_flow);
}

fn test_max_flow_five<
    G: DirectedGraph + StorableGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = u32>,
>(
    graph_constructor: impl Fn() -> G,
    flow_algorithm: impl Fn(&G, G::NodeId, G::NodeId) -> (u32, G::EdgeDataContainer<u32>),
) {
    let mut graph = graph_constructor();
    let mut nodes = Vec::new();
    for _ in 0..6 {
        nodes.push(graph.add_node(()).unwrap());
    }
    for (source, sink, weight) in [
        (0, 1, 16),
        (0, 2, 13),
        (1, 2, 10),
        (1, 3, 12),
        (2, 1, 4),
        (2, 4, 14),
        (3, 2, 9),
        (3, 5, 20),
        (4, 3, 7),
        (4, 5, 4),
    ] {
        graph.add_edge(nodes[source], nodes[sink], weight).unwrap();
    }
    let (max_flow, _) = flow_algorithm(&graph, nodes[0], nodes[5]);
    assert_eq!(23, max_flow);
}

fn test_max_flow_six<
    G: DirectedGraph + StorableGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = u32>,
>(
    graph_constructor: impl Fn() -> G,
    flow_algorithm: impl Fn(&G, G::NodeId, G::NodeId) -> (u32, G::EdgeDataContainer<u32>),
) {
    // Example taken from https://medium.com/@jithmisha/solving-the-maximum-flow-problem-with-ford-fulkerson-method-3fccc2883dc7
    let mut graph = graph_constructor();
    let mut nodes = Vec::new();
    for _ in 0..6 {
        nodes.push(graph.add_node(()).unwrap());
    }
    for (source, sink, weight) in [
        (0, 1, 10),
        (0, 2, 10),
        (1, 2, 2),
        (1, 3, 4),
        (1, 4, 8),
        (2, 4, 9),
        (3, 5, 10),
        (4, 3, 6),
        (4, 5, 10),
    ] {
        graph.add_edge(nodes[source], nodes[sink], weight).unwrap();
    }
    let (max_flow, _) = flow_algorithm(&graph, nodes[0], nodes[5]);
    assert_eq!(19, max_flow);
}

fn test_max_flow_seven<
    G: DirectedGraph + StorableGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = u32>,
>(
    graph_constructor: impl Fn() -> G,
    flow_algorithm: impl Fn(&G, G::NodeId, G::NodeId) -> (u32, G::EdgeDataContainer<u32>),
) {
    // Example that can lead to invalid answers if backward edges
    // in residual network are not considered, resulting in a flow of 3
    // instead of the maximum 4
    let mut graph = graph_constructor();

    let node_s = graph.add_node(()).unwrap();
    let node_a = graph.add_node(()).unwrap();
    let node_b = graph.add_node(()).unwrap();
    let node_c = graph.add_node(()).unwrap();
    let node_d = graph.add_node(()).unwrap();
    let node_t = graph.add_node(()).unwrap();

    graph.add_edge(node_s, node_a, 2).unwrap();
    graph.add_edge(node_s, node_b, 2).unwrap();
    graph.add_edge(node_a, node_c, 1).unwrap(); // misleading edge
    graph.add_edge(node_a, node_d, 2).unwrap();
    graph.add_edge(node_b, node_c, 2).unwrap();
    graph.add_edge(node_c, node_t, 2).unwrap();
    graph.add_edge(node_d, node_t, 2).unwrap();

    let (flow, _) = flow_algorithm(&graph, node_s, node_t);

    assert_eq!(flow, 4);
}

// // TODO: Re-Add this, once proper IndexId etc handling in Algorithms is done.
// pub(crate) fn test_max_flow_eight_call_me<G: DirectedGraph>(
//     graph_constructor: impl Fn() -> G,
//     graph_add_node: impl Fn(&mut G, u32) -> G::NodeId,
//     graph_add_edge: impl Fn(&mut G, G::NodeId, G::NodeId, u32) -> Option<G::EdgeId>,
//     graph_remove_node: impl Fn(&mut G, G::NodeId),
//     _graph_remove_edge: impl Fn(&mut G, G::EdgeId),
//     flow_algorithm: impl Fn(&G, G::NodeId, G::NodeId) -> (u32, Vec<u32>),
// ) {
//     // Example from https://downey.io/blog/max-flow-ford-fulkerson-algorithm-explanation/
//     // Graph Image: https://images.downey.io/max-flow/max-flow-3.png
//     let mut graph = graph_constructor();
//     let mut nodes = Vec::new();
//     nodes.push(graph_add_node(&mut graph, 0));
//     nodes.push(graph_add_node(&mut graph, 1));
//     nodes.push(graph_add_node(&mut graph, 2));
//     nodes.push(graph_add_node(&mut graph, 3));
//     nodes.push(graph_add_node(&mut graph, 4));
//     for (source, sink, weight) in [
//         (0, 1, 3),
//         (0, 2, 2),
//         (1, 2, 5),
//         (1, 4, 2),
//         (2, 4, 3),
//         (0, 3, 1),
//         (3, 4, 3),
//         (1, 3, 3),
//     ] {
//         graph_add_edge(&mut graph, nodes[source], nodes[sink], weight);
//     }
//     graph_remove_node(&mut graph, nodes[3]);
//     let (max_flow, _) = flow_algorithm(&graph, nodes[0], nodes[4]);
//     assert_eq!(5, max_flow);
// }

// pub(crate) fn test_max_flow_nine_call_me<G: DirectedGraph>(
//     graph_constructor: impl Fn() -> G,
//     graph_add_node: impl Fn(&mut G, ()) -> G::NodeId,
//     graph_add_edge: impl Fn(&mut G, G::NodeId, G::NodeId, u32) -> Option<G::EdgeId>,
//     _graph_remove_node: impl Fn(&mut G, G::NodeId),
//     graph_remove_edge: impl Fn(&mut G, G::EdgeId),
//     flow_algorithm: impl Fn(&G, G::NodeId, G::NodeId) -> (u32, Vec<u32>),
// ) {
//     // See issue https://github.com/petgraph/petgraph/issues/792
//     let mut g = graph_constructor();

//     let a = graph_add_node(&mut g, ());
//     let b = graph_add_node(&mut g, ());
//     let c = graph_add_node(&mut g, ());
//     let d = graph_add_node(&mut g, ());

//     let ac = graph_add_edge(&mut g, a, c, 1).unwrap();
//     let _ = graph_add_edge(&mut g, a, b, 1);
//     let _ = graph_add_edge(&mut g, b, c, 1).unwrap();
//     let _ = graph_add_edge(&mut g, b, d, 1);

//     // Current state of graph:
//     // a --1-- b --1-- c --1-- d
//     // |               |
//     // - -- -- 1 -- -- -

//     graph_remove_edge(&mut g, ac);

//     // Current state of graph:
//     // a --1-- b       c --1-- d
//     // |               |
//     // - -- -- 1 -- -- -

//     assert_eq!(1, flow_algorithm(&g, a, d).0);

//     let _ = graph_add_edge(&mut g, b, c, 1);
//     graph_remove_edge(&mut g, ac);

//     // Current state of graph:
//     // a --1-- b --1-- c --1-- d

//     assert_eq!(1, flow_algorithm(&g, a, d).0);

//     let _ = graph_add_edge(&mut g, a, c, 1);
//     let _ = graph_add_edge(&mut g, c, d, 1);

//     // Current state of graph:
//     //                 - --1-- -
//     //                 |       |
//     // a --1-- b --1-- c --1-- d
//     // |               |
//     // - -- -- 1 -- -- -

//     assert_eq!(2, flow_algorithm(&g, a, d).0);
// }
