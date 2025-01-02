#![feature(test)]

extern crate petgraph;
extern crate test;

use petgraph::prelude::*;
use test::Bencher;

use petgraph::algo::bron_kerbosch;

#[bench]
fn bron_kerbosch_bench(bench: &mut Bencher) {
    static NODE_COUNT: usize = 1_000;
    static CLIQUE_SIZE: usize = 5;

    let mut g = UnGraph::new_undirected();
    let nodes: Vec<NodeIndex<_>> = (0..NODE_COUNT).map(|i| g.add_node(i)).collect();

    // Create overlapping cliques throughout the graph
    // Each node i will form a clique with the next CLIQUE_SIZE-1 nodes
    // Creates a graph with multiple overlapping cliques of size CLIQUE_SIZE
    for i in 0..NODE_COUNT {
        for j in i + 1..std::cmp::min(i + CLIQUE_SIZE, NODE_COUNT) {
            g.add_edge(nodes[i], nodes[j], ());
        }
    }

    bench.iter(|| {
        let _cliques = bron_kerbosch(&g).unwrap();
    });
}

#[bench]
fn bron_kerbosch_dense_bench(bench: &mut Bencher) {
    static NODE_COUNT: usize = 100;
    static EDGE_PROBABILITY: f64 = 0.3;

    let mut g = UnGraph::new_undirected();
    let nodes: Vec<NodeIndex<_>> = (0..NODE_COUNT).map(|i| g.add_node(i)).collect();

    // Add edges with probability EDGE_PROBABILITY
    for i in 0..NODE_COUNT {
        for j in i + 1..NODE_COUNT {
            if rand::random::<f64>() < EDGE_PROBABILITY {
                g.add_edge(nodes[i], nodes[j], ());
            }
        }
    }

    bench.iter(|| {
        let _cliques = bron_kerbosch(&g).unwrap();
    });
}

#[bench]
fn bron_kerbosch_worst_case_bench(bench: &mut Bencher) {
    // Test with Moon-Moser graphs, which represent worst-case scenarios
    // These graphs have 3^(n/3) maximal cliques
    static NODE_COUNT: usize = 30; // kept small due to exponential growth
    static PARTITION_SIZE: usize = NODE_COUNT / 3;

    let mut g = UnGraph::new_undirected();
    let nodes: Vec<NodeIndex<_>> = (0..NODE_COUNT).map(|i| g.add_node(i)).collect();

    // Create three independent sets
    for i in 0..3 {
        let start = i * PARTITION_SIZE;
        let end = (i + 1) * PARTITION_SIZE;

        // each vertex in this partition connects to all vertices in the next partition
        for v1 in start..end {
            for v2 in ((i + 1) % 3 * PARTITION_SIZE)..((i + 2) % 3 * PARTITION_SIZE) {
                g.add_edge(nodes[v1], nodes[v2], ());
            }
        }
    }

    bench.iter(|| {
        let _cliques = bron_kerbosch(&g).unwrap();
    });
}