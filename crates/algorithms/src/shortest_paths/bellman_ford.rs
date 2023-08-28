//! Bellman-Ford algorithms.

use alloc::{vec, vec::Vec};

use funty::Floating;
use petgraph_core::visit::{
    EdgeRef, IntoEdges, IntoNodeIdentifiers, NodeCount, NodeIndexable, VisitMap, Visitable,
};

use crate::{error::NegativeCycleError, shortest_paths::FloatMeasure};

#[derive(Debug, Clone)]
pub struct Paths<NodeId, EdgeWeight> {
    pub distances: Vec<EdgeWeight>,
    pub predecessors: Vec<Option<NodeId>>,
}

/// \[Generic\] Compute shortest paths from node `source` to all other.
///
/// Using the [Bellman–Ford algorithm][bf]; negative edge costs are
/// permitted, but the graph must not have a cycle of negative weights
/// (in that case it will return an error).
///
/// On success, return one vec with path costs, and another one which points
/// out the predecessor of a node along a shortest path. The vectors
/// are indexed by the graph's node indices.
///
/// [bf]: https://en.wikipedia.org/wiki/Bellman%E2%80%93Ford_algorithm
///
/// # Example
/// ```rust
/// use petgraph_algorithms::shortest_paths::bellman_ford;
/// use petgraph_core::edge::Undirected;
/// use petgraph_graph::{Graph, NodeIndex};
///
/// let mut g = Graph::new();
/// let a = g.add_node(()); // node with no weight
/// let b = g.add_node(());
/// let c = g.add_node(());
/// let d = g.add_node(());
/// let e = g.add_node(());
/// let f = g.add_node(());
/// g.extend_with_edges(&[
///     (0, 1, 2.0),
///     (0, 3, 4.0),
///     (1, 2, 1.0),
///     (1, 5, 7.0),
///     (2, 4, 5.0),
///     (4, 5, 1.0),
///     (3, 4, 1.0),
/// ]);
///
/// // Graph represented with the weight of each edge
/// //
/// //     2       1
/// // a ----- b ----- c
/// // | 4     | 7     |
/// // d       f       | 5
/// // | 1     | 1     |
/// // \------ e ------/
///
/// let path = bellman_ford(&g, a);
/// assert!(path.is_ok());
/// let path = path.unwrap();
/// assert_eq!(path.distances, vec![0.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
/// assert_eq!(path.predecessors, vec![
///     None,
///     Some(a),
///     Some(b),
///     Some(a),
///     Some(d),
///     Some(e)
/// ]);
///
/// // Node f (indice 5) can be reach from a with a path costing 6.
/// // Predecessor of f is Some(e) which predecessor is Some(d) which predecessor is Some(a).
/// // Thus the path from a to f is a <-> d <-> e <-> f
///
/// let graph_with_neg_cycle = Graph::<(), f32, Undirected>::from_edges(&[
///     (0, 1, -2.0),
///     (0, 3, -4.0),
///     (1, 2, -1.0),
///     (1, 5, -25.0),
///     (2, 4, -5.0),
///     (4, 5, -25.0),
///     (3, 4, -1.0),
/// ]);
///
/// assert!(bellman_ford(&graph_with_neg_cycle, NodeIndex::new(0)).is_err());
/// ```
pub fn bellman_ford<G>(
    g: G,
    source: G::NodeId,
) -> Result<Paths<G::NodeId, G::EdgeWeight>, NegativeCycleError>
where
    G: NodeCount + IntoNodeIdentifiers + IntoEdges + NodeIndexable,
    G::EdgeWeight: FloatMeasure,
{
    let ix = |i| g.to_index(i);

    // Step 1 and Step 2: initialize and relax
    let (distances, predecessors) = bellman_ford_initialize_relax(g, source);

    // Step 3: check for negative weight cycle
    for i in g.node_identifiers() {
        for edge in g.edges(i) {
            let j = edge.target();
            let w = *edge.weight();
            if distances[ix(i)] + w < distances[ix(j)] {
                return Err(NegativeCycleError);
            }
        }
    }

    Ok(Paths {
        distances,
        predecessors,
    })
}

