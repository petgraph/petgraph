extern crate petgraph;

use std::collections::HashSet;

use petgraph::{
    GraphMap,
    Dfs,
};
use petgraph::visit::{
    DfsIter,
};

use petgraph::algo::{
    dijkstra,
};

use petgraph::dot::{Dot, Config};

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

    assert!(gr.add_edge(e, f, 5).is_none());

    // duplicate edges
    assert_eq!(gr.add_edge(f, b, 16), Some(15));
    assert_eq!(gr.add_edge(f, e, 6), Some(5));
    println!("{:?}", gr);
    println!("{}", Dot::with_config(&gr, &[]));

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
    let z = gr.add_node("Z");
    gr.add_edge(h, i, 1.);
    gr.add_edge(h, j, 3.);
    gr.add_edge(i, j, 1.);
    gr.add_edge(i, k, 2.);

    println!("{:?}", gr);

    {
        let mut cnt = 0;
        let mut dfs = Dfs::new(&gr, h);
        while let Some(_) = dfs.next(&gr) { cnt += 1; }
        assert_eq!(cnt, 4);
    }
    {
        let mut cnt = 0;
        let mut dfs = Dfs::new(&gr, z);
        while let Some(_) = dfs.next(&gr) { cnt += 1; }
        assert_eq!(cnt, 1);
    }

    assert_eq!(DfsIter::new(&gr, h).count(), 4);
    assert_eq!(DfsIter::new(&gr, i).count(), 4);
    assert_eq!(DfsIter::new(&gr, z).count(), 1);
}

#[test]
fn edge_iterator() {
    let mut gr: GraphMap<&str, u64> = GraphMap::new();
    let h = gr.add_node("H");
    let i = gr.add_node("I");
    let j = gr.add_node("J");
    let k = gr.add_node("K");
    gr.add_edge(h, i, 1);
    gr.add_edge(h, j, 2);
    gr.add_edge(i, j, 3);
    gr.add_edge(i, k, 4);

    let real_edges: HashSet<_> = gr.all_edges().map(|(a, b, &w)| (a, b, w)).collect();
    let expected_edges: HashSet<_> = vec![
        ("H", "I", 1),
        ("H", "J", 2),
        ("I", "J", 3),
        ("I", "K", 4)
    ].into_iter().collect();

    assert_eq!(real_edges, expected_edges);
}

#[test]
fn from_edges() {
    let gr = GraphMap::from_edges(&[
        ("a", "b", 1),
        ("a", "c", 2),
        ("c", "d", 3),
    ]);
    assert_eq!(gr.node_count(), 4);
    assert_eq!(gr.edge_count(), 3);
    assert_eq!(gr[("a", "c")], 2);

    let gr = GraphMap::<_, ()>::from_edges(&[
        (0, 1), (0, 2), (0, 3),
        (1, 2), (1, 3),
        (2, 3),
    ]);
    assert_eq!(gr.node_count(), 4);
    assert_eq!(gr.edge_count(), 6);
    assert_eq!(gr.neighbors(0).count(), 3);
    assert_eq!(gr.neighbors(1).count(), 3);
    assert_eq!(gr.neighbors(2).count(), 3);
    assert_eq!(gr.neighbors(3).count(), 3);

    println!("{:?}", Dot::with_config(&gr, &[Config::EdgeNoLabel]));
}
