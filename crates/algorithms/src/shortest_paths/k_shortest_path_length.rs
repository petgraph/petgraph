use alloc::{collections::BinaryHeap, vec, vec::Vec};
use core::hash::Hash;

use indexmap::IndexMap;
use petgraph_core::visit::{EdgeRef, IntoEdges, NodeCount, NodeIndexable, Visitable};

use crate::{shortest_paths::Measure, utilities::min_scored::MinScored};

/// \[Generic\] k'th shortest path algorithm.
///
/// Compute the length of the k'th shortest path from `start` to every reachable
/// node.
///
/// The graph should be `Visitable` and implement `IntoEdges`. The function
/// `edge_cost` should return the cost for a particular edge, which is used
/// to compute path costs. Edge costs must be non-negative.
///
/// If `goal` is not `None`, then the algorithm terminates once the `goal` node's
/// cost is calculated.
///
/// Computes in **O(k * (|E| + |V|*log(|V|)))** time (average).
///
/// Returns a `HashMap` that maps `NodeId` to path cost.
/// # Example
/// ```rust
/// use indexmap::IndexMap;
/// use petgraph::{
///     algorithms::shortest_paths::k_shortest_path_length,
///     core::edge::Directed,
///     graph::{Graph, NodeIndex},
/// };
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
/// // z will be in another connected component
/// let z = graph.add_node(());
///
/// graph.extend_with_edges(&[
///     (a, b),
///     (b, c),
///     (c, d),
///     (d, a),
///     (e, f),
///     (b, e),
///     (f, g),
///     (g, h),
///     (h, e),
/// ]);
/// // a ----> b ----> e ----> f
/// // ^       |       ^       |
/// // |       v       |       v
/// // d <---- c       h <---- g
///
/// let expected_res: IndexMap<NodeIndex, usize> = [
///     (a, 7),
///     (b, 4),
///     (c, 5),
///     (d, 6),
///     (e, 5),
///     (f, 6),
///     (g, 7),
///     (h, 8),
/// ]
/// .iter()
/// .cloned()
/// .collect();
/// let res = k_shortest_path_length(&graph, b, None, 2, |_| 1);
/// assert_eq!(res, expected_res);
/// // z is not inside res because there is not path from b to z.
/// ```
pub fn k_shortest_path_length<G, F, K>(
    graph: G,
    start: G::NodeId,
    goal: Option<G::NodeId>,
    k: usize,
    mut edge_cost: F,
) -> IndexMap<G::NodeId, K>
where
    G: IntoEdges + Visitable + NodeCount + NodeIndexable,
    G::NodeId: Eq + Hash,
    F: FnMut(G::EdgeRef) -> K,
    K: Measure + Copy,
{
    let mut counter: Vec<usize> = vec![0; graph.node_count()];
    let mut scores = IndexMap::new();
    let mut visit_next = BinaryHeap::new();
    let zero_score = K::default();

    visit_next.push(MinScored(zero_score, start));

    while let Some(MinScored(node_score, node)) = visit_next.pop() {
        counter[graph.to_index(node)] += 1;
        let current_counter = counter[graph.to_index(node)];

        if current_counter > k {
            continue;
        }

        if current_counter == k {
            scores.insert(node, node_score);
        }

        //Already reached goal k times
        if goal.as_ref() == Some(&node) && current_counter == k {
            break;
        }

        for edge in graph.edges(node) {
            visit_next.push(MinScored(node_score + edge_cost(edge), edge.target()));
        }
    }
    scores
}

#[cfg(test)]
mod tests {
    use alloc::{format, vec::Vec};

    use indexmap::IndexMap;
    use petgraph_core::edge::Directed;
    use petgraph_graph::{Graph, NodeIndex};
    use proptest::{prelude::*, sample::Index};

    use crate::shortest_paths::{dijkstra, k_shortest_path_length};

    /// Graph:
    ///
    /// ```text
    /// A → B → C → D → E
    ///     ↓   ↓     ↙   ↖
    ///     F → G → H → I → M
    ///           ↙ ↓ ↘   ↗
    ///         L → K ← J
    ///          ↘ ↙
    ///           M
    /// ```
    #[test]
    fn integration_second_shortest_path() {
        let mut graph: Graph<(), (), Directed> = Graph::new();

        let a = graph.add_node(());
        let b = graph.add_node(());
        let c = graph.add_node(());
        let d = graph.add_node(());
        let e = graph.add_node(());
        let f = graph.add_node(());
        let g = graph.add_node(());
        let h = graph.add_node(());
        let i = graph.add_node(());
        let j = graph.add_node(());
        let k = graph.add_node(());
        let l = graph.add_node(());
        let m = graph.add_node(());

        graph.extend_with_edges(&[
            (a, b),
            (b, c),
            (c, d),
            (b, f),
            (f, g),
            (c, g),
            (g, h),
            (d, e),
            (e, h),
            (h, i),
            (h, j),
            (h, k),
            (h, l),
            (i, m),
            (l, k),
            (j, k),
            (j, m),
            (k, m),
            (l, m),
            (m, e),
        ]);

        let result = k_shortest_path_length(&graph, a, None, 2, |_| 1);

        let expected: IndexMap<NodeIndex, usize> = [
            (e, 7),
            (g, 3),
            (h, 4),
            (i, 5),
            (j, 5),
            (k, 5),
            (l, 5),
            (m, 6),
        ]
        .into_iter()
        .collect();

        assert_eq!(result, expected);
    }

    fn non_empty_graph() -> impl Strategy<Value = Graph<(), u8, Directed, u8>> {
        any::<Graph<(), u8, Directed, u8>>()
            .prop_filter("graph must not be empty", |graph| graph.node_count() > 0)
    }

    #[cfg(not(miri))]
    proptest! {
        // checks that the distances computed by k'th shortest path is always greater
        // or equal compared to their dijkstra computation
        #[test]
        fn kth_shortest_path_longer_than_dijkstra(graph in non_empty_graph(), index in any::<Index>()) {
            let node = graph.node_indices().nth(index.index(graph.node_count())).unwrap();

            let second_shortest_path = k_shortest_path_length(&graph, node, None, 2, |edge| *edge.weight() as u32);
            let dijkstra = dijkstra(&graph, node, None, |edge| *edge.weight() as u32);

            for (node, distance) in second_shortest_path {
                prop_assert!(dijkstra.get(&node).unwrap() <= &distance);
            }
        }
    }
}
