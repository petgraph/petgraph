mod dinics;
mod ford_fulkerson;

use petgraph_algorithms::flows::maximum_flow::ford_fulkerson;
use petgraph_core::{graph::DirectedGraph, utils::directed::DirectedTestGraph};

use crate::run_macro_for_all_graphs;

macro_rules! test_max_flow_on_graph {
    (
        $graph_constructor:expr,
        $graph_add_node:expr,
        $graph_add_edge:expr,
        $graph_remove_node:expr,
        $graph_remove_edge:expr
    ) => {
        test_max_flow_all_examples!(
            $graph_constructor,
            $graph_add_node,
            $graph_add_edge,
            $graph_remove_node,
            $graph_remove_edge,
            // Due to some lifetime magic, it is important to wrap the algorithm in a closure here.
            |graph, source, destination| ford_fulkerson(graph, source, destination)
        );
    };
}

macro_rules! test_max_flow_all_examples {
    (
        $graph_constructor:expr,
        $graph_add_node:expr,
        $graph_add_edge:expr,
        $graph_remove_node:expr,
        $graph_remove_edge:expr,
        $max_flow_algorithm:expr
    ) => {
        #[test]
        fn test_max_flow_one() {
            crate::flows::maximum_flow::test_max_flow_one_call_me(
                $graph_constructor,
                $graph_add_node,
                $graph_add_edge,
                $graph_remove_node,
                $graph_remove_edge,
                $max_flow_algorithm,
            );
        }

        #[test]
        fn test_max_flow_two() {
            crate::flows::maximum_flow::test_max_flow_two_call_me(
                $graph_constructor,
                $graph_add_node,
                $graph_add_edge,
                $graph_remove_node,
                $graph_remove_edge,
                $max_flow_algorithm,
            );
        }

        #[test]
        fn test_max_flow_three() {
            crate::flows::maximum_flow::test_max_flow_three_call_me(
                $graph_constructor,
                $graph_add_node,
                $graph_add_edge,
                $graph_remove_node,
                $graph_remove_edge,
                $max_flow_algorithm,
            );
        }

        #[test]
        fn test_max_flow_four() {
            crate::flows::maximum_flow::test_max_flow_four_call_me(
                $graph_constructor,
                $graph_add_node,
                $graph_add_edge,
                $graph_remove_node,
                $graph_remove_edge,
                $max_flow_algorithm,
            );
        }

        #[test]
        fn test_max_flow_five() {
            crate::flows::maximum_flow::test_max_flow_five_call_me(
                $graph_constructor,
                $graph_add_node,
                $graph_add_edge,
                $graph_remove_node,
                $graph_remove_edge,
                $max_flow_algorithm,
            );
        }

        #[test]
        fn test_max_flow_six() {
            crate::flows::maximum_flow::test_max_flow_six_call_me(
                $graph_constructor,
                $graph_add_node,
                $graph_add_edge,
                $graph_remove_node,
                $graph_remove_edge,
                $max_flow_algorithm,
            );
        }

        // #[test]
        // fn test_max_flow_seven() {
        //     crate::flows::maximum_flow::test_max_flow_seven_call_me(
        //         $graph_constructor,
        //         $graph_add_node,
        //         $graph_add_edge,
        //         $graph_remove_node,
        //         $graph_remove_edge,
        //         $max_flow_algorithm,
        //     );
        // }
    };
}

run_macro_for_all_graphs!(test_max_flow_on_graph);

pub(crate) fn test_max_flow_one_call_me<G: DirectedGraph>(
    graph_constructor: impl Fn() -> G,
    graph_add_node: impl Fn(&mut G, ()) -> G::NodeId,
    graph_add_edge: impl Fn(&mut G, G::NodeId, G::NodeId, u32) -> Option<G::EdgeId>,
    _graph_remove_node: impl Fn(&mut G, G::NodeId),
    _graph_remove_edge: impl Fn(&mut G, G::EdgeId),
    flow_algorithm: impl Fn(&G, G::NodeId, G::NodeId) -> (u32, Vec<u32>),
) {
    // Example from https://downey.io/blog/max-flow-ford-fulkerson-algorithm-explanation/
    let mut graph = graph_constructor();
    let mut nodes = Vec::new();
    nodes.push(graph_add_node(&mut graph, ()));
    nodes.push(graph_add_node(&mut graph, ()));
    nodes.push(graph_add_node(&mut graph, ()));
    nodes.push(graph_add_node(&mut graph, ()));
    for (source, sink, weight) in [(0, 1, 3), (0, 2, 2), (1, 2, 5), (1, 3, 2), (2, 3, 3)] {
        graph_add_edge(&mut graph, nodes[source], nodes[sink], weight);
    }
    let (max_flow, _) = flow_algorithm(&graph, nodes[0], nodes[3]);
    assert_eq!(5, max_flow);
}

