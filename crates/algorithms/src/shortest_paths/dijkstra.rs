use alloc::collections::BinaryHeap;
use core::hash::Hash;

use indexmap::{map::Entry, IndexMap};
use petgraph_core::visit::{EdgeRef, IntoEdges, VisitMap, Visitable};

use crate::{shortest_paths::Measure, utilities::min_scored::MinScored};

/// \[Generic\] Dijkstra's shortest path algorithm.
///
/// Compute the length of the shortest path from `start` to every reachable
/// node.
///
/// The graph should be `Visitable` and implement `IntoEdges`. The function
/// `edge_cost` should return the cost for a particular edge, which is used
/// to compute path costs. Edge costs must be non-negative.
///
/// If `goal` is not `None`, then the algorithm terminates once the `goal` node's
/// cost is calculated.
///
/// Returns a `HashMap` that maps `NodeId` to path cost.
/// # Example
/// ```rust
/// use std::collections::HashMap;
///
/// use petgraph_algorithms::shortest_paths::dijkstra;
/// use petgraph_core::edge::Directed;
/// use petgraph_graph::{Graph, NodeIndex};
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
/// let expected_res: HashMap<NodeIndex, usize> = [
///     (a, 3),
///     (b, 0),
///     (c, 1),
///     (d, 2),
///     (e, 1),
///     (f, 2),
///     (g, 3),
///     (h, 4),
/// ]
/// .iter()
/// .cloned()
/// .collect();
/// let res = dijkstra(&graph, b, None, |_| 1);
/// assert_eq!(res, expected_res);
/// // z is not inside res because there is not path from b to z.
/// ```
pub fn dijkstra<G, F, K>(
    graph: G,
    start: G::NodeId,
    goal: Option<G::NodeId>,
    mut edge_cost: F,
) -> IndexMap<G::NodeId, K>
where
    G: IntoEdges + Visitable,
    G::NodeId: Eq + Hash,
    F: FnMut(G::EdgeRef) -> K,
    K: Measure + Copy,
{
    let mut visited = graph.visit_map();
    let mut scores = IndexMap::new();
    //let mut predecessor = HashMap::new();
    let mut visit_next = BinaryHeap::new();
    let zero_score = K::default();
    scores.insert(start, zero_score);
    visit_next.push(MinScored(zero_score, start));
    while let Some(MinScored(node_score, node)) = visit_next.pop() {
        if visited.is_visited(&node) {
            continue;
        }
        if goal.as_ref() == Some(&node) {
            break;
        }
        for edge in graph.edges(node) {
            let next = edge.target();
            if visited.is_visited(&next) {
                continue;
            }
            let next_score = node_score + edge_cost(edge);
            match scores.entry(next) {
                Entry::Occupied(ent) => {
                    if next_score < *ent.get() {
                        *ent.into_mut() = next_score;
                        visit_next.push(MinScored(next_score, next));
                        //predecessor.insert(next.clone(), node.clone());
                    }
                }
                Entry::Vacant(ent) => {
                    ent.insert(next_score);
                    visit_next.push(MinScored(next_score, next));
                    //predecessor.insert(next.clone(), node.clone());
                }
            }
        }
        visited.visit(node);
    }
    scores
}

#[cfg(test)]
pub(super) mod tests {
    use petgraph_core::{
        edge::{Directed, Undirected},
        visit::{EdgeRef, IntoNodeReferences},
    };
    use petgraph_graph::{Graph, NodeIndex};
    use proptest::{prelude::*, sample::Index};

    use super::dijkstra;

    /// Uses the graph from networkx
    ///
    /// <https://github.com/networkx/networkx/blob/main/networkx/algorithms/shortest_paths/tests/test_weighted.py>
    pub fn setup() -> Graph<&'static str, i32> {
        let mut graph = Graph::new();

        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");
        let d = graph.add_node("D");
        let e = graph.add_node("E");

        graph.extend_with_edges([
            (a, b, 10),
            (a, c, 5),
            (b, d, 1),
            (b, c, 2),
            (d, e, 1),
            (c, b, 3),
            (c, d, 5),
            (c, e, 2),
            (e, a, 7),
            (e, d, 6),
        ]);

        graph
    }

    #[test]
    fn no_goal_directed() {
        let graph = setup();

        let node = |weight: &'static str| {
            graph
                .node_references()
                .find(|(_, &node_weight)| node_weight == weight)
                .unwrap()
                .0
        };

        let result = dijkstra(&graph, node("A"), None, |edge| *edge.weight());

        let expected = [
            (node("A"), 0),
            (node("B"), 8),
            (node("C"), 5),
            (node("D"), 9),
            (node("E"), 7),
        ];

        assert_eq!(result.len(), expected.len());

        for (node, weight) in expected {
            assert_eq!(result[&node], weight);
        }
    }

    #[test]
    fn no_goal_undirected() {
        let graph = setup().into_edge_type::<Undirected>();

        let node = |weight: &'static str| {
            graph
                .node_references()
                .find(|(_, &node_weight)| node_weight == weight)
                .unwrap()
                .0
        };

        let result = dijkstra(&graph, node("A"), None, |edge| *edge.weight());

        let expected = [
            (node("A"), 0),
            (node("B"), 7),
            (node("C"), 5),
            (node("D"), 8),
            (node("E"), 7),
        ];

        assert_eq!(result.len(), expected.len());

        for (node, weight) in expected {
            assert_eq!(result[&node], weight);
        }
    }

    #[test]
    fn goal_directed() {
        let graph = setup();

        let node = |weight: &'static str| {
            graph
                .node_references()
                .find(|(_, &node_weight)| node_weight == weight)
                .unwrap()
                .0
        };

        let result = dijkstra(&graph, node("A"), Some(node("D")), |edge| *edge.weight());

        // we only guarantee that A - D exists in the result
        assert_eq!(result[&node("D")], 9);
    }

    #[test]
    fn goal_undirected() {
        let graph = setup().into_edge_type::<Undirected>();

        let node = |weight: &'static str| {
            graph
                .node_references()
                .find(|(_, &node_weight)| node_weight == weight)
                .unwrap()
                .0
        };

        let result = dijkstra(&graph, node("A"), Some(node("D")), |edge| *edge.weight());

        // we only guarantee that A - D exists in the result
        assert_eq!(result[&node("D")], 8);
    }

    fn non_empty_graph() -> impl Strategy<Value = Graph<(), u8, Directed, u8>> {
        any::<Graph<(), u8, Directed, u8>>()
            .prop_filter("graph is empty", |graph| graph.node_count() > 0)
    }

    proptest! {
        #[test]
        fn triangle_inequality(
            graph in non_empty_graph(),
            node in any::<Index>()
        ) {
            let node = NodeIndex::new(node.index(graph.node_count()));
            let result = dijkstra(&graph, node, None, |edge| *edge.weight() as u32);

            // triangle inequality:
            // d(v,u) <= d(v,v2) + d(v2,u)
            for (node, weight) in &result {
                for edge in graph.edges(*node) {
                    let next = edge.target();
                    let next_weight = *edge.weight() as u32;

                    if result.contains_key(&next) {
                        assert!(result[&next] <= *weight + next_weight);
                    }
                }
            }
        }
    }
}
