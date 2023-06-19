use petgraph_core::{edge::EdgeType, index::IndexType, visit::IntoNodeReferences};
use petgraph_graph::Graph;

/// \[Generic\] complement of the graph
///
/// Computes the graph complement of the input Graph and stores it
/// in the provided empty output Graph.
///
/// The function does not create self-loops.
///
/// Computes in **O(|V|^2*log(|V|))** time (average).
///
/// Returns the complement.
///
/// # Example
/// ```rust
/// use petgraph_algorithms::operators::complement;
/// use petgraph_core::edge::Directed;
/// use petgraph_graph::Graph;
///
/// let mut graph: Graph<(), (), Directed> = Graph::new();
/// let a = graph.add_node(()); // node with no weight
/// let b = graph.add_node(());
/// let c = graph.add_node(());
/// let d = graph.add_node(());
///
/// graph.extend_with_edges(&[(a, b), (b, c), (c, d)]);
/// // a ----> b ----> c ----> d
///
/// let mut output: Graph<(), (), Directed> = Graph::new();
///
/// complement(&graph, &mut output, ());
///
/// let mut expected_res: Graph<(), (), Directed> = Graph::new();
/// let a = expected_res.add_node(());
/// let b = expected_res.add_node(());
/// let c = expected_res.add_node(());
/// let d = expected_res.add_node(());
/// expected_res.extend_with_edges(&[
///     (a, c),
///     (a, d),
///     (b, a),
///     (b, d),
///     (c, a),
///     (c, b),
///     (d, a),
///     (d, b),
///     (d, c),
/// ]);
///
/// for x in graph.node_indices() {
///     for y in graph.node_indices() {
///         assert_eq!(output.contains_edge(x, y), expected_res.contains_edge(x, y));
///     }
/// }
/// ```
// TODO: rework, make generic, weight over fn, doesn't support parallel edges
pub fn complement<N, E, Ty, Ix>(
    input: &Graph<N, E, Ty, Ix>,
    output: &mut Graph<N, E, Ty, Ix>,
    weight: E,
) where
    Ty: EdgeType,
    Ix: IndexType,
    E: Clone,
    N: Clone,
{
    for (_node, weight) in input.node_references() {
        output.add_node(weight.clone());
    }
    for x in input.node_indices() {
        for y in input.node_indices() {
            if x != y && !input.contains_edge(x, y) {
                output.add_edge(x, y, weight.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::{format, sync::Arc, vec::Vec};

    use petgraph_core::{edge::Directed, visit::EdgeRef};
    use petgraph_graph::Graph;
    use petgraph_proptest::default::graph_strategy;
    use proptest::{collection::SizeRange, prelude::*};

    use crate::operators::complement;

    /// Graph:
    ///
    /// ```text
    /// a → b → c → d
    /// ```
    #[test]
    fn simple() {
        let mut graph: Graph<(), (), Directed> = Graph::new();
        let a = graph.add_node(());
        let b = graph.add_node(());
        let c = graph.add_node(());
        let d = graph.add_node(());

        graph.extend_with_edges([
            (a, b), //
            (b, c),
            (c, d),
        ]);

        let mut output: Graph<(), (), Directed> = Graph::new();
        complement(&graph, &mut output, ());

        let expected = [
            (a, c), //
            (a, d),
            (b, a),
            (b, d),
            (c, a),
            (c, b),
            (d, a),
            (d, b),
            (d, c),
        ];

        let result = output
            .edge_references()
            .map(|e| (e.source(), e.target()))
            .collect::<Vec<_>>();

        assert_eq!(result, expected);
    }

    proptest! {
        #[test]
        fn complement_of_complement_is_original(
            graph in graph_strategy::<Graph::<(), (), Directed, u32>>(
                false,
                false,
                0..u8::MAX as usize,
                Some(Arc::new(|max| {
                    SizeRange::new(0..=usize::min(max.pow(2), (u8::MAX as usize) - 1))
                }))
            ),
        ) {
            let mut output = Graph::default();
            let mut output2 = Graph::default();
            complement(&graph, &mut output, ());
            complement(&output, &mut output2, ());

            for x in graph.node_indices() {
                let received = output2.edges(x).map(|edge| edge.target()).collect::<Vec<_>>();
                let expected = graph.edges(x).map(|edge| edge.target()).collect::<Vec<_>>();

                prop_assert_eq!(received, expected);
            }
        }
    }
}
