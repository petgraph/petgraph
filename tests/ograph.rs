extern crate petgraph;

use petgraph::{
    Graph,
    Bfs,
    BfsIter,
    Dfs,
    DfsIter,
    Incoming,
    Outgoing,
    Directed,
    Undirected,
    EdgeType,
};

use petgraph as pg;

use petgraph::algo::{
    min_spanning_tree,
    is_cyclic_undirected,
};

use petgraph::graph::node_index as n;
use petgraph::graph::{
    NodeIndex,
    EdgeIndex,
    IndexType,
};

use petgraph::visit::{
    Reversed,
    AsUndirected,
    Topo,
};
use petgraph::algo::{
    dijkstra,
};

use petgraph::dot::{
    Dot,
};

use petgraph::visit::GetAdjacencyMatrix;

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

    for edge in og.raw_edges() {
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
    assert_graph_consistent(&og);

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

    println!("{}", Dot::new(&gr));

    assert_eq!(DfsIter::new(&gr, h).count(), 4);
    assert_eq!(DfsIter::new(&gr, h).clone().count(), 4);

    assert_eq!(DfsIter::new(&Reversed(&gr), h).count(), 1);

    assert_eq!(DfsIter::new(&Reversed(&gr), k).count(), 3);

    assert_eq!(DfsIter::new(&gr, i).count(), 3);

    assert_eq!(DfsIter::new(&AsUndirected(&gr), i).count(), 4);
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

    assert_eq!(BfsIter::new(&gr, h).count(), 4);
    assert_eq!(BfsIter::new(&gr, h).clone().count(), 4);

    assert_eq!(BfsIter::new(&Reversed(&gr), h).count(), 1);

    assert_eq!(BfsIter::new(&Reversed(&gr), k).count(), 3);

    assert_eq!(BfsIter::new(&gr, i).count(), 3);

    assert_eq!(BfsIter::new(&AsUndirected(&gr), i).count(), 4);

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

    println!("{}", Dot::new(&gr));

    let mst = min_spanning_tree(&gr);
    println!("{}", Dot::new(&mst));
    println!("{:?}", Dot::new(&mst));
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
    assert_graph_consistent(&gr);
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
    for no in BfsIter::new(&g, a) {
        println!("Visit {:?} = {:?}", no, g.node_weight(no));
    }

    let scores = dijkstra(&g, a, None, |gr, n| gr.edges(n).map(|(n, &e)| (n, e)));
    let mut scores: Vec<_> = scores.into_iter().map(|(n, s)| (g[n], s)).collect();
    scores.sort();
    assert_eq!(scores,
       vec![("A", 0), ("B", 7), ("C", 9), ("D", 11), ("E", 20), ("F", 20)]);

    let scores = dijkstra(&g, a, Some(c), |gr, n| gr.edges(n).map(|(n, &e)| (n, e)));
    assert_eq!(scores[&c], 9);
}

#[test]
fn test_generate_undirected() {
    for size in 0..4 {
        let mut gen = pg::generate::Generator::<Undirected>::all(size, true);
        let nedges = (size * size - size) / 2 + size;
        let mut n = 0;
        while let Some(g) = gen.next_ref() {
            n += 1;
            println!("{:?}", g);
        }

        assert_eq!(n, 1 << nedges);
    }
}

