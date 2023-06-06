use alloc::{vec, vec::Vec};

use petgraph_core::{edge::EdgeType, index::IndexType};
use petgraph_graph::{Graph, NodeIndex};

use crate::components::kosaraju_scc;

/// [Graph] Condense every strongly connected component into a single node and return the result.
///
/// If `make_acyclic` is true, self-loops and multi edges are ignored, guaranteeing that
/// the output is acyclic.
/// # Example
/// ```rust
/// use petgraph_adjacency_matrix::NodeIndex;
/// use petgraph_algorithms::components::condensation;
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
///     (b, e),
///     (e, f),
///     (f, g),
///     (g, h),
///     (h, e),
/// ]);
///
/// // a ----> b ----> e ----> f
/// // ^       |       ^       |
/// // |       v       |       v
/// // d <---- c       h <---- g
///
/// let condensed_graph = condensation(graph, false);
/// let A = NodeIndex::new(0);
/// let B = NodeIndex::new(1);
/// assert_eq!(condensed_graph.node_count(), 2);
/// assert_eq!(condensed_graph.edge_count(), 9);
/// assert_eq!(condensed_graph.neighbors(A).collect::<Vec<_>>(), vec![
///     A, A, A, A
/// ]);
/// assert_eq!(condensed_graph.neighbors(B).collect::<Vec<_>>(), vec![
///     A, B, B, B, B
/// ]);
/// ```
/// If `make_acyclic` is true, self-loops and multi edges are ignored:
///
/// ```rust
/// # use petgraph_algorithms::components::condensation;
/// # use petgraph_core::edge::Directed;
/// # use petgraph_graph::Graph;
/// #
/// # let mut graph : Graph<(),(), Directed> = Graph::new();
/// # let a = graph.add_node(()); // node with no weight
/// # let b = graph.add_node(());
/// # let c = graph.add_node(());
/// # let d = graph.add_node(());
/// # let e = graph.add_node(());
/// # let f = graph.add_node(());
/// # let g = graph.add_node(());
/// # let h = graph.add_node(());
/// #
/// # graph.extend_with_edges(&[
/// #    (a, b),
/// #    (b, c),
/// #    (c, d),
/// #    (d, a),
/// #    (b, e),
/// #    (e, f),
/// #    (f, g),
/// #    (g, h),
/// #    (h, e)
/// # ]);
/// let acyclic_condensed_graph = condensation(graph, true);
/// let A = NodeIndex::new(0);
/// let B = NodeIndex::new(1);
/// assert_eq!(acyclic_condensed_graph.node_count(), 2);
/// assert_eq!(acyclic_condensed_graph.edge_count(), 1);
/// assert_eq!(
///     acyclic_condensed_graph.neighbors(B).collect::<Vec<_>>(),
///     vec![A]
/// );
/// ```
pub fn condensation<N, E, Ty, Ix>(
    g: Graph<N, E, Ty, Ix>,
    make_acyclic: bool,
) -> Graph<Vec<N>, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    let sccs = kosaraju_scc(&g);
    let mut condensed: Graph<Vec<N>, E, Ty, Ix> = Graph::with_capacity(sccs.len(), g.edge_count());

    // Build a map from old indices to new ones.
    let mut node_map = vec![NodeIndex::end(); g.node_count()];
    for comp in sccs {
        let new_nix = condensed.add_node(Vec::new());
        for nix in comp {
            node_map[nix.index()] = new_nix;
        }
    }

    // Consume nodes and edges of the old graph and insert them into the new one.
    let (nodes, edges) = g.into_nodes_edges();
    for (nix, node) in nodes.into_iter().enumerate() {
        condensed[node_map[nix]].push(node.weight);
    }
    for edge in edges {
        let source = node_map[edge.source().index()];
        let target = node_map[edge.target().index()];
        if make_acyclic {
            if source != target {
                condensed.update_edge(source, target, edge.weight);
            }
        } else {
            condensed.add_edge(source, target, edge.weight);
        }
    }
    condensed
}

#[cfg(test)]
mod tests {
    use petgraph_core::visit::EdgeRef;
    use petgraph_graph::{Graph, NodeIndex};

    /// Sets up the example graph
    ///
    /// Uses the graph from: <https://en.wikipedia.org/wiki/Strongly_connected_component>
    fn setup() -> Graph<&'static str, &'static str> {
        let mut graph = Graph::new();

        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");
        let d = graph.add_node("D");
        let e = graph.add_node("E");
        let f = graph.add_node("F");
        let g = graph.add_node("G");
        let h = graph.add_node("H");

        graph.extend_with_edges([
            (a, b, "A → B"), //
            (b, c, "B → C"),
            (b, e, "B → E"),
            (b, f, "B → F"),
            (c, d, "C → D"),
            (c, g, "C → G"),
            (d, c, "D → C"),
            (d, h, "D → H"),
            (e, a, "E → A"),
            (e, f, "E → F"),
            (f, g, "F → G"),
            (g, f, "G → F"),
            (h, d, "H → D"),
            (h, g, "H → G"),
        ]);

        graph
    }

    #[test]
    fn acyclic() {
        let graph = setup();

        let condensed = super::condensation(graph, true);

        assert_eq!(condensed.node_count(), 3);
        assert_eq!(condensed.edge_count(), 3);

        // The graph should look like this:
        // A → B
        //   ↘ ↓
        //     C
        let a = condensed
            .node_weight(NodeIndex::new(0))
            .expect("first strongly connected component");
        let b = condensed
            .node_weight(NodeIndex::new(1))
            .expect("second strongly connected component");
        let c = condensed
            .node_weight(NodeIndex::new(2))
            .expect("third strongly connected component");

        assert_eq!(a, &["F", "G"]);
        assert_eq!(b, &["C", "D", "H"]);
        assert_eq!(c, &["A", "B", "E"]);

        let a = NodeIndex::new(0);
        let b = NodeIndex::new(1);
        let c = NodeIndex::new(2);

        assert!(condensed.find_edge(c, a).is_some());
        assert!(condensed.find_edge(c, b).is_some());
        assert!(condensed.find_edge(b, a).is_some());
    }

    #[test]
    fn not_acyclic() {
        let graph = setup();

        let condensed = super::condensation(graph, false);

        assert_eq!(condensed.node_count(), 3);
        assert_eq!(condensed.edge_count(), 14);

        // The graph should look like this:
        // A → B
        //   ↘ ↓
        //     C
        // but here we do not condense any edges
        let a = condensed
            .node_weight(NodeIndex::new(0))
            .expect("first strongly connected component");
        let b = condensed
            .node_weight(NodeIndex::new(1))
            .expect("second strongly connected component");
        let c = condensed
            .node_weight(NodeIndex::new(2))
            .expect("third strongly connected component");

        assert_eq!(a, &["F", "G"]);
        assert_eq!(b, &["C", "D", "H"]);
        assert_eq!(c, &["A", "B", "E"]);

        let a = NodeIndex::new(0);
        let b = NodeIndex::new(1);
        let c = NodeIndex::new(2);

        assert_eq!(condensed.edges(c).filter(|e| e.target() == a).count(), 2);
        assert_eq!(condensed.edges(c).filter(|e| e.target() == b).count(), 1);
        assert_eq!(condensed.edges(b).filter(|e| e.target() == a).count(), 2);

        assert_eq!(condensed.edges(a).filter(|e| e.target() == a).count(), 2);
        assert_eq!(condensed.edges(b).filter(|e| e.target() == b).count(), 4);
        assert_eq!(condensed.edges(c).filter(|e| e.target() == c).count(), 3);
    }
}
