use crate::visit::{
    GraphBase, IntoNeighbors, IntoNodeIdentifiers, NodeCount, NodeIndexable, VisitMap, Visitable,
};
use alloc::{vec, vec::Vec};
use core::hash::Hash;
use hashbrown::HashSet;

pub fn is_chordal<G>(graph: G) -> bool
where
    G: IntoNeighbors + IntoNodeIdentifiers + NodeCount + NodeIndexable + Visitable,
    G::NodeId: Eq + Hash,
{
    // Any graph with 3 or fewer nodes is chordal
    if graph.node_count() <= 3 {
        return true;
    }

    // Compute the maximum cardinality search ordering
    let mcs_ordering = maximum_cardinality_search(&graph);

    // Check if the ordering is a perfect elimination ordering
    is_perfect_elimination_ordering(&graph, &mcs_ordering)
}

/// State struct for the maximum cardinality search (MCS) algorithm.
///
/// The algorithm requires a max-heap in which one can edit the labels of elements in the heap.
/// We achieve this by using a bucket structure, where each bucket contains all nodes
/// with the same label (i.e. the number of already selected neighbors).
struct MCSState<G: GraphBase + Visitable> {
    /// The resulting elimination ordering of the graph
    elimination_ord: Vec<G::NodeId>,
    /// The index of the last node in the curent ordering
    current_ord_index: usize,
    /// Map of nodes already in the ordering
    visited: G::Map,
    /// Contains a bucket for each number from 0 to n-1, where n is the number
    /// of nodes in the graph. A node is in bucket i if and only if it has i
    /// neighbors that have already been selected.
    buckets: Vec<Vec<G::NodeId>>,
    /// Current highest nonempty bucket index
    current_high: usize,
    /// Position of each node within its bucket
    inner_idx: Vec<usize>,
    /// Number of already selected neighbors per node (i.e. the label)
    num_sel: Vec<usize>,
}

impl<G> MCSState<G>
where
    G: NodeIndexable + Visitable,
{
    fn new(graph: &G, num_nodes: usize, node_bound: usize) -> Self {
        MCSState {
            elimination_ord: vec![graph.from_index(0); num_nodes],
            current_ord_index: num_nodes - 1,
            visited: graph.visit_map(),
            buckets: vec![vec![]; num_nodes],
            current_high: 0,
            inner_idx: vec![0; node_bound],
            num_sel: vec![0; node_bound],
        }
    }

    fn update_label(&mut self, graph: &G, neighbor: G::NodeId) {
        let neighbor_index = graph.to_index(neighbor);
        let neighbor_label = self.num_sel[neighbor_index];
        let inner_bucket_last_element = *self.buckets[neighbor_label]
            .last()
            .expect("Inner bucket shouldn't be empty since neighbor is in it");
        let neighbor_inner_bucket_index = self.inner_idx[neighbor_index];
        self.buckets
            .get_mut(neighbor_label)
            .expect("Inner bucket should exist")
            .swap_remove(neighbor_inner_bucket_index);

        if neighbor != inner_bucket_last_element {
            self.inner_idx[graph.to_index(inner_bucket_last_element)] = neighbor_inner_bucket_index;
        }

        self.num_sel[neighbor_index] = neighbor_label + 1;
        if neighbor_label + 1 > self.current_high {
            self.current_high = neighbor_label + 1;
        }
        self.buckets
            .get_mut(neighbor_label + 1)
            .expect("Inner bucket should exist")
            .push(neighbor);
        self.inner_idx[neighbor_index] = self.buckets[neighbor_label + 1].len() - 1;
    }
}

