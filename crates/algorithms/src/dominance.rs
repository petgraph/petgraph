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

use alloc::{vec, vec::Vec};
use core::{cmp::Ordering, hash::Hash};

use fxhash::FxBuildHasher;
use petgraph_core::deprecated::visit::{DfsPostOrder, GraphBase, IntoNeighbors, Visitable, Walker};

use crate::common::{IndexMap, IndexSet};

/// The dominance relation for some graph and root.
#[derive(Debug, Clone)]
pub struct Dominators<N>
where
    N: Copy + Eq + Hash,
{
    root: N,
    dominators: IndexMap<N, N>,
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
    pub fn strict_dominators(&self, node: N) -> Option<DominatorsIter<N>> {
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
    pub fn dominators(&self, node: N) -> Option<DominatorsIter<N>> {
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
    pub fn immediately_dominated_by(&self, node: N) -> DominatedByIter<N> {
        DominatedByIter {
            iter: self.dominators.iter(),
            node,
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
    iter: indexmap::map::Iter<'a, N, N>,
    node: N,
}

impl<'a, N> Iterator for DominatedByIter<'a, N>
where
    N: 'a + Copy + Eq + Hash,
{
    type Item = N;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.iter.next() {
            if next.1 == &self.node {
                return Some(*next.0);
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.iter.size_hint();
        (0, upper)
    }
}

/// The undefined dominator sentinel, for when we have not yet discovered a
/// node's dominator.
const UNDEFINED: usize = usize::MAX;

/// This is an implementation of the engineered ["Simple, Fast Dominance
/// Algorithm"][0] discovered by Cooper et al.
///
/// This algorithm is **O(|V|²)**, and therefore has slower theoretical running time
/// than the Lengauer-Tarjan algorithm (which is **O(|E| log |V|)**. However,
/// Cooper et al found it to be faster in practice on control flow graphs of up
/// to ~30,000 vertices.
///
/// [0]: http://www.cs.rice.edu/~keith/EMBED/dom.pdf
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
    let node_to_post_order_idx: IndexMap<_, _> = post_order
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

    debug_assert!(!dominators.iter().any(|&dom| dom == UNDEFINED));

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
    node_to_post_order_idx: &IndexMap<N, usize>,
    mut predecessor_sets: IndexMap<N, IndexSet<N>>,
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
                .unwrap_or_else(Vec::new)
        })
        .collect()
}

type PredecessorSets<NodeId> = IndexMap<NodeId, IndexSet<NodeId>>;

fn simple_fast_post_order<G>(
    graph: G,
    root: G::NodeId,
) -> (Vec<G::NodeId>, PredecessorSets<G::NodeId>)
where
    G: IntoNeighbors + Visitable,
    <G as GraphBase>::NodeId: Eq + Hash,
{
    let mut post_order = vec![];
    let mut predecessor_sets = IndexMap::with_hasher(FxBuildHasher::default());

    for node in DfsPostOrder::new(graph, root).iter(graph) {
        post_order.push(node);

        for successor in graph.neighbors(node) {
            predecessor_sets
                .entry(successor)
                .or_insert_with(|| IndexSet::with_hasher(FxBuildHasher::default()))
                .insert(node);
        }
    }

    (post_order, predecessor_sets)
}

#[cfg(test)]
mod tests {
    use petgraph_graph::{Graph, NodeIndex};

    use super::*;

    fn setup() -> Dominators<i32> {
        Dominators {
            root: 0,
            dominators: [(2, 1), (1, 0), (0, 0)].into_iter().collect(),
        }
    }

    #[test]
    fn iterator_dominators() {
        let dominators = setup();
        let all_dominators: Vec<_> = dominators.dominators(2).unwrap().collect();
        assert_eq!(all_dominators, [2, 1, 0]);
    }

    #[test]
    fn iterator_dominators_does_not_exist() {
        let dominators = setup();

        assert!(dominators.dominators(i32::MAX).is_none());
    }

    #[test]
    fn iterator_strict_dominators() {
        let dominators = setup();

        let strict_dominators: Vec<_> = dominators.strict_dominators(2).unwrap().collect();
        assert_eq!(strict_dominators, [1, 0]);
    }

    #[test]
    fn iterator_immediately_dominated_by() {
        let dominators = setup();

        let dominated_by: Vec<_> = dominators.immediately_dominated_by(1).collect();
        assert_eq!(dominated_by, [2]);
    }

    /// Example from <https://en.wikipedia.org/wiki/Dominator_(graph_theory)>
    #[test]
    fn simple_fast_wikipedia() {
        let mut graph = Graph::new();

        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");
        let d = graph.add_node("D");
        let e = graph.add_node("E");
        let f = graph.add_node("F");

        graph.extend_with_edges([
            (a, b, "A → B"), //
            (b, c, "B → C"),
            (b, d, "B → D"),
            (b, f, "B → F"),
            (c, e, "C → E"),
            (d, b, "D → B"),
            (d, e, "D → E"),
        ]);

        let dominators = simple_fast(&graph, a);

        assert_eq!(dominators.root(), a);
        assert_eq!(dominators.immediate_dominator(a), None);
        assert_eq!(dominators.immediate_dominator(b), Some(a));
        assert_eq!(dominators.immediate_dominator(c), Some(b));
        assert_eq!(dominators.immediate_dominator(d), Some(b));
        assert_eq!(dominators.immediate_dominator(e), Some(b));
        assert_eq!(dominators.immediate_dominator(f), Some(b));
    }

    /// Example extracted from the research paper
    /// <http://www.cs.princeton.edu/courses/archive/spr03/cs423/download/dominators.pdf>
    #[test]
    fn simple_fast_princeton() {
        // Fig 1 of the paper
        let mut graph = Graph::new();

        let r = graph.add_node("R");
        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");
        let d = graph.add_node("D");
        let e = graph.add_node("E");
        let f = graph.add_node("F");
        let g = graph.add_node("G");
        let h = graph.add_node("H");
        let i = graph.add_node("I");
        let j = graph.add_node("J");
        let k = graph.add_node("K");
        let l = graph.add_node("L");

        graph.extend_with_edges([
            (r, a, "R → A"), //
            (r, b, "R → B"),
            (r, c, "R → C"),
            (a, d, "A → D"),
            (b, a, "B → A"),
            (b, d, "B → D"),
            (b, e, "B → E"),
            (c, f, "C → F"),
            (c, g, "C → G"),
            (d, l, "D → L"),
            (e, h, "E → H"),
            (f, i, "F → I"),
            (g, i, "G → I"),
            (g, j, "G → J"),
            (h, e, "H → E"),
            (h, k, "H → K"),
            (i, k, "I → K"),
            (j, i, "J → I"),
            (k, r, "K → R"),
            (k, i, "K → I"),
            (l, h, "L → H"),
        ]);

        let dominators = simple_fast(&graph, NodeIndex::new(0));

        // Fig. 2 of the paper
        assert_eq!(dominators.root(), r);
        assert_eq!(dominators.immediate_dominator(r), None);
        assert_eq!(dominators.immediate_dominator(a), Some(r));
        assert_eq!(dominators.immediate_dominator(b), Some(r));
        assert_eq!(dominators.immediate_dominator(c), Some(r));
        assert_eq!(dominators.immediate_dominator(d), Some(r));
        assert_eq!(dominators.immediate_dominator(e), Some(r));
        assert_eq!(dominators.immediate_dominator(f), Some(c));
        assert_eq!(dominators.immediate_dominator(g), Some(c));
        assert_eq!(dominators.immediate_dominator(h), Some(r));
        assert_eq!(dominators.immediate_dominator(i), Some(r));
        assert_eq!(dominators.immediate_dominator(j), Some(g));
        assert_eq!(dominators.immediate_dominator(k), Some(r));
        assert_eq!(dominators.immediate_dominator(l), Some(d));
    }

    #[test]
    fn simple_fast_disconnected() {
        let mut graph = Graph::<_, ()>::new();

        let a = graph.add_node("A");
        let b = graph.add_node("B");

        let dominators = simple_fast(&graph, a);

        assert_eq!(dominators.root(), a);
        assert_eq!(dominators.immediate_dominator(a), None);
        // nodes that aren't reachable from the root have no dominator
        assert_eq!(dominators.immediate_dominator(b), None);
    }

    #[test]
    fn simple_fast_unreachable() {
        let mut graph = Graph::new();

        let a = graph.add_node("A");
        let b = graph.add_node("B");

        graph.add_edge(b, a, "B → A");

        let dominators = simple_fast(&graph, a);

        assert_eq!(dominators.root(), a);
        assert_eq!(dominators.immediate_dominator(a), None);
        assert_eq!(dominators.immediate_dominator(b), None);
    }
}
