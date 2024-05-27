use petgraph::{graph6::Graph6, Graph};

#[test]
fn test_graph6_from_small_graph() {
    let mut gr = Graph::<_, _>::new();
    let _0 = gr.add_node("A");
    let _1 = gr.add_node("B");
    let _2 = gr.add_node("C");
    let _3 = gr.add_node("D");
    let _4 = gr.add_node("E");
    gr.add_edge(_0, _2, 7.);
    gr.add_edge(_0, _4, 7.);
    gr.add_edge(_1, _3, 7.);
    gr.add_edge(_3, _4, 7.);

    let graph6 = Graph6::from_graph(&gr);

    let str_representation = graph6.to_string();
    println!("{}", str_representation);
    assert!(str_representation == "DQc");
}

#[test]
fn test_graph6_from_graph() {
    let mut gr = Graph::<_, _>::new();
    let a = gr.add_node("A");
    let b = gr.add_node("B");
    let c = gr.add_node("C");
    let d = gr.add_node("D");
    let e = gr.add_node("E");
    let f = gr.add_node("F");
    let g = gr.add_node("G");
    gr.add_edge(a, b, 7.);
    gr.add_edge(a, d, 5.);
    gr.add_edge(d, b, 9.);
    gr.add_edge(b, c, 8.);
    gr.add_edge(b, e, 7.);
    gr.add_edge(c, e, 5.);
    gr.add_edge(d, e, 15.);
    gr.add_edge(d, f, 6.);
    gr.add_edge(f, e, 8.);
    gr.add_edge(f, g, 11.);
    gr.add_edge(e, g, 9.);

    // add a disjoint part
    let h = gr.add_node("H");
    let i = gr.add_node("I");
    let j = gr.add_node("J");
    gr.add_edge(h, i, 1.);
    gr.add_edge(h, j, 3.);
    gr.add_edge(i, j, 1.);

    let graph6 = Graph6::from_graph(&gr);

    let str_representation = graph6.to_string();
    println!("{}", str_representation);
    assert!(str_representation == "to-check");
}
