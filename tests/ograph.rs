#![feature(core)]

extern crate petgraph;

use std::iter::AdditiveIterator;

use petgraph::{
    Graph,
    Bfs,
    Dfs,
    Incoming,
    Outgoing,
    Directed,
    Undirected,
};

use petgraph::algo::{
    min_spanning_tree,
    is_cyclic_undirected,
};

use petgraph::graph::NodeIndex;

use petgraph::visit::{
    //Reversed,
    //AsUndirected,
};
use petgraph::algo::{
    dijkstra,
};

#[test]
fn undirected()
{
    let mut og = Graph::new_undirected();
    let a = og.add_node(0);
    let b = og.add_node(1);
    let c = og.add_node(2);
    let d = og.add_node(3);
    let _ = og.add_edge(a, b, 0);
    let _ = og.add_edge(a, c, 1);
    og.add_edge(c, a, 2);
    og.add_edge(a, a, 3);
    og.add_edge(b, c, 4);
    og.add_edge(b, a, 5);
    og.add_edge(a, d, 6);
    assert_eq!(og.node_count(), 4);
    assert_eq!(og.edge_count(), 7);

    assert!(og.find_edge(a, b).is_some());
    assert!(og.find_edge(d, a).is_some());
    assert!(og.find_edge(a, a).is_some());

    for edge in og.raw_edges().iter() {
        assert!(og.find_edge(edge.source(), edge.target()).is_some());
        assert!(og.find_edge(edge.target(), edge.source()).is_some());
    }

    assert_eq!(og.neighbors(b).collect::<Vec<_>>(), vec![a, c, a]);

    og.remove_node(a);
    assert_eq!(og.neighbors(b).collect::<Vec<_>>(), vec![c]);
    assert_eq!(og.node_count(), 3);
    assert_eq!(og.edge_count(), 1);
    assert!(og.find_edge(a, b).is_none());
    assert!(og.find_edge(d, a).is_none());
    assert!(og.find_edge(a, a).is_none());
    assert!(og.find_edge(b, c).is_some());

}

#[test]
fn dfs() {
    let mut gr = Graph::new();
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

    assert_eq!(DfsIter::new(&Reversed(&gr), h).count(), 1);

    assert_eq!(DfsIter::new(&Reversed(&gr), k).count(), 3);

    assert_eq!(DfsIter::new(&gr, i).count(), 3);

    assert_eq!(DfsIter::new(&AsUndirected(&gr), i).count(), 4);
    */

}


#[test]
fn bfs() {
    let mut gr = Graph::new();
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
    assert_eq!(BfsIter::new(&gr, h).count(), 4);

    assert_eq!(BfsIter::new(&Reversed(&gr), h).count(), 1);

    assert_eq!(BfsIter::new(&Reversed(&gr), k).count(), 3);

    assert_eq!(BfsIter::new(&gr, i).count(), 3);

    assert_eq!(BfsIter::new(&AsUndirected(&gr), i).count(), 4);
    */

    let mut bfs = Bfs::new(&gr, h);
    let nx = bfs.next(&gr);
    assert_eq!(nx, Some(h));

    let nx1 = bfs.next(&gr);
    assert!(nx1 == Some(i) || nx1 == Some(j));

    let nx2 = bfs.next(&gr);
    assert!(nx2 == Some(i) || nx2 == Some(j));
    assert!(nx1 != nx2);

    let nx = bfs.next(&gr);
    assert_eq!(nx, Some(k));
    assert_eq!(bfs.next(&gr), None);
}



#[test]
fn mst() {
    let mut gr = Graph::<_,_>::new();
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

    let mst = min_spanning_tree(&gr);
    println!("MST is:\n{:?}", mst);
    assert!(mst.node_count() == gr.node_count());
    // |E| = |N| - 2  because there are two disconnected components.
    assert!(mst.edge_count() == gr.node_count() - 2);

    // check the exact edges are there
    assert!(mst.find_edge(a, b).is_some());
    assert!(mst.find_edge(a, d).is_some());
    assert!(mst.find_edge(b, e).is_some());
    assert!(mst.find_edge(e, c).is_some());
    assert!(mst.find_edge(e, g).is_some());
    assert!(mst.find_edge(d, f).is_some());

    assert!(mst.find_edge(h, i).is_some());
    assert!(mst.find_edge(i, j).is_some());
    
    assert!(mst.find_edge(d, b).is_none());
    assert!(mst.find_edge(b, c).is_none());

}

