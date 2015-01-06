extern crate petgraph;

use petgraph::{
    OGraph,
};

use petgraph::ograph::min_spanning_tree;



#[test]
fn mst() {
    let mut gr = OGraph::<_, f32>::new();
    let a = gr.add_node("A");
    let b = gr.add_node("B");
    let c = gr.add_node("C");
    let d = gr.add_node("D");
    let e = gr.add_node("E");
    let f = gr.add_node("F");
    let g = gr.add_node("gr");
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

    let mst = min_spanning_tree(&gr);
    println!("MST is:\n{}", mst);
    assert!(mst.node_count() == gr.node_count());
    assert!(mst.edge_count() == gr.node_count() - 1);

    let one_of = |&: a: bool, b: bool| {
        (a && !b) || (!a && b)
    };
    let have_one = |&: x, y| {
        one_of(mst.find_edge(x, y).is_some(), mst.find_edge(y, x).is_some())
    };

    assert!(have_one(a, b));
    assert!(have_one(a, d));
    assert!(have_one(b, e));
    assert!(have_one(e, c));
    assert!(have_one(e, g));
    assert!(have_one(d, f));
    
    assert!(mst.find_edge(d, b).is_none());
    assert!(mst.find_edge(b, c).is_none());

    // check the exact edges are there
}
