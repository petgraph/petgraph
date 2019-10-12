#![feature(test)]
extern crate test;

extern crate petgraph;

use std::collections::HashMap;

use petgraph::prelude::*;
use petgraph::EdgeType;
use petgraph::algo::pathfinding::{Dijkstra, IndexableNodeMap};

// replicate chain: a -> ... -> b
fn add_chains<Ty: EdgeType>(g: &mut Graph<(), i32, Ty>,
                            a: NodeIndex, b: NodeIndex,
                            nb_chains: usize, chain_length: usize)
{
    for i in 0..nb_chains {
        let mut prev = a;

        for _ in 0..chain_length {
            let new = g.add_node(());
            g.add_edge(prev, new, 1);
        }

        // first chain's last edge have a slightly lower weight
        g.add_edge(prev, b, if i == 0 { 1 } else { 2 });
    }
}

fn sample_graph<Ty: EdgeType>() -> (Graph<(), i32, Ty>, NodeIndex, NodeIndex) {
    let nb_chains = 50;
    let chain_length = 10;

    let nb_nodes = chain_length * nb_chains + 2;
    let nb_edges = ((chain_length - 1) + 2) * nb_chains;
    let mut g = Graph::with_capacity(nb_nodes, nb_edges);

    let a = g.add_node(());
    let b = g.add_node(());

    add_chains(&mut g, a, b, nb_chains, chain_length);

    (g, a, b)
}

#[allow(dead_code)]
fn sample_digraph() -> (Graph<(), i32, Directed>, NodeIndex, NodeIndex) {
    sample_graph::<Directed>()
}

#[allow(dead_code)]
fn sample_ungraph() -> (Graph<(), i32, Undirected>, NodeIndex, NodeIndex) {
    sample_graph::<Undirected>()
}

#[bench]
fn dijkstra(bench: &mut test::Bencher) {
    let (g, a, b) = sample_digraph();

    bench.iter(|| {
        // equivalent to
        //let _ = dijkstra(&g, a, Some(b), |e| *e.weight())
        let _ = Dijkstra::new(&g)
            .cost_map(HashMap::new())
            .predecessor_map(HashMap::new())
            .path(a, b)
            .into_costs();
    });
}

#[bench]
fn dijkstra_no_predecessors(bench: &mut test::Bencher) {
    let (g, a, b) = sample_digraph();

    bench.iter(|| {
        let _ = Dijkstra::new(&g)
            .cost_map(HashMap::new())
            .path(a, b)
            .into_costs();
    });
}

#[bench]
fn dijkstra_indexable_node_map(bench: &mut test::Bencher) {
    let (g, a, b) = sample_digraph();

    bench.iter(|| {
        let _ = Dijkstra::new(&g)
            .cost_map(IndexableNodeMap::<_, Option<_>>::new())
            .predecessor_map(IndexableNodeMap::new())
            .path(a, b)
            .into_costs();
    });
}
