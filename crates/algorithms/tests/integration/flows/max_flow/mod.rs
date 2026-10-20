use petgraph_algorithms::flows::max_flow::{
    MaxFlow, dinics_mod::Dinics, edmonds_karp_mod::EdmondsKarp,
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
                EdmondsKarp::new_with_capacities(graph, |edge_id| {
                    graph.edge(edge_id).unwrap().data.clone()
                })
                .with_source(source)
                .with_destination(destination)
                .run()
                .unwrap()
                .into_max_flow_and_flow_data()
            });
        }
        mod dinics {
            use super::*;
            test_max_flow_all_examples!($graph_constructor, |graph, source, destination| {
                Dinics::new_with_capacities(graph, |edge_id| {
                    graph.edge(edge_id).unwrap().data.clone()
                })
                .with_source(source)
                .with_destination(destination)
                .run()
                .unwrap()
                .into_max_flow_and_flow_data()
            });
        }
        mod max_flow_auto {
            use super::*;
            test_max_flow_all_examples!($graph_constructor, |graph, source, destination| {
                MaxFlow::new_with_capacities(graph, |edge_id| {
                    graph.edge(edge_id).unwrap().data.clone()
                })
                .with_source(source)
                .with_destination(destination)
                .run()
                .unwrap()
                .into_max_flow_and_flow_data()
            });
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

        #[test]
        fn test_eight() {
            crate::flows::max_flow::test_max_flow_eight($graph_constructor, $max_flow_algorithm);
        }

        #[test]
        fn test_nine() {
            crate::flows::max_flow::test_max_flow_nine($graph_constructor, $max_flow_algorithm);
        }
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
pub(crate) fn test_max_flow_eight<
    G: DirectedGraph + StorableGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = u32>,
>(
    graph_constructor: impl Fn() -> G,
    flow_algorithm: impl Fn(&G, G::NodeId, G::NodeId) -> (u32, G::EdgeDataContainer<u32>),
) {
    // Example from https://downey.io/blog/max-flow-ford-fulkerson-algorithm-explanation/
    // Graph Image: https://images.downey.io/max-flow/max-flow-3.png
    let mut graph = graph_constructor();
    let mut nodes = Vec::new();
    nodes.push(graph.add_node(()).unwrap());
    nodes.push(graph.add_node(()).unwrap());
    nodes.push(graph.add_node(()).unwrap());
    nodes.push(graph.add_node(()).unwrap());
    nodes.push(graph.add_node(()).unwrap());
    for (source, sink, weight) in [
        (0, 1, 3),
        (0, 2, 2),
        (1, 2, 5),
        (1, 4, 2),
        (2, 4, 3),
        (0, 3, 1),
        (3, 4, 3),
        (1, 3, 3),
    ] {
        graph.add_edge(nodes[source], nodes[sink], weight).unwrap();
    }
    graph.remove_node(nodes[3]);
    let (max_flow, _) = flow_algorithm(&graph, nodes[0], nodes[4]);
    assert_eq!(5, max_flow);
}

pub(crate) fn test_max_flow_nine<
    G: DirectedGraph + StorableGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = u32>,
>(
    graph_constructor: impl Fn() -> G,
    flow_algorithm: impl Fn(&G, G::NodeId, G::NodeId) -> (u32, G::EdgeDataContainer<u32>),
) {
    // See issue https://github.com/petgraph/petgraph/issues/792
    let mut graph = graph_constructor();

    let node_a = graph.add_node(()).unwrap();
    let node_b = graph.add_node(()).unwrap();
    let node_c = graph.add_node(()).unwrap();
    let node_d = graph.add_node(()).unwrap();

    let ac = graph.add_edge(node_a, node_c, 1).unwrap();
    let _ = graph.add_edge(node_a, node_b, 1).unwrap();
    let bc = graph.add_edge(node_b, node_c, 1).unwrap();
    let _ = graph.add_edge(node_b, node_d, 1).unwrap();

    // Current state of graph:
    // a --1-- b --1-- c --1-- d
    // |               |
    // - -- -- 1 -- -- -

    graph.remove_edge(bc).unwrap();

    // Current state of graph:
    // a --1-- b       c --1-- d
    // |               |
    // - -- -- 1 -- -- -

    assert_eq!(1, flow_algorithm(&graph, node_a, node_d).0);

    let _ = graph.add_edge(node_b, node_c, 1).unwrap();
    graph.remove_edge(ac).unwrap();

    // Current state of graph:
    // a --1-- b --1-- c --1-- d

    assert_eq!(1, flow_algorithm(&graph, node_a, node_d).0);

    let _ = graph.add_edge(node_a, node_c, 1).unwrap();
    let _ = graph.add_edge(node_c, node_d, 1).unwrap();

    // Current state of graph:
    //                 - --1-- -
    //                 |       |
    // a --1-- b --1-- c --1-- d
    // |               |
    // - -- -- 1 -- -- -

    assert_eq!(2, flow_algorithm(&graph, node_a, node_d).0);
}
