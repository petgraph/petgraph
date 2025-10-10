use alloc::{
    collections::BinaryHeap,
    {vec, vec::Vec},
};
use core::hash::Hash;

use hashbrown::{HashMap, HashSet};

use crate::scored::MaxScored;
use crate::visit::{IntoEdges, IntoNodeIdentifiers, NodeIndexable, VisitMap, Visitable};

pub struct ColorReturn<G>
where
    G: IntoNodeIdentifiers,
{
    pub color_count: usize,
    pub nodes_to_colors: HashMap<G::NodeId, usize>,
}

/// Solves the graph coloring problem for the given graph, returning a ColorReturn struct.
/// Uses Recursive Largest First algorithm.
///
/// The current coloring algorithm implementations are:
/// - Recursive Largest First (RLF)
/// - DStatur coloring
///
/// Both are heuristics, so they are not guaranteed to return the optimal coloring.
/// RLF generally gives better results than DStatur, but runs much slower.
pub fn get_colors<G>(graph: G) -> ColorReturn<G>
where
    G: IntoEdges + IntoNodeIdentifiers + Visitable + NodeIndexable + Clone,
    G::NodeId: Eq + Hash + Ord,
{
    recursive_largest_first_coloring(graph)
}

/// [Recursive Largest First Algorithm][1] to properly color a non weighted undirected graph.
/// The graph must be undirected. It should not contain loops.
///
/// Gives better results than DStatur, but is slower. Use DStatur if you find this algorithm too slow.
///
/// This is a heuristic. So, it does not necessarily return a minimum coloring.
///
/// # Arguments
/// * `graph`: undirected graph without loops.
///
/// # Returns
/// * `ColorReturn<G>`
///
/// # Complexity
/// * Time complexity: **O(|V|^3)**.
/// * Auxiliary space: **0**
///
/// where **|V|** is the number of nodes
///
/// [1]: https://en.wikipedia.org/wiki/Recursive_largest_first_algorithm
/// [2]: https://pmc.ncbi.nlm.nih.gov/articles/PMC6756213/
///
/// # Example
/// ```rust
/// use petgraph::{Graph, Undirected};
/// use petgraph::algo::recursive_largest_first_coloring;
///
/// let mut graph: Graph<(), (), Undirected> = Graph::new_undirected();
/// let a = graph.add_node(());
/// let b = graph.add_node(());
/// let c = graph.add_node(());
/// let d = graph.add_node(());
/// let e = graph.add_node(());
/// let f = graph.add_node(());
///
/// graph.extend_with_edges(&[
///     (a, b),
///     (b, c),
///     (c, d),
///     (d, e),
///     (e, f),
///     (f, a),
/// ]);
///
/// // a ----- b ----- c
/// // |               |
/// // |               |
/// // |               |
/// // f ----- e------ d
///
/// let return = recursive_largest_first_coloring(&graph);
/// assert_eq!(return.color_count, 2);
/// assert_ne!(return.nodes_to_colors[&a], return.nodes_to_colors[&b]);
/// ```
pub fn recursive_largest_first_coloring<G>(graph: G) -> ColorReturn<G>
where
    G: IntoEdges + IntoNodeIdentifiers + Visitable + NodeIndexable + Clone,
    G::NodeId: Eq + Hash + Ord,
{
    let mut color_classes: HashMap<G::NodeId, usize> = HashMap::new();
    let mut remaining: Vec<G::NodeId> = graph.node_identifiers().collect();
    remaining.sort_by_key(|b| core::cmp::Reverse(graph.edges(*b).count()));
    let mut current_color: usize = 0;
    while !remaining.is_empty() {
        // Build maximal independent set
        let mut independent_set = Vec::new();
        let mut used_indices = Vec::new();
        for (idx, &node) in remaining.iter().enumerate() {
            // Check for adjacency with any node already in the independent set
            let is_adjacent = independent_set
                .iter()
                .any(|&set_node| graph.neighbors(node).any(|neighbor| neighbor == set_node));

            if !is_adjacent {
                independent_set.push(node);
                used_indices.push(idx);
                color_classes.insert(node, current_color);
            }
        }
        for &idx in used_indices.iter().rev() {
            remaining.swap_remove(idx);
        }
        current_color += 1;
    }

    ColorReturn {
        color_count: current_color,
        nodes_to_colors: color_classes,
    }
}

/// [DStatur algorithm][1] to properly color a non weighted undirected graph.
///
///
/// This is a heuristic. So, it does not necessarily return a minimum coloring.
/// The graph must be undirected. It should not contain loops.
///
/// # Arguments
/// * `graph`: undirected graph without loops.
///
/// # Returns
/// Returns a tuple of:
/// * [`struct@hashbrown::HashMap`] that associates to each `NodeId` its color.
/// * `usize`: the number of used colors.
///
/// # Complexity
/// * Time complexity: **O((|V| + |E|)log(|V|)**.
/// * Auxiliary space: **O(|V| + |E|)**.
///
/// where **|V|** is the number of nodes and **|E|** is the number of edges.
///
/// [1]: https://en.wikipedia.org/wiki/DSatur
///
/// # Example
/// ```rust
/// use petgraph::{Graph, Undirected};
/// use petgraph::algo::dsatur_coloring;
///
/// let mut graph: Graph<(), (), Undirected> = Graph::new_undirected();
/// let a = graph.add_node(());
/// let b = graph.add_node(());
/// let c = graph.add_node(());
/// let d = graph.add_node(());
/// let e = graph.add_node(());
/// let f = graph.add_node(());
///
/// graph.extend_with_edges(&[
///     (a, b),
///     (b, c),
///     (c, d),
///     (d, e),
///     (e, f),
///     (f, a),
/// ]);
///
/// // a ----- b ----- c
/// // |               |
/// // |               |
/// // |               |
/// // f ----- e------ d
///
/// let (coloring, nb_colors) = dsatur_coloring(&graph);
/// assert_eq!(nb_colors, 2);
/// assert_ne!(coloring[&a], coloring[&b]);
/// ```
pub fn dsatur_coloring<G>(graph: G) -> (HashMap<G::NodeId, usize>, usize)
where
    G: IntoEdges + IntoNodeIdentifiers + Visitable + NodeIndexable,
    G::NodeId: Eq + Hash,
{
    let ix = |v| graph.to_index(v);
    let n = graph.node_bound();

    let mut degree_map = vec![0; n];
    let mut queue = BinaryHeap::with_capacity(n);
    let mut colored = HashMap::with_capacity(n);
    let mut adj_color_map = vec![HashSet::new(); n];
    let mut seen = graph.visit_map();
    let mut max_color = 0;

    for node in graph.node_identifiers() {
        let degree = graph.edges(node).count();
        queue.push(MaxScored((0, degree), node));
        degree_map[ix(node)] = degree;
    }

    while let Some(MaxScored(_, node)) = queue.pop() {
        if seen.is_visited(&node) {
            continue;
        }
        seen.visit(node);

        let adj_color = &adj_color_map[ix(node)];
        let mut color = 0;
        while adj_color.contains(&color) {
            color += 1;
        }

        colored.insert(node, color);
        max_color = max_color.max(color);

        for nbor in graph.neighbors(node) {
            if let Some(adj_color) = adj_color_map.get_mut(ix(nbor)) {
                adj_color.insert(color);
                queue.push(MaxScored((adj_color.len(), degree_map[ix(nbor)]), nbor));
            }
        }
    }

    (colored, max_color + 1)
}
