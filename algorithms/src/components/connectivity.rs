use petgraph_core::visit::{IntoEdgeReferences, NodeCompactIndexable};

use crate::utilities::union_find::UnionFind;

/// \[Generic\] Return the number of connected components of the graph.
///
/// For a directed graph, this is the *weakly* connected components.
/// # Example
/// ```rust
/// use petgraph_core::edge::Directed;
/// use petgraph_graph::Graph;
///
/// let mut graph: Graph<(), (), Directed> = Graph::new();
/// let a = graph.add_node(()); // node with no weight
/// let b = graph.add_node(());
/// let c = graph.add_node(());
/// let d = graph.add_node(());
/// let e = graph.add_node(());
/// let f = graph.add_node(());
/// let g = graph.add_node(());
/// let h = graph.add_node(());
///
/// graph.extend_with_edges(&[
///     (a, b),
///     (b, c),
///     (c, d),
///     (d, a),
///     (e, f),
///     (f, g),
///     (g, h),
///     (h, e),
/// ]);
/// // a ----> b       e ----> f
/// // ^       |       ^       |
/// // |       v       |       v
/// // d <---- c       h <---- g
///
/// assert_eq!(connected_components(&graph), 2);
/// graph.add_edge(b, e, ());
/// assert_eq!(connected_components(&graph), 1);
/// ```
pub fn connected_components<G>(g: G) -> usize
where
    G: NodeCompactIndexable + IntoEdgeReferences,
{
    let mut vertex_sets = UnionFind::new(g.node_bound());
    for edge in g.edge_references() {
        let (a, b) = (edge.source(), edge.target());

        // union the two vertices of the edge
        vertex_sets.union(g.to_index(a), g.to_index(b));
    }
    let mut labels = vertex_sets.into_labeling();
    labels.sort_unstable();
    labels.dedup();
    labels.len()
}