/// Computes an elimination ordering of a graph using the maximum cardinality
/// search (MCS) algorithm.
/// The algorithm was first proposed by [Tarjan and Yannakakis][1] in 1984.
///
/// If the graph is [chordal][2], MCS will return a [perfect elimination ordering][3].
///
/// The runtime of the algorithm and this implementation in particular is
/// O(n + m), where n is the number of nodes and m is the number of edges
/// in the graph. This implementation achieves this runtime.
///
/// [1]: https://epubs.siam.org/doi/10.1137/0213035
/// [2]: https://en.wikipedia.org/wiki/Chordal_graph
/// [3]: https://en.wikipedia.org/wiki/Chordal_graph#Perfect_elimination_and_efficient_recognition
pub fn maximum_cardinality_search<G>(graph: &G) -> Vec<G::NodeId>
where
    G: IntoNeighbors + IntoNodeIdentifiers + NodeCount + NodeIndexable + Visitable,
{
    let num_nodes = graph.node_count();

    if num_nodes == 0 {
        return Vec::new();
    }

    // Take an arbitrary node as the last node in the ordering
    let mut main_node_iter = graph.node_identifiers();
    let last = main_node_iter
        .next()
        .expect("Graph is not empty due to check");

    // Initialize state for MCS
    let mut mcs_state = MCSState::new(graph, num_nodes, num_nodes);

    // Initialize first bucket (reserve space before loop)
    mcs_state
        .buckets
        .get_mut(0)
        .expect("Bucket should exist")
        .reserve(num_nodes);
    for (index, node) in graph.node_identifiers().enumerate() {
        let node_index = graph.to_index(node);
        mcs_state
            .buckets
            .get_mut(0)
            .expect("Bucket should exist")
            .push(node);
        mcs_state.inner_idx[node_index] = index;
        mcs_state.current_high += 1;
        todo!();
    }

    // Insert the last node at the start of the ordering and update the count of its neighbors
    mcs_state.elimination_ord[mcs_state.current_ord_index] = last;
    mcs_state.current_ord_index -= 1;
    mcs_state.visited.visit(last);
    for neighbor in graph.neighbors(last) {
        mcs_state.update_label(graph, neighbor)
    }

    for _ in 0..(num_nodes - 1) {
        // For each iteration, find the node with the highest label (num_sel)
        // and add it to the front of the current ordering and adjust the
        // buckets/MCS state accordingly
        let highest_label_node = mcs_state.buckets[mcs_state.current_high]
            .pop()
            .expect("Highest bucket should not be empty");

        // Insert the node at the start of the ordering
        mcs_state.elimination_ord[mcs_state.current_ord_index] = highest_label_node;
        mcs_state.current_ord_index -= 1;
        mcs_state.visited.visit(last);

        // Check if highest bucket is now empty
        if mcs_state.buckets[mcs_state.current_high].is_empty() {
            mcs_state.current_high -= 1;
        }

        // Update the labels of the neighbors of the highest label node
        for neighbor in graph.neighbors(highest_label_node) {
            // Only update the label if the neighbor is not already in the ordering
            if !mcs_state.visited.is_visited(&neighbor) {
                mcs_state.update_label(graph, neighbor);
            }
        }
    }

    mcs_state.elimination_ord
}

fn is_perfect_elimination_ordering<G>(graph: &G, ordering: &[G::NodeId]) -> bool
where
    G: IntoNeighbors + Visitable,
    G::NodeId: Eq + Hash,
{
    let mut eliminated = graph.visit_map();
    for i in (0..ordering.len()).rev() {
        let node = ordering[i];
        eliminated.visit(node);
        let neighbors_not_visited = graph
            .neighbors(node)
            .filter(|n| !eliminated.is_visited(&n))
            .collect::<HashSet<_>>();
        if !is_clique(&graph, neighbors_not_visited) {
            return false;
        }
    }
    true
}

fn is_clique<G>(graph: &G, nodes: HashSet<G::NodeId>) -> bool
where
    G: IntoNeighbors,
    G::NodeId: Eq + Hash,
{
    for a in nodes.iter() {
        let mut rest = nodes.clone();
        rest.remove(a);
        if !rest.is_subset(&graph.neighbors(*a).collect()) {
            return false;
        }
    }
    true
}
