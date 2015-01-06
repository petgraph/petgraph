
extern crate petgraph;

use petgraph::{
    OGraph,
    //BreadthFirst,
    dijkstra,
};

use petgraph::ograph::toposort;
use petgraph::EdgeDirection;


#[test]
fn ograph_1()
{
    let mut og = OGraph::new();
    let a = og.add_node(0i);
    let b = og.add_node(1i);
    let c = og.add_node(2i);
    let d = og.add_node(3i);
    let _ = og.add_edge(a, b, 0i);
    let _ = og.add_edge(a, c, 1);
    og.add_edge(c, a, 2);
    og.add_edge(a, a, 3);
    og.add_edge(b, c, 4);
    og.add_edge(b, a, 5);
    og.add_edge(a, d, 6);
    assert_eq!(og.node_count(), 4);
    assert_eq!(og.edge_count(), 7);

    assert!(og.find_edge(a, b).is_some());
    assert!(og.find_edge(d, a).is_none());
    assert!(og.find_edge(a, a).is_some());

    assert_eq!(og.neighbors(b, EdgeDirection::Outgoing).collect::<Vec<_>>(), vec![a, c]);

    og.remove_node(a);
    assert_eq!(og.neighbors(b, EdgeDirection::Outgoing).collect::<Vec<_>>(), vec![c]);
    assert_eq!(og.node_count(), 3);
    assert_eq!(og.edge_count(), 1);
    assert!(og.find_edge(a, b).is_none());
    assert!(og.find_edge(d, a).is_none());
    assert!(og.find_edge(a, a).is_none());
    assert!(og.find_edge(b, c).is_some());
}

#[test]
fn ograph_2()
{
    let mut g = OGraph::<_, f32>::new();
    let a = g.add_node("A");
    let b = g.add_node("B");
    let c = g.add_node("C");
    let d = g.add_node("D");
    let e = g.add_node("E");
    let f = g.add_node("F");
    g.add_edge(a, b, 7.);
    g.add_edge(a, c, 9.);
    g.add_edge(a, d, 14.);
    g.add_edge(b, c, 10.);
    g.add_edge(c, d, 2.);
    g.add_edge(d, e, 9.);
    g.add_edge(b, f, 15.);
    g.add_edge(c, f, 11.);
    g.add_edge(e, f, 6.);
    let scores = dijkstra(&g, a, |gr, n| gr.edges(n, EdgeDirection::Outgoing).map(|(n, &e)| (n, e)));
    assert_eq!(scores[f], 20.);

    let x = g.add_node("X");
    let y = g.add_node("Y");
    g.add_edge(x, y, 0.);
    println!("{}", toposort(&g));
}
