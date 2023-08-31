#![cfg(feature = "alloc")]
//! This is the same test code from `algorithm::toposort`, but adapted for the visit trait.
// TODO: unify both, traversal shouldn't be in core

use petgraph::{graph::NodeIndex, Graph};
use petgraph_core::deprecated::{edge::Directed, index::IndexType, visit::Topo};
use petgraph_proptest::dag::graph_dag_strategy;
use proptest::prelude::*;

fn assert_topologically_sorted<N, E, Ix>(graph: &Graph<N, E, Directed, Ix>, order: &[NodeIndex<Ix>])
where
    Ix: IndexType,
{
    assert_eq!(graph.node_count(), order.len());
    // check all the edges of the graph
    for edge in graph.raw_edges() {
        let source = edge.source();
        let target = edge.target();

        let source_index = order
            .iter()
            .position(|x| *x == source)
            .expect("Source node not found");

        let target_index = order
            .iter()
            .position(|x| *x == target)
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
fn setup() -> Graph<&'static str, &'static str> {
    let mut graph = Graph::new();

    let a = graph.add_node("A");
    let b = graph.add_node("B");
    let c = graph.add_node("C");
    let d = graph.add_node("D");
    let e = graph.add_node("E");
    let f = graph.add_node("F");
    let g = graph.add_node("G");
    let h = graph.add_node("H");

    graph.extend_with_edges([
        (b, e, "B → E"), //
        (b, g, "B → G"),
        (c, h, "C → H"),
        (d, e, "D → E"),
        (d, h, "D → H"),
        (e, f, "E → F"),
        (h, a, "H → A"),
        (h, f, "H → F"),
        (h, g, "H → G"),
    ]);

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
    let mut graph = Graph::new();

    let a = graph.add_node("A");
    let b = graph.add_node("B");
    let c = graph.add_node("C");
    let d = graph.add_node("D");

    graph.add_edge(a, b, "A → B");
    graph.add_edge(c, d, "C → D");

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
    let mut graph = Graph::new();

    let a = graph.add_node("A");
    let b = graph.add_node("B");

    graph.add_edge(a, b, "A → B");

    let mut topo = Topo::new(&graph);
    let mut order = vec![];

    while let Some(node) = topo.next(&graph) {
        order.push(node);
    }

    assert_eq!(order, vec![a, b]);
}

#[test]
fn error_on_cycle() {
    let mut graph = Graph::new();

    let a = graph.add_node("A");
    let b = graph.add_node("B");

    graph.add_edge(a, b, "A → B");
    graph.add_edge(b, a, "B → A");

    let mut topo = Topo::new(&graph);
    // toposort silently ignores cycles, therefore `Topo` will output `None`, instead of returning a
    // result.
    let order = topo.next(&graph);
    assert!(order.is_none());
}

fn topo_sort(graph: &Graph<(), (), Directed, u8>) -> Vec<NodeIndex<u8>> {
    let mut topo = Topo::new(graph);
    let mut order = vec![];

    while let Some(node) = topo.next(graph) {
        order.push(node);
    }

    assert_topologically_sorted(graph, &order);
    order
}

#[cfg(not(miri))]
proptest! {
    #[test]
    fn consistent_ordering(graph in graph_dag_strategy::<Graph<(), (), Directed, u8>>(None, None, None)) {
        let order_a = topo_sort(&graph);
        let order_b = topo_sort(&graph);

        assert_eq!(order_a, order_b);
    }
}