#[test]
fn selfloop() {
    let mut gr = Graph::new();
    let a = gr.add_node("A");
    let b = gr.add_node("B");
    let c = gr.add_node("C");
    gr.add_edge(a, b, 7.);
    gr.add_edge(c, a, 6.);
    let sed = gr.add_edge(a, a, 2.);

    assert!(gr.find_edge(a, b).is_some());
    assert!(gr.find_edge(b, a).is_none());
    assert!(gr.find_edge_undirected(b, a).is_some());
    assert!(gr.find_edge(a, a).is_some());
    println!("{:?}", gr);

    gr.remove_edge(sed);
    assert!(gr.find_edge(a, a).is_none());
    println!("{:?}", gr);
}

#[test]
fn cyclic() {
    let mut gr = Graph::new();
    let a = gr.add_node("A");
    let b = gr.add_node("B");
    let c = gr.add_node("C");

    assert!(!is_cyclic_undirected(&gr));
    gr.add_edge(a, b, 7.);
    gr.add_edge(c, a, 6.);
    assert!(!is_cyclic_undirected(&gr));
    {
        let e = gr.add_edge(a, a, 0.);
        assert!(is_cyclic_undirected(&gr));
        gr.remove_edge(e);
        assert!(!is_cyclic_undirected(&gr));
    }

    {
        let e = gr.add_edge(b, c, 0.);
        assert!(is_cyclic_undirected(&gr));
        gr.remove_edge(e);
        assert!(!is_cyclic_undirected(&gr));
    }

    let d = gr.add_node("D");
    let e = gr.add_node("E");
    gr.add_edge(b, d, 0.);
    gr.add_edge(d, e, 0.);
    assert!(!is_cyclic_undirected(&gr));
    gr.add_edge(c, e, 0.);
    assert!(is_cyclic_undirected(&gr));
}

#[test]
fn multi() {
    let mut gr = Graph::new();
    let a = gr.add_node("a");
    let b = gr.add_node("b");
    gr.add_edge(a, b, ());
    gr.add_edge(a, b, ());
    assert_eq!(gr.edge_count(), 2);

}
#[test]
fn update_edge()
{
    {
        let mut gr = Graph::new();
        let a = gr.add_node("a");
        let b = gr.add_node("b");
        let e = gr.update_edge(a, b, 1);
        let f = gr.update_edge(a, b, 2);
        let _ = gr.update_edge(b, a, 3);
        assert_eq!(gr.edge_count(), 2);
        assert_eq!(e, f);
        assert_eq!(*gr.edge_weight(f).unwrap(), 2);
    }

    {
        let mut gr = Graph::new_undirected();
        let a = gr.add_node("a");
        let b = gr.add_node("b");
        let e = gr.update_edge(a, b, 1);
        let f = gr.update_edge(b, a, 2);
        assert_eq!(gr.edge_count(), 1);
        assert_eq!(e, f);
        assert_eq!(*gr.edge_weight(f).unwrap(), 2);
    }
}

