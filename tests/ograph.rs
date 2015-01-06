extern crate petgraph;

use petgraph::{
    OGraph,
    BreadthFirst,
    dijkstra,
};

use petgraph::ograph::toposort;
use petgraph::ograph::min_spanning_tree;
use petgraph::EdgeDirection;



#[test]
fn mst() {
    let mut G = OGraph::<_, f32>::new();
    let a = G.add_node("A");
    let b = G.add_node("B");
    let c = G.add_node("C");
    let d = G.add_node("D");
    let e = G.add_node("E");
    let f = G.add_node("F");
    let g = G.add_node("G");
    G.add_edge(a, b, 7.);
    G.add_edge(a, d, 5.);
    G.add_edge(d, b, 9.);
    G.add_edge(b, c, 8.);
    G.add_edge(b, e, 7.);
    G.add_edge(c, e, 5.);
    G.add_edge(d, e, 15.);
    G.add_edge(d, f, 6.);
    G.add_edge(f, e, 8.);
    G.add_edge(f, g, 11.);
    G.add_edge(e, g, 9.);

    let mst = min_spanning_tree(&G);
    println!("MST is:\n{}", mst);
    assert!(mst.node_count() == G.node_count());
    assert!(mst.edge_count() == G.node_count() - 1);

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
