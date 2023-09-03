#![cfg(not(miri))]
#![cfg(feature = "dot")]

use dot::RenderOption;
use insta::assert_debug_snapshot;
use petgraph::{adjacency_matrix::AdjacencyList, graph::stable::StableGraph, Graph};
use petgraph_core::visit::EdgeRef;
use petgraph_io::dot::{Dot, EdgeAttributes, NodeAttributes};

fn simple_graph() -> Graph<&'static str, &'static str> {
    let mut graph = Graph::<&str, &str>::new();
    let a = graph.add_node("A");
    let b = graph.add_node("B");
    graph.add_edge(a, b, "edge_label");
    graph
}

#[test]
fn node_index_label() {
    let graph = simple_graph();

    let dot = Dot::with_config(&graph, &[])
        .with_node_attributes(&|_, (index, _)| NodeAttributes::new(index.to_string()));

    assert_debug_snapshot!(dot);
}

#[test]
fn edge_index_label() {
    let graph = simple_graph();

    let dot = Dot::with_config(&graph, &[])
        .with_edge_attributes(&|_, edge| EdgeAttributes::new(edge.id().index().to_string()));

    assert_debug_snapshot!(dot);
}

#[test]
fn edge_no_label() {
    let graph = simple_graph();
    let dot = Dot::with_config(&graph, &[RenderOption::NoEdgeLabels]);

    assert_debug_snapshot!(dot);
}

#[test]
fn node_no_label() {
    let graph = simple_graph();
    let dot = Dot::with_config(&graph, &[RenderOption::NoNodeLabels]);

    assert_debug_snapshot!(dot);
}

#[test]
fn label_map_to_weight() {
    let graph = simple_graph();
    let dot = Dot::with_config(&graph, &[])
        .with_node_attributes(&|_, (_, weight)| NodeAttributes::new(weight.to_uppercase()))
        .with_edge_attributes(&|_, edge| EdgeAttributes::new(edge.weight().to_uppercase()));

    assert_debug_snapshot!(dot);
}

#[test]
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
fn stable_graph() {
    let mut graph = StableGraph::new();

    let a = graph.add_node("A");
    let b = graph.add_node("B");

    graph.add_edge(a, a, "A -> A");
    graph.add_edge(a, b, "A -> B");

    assert_debug_snapshot!(Dot::new(&graph));
}