#[test]
fn dijk() {
    let mut g = Graph::new_undirected();
    let a = g.add_node("A");
    let b = g.add_node("B");
    let c = g.add_node("C");
    let d = g.add_node("D");
    let e = g.add_node("E");
    let f = g.add_node("F");
    g.add_edge(a, b, 7);
    g.add_edge(c, a, 9);
    g.add_edge(a, d, 14);
    g.add_edge(b, c, 10);
    g.add_edge(d, c, 2);
    g.add_edge(d, e, 9);
    g.add_edge(b, f, 15);
    g.add_edge(c, f, 11);
    g.add_edge(e, f, 6);
    println!("{:?}", g);
    /*
    for no in BfsIter::new(&g, a) {
        println!("Visit {:?} = {:?}", no, g.node_weight(no));
    }
    */

    let scores = dijkstra(&g, a, None, |gr, n| gr.edges(n).map(|(n, &e)| (n, e)));
    let mut scores: Vec<_> = scores.into_iter().map(|(n, s)| (g[n], s)).collect();
    scores.sort();
    assert_eq!(scores,
       vec![("A", 0), ("B", 7), ("C", 9), ("D", 11), ("E", 20), ("F", 20)]);

    let scores = dijkstra(&g, a, Some(c), |gr, n| gr.edges(n).map(|(n, &e)| (n, e)));
    assert_eq!(scores[c], 9);
}

#[test]
fn without()
{
    let mut og = Graph::new_undirected();
    let a = og.add_node(0);
    let b = og.add_node(1);
    let c = og.add_node(2);
    let d = og.add_node(3);
    let _ = og.add_edge(a, b, 0);
    let _ = og.add_edge(a, c, 1);
    let v: Vec<NodeIndex> = og.without_edges(Outgoing).collect();
    assert_eq!(v, vec![d]);

    let mut og = Graph::new();
    let a = og.add_node(0);
    let b = og.add_node(1);
    let c = og.add_node(2);
    let d = og.add_node(3);
    let _ = og.add_edge(a, b, 0);
    let _ = og.add_edge(a, c, 1);
    let init: Vec<NodeIndex> = og.without_edges(Incoming).collect();
    let term: Vec<NodeIndex> = og.without_edges(Outgoing).collect();
    assert_eq!(init, vec![a, d]);
    assert_eq!(term, vec![b, c, d]);
}


#[test]
fn toposort() {
    let mut gr = Graph::<_,_>::new();
    let a = gr.add_node("A");
    let b = gr.add_node("B");
    let c = gr.add_node("C");
    let d = gr.add_node("D");
    let e = gr.add_node("E");
    let f = gr.add_node("F");
    let g = gr.add_node("G");
    gr.add_edge(a, b, 7.0);
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

    let order = petgraph::algo::toposort(&gr);
    println!("{:?}", order);
    assert_eq!(order.len(), gr.node_count());

    // check all the edges of the graph
    for edge in gr.raw_edges().iter() {
        let a = edge.source();
        let b = edge.target();
        let ai = order.iter().position(|x| *x == a);
        let bi = order.iter().position(|x| *x == b);
        println!("Check that {:?} is before {:?}", a, b);
        assert!(ai < bi);
    }
}

#[test]
fn is_cyclic_directed() {
    let mut gr = Graph::<_,_>::new();
    let a = gr.add_node("A");
    let b = gr.add_node("B");
    let c = gr.add_node("C");
    let d = gr.add_node("D");
    let e = gr.add_node("E");
    let f = gr.add_node("F");
    let g = gr.add_node("G");
    gr.add_edge(a, b, 7.0);
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

    assert!(!petgraph::algo::is_cyclic_directed(&gr));

    // add a disjoint part
    let h = gr.add_node("H");
    let i = gr.add_node("I");
    let j = gr.add_node("J");
    gr.add_edge(h, i, 1.);
    gr.add_edge(h, j, 3.);
    gr.add_edge(i, j, 1.);
    assert!(!petgraph::algo::is_cyclic_directed(&gr));

    gr.add_edge(g, e, 0.);
    assert!(petgraph::algo::is_cyclic_directed(&gr));
}

