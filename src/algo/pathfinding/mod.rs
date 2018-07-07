use std::collections::HashMap;
use std::hash::Hash;

use crate::algo::{Measure, FloatMeasure};
use crate::visit::{IntoEdges, Visitable, NodeCount, IntoNodeIdentifiers, NodeIndexable};

mod astar;
mod bellman_ford;
mod dijkstra;
mod path;

pub use self::astar::Astar;
pub use self::bellman_ford::{BellmanFord, NegativeCycle};
pub use self::dijkstra::Dijkstra;

pub use self::path::{Path, IndexableNodeMap};

/// Builders used for type safe configuration of pathfinding algorithms. Mainly here to appear in
/// generated documentation.
pub mod builders {
    pub use super::astar::{AstarBuilder1, AstarBuilder2, AstarBuilder3, ConfiguredAstar};
    pub use super::bellman_ford::{BellmanFordBuilder1, BellmanFordBuilder2, ConfiguredBellmanFord};
    pub use super::dijkstra::{DijkstraBuilder1, DijkstraBuilder2, ConfiguredDijkstra};
}

/// Traits used to configure the pathfinding, such as predecessors or costs.
pub mod traits {
    pub use super::path::{CostMap, PredecessorMap, PredecessorMapConfigured};
}

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
/// use petgraph::Graph;
/// use petgraph::algo::dijkstra;
/// use petgraph::prelude::*;
/// use std::collections::HashMap;
///
/// let mut graph : Graph<(),(),Directed>= Graph::new();
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
///     (h, e)
/// ]);
/// // a ----> b ----> e ----> f
/// // ^       |       ^       |
/// // |       v       |       v
/// // d <---- c       h <---- g
///
/// let expected_res: HashMap<NodeIndex, usize> = [
///      (a, 3),
///      (b, 0),
///      (c, 1),
///      (d, 2),
///      (e, 1),
///      (f, 2),
///      (g, 3),
///      (h, 4)
///     ].iter().cloned().collect();
/// let res = dijkstra(&graph,b,None, |_| 1);
/// assert_eq!(res, expected_res);
/// // z is not inside res because there is not path from b to z.
/// ```
pub fn dijkstra<G, F, K>(graph: G,
                         start: G::NodeId,
                         goal: Option<G::NodeId>,
                         edge_cost: F)
                         -> HashMap<G::NodeId, K>
    where G: IntoEdges + Visitable,
          G::NodeId: Eq + Hash,
          F: Fn(G::EdgeRef) -> K,
          K: Measure + Copy
{
    let dijkstra = Dijkstra::new(graph)
        .edge_cost(edge_cost)
        .cost_map(HashMap::new());

    let path = {
        if let Some(goal) = goal {
            dijkstra.path(start, goal)
        } else {
            dijkstra.path_all(start)
        }
    };

    path.into_costs()
}

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
/// use petgraph::Graph;
/// use petgraph::algo::astar;
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
pub fn astar<G, F, H, K, IsGoal>(graph: G,
                                 start: G::NodeId,
                                 is_goal: IsGoal,
                                 edge_cost: F,
                                 estimate_cost: H)
                                 -> Option<(K, Vec<G::NodeId>)>
    where G: IntoEdges + Visitable,
          IsGoal: Fn(G::NodeId) -> bool,
          G::NodeId: Eq + Hash,
          F: Fn(G::EdgeRef) -> K,
          H: Fn(G::NodeId) -> K,
          K: Measure + Copy
{
    Astar::new(graph)
        .edge_cost(edge_cost)
        .estimate_cost(estimate_cost)
        .cost_map(HashMap::new())
        .predecessor_map(HashMap::new())
        .path_with(start, is_goal)
        .into_nodes()
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
/// use petgraph::Graph;
/// use petgraph::algo::bellman_ford;
/// use petgraph::prelude::*;
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
/// assert_eq!(path, Ok((vec![0.0 ,     2.0,    3.0,    4.0,     5.0,     6.0],
///                      vec![None, Some(a),Some(b),Some(a), Some(d), Some(e)]
///                    ))
///           );
/// // Node f (indice 5) can be reach from a with a path costing 6.
/// // Predecessor of f is Some(e) which predecessor is Some(d) which predecessor is Some(a).
/// // Thus the path from a to f is a <-> d <-> e <-> f
///
/// let graph_with_neg_cycle = Graph::<(), f32, Undirected>::from_edges(&[
///         (0, 1, -2.0),
///         (0, 3, -4.0),
///         (1, 2, -1.0),
///         (1, 5, -25.0),
///         (2, 4, -5.0),
///         (4, 5, -25.0),
///         (3, 4, -1.0),
/// ]);
///
/// assert!(bellman_ford(&graph_with_neg_cycle, NodeIndex::new(0)).is_err());
/// ```
pub fn bellman_ford<G>(graph: G,
                       start: G::NodeId)
                       -> Result<(Vec<G::EdgeWeight>, Vec<Option<G::NodeId>>), NegativeCycle>
    where G: NodeCount + IntoNodeIdentifiers + IntoEdges + NodeIndexable,
          G::EdgeWeight: FloatMeasure
{
    let paths = BellmanFord::new(graph)
        .cost_map(IndexableNodeMap::new())
        .predecessor_map(IndexableNodeMap::new())
        .path_all(start);

    paths.map(|p| {
                  let (costs, predecessors, _) = p.unpack();
                  (costs.into_node_map(), predecessors.into_node_map())
              })
}

