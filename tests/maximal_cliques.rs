extern crate petgraph;
use core::hash::Hash;
use hashbrown::HashSet;
use petgraph::graph::{DiGraph, UnGraph};
use petgraph::{
    algo::maximal_cliques,
    graph::Graph,
    visit::{GetAdjacencyMatrix, IntoNeighbors, IntoNodeIdentifiers},
    Undirected,
};

/// (reference implementation)
/// Finds maximal cliques containing all the vertices in r, some of the
/// vertices in p, and none of the vertices in x.
///
/// By default, only works on undirected graphs. It can be used on directed graphs
/// if the graph is symmetric. I.e., if an edge (u, v) exists, then (v, u) also exists.
#[allow(dead_code)] // used by tests
fn bron_kerbosch_ref<G>(
    g: G,
    adj_mat: &G::AdjMatrix,
    r: HashSet<G::NodeId>,
    mut p: HashSet<G::NodeId>,
    mut x: HashSet<G::NodeId>,
) -> Vec<HashSet<G::NodeId>>
where
    G: GetAdjacencyMatrix,
    G: IntoNeighbors,
    G::NodeId: Eq + Hash,
{
    let mut cliques = Vec::with_capacity(1);
    if p.is_empty() && x.is_empty() {
        cliques.push(r);
        return cliques;
    }
    let mut todo = p.iter().cloned().collect::<Vec<G::NodeId>>();
    while let Some(v) = todo.pop() {
        p.remove(&v);
        let mut next_r = r.clone();
        next_r.insert(v);

        let mut neighbors = HashSet::new();

        for u in g.neighbors(v) {
            if g.is_adjacent(adj_mat, u, v) {
                neighbors.insert(u);
            }
        }

        let next_p = p
            .intersection(&neighbors)
            .cloned()
            .collect::<HashSet<G::NodeId>>();
        let next_x = x
            .intersection(&neighbors)
            .cloned()
            .collect::<HashSet<G::NodeId>>();

        cliques.extend(bron_kerbosch_ref(g, adj_mat, next_r, next_p, next_x));

        x.insert(v);
    }
    cliques
}

/// (reference implementation)
/// Find all maximal cliques in a graph.
///
/// By default, only works on undirected graphs. It can be used on directed graphs
/// if the graph is symmetric. I.e., if an edge (u, v) exists, then (v, u) also exists.
#[allow(dead_code)] // used by tests
pub(crate) fn maximal_cliques_ref<G>(g: G) -> Vec<HashSet<G::NodeId>>
where
    G: GetAdjacencyMatrix,
    G: IntoNodeIdentifiers + IntoNeighbors,
    G::NodeId: Eq + Hash,
{
    let adj_mat = g.adjacency_matrix();
    let p = g.node_identifiers().collect::<HashSet<G::NodeId>>();
    bron_kerbosch_ref(g, &adj_mat, HashSet::new(), p, HashSet::new())
}

#[test]
fn test_maximal_cliques_empty_graph() {
    // empty graph should not yield any cliques
    let g = Graph::<i32, ()>::new();
    let cliques = maximal_cliques(&g);
    let expected_cliques = vec![HashSet::new()];
    assert_eq!(expected_cliques, cliques);
}

#[test]
fn test_maximal_cliques_ref_empty_graph() {
    // empty graph should not yield any cliques
    let g = Graph::<i32, ()>::new();
    let cliques = maximal_cliques_ref(&g);
    let expected_cliques = vec![HashSet::new()];
    assert_eq!(expected_cliques, cliques);
}

#[test]
fn test_maximal_cliques_undirected_sparse_graph() {
    // c     d
    //
    // b --- a
    let mut g = UnGraph::<i32, ()>::new_undirected();

    let a = g.add_node(0);
    let b = g.add_node(1);
    let c = g.add_node(2);
    let d = g.add_node(3);

    g.extend_with_edges([(a, b), (b, a)]);

    let cliques = maximal_cliques(&g);

    let expected_cliques = vec![vec![a, b], vec![c], vec![d]];
    assert_eq!(expected_cliques.len(), cliques.len());

    for v in expected_cliques {
        assert!(cliques.contains(&v.iter().cloned().collect()));
    }
}

