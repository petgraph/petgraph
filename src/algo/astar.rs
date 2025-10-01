use alloc::{collections::BinaryHeap, vec, vec::Vec};
use core::{hash::Hash, ops::Sub};

use hashbrown::hash_map::{
    Entry::{Occupied, Vacant},
    HashMap,
};

use crate::algo::Measure;
use crate::scored::MinScored;
use crate::visit::{EdgeRef, GraphBase, IntoEdges, Visitable};

/// A* shortest path algorithm.
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
/// # Arguments
/// * `graph`: weighted graph.
/// * `start`: the start node.
/// * `is_goal`: the callback defines the goal node.
/// * `edge_cost`: closure that returns cost of a particular edge.
/// * `estimate_cost`: closure that returns the estimated cost to the finish for particular node.
///
/// # Returns
/// * `Some(K, Vec<G::NodeId>)` - the total cost and path from start to finish, if one was found.
/// * `None` - if such a path was not found.
///
/// # Complexity
/// The time complexity largely depends on the heuristic used. Feel free to contribute and provide the exact time complexity :)
///
/// With a trivial heuristic, the algorithm will behave like [`fn@crate::algo::dijkstra`].
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
    // The Open set
    let mut visit_next = BinaryHeap::new();
    // A node -> (f, h, g) mapping
    // TODO: Derive `g` from `f` and `h`.
    let mut scores = HashMap::new();
    // The search tree
    let mut path_tracker = PathTracker::<G>::new();

    let zero: K = K::default();
    let g: K = zero;
    let h: K = estimate_cost(start);
    let f: K = g + h;
    scores.insert(start, (f, h, g));
    visit_next.push(MinScored((f, h, g), start));

    while let Some(MinScored((f, h, g), node)) = visit_next.pop() {
        if is_goal(node) {
            let path = path_tracker.reconstruct_path_to(node);
            let (goal_f, goal_h, goal_g) = scores[&node];
            debug_assert_eq!(goal_h, zero);
            debug_assert_eq!(goal_f, goal_g);
            return Some((goal_f, path));
        }

        match scores.entry(node) {
            Occupied(mut entry) => {
                let (_, _, old_g) = *entry.get();
                // The node has already been expanded with a better cost.
                if old_g < g {
                    continue;
                }
                // NOTE: Because there's no closed set, we don't know if we expanded this node.
                // if old_g = g we may be re-expanding this node, but won't insert new neigbours.
                entry.insert((f, h, g));
            }
            Vacant(entry) => {
                entry.insert((f, h, g));
            }
        }

        for edge in graph.edges(node) {
            let neigh = edge.target();
            let neigh_g = g + edge_cost(edge);
            let neigh_h = estimate_cost(neigh);
            let neigh_f = neigh_g + neigh_h;
            let neigh_score = (neigh_f, neigh_h, neigh_g);

            match scores.entry(neigh) {
                Occupied(mut entry) => {
                    let (_, _, old_neigh_g) = *entry.get();
                    if neigh_g >= old_neigh_g {
                        // New cost isn't better
                        continue;
                    }
                    entry.insert(neigh_score);
                }
                Vacant(entry) => {
                    entry.insert(neigh_score);
                }
            }

            path_tracker.set_predecessor(neigh, node);
            visit_next.push(MinScored(neigh_score, neigh));
        }
    }

    None
}

/// \[Generic\] A* shortest path algorithm with a timeout.
///
/// Similar to the `astar` function, but with an additional timeout check.
/// If the timeout is reached, the function will return the best node found so far.
/// What is considered the best node is determined by the heuristic score (estimate - cost).
///
/// The function `time_out_reached` should return `true` if the timeout has been reached.
///
/// You can construct such a function using either an atomic boolean (set from another thread), check the system time on each iteration or use a simple counter.
///
/// Example with a simple counter:
/// ```
/// use petgraph::Graph;
/// use petgraph::algo::astar_with_timeout;
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
/// use std::sync::atomic::{AtomicBool, Ordering};
/// use std::sync::Arc;
/// use std::thread;
///
/// // First, we do a test where the timeout is not reached.
/// let mut timeout_counter = 0;
/// let timeout_counter_pointer = &mut timeout_counter;
/// let timeout_reached = || {
///    *timeout_counter_pointer += 1;
///   if *timeout_counter_pointer > 5 { true } else { false }
/// };
///
/// let path = astar_with_timeout( &g, a, |finish| finish == f, timeout_reached, |e| *e.weight(), |_| 0, );
/// assert_eq!(path, Some((6, vec![a, d, e, f])));
///
/// // Now, we do a test where the timeout is reached.
/// *timeout_counter_pointer = 0;
/// let timeout_reached_larger = || {
///    *timeout_counter_pointer += 1;
///   if *timeout_counter_pointer > 4 { true } else { false }
/// };
///
/// let path = astar_with_timeout( &g, a, |finish| finish == f, timeout_reached_larger, |e| *e.weight(), |_| 0, );
/// println!("Path: {:?}", path);
/// // The best node found so far is `a` with a cost of 0.
/// // The reason that we don't get a better result is because we don't use a heuristic here and we are essentially just using dijkstra (there is no usefull notion of best so far).
/// assert_eq!(path, Some((0, vec![a])));
///
/// ```
pub fn astar_with_timeout<G, F, H, K, IsGoal, TimeOut>(
    graph: G,
    start: G::NodeId,
    mut is_goal: IsGoal,
    mut time_out_reached: TimeOut,
    mut edge_cost: F,
    mut estimate_cost: H,
) -> Option<(K, Vec<G::NodeId>)>
where
    G: IntoEdges + Visitable,
    IsGoal: FnMut(G::NodeId) -> bool,
    TimeOut: FnMut() -> bool,
    G::NodeId: Eq + Hash,
    F: FnMut(G::EdgeRef) -> K,
    H: FnMut(G::NodeId) -> K,
    K: Measure + Copy + Sub<Output = K>,
{
    let mut visit_next = BinaryHeap::new();
    let mut scores = HashMap::new(); // g-values, cost to reach the node
    let mut estimate_scores = HashMap::new(); // f-values, cost to reach + estimate cost to goal
    let mut path_tracker = PathTracker::<G>::new();

    let mut best_so_far = start; // The best node so far (the one with the lowest heuristic (estimate - cost) score)
    let mut best_so_far_score = estimate_cost(best_so_far); // The best heuristic (estimate - cost) score so far

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

        if best_so_far_score > estimate_score - node_score {
            best_so_far = node;
            best_so_far_score = estimate_score - node_score;
        }

        if time_out_reached() {
            // If we have reached a timeout, we return the best node so far
            let path = path_tracker.reconstruct_path_to(best_so_far);
            let cost = scores[&best_so_far];
            return Some((cost, path));
        }

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
