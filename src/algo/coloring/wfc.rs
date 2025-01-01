use crate::visit::{IntoNeighbors, NodeCount, NodeIndexable, Visitable};
use std::collections::HashMap;
use std::hash::Hash;

/// [Generic] Wave Function Collapse vertex coloring algorithm.
/// https://arxiv.org/pdf/2108.09329
///
/// Complexity:
///  - Best case: O(|V|(|V| + |E|))
///  - Worst case: O(|V|² * (|V| + |E|))
///
/// Compute a valid vertex coloring for an undirected graph using the Wave Function
/// Collapse algorithm. The algorithm assigns colors (represented as usize values)
/// to vertices such that no adjacent vertices share the same color.
///
/// The graph should be `Visitable` and implement `IntoNeighbors`. The implementation
/// uses 1-based color numbering.
///
/// Returns a `HashMap` that maps node IDs to their assigned colors.
///
/// # Example
/// ```rust
/// use petgraph::Graph;
/// use petgraph::Undirected;
/// use std::collections::HashMap;
/// use petgraph::algo::wfc_coloring;
///
/// let mut graph = Graph::<(), (), Undirected>::new_undirected();
/// let a = graph.add_node(());
/// let b = graph.add_node(());
/// let c = graph.add_node(());
///
/// graph.extend_with_edges(&[
///     (a, b),
///     (b, c),
///     (c, a),
/// ]);
///
/// // a ----- b
/// // \      /
/// //   \  /
/// //    c
///
///
/// let coloring = wfc_coloring(&graph);
/// assert_ne!(coloring[&a], coloring[&b]); // Adjacent vertices have different colors
/// ```
pub fn wfc_coloring<G>(graph: G) -> HashMap<G::NodeId, usize>
where
    G: IntoNeighbors + NodeCount + NodeIndexable + Visitable,
    G::NodeId: Eq + Hash + Copy,
{
    // Helper function for constraint propagation
    fn propagate<G>(
        graph: G,
        start: G::NodeId,
        colors: &mut HashMap<G::NodeId, usize>,
        domains: &mut HashMap<G::NodeId, Vec<usize>>,
    ) where
        G: IntoNeighbors,
        G::NodeId: Eq + Hash + Copy,
    {
        let mut stack = vec![start];
        while let Some(u) = stack.pop() {
            if let Some(&color) = colors.get(&u) {
                for neighbor in graph.neighbors(u) {
                    if !colors.contains_key(&neighbor) {
                        if let Some(domain) = domains.get_mut(&neighbor) {
                            if domain.contains(&color) {
                                domain.retain(|&c| c != color);
                                if domain.len() == 1 {
                                    colors.insert(neighbor, domain[0]);
                                    stack.push(neighbor);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Helper function to calculate entropy of a node
    fn entropy<N>(node: N, colors: &HashMap<N, usize>, domains: &HashMap<N, Vec<usize>>) -> usize
    where
        N: Eq + Hash + Copy,
    {
        if colors.contains_key(&node) {
            0
        } else {
            domains.get(&node).map_or(0, |d| d.len())
        }
    }

    // Initialize state
    let max_degree = (0..graph.node_bound())
        .map(|i| graph.from_index(i))
        .map(|n| graph.neighbors(n).count())
        .max()
        .unwrap_or(0);

    let mut max_colors = max_degree + 1;
    let mut colors = HashMap::new();
    let mut domains = HashMap::new();

    loop {
        // Reset for this attempt
        colors.clear();
        domains = (0..graph.node_bound())
            .map(|i| graph.from_index(i))
            .map(|n| (n, (1..=max_colors).collect()))
            .collect();

        // Color highest degree vertex first
        let start = (0..graph.node_bound())
            .map(|i| graph.from_index(i))
            .max_by_key(|&n| graph.neighbors(n).count())
            .unwrap();

        colors.insert(start, 1);
        propagate(&graph, start, &mut colors, &mut domains);

        // Main coloring loop
        while colors.len() < graph.node_count() {
            // Find uncolored vertex with lowest non-zero entropy
            let next = (0..graph.node_bound())
                .map(|i| graph.from_index(i))
                .filter(|v| !colors.contains_key(v))
                .min_by_key(|&v| {
                    let e = entropy(v, &colors, &domains);
                    if e == 0 {
                        usize::MAX
                    } else {
                        e
                    }
                });

            match next {
                Some(v) => {
                    if let Some(domain) = domains.get(&v) {
                        if !domain.is_empty() {
                            colors.insert(v, domain[0]);
                            propagate(&graph, v, &mut colors, &mut domains);
                        }
                    }
                }
                None => {
                    // If we can't proceed, increase color count and restart
                    max_colors += 1;
                    continue;
                }
            }
        }
        break;
    }

    colors
}
