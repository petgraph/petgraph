use petgraph::{graph6::Graph6, Graph};

#[test]
fn test_graph6_from_small_graph() {
    let mut gr = Graph::new_undirected();
    let a = gr.add_node("A");
    let b = gr.add_node("B");
    let c = gr.add_node("C");
    let d = gr.add_node("D");
    let e = gr.add_node("E");
    gr.add_edge(a, c, ());
    gr.add_edge(a, e, ());
    gr.add_edge(b, d, ());
    gr.add_edge(d, e, ());

    let graph6_string = gr.graph6_string();
    println!("{}", graph6_string);
    assert!(graph6_string == "DQc");
}

#[test]
fn test_graph6_from_graph() {
    let mut gr = Graph::new_undirected();
    let a = gr.add_node("A");
    let b = gr.add_node("B");
    let c = gr.add_node("C");
    let d = gr.add_node("D");
    let e = gr.add_node("E");
    let f = gr.add_node("F");
    let g = gr.add_node("G");
    gr.add_edge(a, b, ());
    gr.add_edge(a, d, ());
    gr.add_edge(d, b, ());
    gr.add_edge(b, c, ());
    gr.add_edge(b, e, ());
    gr.add_edge(c, e, ());
    gr.add_edge(d, e, ());
    gr.add_edge(d, f, ());
    gr.add_edge(f, e, ());
    gr.add_edge(f, g, ());
    gr.add_edge(e, g, ());

    // add a disjoint part
    let h = gr.add_node("H");
    let i = gr.add_node("I");
    let j = gr.add_node("J");
    gr.add_edge(h, i, ());
    gr.add_edge(h, j, ());
    gr.add_edge(i, j, ());

    let graph6 = gr.graph6_string();

    let str_representation = graph6.to_string();
    println!("{}", str_representation);
    assert!(str_representation == "to-check");
}
