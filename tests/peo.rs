use petgraph::algo::peo::{is_chordal, lbfs};
use petgraph::{Graph, Undirected};

// http://www.cs.haifa.ac.il/~golumbic/courses/seminar-2013graph/corneil-LBFSsurvey.pdf
// figure 2
#[test]
fn lbfs_no_panic() {
    let mut graph: Graph<(), (), Undirected> = Graph::new_undirected();

    // origin graph has no n0
    // add n0 to shift node_index
    let _n0 = graph.add_node(());
    let n1 = graph.add_node(());
    let n2 = graph.add_node(());
    let n3 = graph.add_node(());
    let n4 = graph.add_node(());
    let n5 = graph.add_node(());
    let n6 = graph.add_node(());
    let n7 = graph.add_node(());
    let n8 = graph.add_node(());
    let n9 = graph.add_node(());
    let n10 = graph.add_node(());
    let n11 = graph.add_node(());

    graph.add_edge(n1, n2, ());
    graph.add_edge(n1, n3, ());
    graph.add_edge(n2, n3, ());
    graph.add_edge(n2, n4, ());
    graph.add_edge(n3, n4, ());

    graph.add_edge(n6, n11, ());
    graph.add_edge(n6, n5, ());
    graph.add_edge(n6, n7, ());
    graph.add_edge(n6, n8, ());
    graph.add_edge(n6, n9, ());
    graph.add_edge(n7, n8, ());
    graph.add_edge(n8, n9, ());
    graph.add_edge(n8, n10, ());

    graph.add_edge(n2, n5, ());
    graph.add_edge(n2, n6, ());
    graph.add_edge(n2, n7, ());
    graph.add_edge(n2, n8, ());
    graph.add_edge(n2, n9, ());
    graph.add_edge(n2, n10, ());

    graph.add_edge(n4, n5, ());
    graph.add_edge(n4, n6, ());
    graph.add_edge(n4, n7, ());
    graph.add_edge(n4, n8, ());
    graph.add_edge(n4, n9, ());
    graph.add_edge(n4, n10, ());

    let _ = lbfs(&graph);

    assert!(is_chordal(&graph));
}

#[test]
fn square_is_not_chordal() {
    let mut graph: Graph<(), (), Undirected> = Graph::new_undirected();

    let n0 = graph.add_node(());
    let n1 = graph.add_node(());
    let n2 = graph.add_node(());
    let n3 = graph.add_node(());

    graph.add_edge(n0, n1, ());
    graph.add_edge(n1, n2, ());
    graph.add_edge(n2, n3, ());
    graph.add_edge(n3, n0, ());

    assert!(!is_chordal(&graph));
}

//     d          e
//     /\--------/\
//    /  \      /  \
//   /    \    /    \
//  /      \  /      \
// /________\/________\
// a         b        c
#[test]
fn should_be_chordal() {
    let mut graph: Graph<(), (), Undirected> = Graph::new_undirected();

    let a = graph.add_node(());
    let b = graph.add_node(());
    let c = graph.add_node(());
    let d = graph.add_node(());
    let e = graph.add_node(());

    graph.add_edge(a, b, ());
    graph.add_edge(b, c, ());
    graph.add_edge(d, e, ());
    graph.add_edge(d, a, ());
    graph.add_edge(d, b, ());
    graph.add_edge(d, b, ());
    graph.add_edge(d, c, ());

    assert!(is_chordal(&graph));
}
