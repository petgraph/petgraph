use crate::visit::{GetAdjacencyMatrix, IntoNeighbors, IntoNodeIdentifiers, NodeIndexable};
use alloc::vec::Vec;
use core::hash::Hash;
use core::iter::FromIterator;
use fixedbitset::FixedBitSet;
use hashbrown::HashSet;

/// Finds maximal cliques containing all the vertices in r, some of the
/// vertices in p, and none of the vertices in x.
///
/// By default, only works on undirected graphs. It can be used on directed graphs
/// if the graph is symmetric. I.e., if an edge (u, v) exists, then (v, u) also exists.
///
/// Uses the [Bron–Kerbosch algorithm][1] with pivoting.
///
/// [1]: https://en.wikipedia.org/wiki/Bron%E2%80%93Kerbosch_algorithm
fn bron_kerbosch_pivot<G>(
    g: G,
    adj_mat: &G::AdjMatrix,
    r: HashSet<G::NodeId>,
    mut p: FixedBitSet,
    mut x: FixedBitSet,
) -> Vec<HashSet<G::NodeId>>
where
    G: GetAdjacencyMatrix + IntoNeighbors + NodeIndexable,
    G::NodeId: Eq + Hash,
{
    let mut cliques = Vec::with_capacity(1);
    if p.is_clear() {
        if x.is_clear() {
            cliques.push(r);
        }
        return cliques;
    }
    // pick the pivot u to be the vertex with max degree
    let u = p
        .ones()
        .max_by_key(|&v| g.neighbors(g.from_index(v)).count())
        .unwrap();
    let mut todo = p
        .ones()
        .filter(|&v| {
            u == v
                || !g.is_adjacent(adj_mat, g.from_index(u), g.from_index(v))
                || !g.is_adjacent(adj_mat, g.from_index(v), g.from_index(u))
        }) //skip neighbors of pivot
        .collect::<Vec<usize>>();
    while let Some(v) = todo.pop() {
        let mut neighbors =
            FixedBitSet::from_iter(g.neighbors(g.from_index(v)).map(|n| g.to_index(n)));
        p.remove(v);
        let mut next_r = r.clone();
        next_r.insert(g.from_index(v));

        let next_p: FixedBitSet = p.intersection(&neighbors).collect();
        neighbors.intersect_with(&x);
        let next_x = neighbors;

        cliques.extend(bron_kerbosch_pivot(g, adj_mat, next_r, next_p, next_x));

        x.insert(v);
    }

    cliques
}

/// Find all maximal cliques in an undirected graph using [Bron–Kerbosch algorithm][1]
/// with pivoting. Also works on symmetric directed graphs, see the note below.
///
/// A clique is a set of nodes such that every node connects to
/// every other. A maximal clique is a clique that cannot be extended
/// by including one more adjacent vertex. A graph may have multiple
/// maximal cliques.
///
/// This method may also be called on directed graphs, but one needs to ensure that
/// if an edge (u, v) exists, then (v, u) also exists.
///
/// # Arguments
/// * `g`: The graph to find maximal cliques in.
///
/// # Returns
/// * `Vec<HashSet>`: A vector of [`struct@hashbrown::HashSet`] making up the maximal cliques in the graph.
///
/// # Complexity
/// * Time complexity: **O(3^(|V|/3))**
/// * Auxiliary space: **O(|V|² + |V|k)**.
///
/// where **|V|** is the number of nodes and k is the number of maximal cliques in the graph
/// (possibly up to *3^(|V|/3)** many).
///
/// [1]: https://en.wikipedia.org/wiki/Bron%E2%80%93Kerbosch_algorithm
///
/// # Example
///
/// ```
/// use petgraph::algo::maximal_cliques;
/// use petgraph::graph::UnGraph;
///
/// let mut g = UnGraph::<i32, ()>::from_edges([(0, 1), (0, 2), (1, 2), (2, 3)]);
/// g.add_node(4);
/// // The example graph:
/// //
/// // 0 --- 2 -- 3
/// //  \   /
/// //   \ /
/// //    1       4
/// //
/// // maximal cliques: {4}, {2, 3}, {0, 1, 2}
/// // Output the result
/// let cliques = maximal_cliques(&g);
/// println!("{:?}", cliques);
/// // [
/// //   {NodeIndex(4)},
/// //   {NodeIndex(0), NodeIndex(1), NodeIndex(2)},
/// //   {NodeIndex(2), NodeIndex(3)}
/// // ]
/// ```
pub fn maximal_cliques<G>(g: G) -> Vec<HashSet<G::NodeId>>
where
    G: GetAdjacencyMatrix + IntoNodeIdentifiers + IntoNeighbors + NodeIndexable,
    G::NodeId: Eq + Hash,
{
    let adj_mat = g.adjacency_matrix();
    let r = HashSet::new();
    let mut p = FixedBitSet::with_capacity(g.node_bound());
    p.extend(g.node_identifiers().map(|n| g.to_index(n)));
    let x = FixedBitSet::with_capacity(g.node_bound());
    bron_kerbosch_pivot(g, &adj_mat, r, p, x)
}