/// \[Generic\] Find the path of a negative cycle reachable from node `source`.
///
/// Using the [find_negative_cycle][nc]; will search the Graph for negative cycles using
/// [Bellman–Ford algorithm][bf]. If no negative cycle is found the function will return `None`.
///
/// If a negative cycle is found from source, return one vec with a path of `NodeId`s.
///
/// The time complexity of this algorithm should be the same as the Bellman-Ford (O(|V|·|E|)).
///
/// [nc]: https://blogs.asarkar.com/assets/docs/algorithms-curated/Negative-Weight%20Cycle%20Algorithms%20-%20Huang.pdf
/// [bf]: https://en.wikipedia.org/wiki/Bellman%E2%80%93Ford_algorithm
///
/// # Example
/// ```rust
/// use petgraph_algorithms::shortest_paths::find_negative_cycle;
/// use petgraph_core::edge::Directed;
/// use petgraph_graph::{Graph, NodeIndex};
///
/// let graph_with_neg_cycle = Graph::<(), f32, Directed>::from_edges(&[
///     (0, 1, 1.),
///     (0, 2, 1.),
///     (0, 3, 1.),
///     (1, 3, 1.),
///     (2, 1, 1.),
///     (3, 2, -3.),
/// ]);
///
/// let path = find_negative_cycle(&graph_with_neg_cycle, NodeIndex::new(0));
/// assert_eq!(
///     path,
///     Some([NodeIndex::new(1), NodeIndex::new(3), NodeIndex::new(2)].to_vec())
/// );
/// ```
pub fn find_negative_cycle<G>(g: G, source: G::NodeId) -> Option<Vec<G::NodeId>>
where
    G: NodeCount + IntoNodeIdentifiers + IntoEdges + NodeIndexable + Visitable,
    G::EdgeWeight: FloatMeasure,
{
    let ix = |i| g.to_index(i);
    let mut path = Vec::<G::NodeId>::new();

    // Step 1: initialize and relax
    let (distance, predecessor) = bellman_ford_initialize_relax(g, source);

    // Step 2: Check for negative weight cycle
    'outer: for i in g.node_identifiers() {
        for edge in g.edges(i) {
            let j = edge.target();
            let w = *edge.weight();
            if distance[ix(i)] + w < distance[ix(j)] {
                // Step 3: negative cycle found
                let start = j;
                let mut node = start;
                let mut visited = g.visit_map();
                // Go backward in the predecessor chain
                loop {
                    let ancestor = match predecessor[ix(node)] {
                        Some(predecessor_node) => predecessor_node,
                        None => node, // no predecessor, self cycle
                    };
                    // We have only 2 ways to find the cycle and break the loop:
                    // 1. start is reached
                    if ancestor == start {
                        path.push(ancestor);
                        break;
                    }
                    // 2. some node was reached twice
                    else if visited.is_visited(&ancestor) {
                        // Drop any node in path that is before the first ancestor
                        let pos = path
                            .iter()
                            .position(|&p| p == ancestor)
                            .expect("we should always have a position");
                        path = path[pos..path.len()].to_vec();

                        break;
                    }

                    // None of the above, some middle path node
                    path.push(ancestor);
                    visited.visit(ancestor);
                    node = ancestor;
                }
                // We are done here
                break 'outer;
            }
        }
    }

    if !path.is_empty() {
        // Users will probably need to follow the path of the negative cycle
        // so it should be in the reverse order than it was found by the algorithm.
        path.reverse();
        Some(path)
    } else {
        None
    }
}

// Perform Step 1 and Step 2 of the Bellman-Ford algorithm.
#[inline(always)]
fn bellman_ford_initialize_relax<G>(
    g: G,
    source: G::NodeId,
) -> (Vec<G::EdgeWeight>, Vec<Option<G::NodeId>>)
where
    G: NodeCount + IntoNodeIdentifiers + IntoEdges + NodeIndexable,
    G::EdgeWeight: FloatMeasure,
{
    // Step 1: initialize graph
    let mut predecessor = vec![None; g.node_bound()];
    let mut distance = vec![G::EdgeWeight::INFINITY; g.node_bound()];
    let ix = |i| g.to_index(i);
    distance[ix(source)] = G::EdgeWeight::POS_ZERO;

    // Step 2: relax edges repeatedly
    for _ in 1..g.node_count() {
        let mut did_update = false;
        for i in g.node_identifiers() {
            for edge in g.edges(i) {
                let j = edge.target();
                let w = *edge.weight();
                if distance[ix(i)] + w < distance[ix(j)] {
                    distance[ix(j)] = distance[ix(i)] + w;
                    predecessor[ix(j)] = Some(i);
                    did_update = true;
                }
            }
        }
        if !did_update {
            break;
        }
    }
    (distance, predecessor)
}

#[cfg(test)]
mod tests {
    use alloc::format;

    use petgraph_core::edge::{Directed, Undirected};
    use petgraph_graph::Graph;
    use proptest::prelude::*;

    use crate::shortest_paths::bellman_ford;

    #[cfg(not(miri))]
    proptest! {
        #[test]
        fn positive_weights_undirected_always_ok(mut graph in any::<Graph<(), f32, Undirected, u8>>()) {
            for weight in graph.edge_weights_mut() {
                *weight = weight.abs();
            }

            for node in graph.node_indices() {
                bellman_ford(&graph, node).expect("should be possible");
            }
        }

        #[test]
        fn positive_weights_directed_always_ok(mut graph in any::<Graph<(), f32, Directed, u8>>()) {
            for weight in graph.edge_weights_mut() {
                *weight = weight.abs();
            }

            for node in graph.node_indices() {
                bellman_ford(&graph, node).expect("should be possible");
            }
        }

        #[test]
        fn negative_path_never_empty(graph in any::<Graph<(), f32, Directed, u8>>()) {
            for node in graph.node_indices() {
                let path = super::find_negative_cycle(&graph, node);

                if let Some(path) = path {
                    prop_assert!(!path.is_empty());
                }
            }
        }
    }
}
