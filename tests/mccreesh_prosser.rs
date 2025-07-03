extern crate petgraph;
use hashbrown::HashSet;
use petgraph::graph::{DiGraph, UnGraph};
use petgraph::{algo::maximal_cliques::largest_maximal_clique, graph::Graph, Undirected};

#[test]
fn test_largest_maximal_clique_empty_graph() {
    // empty graph should return empty set
    let g = Graph::<i32, ()>::new();
    let largest_clique = largest_maximal_clique(&g);
    let expected_clique = HashSet::new();
    assert_eq!(expected_clique, largest_clique);
}

#[test]
fn test_largest_maximal_clique_undirected_sparse_graph() {
    // c     d
    //
    // b --- a
    let mut g = UnGraph::<i32, ()>::new_undirected();

    let a = g.add_node(0);
    let b = g.add_node(1);

    g.extend_with_edges([(a, b), (b, a)]);

    let largest_clique = largest_maximal_clique(&g);

    // The largest clique should be {a, b} with size 2
    let expected_clique: HashSet<_> = vec![a, b].into_iter().collect();
    assert_eq!(expected_clique, largest_clique);
}

#[test]
fn test_largest_maximal_clique_directed_sparse_graph() {
    // c     d
    //
    // b <-> a
    let mut g = DiGraph::<i32, ()>::new();

    let a = g.add_node(0);
    let b = g.add_node(1);

    g.extend_with_edges([(a, b), (b, a)]);

    let largest_clique = largest_maximal_clique(&g);

    // The largest clique should be {a, b} with size 2
    let expected_clique: HashSet<_> = vec![a, b].into_iter().collect();
    assert_eq!(expected_clique, largest_clique);
}

#[test]
fn test_largest_maximal_clique_undirected() {
    // f - d - e - a
    //     |   | /
    //     c - b
    let mut g = Graph::<i32, (), Undirected>::new_undirected();
    let a = g.add_node(0);
    let b = g.add_node(1);
    let c = g.add_node(2);
    let d = g.add_node(3);
    let e = g.add_node(4);
    let f = g.add_node(5);
    g.extend_with_edges([(a, b), (a, e), (b, e), (b, c), (c, d), (d, e), (e, f)]);

    let largest_clique = largest_maximal_clique(&g);
    println!("Largest clique: {:?}", &largest_clique);

    // The largest clique should be {a, b, e} with size 3
    let expected_clique: HashSet<_> = vec![a, b, e].into_iter().collect();
    assert_eq!(expected_clique, largest_clique);
}

#[test]
fn test_largest_maximal_clique_directed() {
    // f <-> d <-> e <-> a
    //       ^     ^     ^
    //       |     |     |
    //       v     v     |
    //       c <-> b <---|
    let mut g = Graph::<i32, ()>::new();
    let a = g.add_node(0);
    let b = g.add_node(1);
    let c = g.add_node(2);
    let d = g.add_node(3);
    let e = g.add_node(4);
    let f = g.add_node(5);
    g.extend_with_edges([
        (a, b),
        (b, a),
        (a, e),
        (e, a),
        (b, e),
        (e, b),
        (b, c),
        (c, b),
        (c, d),
        (d, c),
        (d, e),
        (e, d),
        (d, f),
        (f, d),
    ]);

    let largest_clique = largest_maximal_clique(&g);
    println!("Largest clique: {:?}", &largest_clique);

    // The largest clique should be {a, b, e} with size 3
    let expected_clique: HashSet<_> = vec![a, b, e].into_iter().collect();
    assert_eq!(expected_clique, largest_clique);
}

#[test]
fn test_largest_maximal_clique_single_node() {
    // Test with a single isolated node
    let mut g = Graph::<i32, (), Undirected>::new_undirected();
    let a = g.add_node(0);

    let largest_clique = largest_maximal_clique(&g);

    // The largest (and only) clique should be {a}
    let expected_clique: HashSet<_> = vec![a].into_iter().collect();
    assert_eq!(expected_clique, largest_clique);
}

#[test]
fn test_largest_maximal_clique_multiple_isolated_nodes() {
    // Test with multiple isolated nodes
    let mut g = Graph::<i32, (), Undirected>::new_undirected();
    let a = g.add_node(0);
    let b = g.add_node(1);
    let c = g.add_node(2);

    let largest_clique = largest_maximal_clique(&g);

    // All isolated nodes form cliques of size 1, so any one of them is valid
    // The function should return one of {a}, {b}, or {c}
    assert_eq!(1, largest_clique.len());
    assert!(
        largest_clique.contains(&a) || largest_clique.contains(&b) || largest_clique.contains(&c)
    );
}

#[test]
fn test_largest_maximal_clique_complete_graph() {
    // Test with a complete graph (triangle)
    let mut g = Graph::<i32, (), Undirected>::new_undirected();
    let a = g.add_node(0);
    let b = g.add_node(1);
    let c = g.add_node(2);

    g.extend_with_edges([(a, b), (b, c), (c, a)]);

    let largest_clique = largest_maximal_clique(&g);

    // The largest clique should be {a, b, c} with size 3
    let expected_clique: HashSet<_> = vec![a, b, c].into_iter().collect();
    assert_eq!(expected_clique, largest_clique);
}
