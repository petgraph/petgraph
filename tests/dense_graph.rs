#![cfg(feature = "dense_graph")]

extern crate petgraph;

use petgraph::dense_graph::*;

#[test]
fn test_new() {
    DenseGraph::<i32, i32>::new();
}

#[test]
fn test_default() {
    DenseGraph::<i32, i32>::default();
}

#[test]
fn test_with_capacity() {
    DenseGraph::<i32, i32>::with_capacity(10, 5);
}

#[test]
fn test_add_node() {
    let mut g: DenseGraph<char, ()> = DenseGraph::new();
    g.add_node('a');
    g.add_node('b');
}

#[test]
fn test_remove_node() {
    let mut g: DenseGraph<char, ()> = DenseGraph::new();
    let a = g.add_node('a');
    g.remove_node(a);
}

#[test]
fn test_add_edge() {
    let mut g: DenseGraph<char, ()> = DenseGraph::new();
    let a = g.add_node('a');
    let b = g.add_node('b');
    let c = g.add_node('c');
    g.add_edge(a, b, ());
    g.add_edge(b, c, ());
}

#[test]
fn test_add_edge_with_weights() {
    let mut g: DenseGraph<char, bool> = DenseGraph::new();
    let a = g.add_node('a');
    let b = g.add_node('b');
    let c = g.add_node('c');
    g.add_edge(a, b, true);
    g.add_edge(b, c, false);
}

#[test]
fn test_node_indexing() {
    let mut g: DenseGraph<char, ()> = DenseGraph::new();
    let a = g.add_node('a');
    let b = g.add_node('b');
    assert_eq!(g[a], 'a');
    assert_eq!(g[b], 'b');
}

#[test]
fn test_edge_indexing() {
    let mut g: DenseGraph<char, bool> = DenseGraph::new();
    let a = g.add_node('a');
    let b = g.add_node('b');
    let c = g.add_node('c');
    let ab = g.add_edge(a, b, true);
    let bc = g.add_edge(b, c, false);
    assert_eq!(g[ab], true);
    assert_eq!(g[bc], false);
}

#[test]
fn test_neighbors() {
    let mut g: DenseGraph<char, ()> = DenseGraph::new();
    let a = g.add_node('a');
    let b = g.add_node('b');
    let c = g.add_node('c');
    let _ = g.add_edge(a, b, ());
    let _ = g.add_edge(a, c, ());

    let a_neighbors: Vec<_> = g.neighbors(a).collect();
    assert_eq!(a_neighbors, vec![b, c]);

    let b_neighbors: Vec<_> = g.neighbors(b).collect();
    assert_eq!(b_neighbors, vec![]);

    let c_neighbors: Vec<_> = g.neighbors(c).collect();
    assert_eq!(c_neighbors, vec![]);
}

#[test]
fn test_neighbors_undirected() {
    let mut g: DenseUnGraph<char, ()> = DenseGraph::new_undirected();
    let a = g.add_node('a');
    let b = g.add_node('b');
    let c = g.add_node('c');
    let _ = g.add_edge(a, b, ());
    let _ = g.add_edge(a, c, ());

    let a_neighbors: Vec<_> = g.neighbors(a).collect();
    assert_eq!(a_neighbors, vec![b, c]);

    let b_neighbors: Vec<_> = g.neighbors(b).collect();
    assert_eq!(b_neighbors, vec![a]);

    let c_neighbors: Vec<_> = g.neighbors(c).collect();
    assert_eq!(c_neighbors, vec![a]);
}

#[test]
fn test_from_edges() {
    let _: DenseGraph<char, bool> = DenseGraph::from_edges(&[
        (0, 5), (0, 2), (0, 3), (0, 1),
        (1, 3),
        (2, 3), (2, 4),
        (4, 0), (4, 5),
    ]);
}

#[test]
fn test_edges() {
    let g: DenseGraph<char, bool> = DenseGraph::from_edges(&[
        (0, 5), (0, 2), (0, 3), (0, 1),
        (1, 3),
        (2, 3), (2, 4),
        (4, 0), (4, 5),
    ]);

    use std::collections::HashSet;
    let edges: HashSet<EdgeIndex<DefaultIx>> = g.edges(node_index(0)).collect();
    assert_eq!(edges.len(), 4);
}