#[test]
fn test_generate_directed() {
    // Number of DAG out of all graphs (all permutations) per node size
    //            0, 1, 2, 3,  4,   5 .. 
    let n_dag = &[1, 1, 3, 25, 543, 29281, 3781503];
    for (size, &dags_exp) in (0..4).zip(n_dag) {
        let mut gen = pg::generate::Generator::<Directed>::all(size, true);
        let nedges = size * size;
        let mut n = 0;
        let mut dags = 0;
        while let Some(g) = gen.next_ref() {
            n += 1;
            if !pg::algo::is_cyclic_directed(g) {
                dags += 1;
            }
        }

        /*
        // check that all generated graphs have unique adjacency matrices
        let mut adjmats = graphs.iter().map(Graph::adjacency_matrix).collect::<Vec<_>>();
        adjmats.sort(); adjmats.dedup();
        assert_eq!(adjmats.len(), n);
        */
        assert_eq!(dags_exp, dags);
        assert_eq!(n, 1 << nedges);
    }
}
#[test]
fn test_generate_dag() {
    for size in 1..5 {
        let gen = pg::generate::Generator::directed_acyclic(size);
        let nedges = (size - 1) * size / 2;
        let graphs = gen.collect::<Vec<_>>();

        assert_eq!(graphs.len(), 1 << nedges);

        // check that all generated graphs have unique adjacency matrices
        let mut adjmats = graphs.iter().map(Graph::adjacency_matrix).collect::<Vec<_>>();
        adjmats.sort();
        adjmats.dedup();
        assert_eq!(adjmats.len(), graphs.len());
        for gr in &graphs {
            assert!(!petgraph::algo::is_cyclic_directed(gr),
                    "Assertion failed: {:?} acyclic", gr);
        }
    }
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

fn assert_is_topo_order<N, E>(gr: &Graph<N, E, Directed>, order: &[NodeIndex])
{
    assert_eq!(gr.node_count(), order.len());
    // check all the edges of the graph
    for edge in gr.raw_edges() {
        let a = edge.source();
        let b = edge.target();
        let ai = order.iter().position(|x| *x == a).unwrap();
        let bi = order.iter().position(|x| *x == b).unwrap();
        println!("Check that {:?} is before {:?}", a, b);
        assert!(ai < bi, "Topo order: assertion that node {:?} is before {:?} failed",
                a, b);
    }
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
    gr.extend_with_edges(&[
        (a, b, 7.),
        (a, d, 5.),
        (d, b, 9.),
        (b, c, 8.),
        (b, e, 7.),
        (c, e, 5.),
        (d, e, 15.),
        (d, f, 6.),
        (f, e, 8.),
        (f, g, 11.),
        (e, g, 9.),
    ]);

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

    assert_is_topo_order(&gr, &order);
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
    fn assert_sccs_eq(mut res: Vec<Vec<NodeIndex>>, normalized: Vec<Vec<NodeIndex>>) {
        // normalize the result and compare with the answer.
        for scc in res.iter_mut() {
            scc.sort();
        }
        // sort by minimum element
        res.sort_by(|v, w| v[0].cmp(&w[0]));
        assert_eq!(res, normalized);
    }

    let gr: Graph<(), ()> = Graph::from_edges(&[
        (6, 0),
        (0, 3),
        (3, 6),
        (8, 6),
        (8, 2),
        (2, 5),
        (5, 8),
        (7, 5),
        (1, 7),
        (7, 4),
        (4, 1)]);

    assert_sccs_eq(petgraph::algo::scc(&gr), vec![
        vec![n(0), n(3), n(6)],
        vec![n(1), n(4), n(7)],
        vec![n(2), n(5), n(8)],
    ]);


    // Test an undirected graph just for fun.
    // Sccs are just connected components.
    let mut hr = gr.into_edge_type::<Undirected>();
    // Delete an edge to disconnect it
    let ed = hr.find_edge(n(6), n(8)).unwrap();
    assert!(hr.remove_edge(ed).is_some());

    assert_sccs_eq(petgraph::algo::scc(&hr), vec![
        vec![n(0), n(3), n(6)],
        vec![n(1), n(2), n(4), n(5), n(7), n(8)],
    ]);


    // acyclic non-tree, #14
    let n = NodeIndex::new;
    let mut gr = Graph::new();
    gr.add_node(0);
    gr.add_node(1);
    gr.add_node(2);
    gr.add_node(3);
    gr.add_edge(n(3), n(2), ());
    gr.add_edge(n(3), n(1), ());
    gr.add_edge(n(2), n(0), ());
    gr.add_edge(n(1), n(0), ());

    assert_sccs_eq(petgraph::algo::scc(&gr), vec![
        vec![n(0)], vec![n(1)], vec![n(2)], vec![n(3)],
    ]);
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

#[test]
fn u8_index()
{
    let mut gr = Graph::<_, (), Undirected, u8>::with_capacity(0, 0);
    for _ in 0..255 {
        gr.add_node(());
    }
}

#[should_panic]
#[test]
fn u8_index_overflow()
{
    let mut gr = Graph::<_, (), Undirected, u8>::with_capacity(0, 0);
    for _ in 0..256 {
        gr.add_node(());
    }
}

#[should_panic]
#[test]
fn u8_index_overflow_edges()
{
    let mut gr = Graph::<_, (), Undirected, u8>::with_capacity(0, 0);
    let a = gr.add_node('a');
    let b = gr.add_node('b');
    for _ in 0..256 {
        gr.add_edge(a, b, ());
    }
}

#[test]
fn test_weight_iterators() {
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

    assert_eq!(gr.node_weights_mut().count(), gr.node_count());
    assert_eq!(gr.edge_weights_mut().count(), gr.edge_count());

    // add a disjoint part
    let h = gr.add_node("H");
    let i = gr.add_node("I");
    let j = gr.add_node("J");
    gr.add_edge(h, i, 1.);
    gr.add_edge(h, j, 3.);
    gr.add_edge(i, j, 1.);

    assert_eq!(gr.node_weights_mut().count(), gr.node_count());
    assert_eq!(gr.edge_weights_mut().count(), gr.edge_count());

    for nw in gr.node_weights_mut() {
        *nw = "x";
    }
    for node in gr.raw_nodes() {
        assert_eq!(node.weight, "x");
    }

    let old = gr.clone();
    for (index, ew) in gr.edge_weights_mut().enumerate() {
        assert_eq!(old[EdgeIndex::new(index)], *ew);
        *ew = - *ew;
    }
    for (index, edge) in gr.raw_edges().iter().enumerate() {
        assert_eq!(edge.weight, -1. * old[EdgeIndex::new(index)]);
    }
}

#[test]
fn walk_edges() {
    let mut gr = Graph::<_,_>::new();
    let a = gr.add_node(0.);
    let b = gr.add_node(1.);
    let c = gr.add_node(2.);
    let d = gr.add_node(3.);
    let e0 = gr.add_edge(a, b, 0.);
    let e1 = gr.add_edge(a, d, 0.);
    let e2 = gr.add_edge(b, c, 0.);
    let e3 = gr.add_edge(c, a, 0.);

    // Set edge weights to difference: target - source.
    let mut dfs = Dfs::new(&gr, a);
    while let Some(source) = dfs.next(&gr) {
        let mut edges = gr.walk_edges_directed(source, Outgoing);
        while let Some((edge, target)) = edges.next_neighbor(&gr) {
            gr[edge] = gr[target] - gr[source];
        }
    }
    assert_eq!(gr[e0], 1.);
    assert_eq!(gr[e1], 3.);
    assert_eq!(gr[e2], 1.);
    assert_eq!(gr[e3], -2.);

    let mut nedges = 0;
    let mut dfs = Dfs::new(&gr, a);
    while let Some(node) = dfs.next(&gr) {
        let mut edges = gr.walk_edges_directed(node, Incoming);
        while let Some((edge, source)) = edges.next_neighbor(&gr) {
            assert_eq!(gr.find_edge(source, node), Some(edge));
            nedges += 1;
        }

        let mut edges = gr.walk_edges_directed(node, Outgoing);
        while let Some((edge, target)) = edges.next_neighbor(&gr) {
            assert_eq!(gr.find_edge(node, target), Some(edge));
            nedges += 1;
        }
    }
    assert_eq!(nedges, 8);
}

#[test]
fn index_twice_mut() {
    let mut gr = Graph::<_,_>::new();
    let a = gr.add_node(0.);
    let b = gr.add_node(0.);
    let c = gr.add_node(0.);
    let d = gr.add_node(0.);
    let e = gr.add_node(0.);
    let f = gr.add_node(0.);
    let g = gr.add_node(0.);
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

    for dir in &[Incoming, Outgoing] {
        for nw in gr.node_weights_mut() { *nw = 0.; }

        // walk the graph and sum incoming edges
        let mut dfs = Dfs::new(&gr, a);
        while let Some(node) = dfs.next(&gr) {
            let mut edges = gr.walk_edges_directed(node, *dir);
            while let Some(edge) = edges.next(&gr) {
                let (nw, ew) = gr.index_twice_mut(node, edge);
                *nw += *ew;
            }
        }

        // check the sums
        for i in 0..gr.node_count() {
            let ni = NodeIndex::new(i);
            let s = gr.edges_directed(ni, *dir).map(|(_, &ew)| ew).fold(0., |a, b| a + b);
            assert_eq!(s, gr[ni]);
        }
        println!("Sum {:?}: {:?}", dir, gr);
    }
}

#[test]
fn toposort_generic() {
    // This is a DAG, visit it in order
    let mut gr = Graph::<_,_>::new();
    let b = gr.add_node(("B", 0.));
    let a = gr.add_node(("A", 0.));
    let c = gr.add_node(("C", 0.));
    let d = gr.add_node(("D", 0.));
    let e = gr.add_node(("E", 0.));
    let f = gr.add_node(("F", 0.));
    let g = gr.add_node(("G", 0.));
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

    assert!(!pg::algo::is_cyclic_directed(&gr));
    let mut index = 0.;
    let mut topo = Topo::new(&gr);
    while let Some(nx) = topo.next(&gr) {
        gr[nx].1 = index;
        index += 1.;
    }

    let mut order = Vec::new();
    index = 0.;
    let mut topo = Topo::new(&gr);
    while let Some(nx) = topo.next(&gr) {
        order.push(nx);
        assert_eq!(gr[nx].1, index);
        index += 1.;
    }
    println!("{:?}", gr);
    assert_is_topo_order(&gr, &order);

    {
        order.clear();
        let mut topo = Topo::new(&gr);
        while let Some(nx) = topo.next(&gr) {
            order.push(nx);
        }
        println!("{:?}", gr);
        assert_is_topo_order(&gr, &order);
    }
}

#[test]
fn map_filter_map() {
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

    let g2 = g.filter_map(|_, name| Some(*name), |_, &weight| if weight >= 10 {
        Some(weight)
    } else { None });
    assert_eq!(g2.edge_count(), 4);
    for edge in g2.raw_edges() {
        assert!(edge.weight >= 10);
    }

    let g3 = g.filter_map(|i, &name| if i == a || i == e { None } else { Some(name) },
                          |i, &weight| {
                              let (source, target) = g.edge_endpoints(i).unwrap();
                              // don't map edges from a removed node
                              assert!(source != a);
                              assert!(target != a);
                              assert!(source != e);
                              assert!(target != e);
                              Some(weight)
                          });
    assert_eq!(g3.node_count(), g.node_count() - 2);
    assert_eq!(g3.edge_count(), g.edge_count() - 5);
    assert_graph_consistent(&g3);

    let mut g4 = g.clone();
    g4.retain_edges(|gr, i| {
        let (s, t) = gr.edge_endpoints(i).unwrap();
        !(s == a || s == e || t == a || t == e)
    });
    assert_eq!(g4.edge_count(), g.edge_count() - 5);
    assert_graph_consistent(&g4);
}

#[test]
fn from_edges() {
    let n = NodeIndex::new;
    let gr = Graph::<(), (), Undirected>::from_edges(&[
        (0, 1), (0, 2), (0, 3),
        (1, 2), (1, 3),
        (2, 3),
    ]);
    assert_eq!(gr.node_count(), 4);
    assert_eq!(gr.edge_count(), 6);
    assert_eq!(gr.neighbors(n(0)).count(), 3);
    assert_eq!(gr.neighbors(n(1)).count(), 3);
    assert_eq!(gr.neighbors(n(2)).count(), 3);
    assert_eq!(gr.neighbors(n(3)).count(), 3);
    assert_graph_consistent(&gr);
}

#[test]
fn retain() {
    let mut gr = Graph::<i32, i32, Undirected>::from_edges(&[
        (0, 1, 2),
        (1, 1, 1),
        (0, 2, 0),
        (1, 2, 3),
        (2, 3, 3),
    ]);
    gr.retain_edges(|gr, i| {
        if gr[i] <= 0 { false }
        else {
            gr[i] -= 1;
            let (s, t) = gr.edge_endpoints(i).unwrap();
            gr[s] += 1;
            gr[t] += 1;

            gr[i] > 0
        }
    });

    assert_eq!(gr.edge_count(), 3);
    assert_eq!(gr[n(0)], 1);
    assert_eq!(gr[n(1)], 4);
    assert_eq!(gr[n(2)], 2);
    assert_eq!(gr[n(3)], 1);
    assert!(gr.find_edge(n(1), n(1)).is_none());
    assert!(gr.find_edge(n(0), n(2)).is_none());
    assert_graph_consistent(&gr);
}

fn assert_graph_consistent<N, E, Ty, Ix>(g: &Graph<N, E, Ty, Ix>)
    where Ty: EdgeType,
          Ix: IndexType,
{
    assert_eq!(g.node_count(), g.node_indices().count());
    assert_eq!(g.edge_count(), g.edge_indices().count());
    for edge in g.raw_edges() {
        assert!(g.find_edge(edge.source(), edge.target()).is_some(),
                "Edge not in graph! {:?} to {:?}", edge.source(), edge.target());
    }
}
