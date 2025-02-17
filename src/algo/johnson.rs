//! Johnson's algorithm implementation.
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Sub;

use super::{dijkstra, spfa::spfa_loop};
pub use super::{BoundedMeasure, NegativeCycle};
use crate::visit::{EdgeRef, IntoEdges, IntoNodeIdentifiers, NodeIndexable, Visitable};

/// [Johnson algorithm][johnson] for all pairs shortest path problem.
///
/// Сompute the lengths of shortest paths in a weighted graph with
/// positive or negative edge weights, but no negative cycles.
///
/// ## Arguments
/// * `graph`: weighted graph.
/// * `edge_cost`: closure that returns cost of a particular edge.
///
/// ## Returns
/// * `Err`: if graph contains negative cycle.
/// * `Ok`: `HashMap` of shortest distances.
///
/// [johnson]: https://en.wikipedia.org/wiki/Johnson%27s_algorithm
///
/// # Examples
///
/// ```
/// use petgraph::{prelude::*, Graph, Directed};
/// use petgraph::algo::johnson;
/// use std::collections::HashMap;
///
/// let mut graph: Graph<(), i32, Directed> = Graph::new();
/// let a = graph.add_node(());
/// let b = graph.add_node(());
/// let c = graph.add_node(());
/// let d = graph.add_node(());
///
/// graph.extend_with_edges(&[
///    (a, b, 1),
///    (a, c, 4),
///    (a, d, 10),
///    (b, c, 2),
///    (b, d, 2),
///    (c, d, 2)
/// ]);
///
/// //     ----- b --------
/// //    |      ^         | 2
/// //    |    1 |    4    v
/// //  2 |      a ------> c
/// //    |   10 |         | 2
/// //    |      v         v
/// //     --->  d <-------
///
/// let expected_res: HashMap<(NodeIndex, NodeIndex), i32> = [
///    ((a, a), 0), ((a, b), 1), ((a, c), 3), ((a, d), 3),
///    ((b, b), 0), ((b, c), 2), ((b, d), 2),
///    ((c, c), 0), ((c, d), 2),
///    ((d, d), 0),
/// ].iter().cloned().collect();
///
///
/// let res = johnson(&graph, |edge| {
///     *edge.weight()
/// }).unwrap();
///
/// let nodes = [a, b, c, d];
/// for node1 in &nodes {
///     for node2 in &nodes {
///         assert_eq!(res.get(&(*node1, *node2)), expected_res.get(&(*node1, *node2)));
///     }
/// }
/// ```
#[allow(clippy::type_complexity)]
pub fn johnson<G, F, K>(
    graph: G,
    mut edge_cost: F,
) -> Result<HashMap<(G::NodeId, G::NodeId), K>, NegativeCycle>
where
    G: IntoEdges + IntoNodeIdentifiers + NodeIndexable + Visitable,
    G::NodeId: Eq + Hash,
    F: FnMut(G::EdgeRef) -> K,
    K: BoundedMeasure + Copy + Sub<K, Output = K>,
{
    // Add a virtual node to the graph with oriented edges with zero weight
    // to all other vertices, and then run SPFA from it.
    // The found distances will be used to change the edge weights in Dijkstra's
    // algorithm to make them non-negative.

    let ix = |i| graph.to_index(i);
    let node_bound = graph.node_bound();

    let reweight = vec![K::default(); node_bound];

    // Queue of vertices capable of relaxation of the found shortest distances.
    let mut queue: Vec<G::NodeId> = Vec::with_capacity(node_bound);

    // Adding all vertices to the queue is the same as starting the algorithm from a virtual node.
    queue.extend(graph.node_identifiers());
    let in_queue = vec![true; node_bound];

    let (reweight, _) = spfa_loop(graph, reweight, None, queue, in_queue, &mut edge_cost)?;

    let mut distance_map: HashMap<(G::NodeId, G::NodeId), K> =
        HashMap::with_capacity(node_bound * node_bound);

    // Reweight edges.
    let mut new_cost = |edge: G::EdgeRef| {
        let (sum, _overflow) = edge_cost(edge).overflowing_add(reweight[ix(edge.source())]);
        debug_assert!(!_overflow);
        sum - reweight[ix(edge.target())]
    };

    // Run Dijkstra's algorithm from each node.
    for source in graph.node_identifiers() {
        for (target, dist) in dijkstra(graph, source, None, &mut new_cost) {
            distance_map.insert(
                (source, target),
                dist + reweight[ix(target)] - reweight[ix(source)],
            );
        }
    }

    Ok(distance_map)
}
