use alloc::collections::BinaryHeap;
use alloc::vec::Vec;
use core::hash::Hash;
use hashbrown::hash_map::{
    Entry::{Occupied, Vacant},
    HashMap,
};

use crate::algo::Measure;
use crate::scored::MinScored;
use crate::visit::{EdgeRef, IntoEdges, IntoEdgesDirected, VisitMap, Visitable};
use crate::Direction;

pub struct Dijkstra<G, GoalFn, CostFn, K>
where
    G: IntoEdges + Visitable,
    G::NodeId: Eq + Hash,
    GoalFn: FnMut(&G::NodeId) -> bool,
    CostFn: FnMut(G::EdgeRef) -> K,
    K: Measure + Copy,
{
    graph: G,
    start: G::NodeId,
    goal: GoalFn,
    edge_cost: CostFn,
    distances: HashMap<G::NodeId, K>,
    visited: G::Map,
}

/// Return value of [`Dijkstra::run`]. To access values, use the respective methods.
pub struct DijkstraOutput<'a, N, C>
where
    N: Eq + Hash,
    C: Measure + Copy,
{
    paths: HashMap<N, Vec<N>>,
    distances: &'a HashMap<N, C>,
}

/// Owned return value of [`Dijkstra::run_once`]. To access values, use `into_` methods.
pub struct DijkstraOutputOwned<N, C>
where
    N: Eq + Hash,
    C: Measure + Copy,
{
    paths: HashMap<N, Vec<N>>,
    distances: HashMap<N, C>,
}

// The GoalFn is instantiated to `fn(&G::NodeId) -> bool` in this impl, because new supplies a new
// type for GoalFn, and thus the type of the Dijkstra of the impl block is actually irrelevant.
// Thus, we can use an arbitrary type for GoalFn, as it is just a placeholder.
impl<G, CostFn, K> Dijkstra<G, fn(&G::NodeId) -> bool, CostFn, K>
where
    G: IntoEdges + Visitable,
    G::NodeId: Eq + Hash,
    CostFn: FnMut(G::EdgeRef) -> K,
    K: Measure + Copy,
{
    /// Create a new `Dijkstra` instance.
    pub fn new(
        graph: G,
        start: G::NodeId,
        edge_cost: CostFn,
    ) -> Dijkstra<G, impl FnMut(&G::NodeId) -> bool, CostFn, K> {
        Dijkstra {
            graph,
            start,
            goal: |_: &G::NodeId| false,
            edge_cost,
            distances: HashMap::new(),
            visited: graph.visit_map(),
        }
    }
}

impl<G, GoalFn, CostFn, K> Dijkstra<G, GoalFn, CostFn, K>
where
    G: IntoEdges + Visitable,
    G::NodeId: Eq + Hash,
    GoalFn: FnMut(&G::NodeId) -> bool,
    CostFn: FnMut(G::EdgeRef) -> K,
    K: Measure + Copy,
{
    /// Dijkstra's shortest path algorithm.
    ///
    /// This algorithm is identical to [`dijkstra`],
    /// but allows matching multiple goal nodes, whichever is reached first.
    /// A node is considered a goal if `goal_fn` returns `true` for it.
    ///
    /// See the [`dijkstra`] function for more details.
    ///
    /// # Example
    /// ```rust
    /// use petgraph::Graph;
    /// use petgraph::algo::dijkstra;
    /// use petgraph::prelude::*;
    /// use hashbrown::HashMap;
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
    ///     (b, 0),
    ///     (c, 1),
    ///     (d, 2),
    ///     (e, 1),
    ///     (f, 2),
    /// ].iter().cloned().collect();
    /// let mut dijkstra = dijkstra::Dijkstra::new(&graph, b, |_| 1).with_goal(|&node| node == d || node == f);
    /// let res = dijkstra.run();
    /// assert_eq!(res.distances(), &expected_res);
    /// ```
    pub fn run(&mut self) -> DijkstraOutput<'_, G::NodeId, K> {
        let mut visit_next = BinaryHeap::new();
        let zero_score = K::default();
        self.distances.insert(self.start, zero_score);
        visit_next.push(MinScored(zero_score, self.start));
        while let Some(MinScored(node_score, node)) = visit_next.pop() {
            if self.visited.is_visited(&node) {
                continue;
            }
            if (self.goal)(&node) {
                break;
            }
            for edge in self.graph.edges(node) {
                let next = edge.target();
                if self.visited.is_visited(&next) {
                    continue;
                }
                let next_score = node_score + (self.edge_cost)(edge);
                match self.distances.entry(next) {
                    Occupied(ent) => {
                        if next_score < *ent.get() {
                            *ent.into_mut() = next_score;
                            visit_next.push(MinScored(next_score, next));
                            //predecessor.insert(next.clone(), node.clone());
                        }
                    }
                    Vacant(ent) => {
                        ent.insert(next_score);
                        visit_next.push(MinScored(next_score, next));
                        //predecessor.insert(next.clone(), node.clone());
                    }
                }
            }
            self.visited.visit(node);
        }
        DijkstraOutput {
            paths: HashMap::new(),
            distances: &self.distances,
        }
    }

    pub fn run_once(mut self) -> DijkstraOutputOwned<G::NodeId, K> {
        let output = self.run();
        DijkstraOutputOwned {
            paths: output.paths,
            distances: self.distances,
        }
    }

    pub fn with_goal<NewGoalFn>(self, goal: NewGoalFn) -> Dijkstra<G, NewGoalFn, CostFn, K>
    where
        NewGoalFn: FnMut(&G::NodeId) -> bool,
    {
        Dijkstra {
            graph: self.graph,
            start: self.start,
            goal,
            edge_cost: self.edge_cost,
            distances: self.distances,
            visited: self.visited,
        }
    }

    pub fn into_distances(self) -> HashMap<G::NodeId, K> {
        self.distances
    }
}