#[test]
fn test_maximal_cliques_undirected_ref_sparse_graph() {
    // c     d
    //
    // b --- a
    let mut g = UnGraph::<i32, ()>::new_undirected();

    let a = g.add_node(0);
    let b = g.add_node(1);
    let c = g.add_node(2);
    let d = g.add_node(3);

    g.extend_with_edges([(a, b), (b, a)]);

    let cliques = maximal_cliques_ref(&g);

    let expected_cliques = vec![vec![a, b], vec![c], vec![d]];
    assert_eq!(expected_cliques.len(), cliques.len());

    for v in expected_cliques {
        assert!(cliques.contains(&v.iter().cloned().collect()));
    }
}

#[test]
fn test_maximal_cliques_directed_sparse_graph() {
    // c     d
    //
    // b <-> a
    let mut g = DiGraph::<i32, ()>::new();

    let a = g.add_node(0);
    let b = g.add_node(1);
    let c = g.add_node(2);
    let d = g.add_node(3);

    g.extend_with_edges([(a, b), (b, a)]);

    let cliques = maximal_cliques(&g);

    let expected_cliques = vec![vec![a, b], vec![c], vec![d]];
    assert_eq!(expected_cliques.len(), cliques.len());

    for v in expected_cliques {
        assert!(cliques.contains(&v.iter().cloned().collect()));
    }
}

#[test]
fn test_maximal_cliques_directed_ref_sparse_graph() {
    // c     d
    //
    // b <-> a
    let mut g = DiGraph::<i32, ()>::new();

    let a = g.add_node(0);
    let b = g.add_node(1);
    let c = g.add_node(2);
    let d = g.add_node(3);

    g.extend_with_edges([(a, b), (b, a)]);

    let cliques = maximal_cliques_ref(&g);

    let expected_cliques = vec![vec![a, b], vec![c], vec![d]];
    assert_eq!(expected_cliques.len(), cliques.len());

    for v in expected_cliques {
        assert!(cliques.contains(&v.iter().cloned().collect()));
    }
}

#[test]
fn test_maximal_cliques_undirected() {
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

    let cliques = maximal_cliques(&g);
    println!("{:?}", &cliques);

    let expected_cliques = vec![
        vec![a, b, e],
        vec![b, c],
        vec![c, d],
        vec![d, e],
        vec![e, f],
    ];
    assert_eq!(expected_cliques.len(), cliques.len());

    for v in expected_cliques {
        assert!(cliques.contains(&v.iter().cloned().collect()));
    }
}

#[test]
fn test_maximal_cliques_ref_undirected() {
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

    let cliques = maximal_cliques_ref(&g);
    println!("{:?}", &cliques);

    let expected_cliques = vec![
        vec![a, b, e],
        vec![b, c],
        vec![c, d],
        vec![d, e],
        vec![e, f],
    ];
    assert_eq!(expected_cliques.len(), cliques.len());

    for v in expected_cliques {
        assert!(cliques.contains(&v.iter().cloned().collect()));
    }
}

#[test]
fn test_maximal_cliques_directed() {
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

    let cliques = maximal_cliques(&g);
    println!("{:?}", &cliques);

    let expected_cliques = vec![
        vec![a, b, e],
        vec![b, c],
        vec![c, d],
        vec![d, e],
        vec![d, f],
    ];
    assert_eq!(expected_cliques.len(), cliques.len());

    for v in expected_cliques {
        assert!(cliques.contains(&v.iter().cloned().collect()));
    }
}

#[test]
fn test_maximal_cliques_ref_directed() {
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

    let cliques = maximal_cliques_ref(&g);
    println!("{:?}", &cliques);

    let expected_cliques = vec![
        vec![a, b, e],
        vec![b, c],
        vec![c, d],
        vec![d, e],
        vec![d, f],
    ];
    assert_eq!(expected_cliques.len(), cliques.len());

    for v in expected_cliques {
        assert!(cliques.contains(&v.iter().cloned().collect()));
    }
}
