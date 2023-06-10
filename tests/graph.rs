extern crate petgraph;

use std::{collections::HashSet, hash::Hash};

use petgraph as pg;
use petgraph::{
    algo::{
        astar, dijkstra, dominators, has_path_connecting, is_bipartite_undirected,
        is_cyclic_undirected, is_isomorphic_matching, min_spanning_tree, DfsSpace,
    },
    dot::Dot,
    graph::{node_index as n, IndexType},
    prelude::*,
    visit::{
        IntoEdges, IntoEdgesDirected, IntoNeighbors, IntoNodeIdentifiers, NodeFiltered, Reversed,
        Topo, VisitMap, Walker,
    },
    EdgeType,
};

fn set<I>(iter: I) -> HashSet<I::Item>
where
    I: IntoIterator,
    I::Item: Hash + Eq,
{
    iter.into_iter().collect()
}

fn assert_is_topo_order<N, E>(gr: &Graph<N, E, Directed>, order: &[NodeIndex]) {
    assert_eq!(gr.node_count(), order.len());
    // check all the edges of the graph
    for edge in gr.raw_edges() {
        let a = edge.source();
        let b = edge.target();
        let ai = order.iter().position(|x| *x == a).unwrap();
        let bi = order.iter().position(|x| *x == b).unwrap();
        println!("Check that {:?} is before {:?}", a, b);
        assert!(
            ai < bi,
            "Topo order: assertion that node {:?} is before {:?} failed",
            a,
            b
        );
    }
}

// TODO: move to core
#[test]
fn toposort_generic() {
    // This is a DAG, visit it in order
    let mut gr = Graph::<_, _>::new();
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
    let mut gr2 = gr.clone();
    gr.add_edge(e, d, -1.);
    assert!(pg::algo::is_cyclic_directed(&gr));
    assert!(pg::algo::toposort(&gr, None).is_err());
    gr2.add_edge(d, d, 0.);
    assert!(pg::algo::is_cyclic_directed(&gr2));
    assert!(pg::algo::toposort(&gr2, None).is_err());
}