impl<N, K> DijkstraOutput<'_, N, K>
where
    N: Eq + Hash + Clone,
    K: Measure + Copy,
{
    /// Get a reference to a HashMap of the distances.
    ///
    /// To get an owned HashMap, run [`Dijkstra::run_once`] or use [`Dijkstra::into_distances`].
    pub fn distances(&self) -> &HashMap<N, K> {
        self.distances
    }

    /// Get the distance to a specific node.
    pub fn distance_to_node(&self, node: &N) -> Option<K> {
        self.distances.get(node).cloned()
    }

    /// Get the reference to a HashMap of the paths.
    ///
    /// To get an owned HashMap, use [`DijkstraOutput::into_paths`].
    pub fn paths(&self) -> &HashMap<N, Vec<N>> {
        &self.paths
    }

    /// Get the the reference to a path to a specific node.
    ///
    /// To get an owned HashMap, use [`DijkstraOutput::into_paths`].
    pub fn path_to_node(&self, node: &N) -> Option<&Vec<N>> {
        self.paths.get(node)
    }

    /// Turn the output into the paths HashMap
    pub fn into_paths(self) -> HashMap<N, Vec<N>> {
        self.paths
    }
}

impl<N, K> DijkstraOutputOwned<N, K>
where
    N: Eq + Hash + Clone,
    K: Measure + Copy,
{
    /// Turn the output into the paths HashMap. To get both paths and distances, use
    /// [`DijkstraOutputOwned::into_paths_and_distances`].
    pub fn into_paths(self) -> HashMap<N, Vec<N>> {
        self.paths
    }

    /// Turn the output into the distances HashMap. To get both paths and distances, use
    /// [`DijkstraOutputOwned::into_paths_and_distances`].
    pub fn into_distances(self) -> HashMap<N, K> {
        self.distances
    }

    /// Turn the output into both paths and distances HashMaps.
    pub fn into_paths_and_distances(self) -> (HashMap<N, Vec<N>>, HashMap<N, K>) {
        (self.paths, self.distances)
    }
}

/// Dijkstra's shortest path algorithm.
///
/// Compute the length of the shortest path from `start` to every reachable
/// node.
///
/// The function `edge_cost` should return the cost for a particular edge, which is used
/// to compute path costs. Edge costs must be non-negative.
///
/// If `goal` is not `None`, then the algorithm terminates once the `goal` node's
/// cost is calculated.
///
/// # Arguments
/// * `graph`: weighted graph.
/// * `start`: the start node.
/// * `goal`: optional *goal* node.
/// * `edge_cost`: closure that returns cost of a particular edge.
///
/// # Returns
/// * `HashMap`: [`struct@hashbrown::HashMap`] that maps `NodeId` to path cost.
///
/// # Complexity
/// * Time complexity: **O((|V|+|E|)log(|V|))**.
/// * Auxiliary space: **O(|V|+|E|)**.
///
/// where **|V|** is the number of nodes and **|E|** is the number of edges.
///
/// # Example
/// ```rust
/// use petgraph::Graph;
/// use petgraph::algo::dijkstra;
/// use petgraph::prelude::*;
/// use hashbrown::HashMap;
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
/// ].iter().cloned().collect();
/// let res = dijkstra(&graph, b, None, |_| 1);
/// assert_eq!(res, expected_res);
/// // z is not inside res because there is not path from b to z.
/// ```
pub fn dijkstra<G, F, K>(
    graph: G,
    start: G::NodeId,
    goal: Option<G::NodeId>,
    edge_cost: F,
) -> HashMap<G::NodeId, K>
where
    G: IntoEdges + Visitable,
    G::NodeId: Eq + Hash,
    F: FnMut(G::EdgeRef) -> K,
    K: Measure + Copy,
{
    Dijkstra::new(graph, start, edge_cost)
        .with_goal(|node| goal.as_ref() == Some(node))
        .run_once()
        .distances
}

