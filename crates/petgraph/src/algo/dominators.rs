//! Compute dominators of a control-flow graph.
//!
//! # The Dominance Relation
//!
//! In a directed graph with a root node **R**, a node **A** is said to *dominate* a
//! node **B** iff every path from **R** to **B** contains **A**.
//!
//! The node **A** is said to *strictly dominate* the node **B** iff **A** dominates
//! **B** and **A ≠ B**.
//!
//! The node **A** is said to be the *immediate dominator* of a node **B** iff it
//! strictly dominates **B** and there does not exist any node **C** where **A**
//! dominates **C** and **C** dominates **B**.

use alloc::{collections::VecDeque, vec, vec::Vec};
use core::{cmp::Ordering, hash::Hash};

use hashbrown::{HashMap, HashSet, hash_map::Iter};

use crate::EdgeDirection;
use crate::visit::{
    DfsPostOrder, GraphBase, IntoNeighbors, IntoNeighborsDirected, IntoNodeIdentifiers,
    NodeIndexable, Visitable, Walker,
};

/// The dominance relation for some graph and root.
#[derive(Debug, Clone)]
pub struct Dominators<N>
where
    N: Copy + Eq + Hash,
{
    root: N,
    dominators: HashMap<N, N>,
}

impl<N> Dominators<N>
where
    N: Copy + Eq + Hash,
{
    /// Get the root node used to construct these dominance relations.
    pub fn root(&self) -> N {
        self.root
    }

    /// Get the immediate dominator of the given node.
    ///
    /// Returns `None` for any node that is not reachable from the root, and for
    /// the root itself.
    pub fn immediate_dominator(&self, node: N) -> Option<N> {
        if node == self.root {
            None
        } else {
            self.dominators.get(&node).cloned()
        }
    }

    /// Iterate over the given node's strict dominators.
    ///
    /// If the given node is not reachable from the root, then `None` is
    /// returned.
    pub fn strict_dominators(&self, node: N) -> Option<DominatorsIter<'_, N>> {
        if self.dominators.contains_key(&node) {
            Some(DominatorsIter {
                dominators: self,
                node: self.immediate_dominator(node),
            })
        } else {
            None
        }
    }

    /// Iterate over all of the given node's dominators (including the given
    /// node itself).
    ///
    /// If the given node is not reachable from the root, then `None` is
    /// returned.
    pub fn dominators(&self, node: N) -> Option<DominatorsIter<'_, N>> {
        if self.dominators.contains_key(&node) {
            Some(DominatorsIter {
                dominators: self,
                node: Some(node),
            })
        } else {
            None
        }
    }

    /// Iterate over all nodes immediately dominated by the given node (not
    /// including the given node itself).
    pub fn immediately_dominated_by(&self, node: N) -> DominatedByIter<'_, N> {
        DominatedByIter {
            iter: self.dominators.iter(),
            node,
        }
    }

    /// Iterate over all nodes dominated by the given node (including
    /// indirectly dominated nodes, not including the given node itself).
    ///
    /// This returns all nodes that the given node dominates, including those
    /// that are indirectly dominated (i.e., descendants in the dominance tree).
    pub fn dominated_by(&self, node: N) -> DominatedByAllIter<'_, N> {
        // Build a set of all nodes dominated by the given node using BFS
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        // Start with all nodes immediately dominated by the given node
        for (&dominator, dominated) in &self.dominators {
            if dominated == &dominator {
                continue; // Skip the root node that dominates itself
            }
            if dominated == &node {
                queue.push_back(dominator);
                visited.insert(dominator);
            }
        }

        DominatedByAllIter {
            dominators: self,
            queue,
            visited,
        }
    }
}

/// Iterator for a node's dominators.
#[derive(Debug, Clone)]
pub struct DominatorsIter<'a, N>
where
    N: 'a + Copy + Eq + Hash,
{
    dominators: &'a Dominators<N>,
    node: Option<N>,
}

impl<'a, N> Iterator for DominatorsIter<'a, N>
where
    N: 'a + Copy + Eq + Hash,
{
    type Item = N;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.node.take();
        if let Some(next) = next {
            self.node = self.dominators.immediate_dominator(next);
        }
        next
    }
}

/// Iterator for nodes dominated by a given node.
#[derive(Debug, Clone)]
pub struct DominatedByIter<'a, N>
where
    N: 'a + Copy + Eq + Hash,
{
    iter: Iter<'a, N, N>,
    node: N,
}

