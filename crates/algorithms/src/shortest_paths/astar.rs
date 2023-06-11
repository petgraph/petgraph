use alloc::{collections::BinaryHeap, vec, vec::Vec};
use core::hash::Hash;

use indexmap::{map::Entry, IndexMap};
use petgraph_core::visit::{EdgeRef, GraphBase, IntoEdges, Visitable};

use crate::{shortest_paths::Measure, utilities::min_scored::MinScored};

/// \[Generic\] A* shortest path algorithm.
///
/// Computes the shortest path from `start` to `finish`, including the total path cost.
///
/// `finish` is implicitly given via the `is_goal` callback, which should return `true` if the
/// given node is the finish node.
///
/// The function `edge_cost` should return the cost for a particular edge. Edge costs must be
/// non-negative.
///
/// The function `estimate_cost` should return the estimated cost to the finish for a particular
/// node. For the algorithm to find the actual shortest path, it should be admissible, meaning that
/// it should never overestimate the actual cost to get to the nearest goal node. Estimate costs
/// must also be non-negative.
///
/// The graph should be `Visitable` and implement `IntoEdges`.
///
/// # Example
/// ```
/// use petgraph_algorithms::shortest_paths::astar;
/// use petgraph_graph::Graph;
///
/// let mut g = Graph::new();
/// let a = g.add_node((0., 0.));
/// let b = g.add_node((2., 0.));
/// let c = g.add_node((1., 1.));
/// let d = g.add_node((0., 2.));
/// let e = g.add_node((3., 3.));
/// let f = g.add_node((4., 2.));
/// g.extend_with_edges(&[
///     (a, b, 2),
///     (a, d, 4),
///     (b, c, 1),
///     (b, f, 7),
///     (c, e, 5),
///     (e, f, 1),
///     (d, e, 1),
/// ]);
///
/// // Graph represented with the weight of each edge
/// // Edges with '*' are part of the optimal path.
/// //
/// //     2       1
/// // a ----- b ----- c
/// // | 4*    | 7     |
/// // d       f       | 5
/// // | 1*    | 1*    |
/// // \------ e ------/
///
/// let path = astar(&g, a, |finish| finish == f, |e| *e.weight(), |_| 0);
/// assert_eq!(path, Some((6, vec![a, d, e, f])));
/// ```
///
/// Returns the total cost + the path of subsequent `NodeId` from start to finish, if one was
/// found.
pub fn astar<G, F, H, K, IsGoal>(
    graph: G,
    start: G::NodeId,
    mut is_goal: IsGoal,
    mut edge_cost: F,
    mut estimate_cost: H,
) -> Option<(K, Vec<G::NodeId>)>
where
    G: IntoEdges + Visitable,
    IsGoal: FnMut(G::NodeId) -> bool,
    G::NodeId: Eq + Hash,
    F: FnMut(G::EdgeRef) -> K,
    H: FnMut(G::NodeId) -> K,
    K: Measure + Copy,
{
    let mut visit_next = BinaryHeap::new();
    let mut scores = IndexMap::new(); // g-values, cost to reach the node
    let mut estimate_scores = IndexMap::new(); // f-values, cost to reach + estimate cost to goal
    let mut path_tracker = PathTracker::<G>::new();

    let zero_score = K::default();
    scores.insert(start, zero_score);
    visit_next.push(MinScored(estimate_cost(start), start));

    while let Some(MinScored(estimate_score, node)) = visit_next.pop() {
        if is_goal(node) {
            let path = path_tracker.reconstruct_path_to(node);
            let cost = scores[&node];
            return Some((cost, path));
        }

        // This lookup can be unwrapped without fear of panic since the node was necessarily scored
        // before adding it to `visit_next`.
        let node_score = scores[&node];

        match estimate_scores.entry(node) {
            Entry::Occupied(mut entry) => {
                // If the node has already been visited with an equal or lower score than now, then
                // we do not need to re-visit it.
                if *entry.get() <= estimate_score {
                    continue;
                }
                entry.insert(estimate_score);
            }
            Entry::Vacant(entry) => {
                entry.insert(estimate_score);
            }
        }

        for edge in graph.edges(node) {
            let next = edge.target();
            let next_score = node_score + edge_cost(edge);

            match scores.entry(next) {
                Entry::Occupied(mut entry) => {
                    // No need to add neighbors that we have already reached through a shorter path
                    // than now.
                    if *entry.get() <= next_score {
                        continue;
                    }
                    entry.insert(next_score);
                }
                Entry::Vacant(entry) => {
                    entry.insert(next_score);
                }
            }

            path_tracker.set_predecessor(next, node);
            let next_estimate_score = next_score + estimate_cost(next);
            visit_next.push(MinScored(next_estimate_score, next));
        }
    }

    None
}