pub(crate) fn test_max_flow_two_call_me<G: DirectedGraph>(
    graph_constructor: impl Fn() -> G,
    graph_add_node: impl Fn(&mut G, ()) -> G::NodeId,
    graph_add_edge: impl Fn(&mut G, G::NodeId, G::NodeId, u32) -> Option<G::EdgeId>,
    _graph_remove_node: impl Fn(&mut G, G::NodeId),
    _graph_remove_edge: impl Fn(&mut G, G::EdgeId),
    flow_algorithm: impl Fn(&G, G::NodeId, G::NodeId) -> (u32, Vec<u32>),
) {
    // Example from https://brilliant.org/wiki/ford-fulkerson-algorithm/
    let mut graph = graph_constructor();
    let mut nodes = Vec::new();
    for _ in 0..6 {
        nodes.push(graph_add_node(&mut graph, ()));
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
        graph_add_edge(&mut graph, nodes[source], nodes[sink], weight);
    }
    let (max_flow, _) = flow_algorithm(&graph, nodes[0], nodes[5]);
    assert_eq!(7, max_flow);
}

pub(crate) fn test_max_flow_three_call_me<G: DirectedGraph>(
    graph_constructor: impl Fn() -> G,
    graph_add_node: impl Fn(&mut G, ()) -> G::NodeId,
    graph_add_edge: impl Fn(&mut G, G::NodeId, G::NodeId, u32) -> Option<G::EdgeId>,
    _graph_remove_node: impl Fn(&mut G, G::NodeId),
    _graph_remove_edge: impl Fn(&mut G, G::EdgeId),
    flow_algorithm: impl Fn(&G, G::NodeId, G::NodeId) -> (u32, Vec<u32>),
) {
    // Example from https://cp-algorithms.com/graph/edmonds_karp.html
    let mut graph = graph_constructor();
    let mut nodes = Vec::new();
    for _ in 0..6 {
        nodes.push(graph_add_node(&mut graph, ()));
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
        graph_add_edge(&mut graph, nodes[source], nodes[sink], weight);
    }
    let (max_flow, _) = flow_algorithm(&graph, nodes[0], nodes[5]);
    assert_eq!(10, max_flow);
}

pub(crate) fn test_max_flow_four_call_me<G: DirectedGraph>(
    graph_constructor: impl Fn() -> G,
    graph_add_node: impl Fn(&mut G, ()) -> G::NodeId,
    graph_add_edge: impl Fn(&mut G, G::NodeId, G::NodeId, u32) -> Option<G::EdgeId>,
    _graph_remove_node: impl Fn(&mut G, G::NodeId),
    _graph_remove_edge: impl Fn(&mut G, G::EdgeId),
    flow_algorithm: impl Fn(&G, G::NodeId, G::NodeId) -> (u32, Vec<u32>),
) {
    // Example from https://www.programiz.com/dsa/ford-fulkerson-algorithm (corrected: result not 6 but 5)
    let mut graph = graph_constructor();
    let mut nodes = Vec::new();
    for _ in 0..6 {
        nodes.push(graph_add_node(&mut graph, ()));
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
        graph_add_edge(&mut graph, nodes[source], nodes[sink], weight);
    }
    let (max_flow, _) = flow_algorithm(&graph, nodes[0], nodes[5]);
    assert_eq!(5, max_flow);
}

pub(crate) fn test_max_flow_five_call_me<G: DirectedGraph>(
    graph_constructor: impl Fn() -> G,
    graph_add_node: impl Fn(&mut G, ()) -> G::NodeId,
    graph_add_edge: impl Fn(&mut G, G::NodeId, G::NodeId, u32) -> Option<G::EdgeId>,
    _graph_remove_node: impl Fn(&mut G, G::NodeId),
    _graph_remove_edge: impl Fn(&mut G, G::EdgeId),
    flow_algorithm: impl Fn(&G, G::NodeId, G::NodeId) -> (u32, Vec<u32>),
) {
    let mut graph = graph_constructor();
    let mut nodes = Vec::new();
    for _ in 0..6 {
        nodes.push(graph_add_node(&mut graph, ()));
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
        graph_add_edge(&mut graph, nodes[source], nodes[sink], weight);
    }
    let (max_flow, _) = flow_algorithm(&graph, nodes[0], nodes[5]);
    assert_eq!(23, max_flow);
}

pub(crate) fn test_max_flow_six_call_me<G: DirectedGraph>(
    graph_constructor: impl Fn() -> G,
    graph_add_node: impl Fn(&mut G, ()) -> G::NodeId,
    graph_add_edge: impl Fn(&mut G, G::NodeId, G::NodeId, u32) -> Option<G::EdgeId>,
    _graph_remove_node: impl Fn(&mut G, G::NodeId),
    _graph_remove_edge: impl Fn(&mut G, G::EdgeId),
    flow_algorithm: impl Fn(&G, G::NodeId, G::NodeId) -> (u32, Vec<u32>),
) {
    // Example taken from https://medium.com/@jithmisha/solving-the-maximum-flow-problem-with-ford-fulkerson-method-3fccc2883dc7
    let mut graph = graph_constructor();
    let mut nodes = Vec::new();
    for _ in 0..6 {
        nodes.push(graph_add_node(&mut graph, ()));
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
        graph_add_edge(&mut graph, nodes[source], nodes[sink], weight);
    }
    let (max_flow, _) = flow_algorithm(&graph, nodes[0], nodes[5]);
    assert_eq!(19, max_flow);
}

// TODO: Re-Add this, once proper IndexId etc handling in Algorithms is done.
// pub(crate) fn test_max_flow_seven_call_me<G: DirectedGraph>(
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
