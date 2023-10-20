use petgraph_core::deprecated::visit::{
    EdgeRef, IntoEdgeReferences, NodeCompactIndexable, NodeRef,
};

use crate::utilities::union_find::UnionFind;

/// \[Generic\] Return the number of connected components of the graph.
///
/// For a directed graph, this is the *weakly* connected components.
/// # Example
/// ```rust
/// use petgraph_algorithms::components::connected_components;
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

#[cfg(test)]
mod tests {
    use alloc::format;

    use petgraph_core::edge::Directed;
    use petgraph_graph::Graph;
    use proptest::prelude::*;

    use crate::components::kosaraju_scc;

    #[test]
    fn link() {
        let mut graph = Graph::new();

        let a = graph.add_node("A");
        let b = graph.add_node("B");

        assert_eq!(super::connected_components(&graph), 2);

        graph.add_edge(a, b, "A → B");

        assert_eq!(super::connected_components(&graph), 1);

        graph.add_edge(b, a, "B → A");

        assert_eq!(super::connected_components(&graph), 1);
    }

    #[test]
    fn triangle() {
        let mut graph = Graph::new();

        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");

        assert_eq!(super::connected_components(&graph), 3);

        graph.add_edge(a, b, "A → B");
        graph.add_edge(b, c, "B → C");
        graph.add_edge(c, a, "C → A");

        assert_eq!(super::connected_components(&graph), 1);
    }

    #[test]
    fn square() {
        let mut graph = Graph::new();

        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");
        let d = graph.add_node("D");

        assert_eq!(super::connected_components(&graph), 4);

        // A → B
        //
        // C → D
        graph.add_edge(a, b, "A → B");
        graph.add_edge(c, d, "C → D");

        assert_eq!(super::connected_components(&graph), 2);

        // A → B
        // ↑   ↓
        // D ← C
        graph.add_edge(b, c, "B → C");
        graph.add_edge(d, a, "D → A");

        assert_eq!(super::connected_components(&graph), 1);
        // also a strongly connected component
        assert_eq!(kosaraju_scc(&graph).len(), 1);
    }

    #[test]
    fn wcc_but_not_scc() {
        let mut graph = Graph::new();

        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");
        let d = graph.add_node("D");

        assert_eq!(super::connected_components(&graph), 4);

        // A → B
        //     ↓
        // D ← C
        graph.add_edge(a, b, "A → B");
        graph.add_edge(c, d, "C → D");
        graph.add_edge(b, c, "B → C");

        assert_eq!(super::connected_components(&graph), 1);
        // but not a strongly connected component
        assert_eq!(kosaraju_scc(&graph).len(), 4);
    }

    #[cfg(not(miri))]
    proptest! {
        #[test]
        fn invariant_over_direction(directed in any::<Graph<(), (), Directed, u8>>()) {
            let undirected = directed.clone().into_edge_type::<petgraph_core::edge::Undirected>();

            prop_assert_eq!(
                super::connected_components(&directed),
                super::connected_components(&undirected)
            );
        }
    }
}
