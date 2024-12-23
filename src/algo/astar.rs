use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::{BinaryHeap, HashMap};

use std::hash::Hash;

use crate::scored::MinScored;
use crate::visit::{EdgeRef, GraphBase, IntoEdges, VisitMap, Visitable};

use crate::algo::Measure;

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
    let mut scores = HashMap::new(); // g-values, cost to reach the node
    let mut estimate_scores = HashMap::new(); // f-values, cost to reach + estimate cost to goal
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
            Occupied(mut entry) => {
                // If the node has already been visited with an equal or lower score than now, then
                // we do not need to re-visit it.
                if *entry.get() <= estimate_score {
                    continue;
                }
                entry.insert(estimate_score);
            }
            Vacant(entry) => {
                entry.insert(estimate_score);
            }
        }

        for edge in graph.edges(node) {
            let next = edge.target();
            let next_score = node_score + edge_cost(edge);

            match scores.entry(next) {
                Occupied(mut entry) => {
                    // No need to add neighbors that we have already reached through a shorter path
                    // than now.
                    if *entry.get() <= next_score {
                        continue;
                    }
                    entry.insert(next_score);
                }
                Vacant(entry) => {
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

/// \[Generic\] A* shortest path algorithm instance for a given graph with a single target.
///
/// Creates a new A*-instance that may be used to calculate the shortest path
/// from any node in the graph to `finish`, including the total path cost.
///
/// `finish` is implicitly given via the `is_goal` callback, which should return `true`
/// if, and only if, the given node is the finish node.
///
/// The function `edge_cost` should return the cost for a particular edge. Edge cost must
/// be non-negative.
///
/// The function `estimate_cost` should return the estimated cost to the finish for a
/// particular node. **For the algoirthm to find the actual shortest path**, it should be
/// admissible, meaning that **it should never overestimate** the actual cost to get
/// to the nearest goal node. Estimated costs **must also be non-negative**.
///
/// The graph should be `Visitable` and implement `IntoEdges`.
///
/// # Example
/// ```
/// use petgraph::Graph;
/// use petgraph::algo::{astar, AstarInstance};
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
/// let mut instance = AstarInstance::new(&g, |finish| finish == f, |e| *e.weight(), |_| 0);
///
/// for start in g.node_indices() {
///     let path = astar(&g, start, |finish| finish == f, |e| *e.weight(), |_| 0);
///     let instance_path = instance.run(start);
///     assert_eq!(path, instance_path);
/// }
/// ```
#[derive(Debug)]
pub struct AstarInstance<G, F, H, K, IsGoal>
where
    G: IntoEdges + Visitable,
    IsGoal: FnMut(G::NodeId) -> bool,
    G::NodeId: Eq + Hash,
    F: FnMut(G::EdgeRef) -> K,
    H: FnMut(G::NodeId) -> K,
    K: Measure + Copy,
{
    graph: G,
    is_goal: IsGoal,
    edge_cost: F,
    estimate_cost: H,

    estimate_costs: HashMap<G::NodeId, K>,
}

impl<G, F, H, K, IsGoal> AstarInstance<G, F, H, K, IsGoal>
where
    G: IntoEdges + Visitable,
    IsGoal: FnMut(G::NodeId) -> bool,
    G::NodeId: Eq + Hash,
    F: FnMut(G::EdgeRef) -> K,
    H: FnMut(G::NodeId) -> K,
    K: Measure + Copy,
{
    pub fn new(
        graph: G,
        is_goal: IsGoal,
        edge_cost: F,
        estimate_cost: H,
    ) -> AstarInstance<G, F, H, K, IsGoal> {
        AstarInstance {
            graph,
            is_goal,
            edge_cost,
            estimate_cost,
            estimate_costs: HashMap::new(),
        }
    }

    /// Retrieves the estimate cost of a specific node from the cache
    /// (estimate_costs), or if the value has not yet been requested,
    /// calculates it and saves it to the cache.
    fn estimate_cost(&mut self, node: G::NodeId) -> K {
        match self.estimate_costs.entry(node) {
            Occupied(ent) => *ent.get(),
            Vacant(ent) => {
                let est = (self.estimate_cost)(node); // Differentiation between the field and the method self.estimate_cost
                ent.insert(est);
                est
            }
        }
    }

    /// Runs the A* algorithm from a given start point.
    pub fn run(&mut self, start: G::NodeId) -> Option<(K, Vec<G::NodeId>)> {
        let mut visited = self.graph.visit_map();
        let mut visit_next = BinaryHeap::new();
        // The scores cannot be cached because they depend on a specific sequence of edges
        let mut scores = HashMap::new();
        let mut path_tracker = PathTracker::<G>::new();

        let zero_score = K::default();
        scores.insert(start, zero_score);
        visit_next.push(MinScored(self.estimate_cost(start), start));

        while let Some(MinScored(_, node)) = visit_next.pop() {
            if (self.is_goal)(node) {
                let path = path_tracker.reconstruct_path_to(node);
                let cost = scores[&node];
                return Some((cost, path));
            }
            if !visited.visit(node) {
                continue;
            }
            let node_score = scores[&node];
            for edge in self.graph.edges(node) {
                let next = edge.target();
                if visited.is_visited(&next) {
                    continue;
                }
                let mut next_score = node_score + (self.edge_cost)(edge);
                match scores.entry(next) {
                    Occupied(ent) => {
                        let old_score = *ent.get();
                        if next_score < old_score {
                            *ent.into_mut() = next_score;
                            path_tracker.set_predecessor(next, node);
                        } else {
                            next_score = old_score;
                        }
                    }
                    Vacant(ent) => {
                        ent.insert(next_score);
                        path_tracker.set_predecessor(next, node);
                    }
                }
                let next_estimate_score = next_score + self.estimate_cost(next);
                visit_next.push(MinScored(next_estimate_score, next));
            }
        }

        None
    }
}

struct PathTracker<G>
where
    G: GraphBase,
    G::NodeId: Eq + Hash,
{
    came_from: HashMap<G::NodeId, G::NodeId>,
}

impl<G> PathTracker<G>
where
    G: GraphBase,
    G::NodeId: Eq + Hash,
{
    fn new() -> PathTracker<G> {
        PathTracker {
            came_from: HashMap::new(),
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
