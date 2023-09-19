//! This is the same test code from `algorithm::toposort`, but adapted for the visit trait.
// TODO: unify both, traversal shouldn't be in core

use petgraph_core::deprecated::{edge::Directed, index::IndexType, visit::Topo};
use petgraph_dino::{DiDinoGraph, NodeId};
use petgraph_proptest::dag::graph_dag_strategy;
use proptest::prelude::*;

fn assert_topologically_sorted<N, E>(graph: &DiDinoGraph<N, E>, order: &[NodeId]) {
    assert_eq!(graph.num_nodes(), order.len());

    // check all the edges of the graph
    for edge in graph.edges() {
        let source = edge.source_id();
        let target = edge.target_id();

        let source_index = order
            .iter()
            .position(|x| x == source)
            .expect("Source node not found");

        let target_index = order
            .iter()
            .position(|x| x == target)
            .expect("Target node not found");

        assert!(
            source_index < target_index,
            "Graph is not topologically sorted ({target} comes before {source})",
        );
    }
}

/// This uses the example from the Wikipedia page on topological sorting:
/// <https://en.wikipedia.org/wiki/Topological_sorting#Examples>
///
/// Node to name mapping:
/// * 2: "A"
/// * 3: "B"
/// * 5: "C"
/// * 7: "D"
/// * 8: "E"
/// * 9: "F"
/// * 10: "G"
/// * 11: "H"
fn setup() -> DiDinoGraph<&'static str, &'static str> {
    let mut graph = DiDinoGraph::new();

    let a = *graph.insert_node("A").id();
    let b = *graph.insert_node("B").id();
    let c = *graph.insert_node("C").id();
    let d = *graph.insert_node("D").id();
    let e = *graph.insert_node("E").id();
    let f = *graph.insert_node("F").id();
    let g = *graph.insert_node("G").id();
    let h = *graph.insert_node("H").id();

    for (source, target, weight) in [
        (b, e, "B → E"), //
        (b, g, "B → G"),
        (c, h, "C → H"),
        (d, e, "D → E"),
        (d, h, "D → H"),
        (e, f, "E → F"),
        (h, a, "H → A"),
        (h, f, "H → F"),
        (h, g, "H → G"),
    ] {
        graph.insert_edge(weight, &source, &target);
    }

    graph
}

#[test]
fn example() {
    let graph = setup();

    let mut topo = Topo::new(&graph);
    let mut order = vec![];

    while let Some(node) = topo.next(&graph) {
        order.push(node);
    }

    assert_topologically_sorted(&graph, &order);
}

#[test]
fn disjoint() {
    let mut graph = DiDinoGraph::new();

    let a = *graph.insert_node("A").id();
    let b = *graph.insert_node("B").id();
    let c = *graph.insert_node("C").id();
    let d = *graph.insert_node("D").id();

    graph.insert_edge("A → B", &a, &b);
    graph.insert_edge("C → D", &c, &d);

    let mut topo = Topo::new(&graph);
    let mut order = vec![];

    while let Some(node) = topo.next(&graph) {
        order.push(node);
    }

    assert_eq!(order.len(), 4);
    assert_topologically_sorted(&graph, &order);
}

#[test]
fn path() {
    let mut graph = DiDinoGraph::new();

    let a = *graph.insert_node("A").id();
    let b = *graph.insert_node("B").id();

    graph.insert_edge("A → B", &a, &b);

    let mut topo = Topo::new(&graph);
    let mut order = vec![];

    while let Some(node) = topo.next(&graph) {
        order.push(node);
    }

    assert_eq!(order, vec![a, b]);
}

#[test]
fn error_on_cycle() {
    let mut graph = DiDinoGraph::new();

    let a = *graph.insert_node("A").id();
    let b = *graph.insert_node("B").id();

    graph.insert_edge("A → B", &a, &b);
    graph.insert_edge("B → A", &b, &a);

    let mut topo = Topo::new(&graph);
    // toposort silently ignores cycles, therefore `Topo` will output `None`, instead of returning a
    // result.
    let order = topo.next(&graph);
    assert!(order.is_none());
}

fn topo_sort(graph: &DiDinoGraph<(), ()>) -> Vec<NodeId> {
    let mut topo = Topo::new(graph);
    let mut order = vec![];

    while let Some(node) = topo.next(graph) {
        order.push(node);
    }

    assert_topologically_sorted(graph, &order);
    order
}

// TODO: proptest isn't converted yet
// #[cfg(not(miri))]
// proptest! {
//     #[test]
//     fn consistent_ordering(graph in graph_dag_strategy::<Graph<(), (), Directed, u8>>(None, None,
// None)) {         let order_a = topo_sort(&graph);
//         let order_b = topo_sort(&graph);
//
//         assert_eq!(order_a, order_b);
//     }
// }