impl<'a, N> Iterator for DominatedByIter<'a, N>
where
    N: 'a + Copy + Eq + Hash,
{
    type Item = N;

    fn next(&mut self) -> Option<Self::Item> {
        for (dominator, dominated) in self.iter.by_ref() {
            // The root node dominates itself, but it should not be included in
            // the results.
            if dominated == &self.node && dominated != dominator {
                return Some(*dominator);
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.iter.size_hint();
        (0, upper)
    }
}

/// Iterator for all nodes dominated by a given node (including indirectly
/// dominated nodes).
#[derive(Debug, Clone)]
pub struct DominatedByAllIter<'a, N>
where
    N: 'a + Copy + Eq + Hash,
{
    dominators: &'a Dominators<N>,
    queue: VecDeque<N>,
    visited: HashSet<N>,
}

impl<'a, N> Iterator for DominatedByAllIter<'a, N>
where
    N: 'a + Copy + Eq + Hash,
{
    type Item = N;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(current) = self.queue.pop_front() {
            // Add all nodes immediately dominated by current to the queue
            for (&dominator, dominated) in &self.dominators.dominators {
                if dominated == &dominator {
                    continue; // Skip the root node that dominates itself
                }
                if dominated == &current && !self.visited.contains(&dominator) {
                    self.visited.insert(dominator);
                    self.queue.push_back(dominator);
                }
            }
            return Some(current);
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let lower = self.queue.len();
        let upper = self.dominators.dominators.len();
        (lower, Some(upper))
    }
}

/// The undefined dominator sentinel, for when we have not yet discovered a
/// node's dominator.
const UNDEFINED: usize = usize::MAX;

/// This is an implementation of the engineered ["Simple, Fast Dominance
/// Algorithm"][0] discovered by Cooper et al.
///
/// This algorithm is **O(|V|²)** where V is the set of nodes, and therefore has slower theoretical
/// running time than the Lengauer-Tarjan algorithm (which is **O(|E| log |V|)** where E is the set
/// of edges). However, Cooper et al found it to be faster in practice on control flow graphs of up
/// to ~30,000 nodes.
///
/// # Arguments
/// * `graph`: a control-flow graph.
/// * `root`: the *root* node of the `graph`.
///
/// # Returns
/// * `Dominators`: the dominance relation for given `graph` and `root` represented by
///   [`struct@Dominators`].
///
/// # Complexity
/// * Time complexity: **O(|V|²)**.
/// * Auxiliary space: **O(|V| + |E|)**.
///
/// where **|V|** is the number of nodes and **|E|** is the number of edges.
///
/// [0]: http://www.hipersoft.rice.edu/grads/publications/dom14.pdf
pub fn simple_fast<G>(graph: G, root: G::NodeId) -> Dominators<G::NodeId>
where
    G: IntoNeighbors + Visitable,
    <G as GraphBase>::NodeId: Eq + Hash,
{
    let (post_order, predecessor_sets) = simple_fast_post_order(graph, root);
    let length = post_order.len();
    debug_assert!(length > 0);
    debug_assert!(post_order.last() == Some(&root));

    // From here on out we use indices into `post_order` instead of actual
    // `NodeId`s wherever possible. This greatly improves the performance of
    // this implementation, but we have to pay a little bit of upfront cost to
    // convert our data structures to play along first.

    // Maps a node to its index into `post_order`.
    let node_to_post_order_idx: HashMap<_, _> = post_order
        .iter()
        .enumerate()
        .map(|(idx, &node)| (node, idx))
        .collect();

    // Maps a node's `post_order` index to its set of predecessors's indices
    // into `post_order` (as a vec).
    let idx_to_predecessor_vec =
        predecessor_sets_to_idx_vecs(&post_order, &node_to_post_order_idx, predecessor_sets);

    let mut dominators = vec![UNDEFINED; length];
    dominators[length - 1] = length - 1;

    let mut changed = true;
    while changed {
        changed = false;

        // Iterate in reverse post order, skipping the root.

        for idx in (0..length - 1).rev() {
            debug_assert!(post_order[idx] != root);

            // Take the intersection of every predecessor's dominator set; that
            // is the current best guess at the immediate dominator for this
            // node.

            let new_idom_idx = {
                let mut predecessors = idx_to_predecessor_vec[idx]
                    .iter()
                    .filter(|&&p| dominators[p] != UNDEFINED);
                let new_idom_idx = predecessors.next().expect(
                    "Because the root is initialized to dominate itself, and is the first node in \
                     every path, there must exist a predecessor to this node that also has a \
                     dominator",
                );
                predecessors.fold(*new_idom_idx, |new_idom_idx, &predecessor_idx| {
                    intersect(&dominators, new_idom_idx, predecessor_idx)
                })
            };

            debug_assert!(new_idom_idx < length);

            if new_idom_idx != dominators[idx] {
                dominators[idx] = new_idom_idx;
                changed = true;
            }
        }
    }

    // All done! Translate the indices back into proper `G::NodeId`s.
    debug_assert!(!dominators.contains(&UNDEFINED));

    Dominators {
        root,
        dominators: dominators
            .into_iter()
            .enumerate()
            .map(|(idx, dom_idx)| (post_order[idx], post_order[dom_idx]))
            .collect(),
    }
}

fn intersect(dominators: &[usize], mut finger1: usize, mut finger2: usize) -> usize {
    loop {
        match finger1.cmp(&finger2) {
            Ordering::Less => finger1 = dominators[finger1],
            Ordering::Greater => finger2 = dominators[finger2],
            Ordering::Equal => return finger1,
        }
    }
}

fn predecessor_sets_to_idx_vecs<N>(
    post_order: &[N],
    node_to_post_order_idx: &HashMap<N, usize>,
    mut predecessor_sets: HashMap<N, HashSet<N>>,
) -> Vec<Vec<usize>>
where
    N: Copy + Eq + Hash,
{
    post_order
        .iter()
        .map(|node| {
            predecessor_sets
                .remove(node)
                .map(|predecessors| {
                    predecessors
                        .into_iter()
                        .map(|p| *node_to_post_order_idx.get(&p).unwrap())
                        .collect()
                })
                .unwrap_or_default()
        })
        .collect()
}

type PredecessorSets<NodeId> = HashMap<NodeId, HashSet<NodeId>>;

fn simple_fast_post_order<G>(
    graph: G,
    root: G::NodeId,
) -> (Vec<G::NodeId>, PredecessorSets<G::NodeId>)
where
    G: IntoNeighbors + Visitable,
    <G as GraphBase>::NodeId: Eq + Hash,
{
    let mut post_order = vec![];
    let mut predecessor_sets = HashMap::new();

    for node in DfsPostOrder::new(graph, root).iter(graph) {
        post_order.push(node);

        for successor in graph.neighbors(node) {
            predecessor_sets
                .entry(successor)
                .or_insert_with(HashSet::new)
                .insert(node);
        }
    }

    (post_order, predecessor_sets)
}

/// Construct the dominance tree for a DAG with a single root.
///
/// This function uses a topological order approach which is more efficient
/// for DAGs. For a node `u`, its immediate dominator is the LCA (Lowest Common
/// Ancestor) of all its predecessors in the dominance tree.
///
/// # Arguments
/// * `graph`: a directed acyclic graph with exactly one root.
/// * `root`: the root node (node with in-degree 0).
///
/// # Returns
/// * `Ok(Dominators)`: the dominance relation for the given graph and root.
/// * `Err(Cycle)`: if the graph contains a cycle.
///
/// # Complexity
/// * Time complexity: **O((V + E) log V)**
/// * Auxiliary space: **O(V log V)**
///
/// # Errors
/// Returns `Err(Cycle)` if the graph contains a cycle.
pub fn dag_dominators<G>(
    graph: G,
    root: G::NodeId,
) -> Result<Dominators<G::NodeId>, crate::algo::Cycle<G::NodeId>>
where
    G: IntoNeighborsDirected + Visitable + NodeIndexable + IntoNodeIdentifiers,
    <G as GraphBase>::NodeId: Eq + Hash + Copy,
{
    use crate::algo::toposort;

    // Get topological order
    let topological_order = toposort(&graph, None)?;

    // Binary lifting table: binary_lifting[node][k] = 2^k-th ancestor of node
    let mut binary_lifting: HashMap<G::NodeId, Vec<G::NodeId>> = HashMap::new();
    // Depth of each node in the dominance tree
    let mut depth: HashMap<G::NodeId, usize> = HashMap::new();
    // Immediate dominator of each node
    let mut dominators: HashMap<G::NodeId, G::NodeId> = HashMap::new();

    // Initialize root
    dominators.insert(root, root);
    depth.insert(root, 0);

    // Initialize binary lifting for root (root's ancestors are itself)
    let max_log = 0;
    binary_lifting.insert(root, vec![root; max_log + 1]);

    // Process nodes in topological order
    for &u in &topological_order {
        if u == root {
            continue;
        }

        // Get predecessors of u
        let predecessors: Vec<G::NodeId> = graph
            .neighbors_directed(u, EdgeDirection::Incoming)
            .collect();

        if predecessors.is_empty() {
            // Node is not reachable from root, skip
            continue;
        }

        // Find LCA of all predecessors
        let mut lca = predecessors[0];
        for &pred in &predecessors[1..] {
            lca = lca_with_binary_lifting(lca, pred, &binary_lifting, &depth);
        }

        // Set immediate dominator
        dominators.insert(u, lca);

        // Update depth and binary lifting for u
        let u_depth = depth[&lca] + 1;
        depth.insert(u, u_depth);

        // Calculate required log size for binary lifting
        let required_log = (u_depth + 1).next_power_of_two().ilog2() as usize + 1;

        // Update binary lifting table for this node and potentially for ancestors
        let mut ancestors = vec![root; required_log];
        ancestors[0] = lca;

        for k in 1..required_log {
            let ancestor_k_minus_1 = ancestors[k - 1];
            if let Some(ancestor_table) = binary_lifting.get(&ancestor_k_minus_1) {
                if k - 1 < ancestor_table.len() {
                    ancestors[k] = ancestor_table[k - 1];
                }
            }
        }

        binary_lifting.insert(u, ancestors);
    }

    Ok(Dominators { root, dominators })
}

/// Find the Lowest Common Ancestor of two nodes using binary lifting.
fn lca_with_binary_lifting<N>(
    mut u: N,
    mut v: N,
    binary_lifting: &HashMap<N, Vec<N>>,
    depth: &HashMap<N, usize>,
) -> N
where
    N: Copy + Eq + Hash,
{
    // Ensure u and v are in the dominance tree
    if !depth.contains_key(&u) || !depth.contains_key(&v) {
        // If one node is not in the tree, return the other
        return if depth.contains_key(&u) { u } else { v };
    }

    // Make u the deeper node
    let mut du = depth[&u];
    let mut dv = depth[&v];
    if du < dv {
        core::mem::swap(&mut u, &mut v);
        core::mem::swap(&mut du, &mut dv);
    }

    // Bring u to the same depth as v
    let mut diff = du - dv;
    let mut k = 0;
    while diff > 0 {
        if diff % 2 == 1 {
            if let Some(table) = binary_lifting.get(&u) {
                if k < table.len() {
                    u = table[k];
                }
            }
        }
        diff /= 2;
        k += 1;
    }

    if u == v {
        return u;
    }

    // Lift both u and v up until their ancestors match
    if let Some(table_u) = binary_lifting.get(&u) {
        if let Some(table_v) = binary_lifting.get(&v) {
            let max_log = table_u.len().min(table_v.len());
            for k in (0..max_log).rev() {
                if table_u[k] != table_v[k] {
                    u = table_u[k];
                }
            }
        }
    }

    // Return parent of u (or v, they should have the same parent now)
    if let Some(table) = binary_lifting.get(&u) {
        if !table.is_empty() {
            return table[0];
        }
    }
    u
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Directed, Graph};

    #[test]
    fn test_iter_dominators() {
        let doms: Dominators<u32> = Dominators {
            root: 0,
            dominators: [(2, 1), (1, 0), (0, 0)].iter().cloned().collect(),
        };

        let all_doms: Vec<_> = doms.dominators(2).unwrap().collect();
        assert_eq!(vec![2, 1, 0], all_doms);

        assert_eq!(None::<()>, doms.dominators(99).map(|_| unreachable!()));

        let strict_doms: Vec<_> = doms.strict_dominators(2).unwrap().collect();
        assert_eq!(vec![1, 0], strict_doms);

        assert_eq!(
            None::<()>,
            doms.strict_dominators(99).map(|_| unreachable!())
        );

        let dom_by: Vec<_> = doms.immediately_dominated_by(1).collect();
        assert_eq!(vec![2], dom_by);
        assert_eq!(None, doms.immediately_dominated_by(99).next());
        assert_eq!(1, doms.immediately_dominated_by(0).count());
    }

    #[test]
    fn test_dominated_by() {
        // Test with a dominance tree:
        //     0
        //    / \
        //   1   2
        //  / \
        // 3   4
        let doms: Dominators<u32> = Dominators {
            root: 0,
            dominators: [(1, 0), (2, 0), (3, 1), (4, 1), (0, 0)]
                .iter()
                .cloned()
                .collect(),
        };

        // Test dominated_by for root node (should return all other nodes)
        let dom_by_root: Vec<_> = doms.dominated_by(0).collect();
        let mut dom_by_root_sorted = dom_by_root.clone();
        dom_by_root_sorted.sort();
        assert_eq!(vec![1, 2, 3, 4], dom_by_root_sorted);
        assert_eq!(4, dom_by_root.len());

        // Test dominated_by for node 1 (should return 3 and 4)
        let dom_by_1: Vec<_> = doms.dominated_by(1).collect();
        let mut dom_by_1_sorted = dom_by_1.clone();
        dom_by_1_sorted.sort();
        assert_eq!(vec![3, 4], dom_by_1_sorted);
        assert_eq!(2, dom_by_1.len());

        // Test dominated_by for node 2 (should return nothing)
        let dom_by_2: Vec<_> = doms.dominated_by(2).collect();
        assert!(dom_by_2.is_empty());

        // Test dominated_by for node 3 (should return nothing)
        let dom_by_3: Vec<_> = doms.dominated_by(3).collect();
        assert!(dom_by_3.is_empty());

        // Test dominated_by for node that doesn't exist
        let dom_by_99: Vec<_> = doms.dominated_by(99).collect();
        assert!(dom_by_99.is_empty());
    }

    #[test]
    fn test_dag_dominators_complex() {
        // Construct a complex DAG to test dag_dominators
        //
        //       0 (root)
        //      /|\
        //     / | \
        //    /  |  \
        //   1   2   3
        //  /|   |\  |\
        // / |   | \ | \
        //4  5   6  7 8 9
        //|  |   |  | |  |
        //10 11  12 13 14 15
        // \ /    \ /   \ /
        //  16     17    18
        //   \     /     /
        //    \   /     /
        //     \ /     /
        //      19    /
        //       \   /
        //        20

        let mut graph: Graph<(), (), Directed> = Graph::new();
        let nodes: Vec<_> = (0..=20).map(|_| graph.add_node(())).collect();

        // Root's children
        graph.add_edge(nodes[0], nodes[1], ());
        graph.add_edge(nodes[0], nodes[2], ());
        graph.add_edge(nodes[0], nodes[3], ());

        // Level 2 edges
        graph.add_edge(nodes[1], nodes[4], ());
        graph.add_edge(nodes[1], nodes[5], ());
        graph.add_edge(nodes[2], nodes[6], ());
        graph.add_edge(nodes[2], nodes[7], ());
        graph.add_edge(nodes[3], nodes[8], ());
        graph.add_edge(nodes[3], nodes[9], ());

        // Level 3 edges
        graph.add_edge(nodes[4], nodes[10], ());
        graph.add_edge(nodes[5], nodes[11], ());
        graph.add_edge(nodes[6], nodes[12], ());
        graph.add_edge(nodes[7], nodes[13], ());
        graph.add_edge(nodes[8], nodes[14], ());
        graph.add_edge(nodes[9], nodes[15], ());

        // Level 4 edges (converging)
        graph.add_edge(nodes[10], nodes[16], ());
        graph.add_edge(nodes[11], nodes[16], ());
        graph.add_edge(nodes[12], nodes[17], ());
        graph.add_edge(nodes[13], nodes[17], ());
        graph.add_edge(nodes[14], nodes[18], ());
        graph.add_edge(nodes[15], nodes[18], ());

        // Final converging edges
        graph.add_edge(nodes[16], nodes[19], ());
        graph.add_edge(nodes[17], nodes[19], ());
        graph.add_edge(nodes[18], nodes[20], ());
        graph.add_edge(nodes[19], nodes[20], ());

        let root = nodes[0];

        // Get dominators using both algorithms
        let doms_dag = dag_dominators(&graph, root).unwrap();
        let doms_simple = simple_fast(&graph, root);

        // Compare results
        assert_eq!(doms_dag.root(), doms_simple.root());

        // Check that immediate dominators match for all nodes
        for node in graph.node_indices() {
            let idom_dag = doms_dag.immediate_dominator(node);
            let idom_simple = doms_simple.immediate_dominator(node);
            assert_eq!(
                idom_dag, idom_simple,
                "Immediate dominator mismatch for node {:?}: dag={:?}, simple={:?}",
                node, idom_dag, idom_simple
            );
        }
    }

    #[test]
    fn test_dag_dominators_cycle() {
        // Test that dag_dominators returns an error for graphs with cycles
        //
        //   0 ---> 1 ---> 2
        //          ^       |
        //          |_______|

        let mut graph: Graph<(), (), Directed> = Graph::new();
        let nodes: Vec<_> = (0..=2).map(|_| graph.add_node(())).collect();

        graph.add_edge(nodes[0], nodes[1], ());
        graph.add_edge(nodes[1], nodes[2], ());
        graph.add_edge(nodes[2], nodes[1], ()); // Creates a cycle

        let root = nodes[0];

        // dag_dominators should return an error when graph has a cycle
        let result = dag_dominators(&graph, root);
        assert!(result.is_err());
    }
}
