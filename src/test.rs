use std::ops::Index;
use crate::algo::steiner_tree::{compute_metric_closure, compute_shortest_path, minimal_steiner_tree};
use crate::dot::{Config, Dot};
use crate::graph::UnGraph;
use crate::visit::EdgeIndexable;

#[test]
fn main() {
    // Create a new undirected graph
    let mut graph = UnGraph::<&str, u32>::new_undirected();

    // Add nodes
    let a = graph.add_node("A");
    let b = graph.add_node("B");
    let c = graph.add_node("C");

    // Add edges
    graph.add_edge(a, b, 0);
    graph.add_edge(b, c, 0);
    graph.add_edge(c, a, 3);


    let metric = minimal_steiner_tree(&graph, vec![a, b, c]);

    //let mst = _kou_steiner_tree(g, terminals);

    println!("{:?}", Dot::with_config(&graph, &[]));
    println!("{:?}", Dot::with_config(&metric, &[]));
}