struct PathTracker<G>
where
    G: GraphBase,
    G::NodeId: Eq + Hash,
{
    came_from: IndexMap<G::NodeId, G::NodeId>,
}

impl<G> PathTracker<G>
where
    G: GraphBase,
    G::NodeId: Eq + Hash,
{
    fn new() -> PathTracker<G> {
        PathTracker {
            came_from: IndexMap::new(),
        }
    }

    fn set_predecessor(&mut self, node: G::NodeId, previous: G::NodeId) {
        self.came_from.insert(node, previous);
    }

    fn reconstruct_path_to(&self, last: G::NodeId) -> Vec<G::NodeId> {
        let mut path = vec![last];

        let mut current = last;
        while let Some(&previous) = self.came_from.get(&current) {
            path.push(previous);
            current = previous;
        }

        path.reverse();

        path
    }
}

#[cfg(test)]
mod tests {
    //! This uses the same graph as the dijkstra test.

    use alloc::vec;

    use approx::assert_relative_eq;
    use petgraph_core::{
        edge::{Directed, Undirected},
        visit::IntoNodeReferences,
    };
    use petgraph_graph::{DiGraph, Graph, NodeIndex};
    use proptest::prelude::*;

    use super::{super::dijkstra::tests::setup, astar};
    use crate::shortest_paths::dijkstra;

    /// A* is a generalization of Dijkstra's algorithm that uses a heuristic to guide the search,
    /// therefore if the `cost_estimate` is always 0, A* should behave exactly like Dijkstra's
    /// algorithm.
    #[test]
    fn null_heuristic_directed() {
        let graph = setup();

        let node = |weight: &str| {
            graph
                .node_references()
                .find(|(_, &node_weight)| node_weight == weight)
                .unwrap()
                .0
        };

        let d = node("D");

        let path = astar(
            &graph,
            node("A"),
            |node| node == d,
            |edge| *edge.weight(),
            |_| 0,
        );

        assert_eq!(
            path,
            Some((9, vec![
                node("A"), //
                node("C"),
                node("B"),
                node("D")
            ]))
        );
    }

    #[test]
    fn null_heuristic_directed_no_path() {
        let mut graph = Graph::new();

        let a = graph.add_node("A");
        let b = graph.add_node("B");

        graph.add_edge(a, b, 1);

        let path = astar(&graph, b, |node| node == a, |edge| *edge.weight(), |_| 0);

        assert_eq!(path, None);
    }

    #[test]
    fn null_heuristic_undirected() {
        let graph = setup().into_edge_type::<Undirected>();

        let node = |weight: &str| {
            graph
                .node_references()
                .find(|(_, &node_weight)| node_weight == weight)
                .unwrap()
                .0
        };

        let d = node("D");

        let path = astar(
            &graph,
            node("A"),
            |node| node == d,
            |edge| *edge.weight(),
            |_| 0,
        );

        assert_eq!(
            path,
            Some((8, vec![
                node("A"), //
                node("E"),
                node("D")
            ]))
        );
    }

    #[test]
    fn null_heuristic_undirected_no_path() {
        let mut graph = Graph::new();

        let a = graph.add_node("A");
        let b = graph.add_node("B");

        let path = astar(&graph, b, |node| node == a, |edge| *edge.weight(), |_| 0);

        assert_eq!(path, None);
    }

    #[derive(Debug, Copy, Clone, PartialEq)]
    struct Point {
        x: f32,
        y: f32,
    }

    impl Point {
        fn distance(self, other: Self) -> f32 {
            ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
        }

        fn manhattan_distance(self, other: Self) -> f32 {
            (self.x - other.x).abs() + (self.y - other.y).abs()
        }
    }

    #[test]
    fn manhattan_heuristic() {
        let mut graph = Graph::new();

        let a = graph.add_node(Point { x: 0.0, y: 0.0 });
        let b = graph.add_node(Point { x: 2.0, y: 0.0 });
        let c = graph.add_node(Point { x: 1.0, y: 1.0 });
        let d = graph.add_node(Point { x: 0.0, y: 2.0 });
        let e = graph.add_node(Point { x: 3.0, y: 3.0 });
        let f = graph.add_node(Point { x: 4.0, y: 2.0 });
        let _ = graph.add_node(Point { x: 5.0, y: 5.0 });

        let distance = |a: NodeIndex, b: NodeIndex| {
            let a = graph[a];
            let b = graph[b];
            a.distance(b)
        };

        let expected_distance = distance(a, b) + distance(b, f);

        let edges = [
            (a, b, distance(a, b)),
            (a, d, distance(a, d)),
            (b, c, distance(b, c)),
            (b, f, distance(b, f)),
            (c, e, distance(c, e)),
            (e, f, distance(e, f)),
            (d, e, distance(d, e)),
        ];

        for (start, end, weight) in edges {
            graph.add_edge(start, end, weight);
        }

        let path = astar(
            &graph,
            a,
            |node| node == f,
            |edge| *edge.weight(),
            |node| {
                let node = graph[node];
                let end = graph[f];

                node.manhattan_distance(end)
            },
        );

        let (distance, path) = path.expect("path not found");

        assert_relative_eq!(distance, expected_distance);
        assert_eq!(path, vec![a, b, f]);
    }

