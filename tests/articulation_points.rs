use petgraph::{
    algo::articulation_points::articulation_points,
    csr::IndexType,
    dot::Dot,
    graph::{NodeIndex, UnGraph},
    visit::{IntoEdges, NodeRef},
    Graph, Undirected,
};

use std::collections::HashSet;

#[test]
fn art_simple1() {
    let mut gr = Graph::<&str, ()>::new();
    let a = gr.add_node("A");
    let b = gr.add_node("B");
    let c = gr.add_node("C");
    let d = gr.add_node("D");
    gr.add_edge(a, b, ());
    gr.add_edge(b, c, ());
    gr.add_edge(b, d, ());

    let set: HashSet<NodeIndex> = [b].iter().cloned().collect();

    assert_eq!(articulation_points(&gr), set);
}

#[test]
fn art_simple2() {
    let mut gr = Graph::<&str, ()>::new();
    let a = gr.add_node("A");
    let b = gr.add_node("B");
    let c = gr.add_node("C");
    let d = gr.add_node("D");
    let e = gr.add_node("E");

    gr.add_edge(a, b, ());
    gr.add_edge(b, c, ());
    gr.add_edge(b, d, ());
    gr.add_edge(d, e, ());

    let set: HashSet<NodeIndex> = [b, d].iter().cloned().collect();

    assert_eq!(articulation_points(&gr), set);
}
