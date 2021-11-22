//! Shortest Path Faster Algorithm.

use crate::algo::bellman_ford::Paths;
use crate::algo::{FloatMeasure, NegativeCycle};
use crate::visit::{EdgeRef, IntoEdges, IntoNodeIdentifiers, NodeIndexable};
use std::collections::VecDeque;

/// \[Generic\] [Shortest Path Faster Algorithm](https://en.wikipedia.org/wiki/Shortest_Path_Faster_Algorithm).
/// Compute shortest distances from node `source` to all other.
///
/// Compute shortest paths lengths in a weighted graph with positive or negative edge weights,
/// but with no negative cycles.
///
/// This algorithm is an improved version of the Bellman-Ford and has an average time complexity
/// of **O(|E|)**, but in the worst case it is still **O(|V|*|E|)**, which equals to the usual Bellman-Ford.
/// However, in some cases, SPFA still loses due to a larger runtime constant.
/// It is believed that SPFA gives the maximum performance gain on sparse graphs.
///
/// ## Arguments
/// * `graph`: weighted graph.
/// * `source`: the source vertex, for which we calculate the lengths of the shortest paths to all the others.
///
/// ## Returns
/// * `Err`: if graph contains negative cycle.
/// * `Ok`: a pair of a vector of shortest distances and a vector
///         of predecessors of each vertex along the shortest path.
///
/// # Examples
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
/// let path = spfa(&g, a);
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
/// assert!(spfa(&graph, 0.into()).is_err());
/// ```
pub fn spfa<G>(
    graph: G,
    source: G::NodeId,
) -> Result<Paths<G::NodeId, G::EdgeWeight>, NegativeCycle>
where
    G: IntoEdges + IntoNodeIdentifiers + NodeIndexable,
    G::EdgeWeight: FloatMeasure,
{
    let ix = |i| graph.to_index(i);

    let mut predecessors = vec![None; graph.node_bound()];
    let mut distances = vec![<_>::infinite(); graph.node_bound()];
    distances[ix(source)] = <_>::zero();

    // Queue of vertices capable of relaxation of the found shortest distances.
    let mut queue: VecDeque<G::NodeId> = VecDeque::with_capacity(graph.node_bound());
    let mut in_queue = vec![false; graph.node_bound()];
    queue.push_back(source);
    in_queue[ix(source)] = true;

    // We will keep track of how many times each vertex appeared
    // in the queue to be able to detect a negative cycle.
    let mut visits = vec![0; graph.node_bound()];

    while !queue.is_empty() {
        let i = queue.pop_front().unwrap();
        in_queue[ix(i)] = false;

        // In a graph without a negative cycle, no vertex can improve
        // the shortest distances by more than |V| times.
        if visits[ix(i)] >= graph.node_bound() {
            return Err(NegativeCycle(()));
        }
        visits[ix(i)] += 1;

        for edge in graph.edges(i) {
            let j = edge.target();
            let w = *edge.weight();

            if distances[ix(i)] + w < distances[ix(j)] {
                distances[ix(j)] = distances[ix(i)] + w;
                predecessors[ix(j)] = Some(i);

                if !in_queue[ix(j)] {
                    in_queue[ix(j)] = true;
                    queue.push_back(j);
                }
            }
        }
    }

    Ok(Paths {
        distances,
        predecessors,
    })
}