#[test]
fn scc() {
    let n = NodeIndex::new;
    let mut gr = Graph::new();
    gr.add_node(0);
    gr.add_node(1);
    gr.add_node(2);
    gr.add_node(3);
    gr.add_node(4);
    gr.add_node(5);
    gr.add_node(6);
    gr.add_node(7);
    gr.add_node(8);
    gr.add_edge(n(6), n(0), ());
    gr.add_edge(n(0), n(3), ());
    gr.add_edge(n(3), n(6), ());
    gr.add_edge(n(8), n(6), ());
    gr.add_edge(n(8), n(2), ());
    gr.add_edge(n(2), n(5), ());
    gr.add_edge(n(5), n(8), ());
    gr.add_edge(n(7), n(5), ());
    gr.add_edge(n(1), n(7), ());
    gr.add_edge(n(7), n(4), ());
    gr.add_edge(n(4), n(1), ());

    let mut sccs = petgraph::algo::scc(&gr);
    assert_eq!(sccs.iter().map(|v| v.len()).sum(), gr.node_count());

    let scc_answer = vec![
        vec![n(0), n(3), n(6)],
        vec![n(1), n(4), n(7)],
        vec![n(2), n(5), n(8)]];

    // normalize the result and compare with the answer.
    for sc in sccs.iter_mut() {
        sc.sort();
    }
    // sort by minimum element
    sccs.sort_by(|v, w| v[0].cmp(&w[0]));
    assert_eq!(sccs, scc_answer);

    // Test an undirected graph just for fun.
    // Sccs are just connected components.
    let mut hr = gr.into_edge_type::<Undirected>();
    // Delete an edge to disconnect it
    let ed = hr.find_edge(n(6), n(8)).unwrap();
    assert!(hr.remove_edge(ed).is_some());

    let mut sccs = petgraph::algo::scc(&hr);

    let scc_undir_answer = vec![
        vec![n(0), n(3), n(6)],
        vec![n(1), n(2), n(4), n(5), n(7), n(8)]];

    for sc in sccs.iter_mut() {
        sc.sort();
        sc.dedup();
    }
    sccs.sort_by(|v, w| v[0].cmp(&w[0]));
    assert_eq!(sccs, scc_undir_answer);
}

#[test]
fn connected_comp()
{
    let n = NodeIndex::new;
    let mut gr = Graph::new();
    gr.add_node(0);
    gr.add_node(1);
    gr.add_node(2);
    gr.add_node(3);
    gr.add_node(4);
    gr.add_node(5);
    gr.add_node(6);
    gr.add_node(7);
    gr.add_node(8);
    gr.add_edge(n(6), n(0), ());
    gr.add_edge(n(0), n(3), ());
    gr.add_edge(n(3), n(6), ());
    gr.add_edge(n(8), n(6), ());
    gr.add_edge(n(8), n(2), ());
    gr.add_edge(n(2), n(5), ());
    gr.add_edge(n(5), n(8), ());
    gr.add_edge(n(7), n(5), ());
    gr.add_edge(n(1), n(7), ());
    gr.add_edge(n(7), n(4), ());
    gr.add_edge(n(4), n(1), ());
    assert_eq!(petgraph::algo::connected_components(&gr), 1);

    gr.add_node(9);
    gr.add_node(10);
    assert_eq!(petgraph::algo::connected_components(&gr), 3);

    gr.add_edge(n(9), n(10), ());
    assert_eq!(petgraph::algo::connected_components(&gr), 2);

    let gr = gr.into_edge_type::<Undirected>();
    assert_eq!(petgraph::algo::connected_components(&gr), 2);
}

#[should_panic]
#[test]
fn oob_index()
{
    let mut gr = Graph::<_, ()>::new();
    let a = gr.add_node(0);
    let b = gr.add_node(1);
    gr.remove_node(a);
    gr[b];
}

#[test]
fn usize_index()
{
    let mut gr = Graph::<_, _, Directed, usize>::with_capacity(0, 0);
    let a = gr.add_node(0);
    let b = gr.add_node(1);
    let e = gr.add_edge(a, b, 1.2);
    let mut dfs = Dfs::new(&gr, a);
    while let Some(nx) = dfs.next(&gr) {
        gr[nx] += 1;
    }
    assert_eq!(gr[a], 1);
    assert_eq!(gr[b], 2);
    assert_eq!(gr[e], 1.2);
}
