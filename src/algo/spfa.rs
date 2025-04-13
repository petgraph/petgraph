//! Shortest Path Faster Algorithm.
use super::{bellman_ford::Paths, BoundedMeasure, NegativeCycle};
use crate::prelude::*;
use crate::visit::{IntoEdges, IntoNodeIdentifiers, NodeIndexable};
use alloc::{vec, vec::Vec};

/// \[Generic\] Compute shortest paths from node `source` to all other.
///
/// Using the [Shortest Path Faster Algorithm][spfa].
/// Compute shortest distances from node `source` to all other.
///
/// Compute shortest paths lengths in a weighted graph with positive or negative edge weights,
/// but with no negative cycles.
///
/// ## Arguments
/// * `graph`: weighted graph.
/// * `source`: the source vertex, for which we calculate the lengths of the shortest paths to all the others.
/// * `edge_cost`: closure that returns cost of a particular edge
///
/// ## Returns
/// * `Err`: if graph contains negative cycle.
/// * `Ok`: a pair of a vector of shortest distances and a vector
///   of predecessors of each vertex along the shortest path.
///
/// [spfa]: https://www.geeksforgeeks.org/shortest-path-faster-algorithm/
///
/// # Example
///
/// ```
/// use petgraph::Graph;
/// use petgraph::algo::spfa;
///
/// let mut g = Graph::new();
/// let a = g.add_node(()); // node with no weight
/// let b = g.add_node(());
/// let c = g.add_node(());
/// let d = g.add_node(());
/// let e = g.add_node(());
/// let f = g.add_node(());
/// g.extend_with_edges(&[
///     (0, 1, 3.0),
///     (0, 3, 2.0),
///     (1, 2, 1.0),
///     (1, 5, 7.0),
///     (2, 4, -4.0),
///     (3, 4, -1.0),
///     (4, 5, 1.0),
/// ]);
///
/// // Graph represented with the weight of each edge.
/// //
/// //     3       1
/// // a ----- b ----- c
/// // | 2     | 7     |
/// // d       f       | -4
/// // | -1    | 1     |
/// // \------ e ------/
///
/// let path = spfa(&g, a, |edge| *edge.weight());
/// assert!(path.is_ok());
/// let path = path.unwrap();
/// assert_eq!(path.distances, vec![0.0 ,     3.0,     4.0,    2.0,     0.0,     1.0]);
/// assert_eq!(path.predecessors, vec![None, Some(a), Some(b), Some(a), Some(c), Some(e)]);
///
///
/// // Negative cycle.
/// let graph = Graph::<(), f32>::from_edges(&[
///     (0, 1, 2.0), (1, 2, 2.0), (2, 0, -10.0)]);
///
/// assert!(spfa(&graph, 0.into(), |edge| *edge.weight()).is_err());
/// ```
pub fn spfa<G, F, K>(
    graph: G,
    source: G::NodeId,
    mut edge_cost: F,
) -> Result<Paths<G::NodeId, K>, NegativeCycle>
where
    G: IntoEdges + IntoNodeIdentifiers + NodeIndexable,
    F: FnMut(G::EdgeRef) -> K,
    K: BoundedMeasure + Copy,
{
    let ix = |i| graph.to_index(i);

    let mut predecessors = vec![None; graph.node_bound()];
    let mut distances = vec![K::max(); graph.node_bound()];
    distances[ix(source)] = K::default();

    // Queue of vertices capable of relaxation of the found shortest distances.
    let mut queue: Vec<G::NodeId> = Vec::with_capacity(graph.node_bound());
    let mut in_queue = vec![false; graph.node_bound()];

    queue.push(source);
    in_queue[ix(source)] = true;

    // Keep track of how many times each vertex appeared
    // in the queue to be able to detect a negative cycle.
    let mut visits = vec![0; graph.node_bound()];

    while let Some(i) = queue.pop() {
        in_queue[ix(i)] = false;

        // In a graph without a negative cycle, no vertex can improve
        // the shortest distances by more than |V| times.
        if visits[ix(i)] >= graph.node_bound() {
            return Err(NegativeCycle(()));
        }
        visits[ix(i)] += 1;

        for edge in graph.edges(i) {
            let j = edge.target();
            let w = edge_cost(edge);

            let (dist, overflow) = distances[ix(i)].overflowing_add(w);

            if !overflow && dist < distances[ix(j)] {
                distances[ix(j)] = dist;
                predecessors[ix(j)] = Some(i);

                if !in_queue[ix(j)] {
                    in_queue[ix(j)] = true;
                    queue.push(j);
                }
            }
        }
    }

    Ok(Paths {
        distances,
        predecessors,
    })
}
