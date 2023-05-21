use alloc::vec;
use core::hash::Hash;

use indexmap::IndexMap;
use petgraph_core::visit::{
    EdgeRef, GraphProp, IntoEdgeReferences, IntoNodeIdentifiers, NodeCompactIndexable,
};

use crate::{error::NegativeCycleError, shortest_paths::BoundedMeasure};

/// \[Generic\] [Floyd–Warshall algorithm](https://en.wikipedia.org/wiki/Floyd%E2%80%93Warshall_algorithm) is an algorithm for all pairs shortest path problem
///
/// Compute shortest paths in a weighted graph with positive or negative edge weights (but with no
/// negative cycles)
///
/// # Arguments
/// * `graph`: graph with no negative cycle
/// * `edge_cost`: closure that returns cost of a particular edge
///
/// # Returns
/// * `Ok`: (if graph contains no negative cycle) a hashmap containing all pairs shortest paths
/// * `Err`: if graph contains negative cycle.
///
/// # Examples
/// ```rust
/// use std::collections::HashMap;
///
/// use petgraph_algorithms::shortest_paths::floyd_warshall;
/// use petgraph_core::edge::Directed;
/// use petgraph_graph::{Graph, NodeIndex};
///
/// let mut graph: Graph<(), (), Directed> = Graph::new();
/// let a = graph.add_node(());
/// let b = graph.add_node(());
/// let c = graph.add_node(());
/// let d = graph.add_node(());
///
/// graph.extend_with_edges(&[(a, b), (a, c), (a, d), (b, c), (b, d), (c, d)]);
///
/// let weight_map: HashMap<(NodeIndex, NodeIndex), i32> = [
///     ((a, a), 0),
///     ((a, b), 1),
///     ((a, c), 4),
///     ((a, d), 10),
///     ((b, b), 0),
///     ((b, c), 2),
///     ((b, d), 2),
///     ((c, c), 0),
///     ((c, d), 2),
/// ]
/// .iter()
/// .cloned()
/// .collect();
/// //     ----- b --------
/// //    |      ^         | 2
/// //    |    1 |    4    v
/// //  2 |      a ------> c
/// //    |   10 |         | 2
/// //    |      v         v
/// //     --->  d <-------
///
/// let inf = i32::MAX;
///
/// let expected_res: HashMap<(NodeIndex, NodeIndex), i32> = [
///     ((a, a), 0),
///     ((a, b), 1),
///     ((a, c), 3),
///     ((a, d), 3),
///     ((b, a), inf),
///     ((b, b), 0),
///     ((b, c), 2),
///     ((b, d), 2),
///     ((c, a), inf),
///     ((c, b), inf),
///     ((c, c), 0),
///     ((c, d), 2),
///     ((d, a), inf),
///     ((d, b), inf),
///     ((d, c), inf),
///     ((d, d), 0),
/// ]
/// .iter()
/// .cloned()
/// .collect();
///
/// let res = floyd_warshall(&graph, |edge| {
///     if let Some(weight) = weight_map.get(&(edge.source(), edge.target())) {
///         *weight
///     } else {
///         inf
///     }
/// })
/// .unwrap();
///
/// let nodes = [a, b, c, d];
/// for node1 in &nodes {
///     for node2 in &nodes {
///         assert_eq!(
///             res.get(&(*node1, *node2)).unwrap(),
///             expected_res.get(&(*node1, *node2)).unwrap()
///         );
///     }
/// }
/// ```
pub fn floyd_warshall<G, F, K>(
    graph: G,
    mut edge_cost: F,
) -> Result<IndexMap<(G::NodeId, G::NodeId), K>, NegativeCycleError>
where
    G: NodeCompactIndexable + IntoEdgeReferences + IntoNodeIdentifiers + GraphProp,
    G::NodeId: Eq + Hash,
    F: FnMut(G::EdgeRef) -> K,
    K: BoundedMeasure + Copy,
{
    let num_of_nodes = graph.node_count();

    // |V|x|V| matrix
    let mut dist = vec![vec![K::MAX; num_of_nodes]; num_of_nodes];

    // init distances of paths with no intermediate nodes
    for edge in graph.edge_references() {
        dist[graph.to_index(edge.source())][graph.to_index(edge.target())] = edge_cost(edge);
        if !graph.is_directed() {
            dist[graph.to_index(edge.target())][graph.to_index(edge.source())] = edge_cost(edge);
        }
    }

    // distance of each node to itself is 0(default value)
    for node in graph.node_identifiers() {
        dist[graph.to_index(node)][graph.to_index(node)] = K::default();
    }

    for k in 0..num_of_nodes {
        for i in 0..num_of_nodes {
            for j in 0..num_of_nodes {
                if let Some(result) = dist[i][k].checked_add(dist[k][j]) {
                    if result < dist[i][j] {
                        dist[i][j] = result;
                    }
                }
            }
        }
    }

    // value less than 0(default value) indicates a negative cycle
    for i in 0..num_of_nodes {
        if dist[i][i] < K::default() {
            return Err(NegativeCycleError);
        }
    }

    let mut distance_map: IndexMap<(G::NodeId, G::NodeId), K> =
        IndexMap::with_capacity(num_of_nodes * num_of_nodes);

    for i in 0..num_of_nodes {
        for j in 0..num_of_nodes {
            distance_map.insert((graph.from_index(i), graph.from_index(j)), dist[i][j]);
        }
    }

    Ok(distance_map)
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use petgraph_core::edge::{Directed, Undirected};
    use petgraph_graph::Graph;

    use crate::{error::NegativeCycleError, shortest_paths::floyd_warshall};

    /// Helper Macro to create a map of expected results
    ///
    /// Technically this macro is not necessarily needed, but it makes the test code much more
    /// readable
    macro_rules! expected {
        [$($from:ident -> $to:ident : $cost:tt),* $(,)?] => {
            {
                [$(expected!(@rule $from -> $to : $cost)),*].into_iter().collect::<IndexMap<_, _>>()
            }
        };
        (@rule $from:ident -> $to:ident : !) => {
            (($from, $to), i32::MAX)
        };
        (@rule $from:ident -> $to:ident : $cost:literal) => {
            (($from, $to), $cost)
        };
    }

    /// Graph:
    ///
    /// ```text
    /// A → B → E → F
    /// ↑   ↓   ↑   ↓
    /// D ← C   H ← G
    /// ```
    #[test]
    fn uniform_weight() {
        let mut graph: Graph<(), (), Directed> = Graph::new();
        let a = graph.add_node(());
        let b = graph.add_node(());
        let c = graph.add_node(());
        let d = graph.add_node(());
        let e = graph.add_node(());
        let f = graph.add_node(());
        let g = graph.add_node(());
        let h = graph.add_node(());

        graph.extend_with_edges([
            (a, b),
            (b, c),
            (c, d),
            (d, a),
            (e, f),
            (b, e),
            (f, g),
            (g, h),
            (h, e),
        ]);

        let expected = expected![
            a -> a: 0,
            a -> b: 1,
            a -> c: 2,
            a -> d: 3,
            a -> e: 2,
            a -> f: 3,
            a -> g: 4,
            a -> h: 5,

            b -> a: 3,
            b -> b: 0,
            b -> c: 1,
            b -> d: 2,
            b -> e: 1,
            b -> f: 2,
            b -> g: 3,
            b -> h: 4,

            c -> a: 2,
            c -> b: 3,
            c -> c: 0,
            c -> d: 1,
            c -> e: 4,
            c -> f: 5,
            c -> g: 6,
            c -> h: 7,

            d -> a: 1,
            d -> b: 2,
            d -> c: 3,
            d -> d: 0,
            d -> e: 3,
            d -> f: 4,
            d -> g: 5,
            d -> h: 6,

            e -> a: !,
            e -> b: !,
            e -> c: !,
            e -> d: !,
            e -> e: 0,
            e -> f: 1,
            e -> g: 2,
            e -> h: 3,

            f -> a: !,
            f -> b: !,
            f -> c: !,
            f -> d: !,
            f -> e: 3,
            f -> f: 0,
            f -> g: 1,
            f -> h: 2,

            g -> a: !,
            g -> b: !,
            g -> c: !,
            g -> d: !,
            g -> e: 2,
            g -> f: 3,
            g -> g: 0,
            g -> h: 1,

            h -> a: !,
            h -> b: !,
            h -> c: !,
            h -> d: !,
            h -> e: 1,
            h -> f: 2,
            h -> g: 3,
            h -> h: 0,
        ];

        let result = floyd_warshall(&graph, |_| 1i32).unwrap();

        assert_eq!(result, expected);
    }

    /// Graph:
    ///
    /// ```text
    /// A → B
    /// ↓ ⤩ ↓
    /// D ← C
    /// ```
    #[test]
    fn weighted() {
        let mut graph: Graph<(), i32, Directed> = Graph::new();
        let a = graph.add_node(());
        let b = graph.add_node(());
        let c = graph.add_node(());
        let d = graph.add_node(());

        graph.extend_with_edges([
            (a, b, 1), //
            (a, c, 4),
            (a, d, 10),
            (b, c, 2),
            (b, d, 2),
            (c, d, 2),
        ]);

        let expected = expected![
            a -> a: 0,
            a -> b: 1,
            a -> c: 3,
            a -> d: 3,

            b -> a: !,
            b -> b: 0,
            b -> c: 2,
            b -> d: 2,

            c -> a: !,
            c -> b: !,
            c -> c: 0,
            c -> d: 2,

            d -> a: !,
            d -> b: !,
            d -> c: !,
            d -> d: 0,
        ];

        let result = floyd_warshall(&graph, |edge| *edge.weight()).unwrap();

        assert_eq!(result, expected);
    }

    /// Graph:
    ///
    /// ```text
    /// A - B
    /// | x |
    /// D - C
    /// ```
    #[test]
    fn weighted_undirected() {
        let mut graph: Graph<(), i32, Undirected> = Graph::new_undirected();
        let a = graph.add_node(());
        let b = graph.add_node(());
        let c = graph.add_node(());
        let d = graph.add_node(());

        graph.extend_with_edges([
            (a, b, 1), //
            (a, c, 4),
            (a, d, 10),
            (b, d, 2),
            (c, b, 2),
            (c, d, 2),
        ]);

        let expected = expected![
            a -> a: 0,
            a -> b: 1,
            a -> c: 3,
            a -> d: 3,

            b -> a: 1,
            b -> b: 0,
            b -> c: 2,
            b -> d: 2,

            c -> a: 3,
            c -> b: 2,
            c -> c: 0,
            c -> d: 2,

            d -> a: 3,
            d -> b: 2,
            d -> c: 2,
            d -> d: 0,
        ];

        let result = floyd_warshall(&graph, |edge| *edge.weight()).unwrap();

        assert_eq!(result, expected);
    }

    /// Graph:
    ///
    /// ```text
    /// A → B
    /// ↑ ↙
    /// C
    /// ```
    #[test]
    fn negative_cycle() {
        let mut graph: Graph<(), i32, Directed> = Graph::new();

        let a = graph.add_node(());
        let b = graph.add_node(());
        let c = graph.add_node(());

        graph.extend_with_edges([
            (a, b, 1), //
            (b, c, -3),
            (c, a, 1),
        ]);

        let result = floyd_warshall(&graph, |edge| *edge.weight());
        let error = result.expect_err("expected negative cycle");

        assert_eq!(error, NegativeCycleError);
    }
}
