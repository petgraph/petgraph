#![cfg(feature = "dot")]

use insta::{assert_debug_snapshot, assert_display_snapshot};
#[cfg(feature = "stable-graph")]
use petgraph::graph::stable::StableGraph;
use petgraph::{dot::Dot, Graph};
#[cfg(feature = "adjacency-matrix")]
use petgraph_adjacency_matrix::AdjacencyList;

#[test]
#[cfg(feature = "adjacency-matrix")]
fn adjacency_list() {
    let mut graph = AdjacencyList::new();

    let a = graph.add_node();
    let b = graph.add_node();

    graph.add_edge(a, a, "A -> A");
    graph.add_edge(a, b, "A -> B");
    graph.add_edge(b, a, "B -> A");

    assert_debug_snapshot!(Dot::new(&graph));
}

#[test]
fn graph() {
    let mut graph = Graph::new();

    let a = graph.add_node("A");
    let b = graph.add_node("B");

    graph.add_edge(a, a, "A -> A");
    graph.add_edge(a, b, "A -> B");
    graph.add_edge(b, a, "B -> A");

    assert_debug_snapshot!(Dot::new(&graph));
}

#[test]
fn debug_formatting() {
    // test alternate formatting
    #[derive(Debug)]
    struct Record {
        a: i32,
        b: &'static str,
    }
    let mut graph = Graph::new();
    let a = graph.add_node(Record { a: 1, b: r"abc\" });
    graph.add_edge(a, a, (1, 2));

    assert_debug_snapshot!(Dot::new(&graph));
}

#[test]
#[cfg(feature = "stable-graph")]
fn stable_graph() {
    let mut graph = StableGraph::new();

    let a = graph.add_node("A");
    let b = graph.add_node("B");

    graph.add_edge(a, a, "A -> A");
    graph.add_edge(a, b, "A -> B");

    assert_debug_snapshot!(Dot::new(&graph));
}