    #[test]
    fn optimal_runtime() {
        let mut graph = Graph::new();

        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");
        let d = graph.add_node("D");
        let e = graph.add_node("E");

        graph.add_edge(a, b, 2);
        graph.add_edge(a, c, 3);
        graph.add_edge(b, d, 3);
        graph.add_edge(c, d, 1);
        graph.add_edge(d, e, 1);

        let mut calls = 0;

        astar(
            &graph,
            a,
            |node| node == e,
            |edge| {
                calls += 1;
                *edge.weight()
            },
            |_| 0,
        );

        // A* is runtime optimal in the sense it won't expand more nodes than needed, for the given
        // heuristic. Here, A* should expand, in order: A, B, C, D, E. This should should ask for
        // the costs of edges (A, B), (A, C), (B, D), (C, D), (D, E). Node D will be added
        // to `visit_next` twice, but should only be expanded once. If it is erroneously
        // expanded twice, it will call for (D, E) again and `times_called` will be 6.
        assert_eq!(calls, 5);
    }

    /// Excerpt from https://en.wikipedia.org/wiki/A*_search_algorithm#Admissibility_and_optimality
    ///
    /// > If the heuristic function is admissible – meaning that it never overestimates the actual
    /// > cost to get to the goal – A* is guaranteed to return a least-cost path from start to goal.
    ///
    /// If a heuristic is admissible, but inconsistent, A* will still find the optimal path, but it
    /// may expand more nodes than needed.
    ///
    /// Papers:
    /// * <https://www.sciencedirect.com/science/article/pii/S0004370211000221>
    /// * <https://citeseerx.ist.psu.edu/document?repid=rep1&type=pdf&doi=1f81b34c3729709e5d81e4d2dc33fa609b335473>
    #[test]
    fn admissible_inconsistent() {
        let mut graph = Graph::new();

        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");
        let d = graph.add_node("D");

        graph.add_edge(a, b, 3);
        graph.add_edge(b, c, 3);
        graph.add_edge(c, d, 3);
        graph.add_edge(a, c, 8);
        graph.add_edge(a, d, 10);

        let admissible_inconsistent = |n: NodeIndex| match graph[n] {
            "A" => 9,
            "B" => 6,
            _ => 0,
        };

        let optimal = astar(
            &graph,
            a,
            |n| n == d,
            |e| *e.weight(),
            admissible_inconsistent,
        );
        assert_eq!(optimal, Some((9, vec![a, b, c, d])));
    }

    fn expand_graph_value_space(graph: &DiGraph<(), u8, u8>) -> Graph<(), u64, Directed, u8> {
        graph.map(|_, _| (), |_, weight| u64::from(*weight))
    }

    prop_compose! {
        // we allow selecting the same node as start and end, because it's a valid use case.
        // we also expand the value space from the initial `u8` to `u64` to avoid overflows.
        fn graph_with_two_nodes()
           (graph in any::<DiGraph::<(), u8, u8>>().prop_filter("graph must have at least one node", |graph| graph.node_count() >= 1))
           (start in 0..graph.node_count(), end in 0..graph.node_count(), graph in Just(graph))
            -> (DiGraph<(), u64, u8>, NodeIndex<u8>, NodeIndex<u8>) {
            (expand_graph_value_space(&graph), NodeIndex::new(start), NodeIndex::new(end))
        }
    }

    proptest! {
        #[test]
        fn null_heuristic_is_dijkstra(
            (graph, start, end) in graph_with_two_nodes()
        ) {
            let astar_path = astar(&graph, start, |node| node == end, |edge| *edge.weight(), |_| 0);
            let dijkstra_path = dijkstra(&graph, start, Some(end), |edge| *edge.weight());


            match astar_path {
                None => {
                    prop_assert_eq!(dijkstra_path.get(&end), None);
                }
                Some((distance, _)) => {
                    prop_assert_eq!(dijkstra_path.get(&end), Some(&distance));
                }
            }
        }
    }
}
