extern crate petgraph;

#[cfg(feature = "quickcheck")]
extern crate quickcheck;

use petgraph::{
    algo::maximal_cliques,
    graph::{Graph, NodeIndex},
    visit::{GetAdjacencyMatrix, IntoNeighbors, IntoNodeIdentifiers},
    Undirected,
};
use std::collections::HashSet;
use std::hash::Hash;

/// (reference implementation)
/// Finds maximal cliques containing all the vertices in r, some of the
/// vertices in p, and none of the vertices in x.
#[allow(dead_code)] //used by tests
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
        let mut walker = g.neighbors(v);
        while let Some(u) = walker.next() {
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
    return cliques;
}

/// (reference implementation)
/// Find all maximal cliques in a graph.
#[allow(dead_code)] //used by tests
fn maximal_cliques_ref<G>(g: G) -> Vec<HashSet<G::NodeId>>
where
    G: GetAdjacencyMatrix,
    G: IntoNodeIdentifiers + IntoNeighbors,
    G::NodeId: Eq + Hash,
{
    let adj_mat = g.adjacency_matrix();
    let p = g.node_identifiers().collect::<HashSet<G::NodeId>>();
    return bron_kerbosch_ref(g, &adj_mat, HashSet::new(), p, HashSet::new());
}

#[test]
fn maximal_cliques_ref_directed() {
    // 5 <- 3 = 4 == 0
    //      ||  || //
    //      2 <- 1
    let mut g = Graph::<i32, ()>::new();
    let a = g.add_node(0);
    let b = g.add_node(1);
    let c = g.add_node(2);
    let d = g.add_node(3);
    let e = g.add_node(4);
    let f = g.add_node(5);
    g.extend_with_edges(&[
        (a, b),
        (b, a),
        (a, e),
        (e, a),
        (b, e),
        (e, b),
        (b, c),
        (c, d),
        (d, c),
        (d, e),
        (e, d),
        (e, f),
    ]);

    let mut cliques = maximal_cliques_ref(&g);
    println!("{:?}", &cliques);

    let answer = vec![vec![a, b, e], vec![c, d], vec![d, e], vec![f]];
    assert_eq!(cliques.len(), answer.len());

    for a in answer {
        let s = a.iter().cloned().collect::<HashSet<NodeIndex>>();
        cliques.retain(|c| *c != s);
    }
    assert!(cliques.is_empty());
}

#[test]
fn maximal_cliques_ref_undirected() {
    // 5 - 3 - 4 - 0
    //     |   | /
    //     2 - 1
    let mut g = Graph::<i32, (), Undirected>::new_undirected();
    let a = g.add_node(0);
    let b = g.add_node(1);
    let c = g.add_node(2);
    let d = g.add_node(3);
    let e = g.add_node(4);
    let f = g.add_node(5);
    g.extend_with_edges(&[(a, b), (a, e), (b, e), (b, c), (c, d), (d, e), (e, f)]);

    let mut cliques = maximal_cliques_ref(&g);
    println!("{:?}", &cliques);

    let answer = vec![
        vec![a, b, e],
        vec![b, c],
        vec![c, d],
        vec![d, e],
        vec![e, f],
    ];
    assert_eq!(cliques.len(), answer.len());

    for a in answer {
        let s = a.iter().cloned().collect::<HashSet<NodeIndex>>();
        cliques.retain(|c| *c != s);
    }
    assert!(cliques.is_empty());
}

#[test]
fn maximal_cliques_empty_graph() {
    // empty graph should not yield any cliques
    let g = Graph::<i32, ()>::new();
    let cliques = maximal_cliques(&g);
    let answer = vec![HashSet::new()];
    assert_eq!(cliques, answer);
}

#[test]
fn maximal_cliques_directed() {
    // 5 <- 3 = 4 == 0
    //      ||  || //
    //      2 <- 1
    let mut g = Graph::<i32, ()>::new();
    let a = g.add_node(0);
    let b = g.add_node(1);
    let c = g.add_node(2);
    let d = g.add_node(3);
    let e = g.add_node(4);
    let f = g.add_node(5);
    g.extend_with_edges(&[
        (a, b),
        (b, a),
        (a, e),
        (e, a),
        (b, e),
        (e, b),
        (b, c),
        (c, d),
        (d, c),
        (d, e),
        (e, d),
        (e, f),
    ]);

    let mut cliques = maximal_cliques_ref(&g);
    println!("{:?}", &cliques);

    let answer = vec![vec![a, b, e], vec![c, d], vec![d, e], vec![f]];
    assert_eq!(cliques.len(), answer.len());

    for a in answer {
        let s = a.iter().cloned().collect::<HashSet<NodeIndex>>();
        cliques.retain(|c| *c != s);
    }
    assert!(cliques.is_empty());
}

#[test]
fn maximal_cliques_undirected() {
    // 5 - 3 - 4 - 0
    //     |   | /
    //     2 - 1
    let mut g = Graph::<i32, (), Undirected>::new_undirected();
    let a = g.add_node(0);
    let b = g.add_node(1);
    let c = g.add_node(2);
    let d = g.add_node(3);
    let e = g.add_node(4);
    let f = g.add_node(5);
    g.extend_with_edges(&[(a, b), (a, e), (b, e), (b, c), (c, d), (d, e), (e, f)]);

    let mut cliques = maximal_cliques_ref(&g);
    println!("{:?}", &cliques);

    let answer = vec![
        vec![a, b, e],
        vec![b, c],
        vec![c, d],
        vec![d, e],
        vec![e, f],
    ];
    assert_eq!(cliques.len(), answer.len());

    for a in answer {
        let s = a.iter().cloned().collect::<HashSet<NodeIndex>>();
        cliques.retain(|c| *c != s);
    }
    assert!(cliques.is_empty());
}

#[cfg(feature = "quickcheck")]
#[test]
fn maximal_cliques_matches_ref_impl() {
    use petgraph::{Directed, EdgeType, Undirected};

    fn prop<Ty>(g: Graph<(), (), Ty>) -> bool
    where
        Ty: EdgeType,
    {
        if g.edge_count() <= 200 && g.node_count() <= 200 {
            let cliques = maximal_cliques(&g);
            let cliques_ref = maximal_cliques_ref(&g);

            assert!(cliques.len() == cliques_ref.len());

            for c in &cliques_ref {
                assert!(cliques.contains(c));
            }
        }
        true
    }
    quickcheck::quickcheck(prop as fn(Graph<_, _, Undirected>) -> bool);
    quickcheck::quickcheck(prop as fn(Graph<_, _, Directed>) -> bool);
}