/// Bidirectional Dijkstra's shortest path algorithm.
///
/// Compute the length of the shortest path from `start` to `target`.
///
/// Bidirectional Dijkstra has the same time complexity as standard [`Dijkstra`][dijkstra]. However, because it
/// searches simultaneously from both the start and goal nodes, meeting in the middle, it often
/// explores roughly half the nodes that regular [`Dijkstra`][dijkstra] would explore. This is especially the case
/// when the path is long relative to the graph size or when working with sparse graphs.
///
/// However, regular [`Dijkstra`][dijkstra] may be preferable when you need the shortest paths from the start node
/// to multiple goals or when the start and goal are relatively close to each other.
///
/// The function `edge_cost` should return the cost for a particular edge, which is used
/// to compute path costs. Edge costs must be non-negative.
///
/// # Arguments
/// * `graph`: weighted graph.
/// * `start`: the start node.
/// * `goal`: the goal node.
/// * `edge_cost`: closure that returns the cost of a particular edge.
///
/// # Returns
/// * `Some(K)` - the total cost from start to finish, if one was found.
/// * `None` - if such a path was not found.
///
/// # Complexity
/// * Time complexity: **O((|V|+|E|)log(|V|))**.
/// * Auxiliary space: **O(|V|+|E|)**.
///
/// where **|V|** is the number of nodes and **|E|** is the number of edges.
///
/// # Example
/// ```rust
/// use petgraph::Graph;
/// use petgraph::algo::bidirectional_dijkstra;
/// use petgraph::prelude::*;
/// use hashbrown::HashMap;
///
/// let mut graph: Graph<(), (), Directed> = Graph::new();
/// let a = graph.add_node(());
/// let b = graph.add_node(());
/// let c = graph.add_node(());
/// let d = graph.add_node(());
/// let e = graph.add_node(());
/// let f = graph.add_node(());
/// let g = graph.add_node(());
/// let h = graph.add_node(());
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
/// let output = bidirectional_dijkstra(&graph, a, g, |_| 1);
/// assert_eq!(output, Some(4));
/// ```
pub fn bidirectional_dijkstra<G, F, K>(
    graph: G,
    start: G::NodeId,
    goal: G::NodeId,
    mut edge_cost: F,
) -> Option<K>
where
    G: Visitable + IntoEdgesDirected,
    G::NodeId: Eq + Hash,
    F: FnMut(G::EdgeRef) -> K,
    K: Measure + Copy,
{
    let mut forward_visited = graph.visit_map();
    let mut forward_distance = HashMap::new();
    forward_distance.insert(start, K::default());

    let mut backward_visited = graph.visit_map();
    let mut backward_distance = HashMap::new();
    backward_distance.insert(goal, K::default());

    let mut forward_heap = BinaryHeap::new();
    let mut backward_heap = BinaryHeap::new();

    forward_heap.push(MinScored(K::default(), start));
    backward_heap.push(MinScored(K::default(), goal));

    let mut best_value = None;

    while !forward_heap.is_empty() && !backward_heap.is_empty() {
        let MinScored(_, u) = forward_heap.pop().unwrap();
        let MinScored(_, v) = backward_heap.pop().unwrap();

        forward_visited.visit(u);
        backward_visited.visit(v);

        let distance_to_u = forward_distance[&u];
        let distance_to_v = backward_distance[&v];

        for edge in graph.edges_directed(u, Direction::Outgoing) {
            let x = edge.target();
            let current_edge_cost = edge_cost(edge);

            if !forward_visited.is_visited(&x) {
                let next_score = distance_to_u + current_edge_cost;

                match forward_distance.entry(x) {
                    Occupied(entry) => {
                        if next_score < *entry.get() {
                            *entry.into_mut() = next_score;
                            forward_heap.push(MinScored(next_score, x));
                        }
                    }
                    Vacant(entry) => {
                        entry.insert(next_score);
                        forward_heap.push(MinScored(next_score, x));
                    }
                }
            }

            if !backward_visited.is_visited(&x) {
                continue;
            }

            let potential_best_value = distance_to_u + current_edge_cost + backward_distance[&x];

            let improves_best_value = match best_value {
                None => true,
                Some(current_best_value) => potential_best_value < current_best_value,
            };

            if improves_best_value {
                best_value = Some(potential_best_value);
            }
        }

        for edge in graph.edges_directed(v, Direction::Incoming) {
            let x = edge.source();
            let edge_cost = edge_cost(edge);

            if !backward_visited.is_visited(&x) {
                let next_score = distance_to_v + edge_cost;

                match backward_distance.entry(x) {
                    Occupied(entry) => {
                        if next_score < *entry.get() {
                            *entry.into_mut() = next_score;
                            backward_heap.push(MinScored(next_score, x));
                        }
                    }
                    Vacant(entry) => {
                        entry.insert(next_score);
                        backward_heap.push(MinScored(next_score, x));
                    }
                }
            }

            if !forward_visited.is_visited(&x) {
                continue;
            }

            let potential_best_value = distance_to_v + edge_cost + forward_distance[&x];

            let improves_best_value = match best_value {
                None => true,
                Some(mu) => potential_best_value < mu,
            };

            if improves_best_value {
                best_value = Some(potential_best_value);
            }
        }

        if let Some(best_value) = best_value {
            if distance_to_u + distance_to_v >= best_value {
                return Some(best_value);
            }
        }
    }

    None
}
