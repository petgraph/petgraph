use crate::graph::{Graph, NodeIndex};
use crate::Undirected;
use std::collections::HashSet;

/// \[Generic\] Bron-Kerbosch algorithm for finding maximal cliques in an undirected graph.
///
/// Arguments:
/// * `graph` - The input graph
///
/// Returns:
/// A vector of vectors, where each inner vector contains the NodeIndices forming a maximal clique
///
/// # Example
/// ```rust
/// use petgraph::graph::UnGraph;
/// use std::collections::HashSet;
/// use petgraph::algo::bron_kerbosch;
///
/// let mut graph = UnGraph::<(), ()>::new_undirected();
///
/// let a = graph.add_node(());
/// let b = graph.add_node(());
/// let c = graph.add_node(());
/// let d = graph.add_node(());
///
/// graph.extend_with_edges(&[
///    (a, b),
///    (b, c),
///    (c, a),
///    (c, d),
/// ]);
///
/// // a ---- c ---- d
/// // \     /
/// //  \   /
/// //    b
///
/// let cliques = bron_kerbosch(&graph);
///
/// assert_eq!(cliques.len(), 2);
/// assert_eq!(cliques[0].len(), 3);
/// assert_eq!(cliques[1].len(), 2);
/// ```

pub fn bron_kerbosch<N, E>(graph: &Graph<N, E, Undirected>) -> Vec<Vec<NodeIndex>> {
    let mut maximal_cliques = Vec::new();
    let mut r = HashSet::new();
    let p: HashSet<NodeIndex> = graph.node_indices().collect();
    let mut x = HashSet::new();

    bron_kerbosch_recursive(graph, &mut r, &p, &mut x, &mut maximal_cliques);
    maximal_cliques
}

fn bron_kerbosch_recursive<N, E>(
    graph: &Graph<N, E, Undirected>,
    r: &mut HashSet<NodeIndex>,
    p: &HashSet<NodeIndex>,
    x: &mut HashSet<NodeIndex>,
    maximal_cliques: &mut Vec<Vec<NodeIndex>>,
) {
    // If P and X are empty, we found a maximal clique
    if p.is_empty() && x.is_empty() {
        maximal_cliques.push(r.iter().cloned().collect());
        return;
    }

    // Choose pivot vertex from P ∪ X that maximizes |P ∩ N(u)|
    let pivot = p
        .iter()
        .chain(x.iter())
        .max_by_key(|&&u| p.iter().filter(|&&v| graph.contains_edge(u, v)).count())
        .cloned();

    // Get vertices to process
    let vertices_to_process = if let Some(pivot) = pivot {
        p.iter()
            .cloned()
            .filter(|&v| !graph.contains_edge(pivot, v))
            .collect::<Vec<_>>()
    } else {
        p.iter().cloned().collect()
    };

    // Process each vertex
    for v in vertices_to_process {
        // Get neighbors of v
        let neighbors: HashSet<NodeIndex> = graph.neighbors(v).collect();

        // Recursively find cliques
        let new_p: HashSet<_> = p.intersection(&neighbors).cloned().collect();
        let mut new_x: HashSet<_> = x.intersection(&neighbors).cloned().collect();

        r.insert(v);
        bron_kerbosch_recursive(graph, r, &new_p, &mut new_x, maximal_cliques);
        r.remove(&v);

        x.insert(v);
    }
}
