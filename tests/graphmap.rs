extern crate petgraph;

use petgraph::{
    GraphMap,
};

use petgraph::algo::{
    dijkstra,
};

#[test]
fn simple() {
    //let root = TypedArena::<Node<_>>::new();
    let mut gr = GraphMap::new();
    //let node = |&: name: &'static str| Ptr(root.alloc(Node(name.to_string())));
    let a = gr.add_node(("A"));
    let b = gr.add_node(("B"));
    let c = gr.add_node(("C"));
    let d = gr.add_node(("D"));
    let e = gr.add_node(("E"));
    let f = gr.add_node(("F"));
    gr.add_edge(a, b, 7);
    gr.add_edge(a, c, 9);
    gr.add_edge(a, d, 14);
    gr.add_edge(b, c, 10);
    gr.add_edge(c, d, 2);
    gr.add_edge(d, e, 9);
    gr.add_edge(b, f, 15);
    gr.add_edge(c, f, 11);

    assert!(gr.add_edge(e, f, 5));

    // duplicate edges
    assert!(!gr.add_edge(f, b, 15));
    assert!(!gr.add_edge(f, e, 6));
    println!("{:?}", gr);

    assert_eq!(gr.node_count(), 6);
    assert_eq!(gr.edge_count(), 9);

    // check updated edge weight
    assert_eq!(gr.edge_weight(e, f), Some(&6));
    let scores = dijkstra(&gr, a, None, |gr, n| gr.edges(n).map(|(n, &e)| (n, e)));
    let mut scores: Vec<_> = scores.into_iter().collect();
    scores.sort();
    assert_eq!(scores,
       vec![("A", 0), ("B", 7), ("C", 9), ("D", 11), ("E", 20), ("F", 20)]);
}

#[test]
fn remov()
{
    let mut g = GraphMap::new();
    g.add_node(1);
    g.add_node(2);
    g.add_edge(1, 2, -1);

    assert_eq!(g.edge_weight(1, 2), Some(&-1));
    assert_eq!(g.edge_weight(2, 1), Some(&-1));
    assert_eq!(g.neighbors(1).count(), 1);

    let noexist = g.remove_edge(2, 3);
    assert_eq!(noexist, None);

    let exist = g.remove_edge(2, 1);
    assert_eq!(exist, Some(-1));
    assert_eq!(g.edge_count(), 0);
    assert_eq!(g.edge_weight(1, 2), None);
    assert_eq!(g.edge_weight(2, 1), None);
    assert_eq!(g.neighbors(1).count(), 0);
}

#[test]
fn dfs() {
    let mut gr = GraphMap::new();
    let h = gr.add_node("H");
    let i = gr.add_node("I");
    let j = gr.add_node("J");
    let k = gr.add_node("K");
    // Z is disconnected.
    let _ = gr.add_node("Z");
    gr.add_edge(h, i, 1.);
    gr.add_edge(h, j, 3.);
    gr.add_edge(i, j, 1.);
    gr.add_edge(i, k, 2.);

    /*
    assert_eq!(DfsIter::new(&gr, h).count(), 4);
    assert_eq!(DfsIter::new(&gr, i).count(), 4);
    assert_eq!(DfsIter::new(&gr, z).count(), 1);
    */
}

