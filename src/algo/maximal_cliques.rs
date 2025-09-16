use crate::visit::{
    GetAdjacencyMatrix, IntoEdges, IntoNeighbors, IntoNodeIdentifiers, NodeIndexable, Visitable,
};
use alloc::vec::Vec;
use core::hash::Hash;
use core::iter::FromIterator;
use hashbrown::{HashMap, HashSet};

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
    mut p: HashSet<G::NodeId>,
    mut x: HashSet<G::NodeId>,
) -> Vec<HashSet<G::NodeId>>
where
    G: GetAdjacencyMatrix + IntoNeighbors,
    G::NodeId: Eq + Hash,
{
    let mut cliques = Vec::with_capacity(1);
    if p.is_empty() {
        if x.is_empty() {
            cliques.push(r);
        }
        return cliques;
    }
    // pick the pivot u to be the vertex with max degree
    let u = p.iter().max_by_key(|&v| g.neighbors(*v).count()).unwrap();
    let mut todo = p
        .iter()
        .filter(|&v| *u == *v || !g.is_adjacent(adj_mat, *u, *v) || !g.is_adjacent(adj_mat, *v, *u)) //skip neighbors of pivot
        .cloned()
        .collect::<Vec<G::NodeId>>();
    while let Some(v) = todo.pop() {
        let neighbors = HashSet::from_iter(g.neighbors(v));
        p.remove(&v);
        let mut next_r = r.clone();
        next_r.insert(v);

        let next_p = p
            .intersection(&neighbors)
            .cloned()
            .collect::<HashSet<G::NodeId>>();
        let next_x = x
            .intersection(&neighbors)
            .cloned()
            .collect::<HashSet<G::NodeId>>();

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
    G: GetAdjacencyMatrix + IntoNodeIdentifiers + IntoNeighbors,
    G::NodeId: Eq + Hash,
{
    let adj_mat = g.adjacency_matrix();
    let r = HashSet::new();
    let p = g.node_identifiers().collect::<HashSet<G::NodeId>>();
    let x = HashSet::new();
    bron_kerbosch_pivot(g, &adj_mat, r, p, x)
}

/// Finds the largest maximal clique in an undirected graph using the algorithm outlined in McCreesh and Prosser's paper [1].
/// Much faster than the standard Bron-Kerbosch technique, but only will get the largest maximal clique
/// instead of all maximal cliques
///  
/// Uses a graph coloring algorithm to narrow the search space and start from the nodes most likely to be in a large clique,
/// and ignores any cliques that are not capable of reaching the max size to further reduce the time taken. This implementation
/// can be dramatically improved through parallelization, as was originally outlined in the aforementioned paper [1]
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
/// * Time complexity: **O((n+m) * 3^(n/3))**
/// * Auxiliary space: **O(n^2)**
///
/// where `n` is the number of nodes and `m` is the of edges
///
/// [1]: https://doi.org/10.3390/a6040618
///
/// # Example
///
/// ```
/// use petgraph::algo::maximal_cliques::largest_maximal_clique;
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
/// let cliques = largest_maximal_clique(&g);
/// println!("{:?}", cliques);
/// //   {NodeIndex(0), NodeIndex(1), NodeIndex(2)}
/// ```
pub fn largest_maximal_clique<G>(g: G) -> HashSet<G::NodeId>
where
    G: IntoEdges + IntoNodeIdentifiers + Visitable + NodeIndexable,
    G::NodeId: Eq + Hash + Ord,
{
    mccreesh_prosser(g)
}

fn mccreesh_prosser<G>(g: G) -> HashSet<G::NodeId>
where
    G: IntoEdges + IntoNodeIdentifiers + Visitable + NodeIndexable,
    G::NodeId: Eq + Hash + Clone + Ord,
{
    let mut c_max = HashSet::new();
    let mut nodes: Vec<_> = g.node_identifiers().collect();

    // sort nodes by degree as high edge nodes are more likely to be in a larger clique
    nodes.sort_by_key(|&n| core::cmp::Reverse(g.edges(n).count()));
    let mut clique = HashSet::new();
    let candidate_vertices: HashSet<_> = nodes.into_iter().collect();

    expand(&g, &mut clique, candidate_vertices, &mut c_max);
    c_max
}

/// A recursive Function as described in the [McCreesh-Prosser paper](https://doi.org/10.3390/a6040618)
/// Which takes in a clique and attempts to expand it using the candidate vertices
fn expand<G>(
    graph: &G,
    clique: &mut HashSet<G::NodeId>,
    mut candidate_vertices: HashSet<G::NodeId>,
    largest_clique: &mut HashSet<G::NodeId>,
) where
    G: IntoEdges + Visitable,
    G::NodeId: Eq + Hash + Clone + Ord,
{
    // color the candidate vertices to get an upper bound for pruning and roughly rank the nodes most -> least connections
    let (order, colors) = color_order(graph, &candidate_vertices);

    // iterate through candidates from rarest color to most common color
    for node in order.into_iter().rev() {
        let v_color = colors.get(&node).copied().unwrap_or(0);

        // if the best potential size is worse than the largest we've found, ignore it
        if clique.len() + v_color <= largest_clique.len() {
            return;
        }
        // test how the clique changes when we add the new node (if it's worse we will remove it)
        clique.insert(node);
        let neighbors: HashSet<_> = graph
            .neighbors(node)
            .filter(|&neighbor| neighbor != node) // filter out self-references
            .collect();
        let new_candidate_vertices: HashSet<G::NodeId> = candidate_vertices
            .intersection(&neighbors)
            .cloned()
            .collect();

        // this fires when we have a maximal clique
        if new_candidate_vertices.is_empty() {
            // update largest_clique if applicable
            if clique.len() > largest_clique.len() {
                *largest_clique = clique.clone();
            }
        } else {
            // still have more nodes to add, recurse
            expand(graph, clique, new_candidate_vertices, largest_clique);
        }

        // backtrack: remove the node from the clique so we can explore other possibilities.
        clique.remove(&node);
        candidate_vertices.remove(&node);
    }
}

/// Colors the given set of nodes using a greedy algorithm and returns an ordering.
/// Simple and fast, but will produce results much worse than most other colorings
///
/// This algorithm specifically is extremely useful for use in `mccreesh_prosser` clique finding as speed is
/// very important and using a good coloring can be detrimental for that task.
///
/// # Returns
/// A tuple containing:
/// * `order`: A vector of the nodes in `p`, sorted by their assigned color number.
/// * `colors`: A map from each node to its color number (a `usize`).
fn color_order<G>(graph: &G, p: &HashSet<G::NodeId>) -> (Vec<G::NodeId>, HashMap<G::NodeId, usize>)
where
    G: IntoEdges + Visitable,
    G::NodeId: Eq + Hash + Clone + Ord,
{
    // A simple greedy coloring implementation.
    let mut colors = HashMap::new();
    let mut ordered_nodes: Vec<_> = p.iter().cloned().collect();

    // sort nodes first so we have deterministic coloring
    ordered_nodes.sort();

    for node in ordered_nodes {
        let mut neighboring_colors = HashSet::new();
        for neighbor in graph.neighbors(node) {
            if let Some(color) = colors.get(&neighbor) {
                neighboring_colors.insert(*color);
            }
        }

        let mut color = 1;
        while neighboring_colors.contains(&color) {
            color += 1;
        }
        colors.insert(node, color);
    }

    // Now create the final `order` vector, sorted by color.
    let mut final_order: Vec<_> = p.iter().cloned().collect();
    final_order.sort_by_key(|n| colors.get(n).copied().unwrap_or(0));

    (final_order, colors)
}
