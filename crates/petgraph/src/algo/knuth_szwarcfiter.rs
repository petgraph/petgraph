//! Generator for all topological sorts
//! Algorithm used is Donald Knuth's and Jayme Szwarcfiter's
//! algorithm described in "A STRUCTURED PROGRAM TO
//! GENERATE ALL TOPOLOGICAL SORTING ARRANGEMENTS"
use core::hash::Hash;
use std::{collections::VecDeque, iter::Iterator, vec::Vec};

use hashbrown::{HashMap, HashSet};

use crate::algo::{
    Cycle, EdgeRef, IntoEdgeReferences, IntoNodeIdentifiers, NodeIndexable, Visitable,
};

pub struct TopologicalSortGenerator<G>
where
    G: IntoEdgeReferences + IntoNodeIdentifiers + Visitable + NodeIndexable,
{
    delta: VecDeque<G::NodeId>,
    counts: HashMap<G::NodeId, usize>,
    partials: HashMap<G::NodeId, HashSet<G::NodeId>>,
    stored_partials: Vec<(G::NodeId, HashSet<G::NodeId>)>,
    bases: Vec<G::NodeId>,
    seq: Option<Vec<G::NodeId>>,
}

impl<G> TopologicalSortGenerator<G>
where
    G: IntoEdgeReferences + IntoNodeIdentifiers + Visitable + NodeIndexable,
    G::NodeId: Eq + Hash,
{
    fn push_node(&mut self, node: G::NodeId) {
        assert!(!self.divergent_seq_state());
        let targets = self.partials.remove(&node).unwrap();
        for target in &targets {
            if let Some(count) = self.counts.get_mut(target) {
                *count -= 1;
                if *count == 0 {
                    self.delta.push_back(*target);
                }
            }
        }

        self.stored_partials.push((node, targets));
        self.push_seq(node);
    }

    fn pop_node(&mut self) -> G::NodeId {
        assert!(!self.divergent_seq_state());
        let (node, targets) = self.stored_partials.pop().unwrap();

        targets.iter().for_each(|target| {
            self.counts.entry(*target).and_modify(|count| *count += 1);
        });

        // Pop delta until we get a valid node
        while !self.delta.is_empty() && *self.counts.get(self.delta.back().unwrap()).unwrap() != 0 {
            self.delta.pop_back();
        }
        assert!(!self.partials.contains_key(&node));
        self.partials.insert(node, targets);

        let last_seq = self.pop_seq();
        assert!(last_seq == node);

        return node;
    }

    fn push_seq(&mut self, node: G::NodeId) {
        self.seq.as_mut().unwrap().push(node);
    }

    fn pop_seq(&mut self) -> G::NodeId {
        self.seq.as_mut().unwrap().pop().unwrap()
    }

    /// Helper function that says whether
    /// the seq and the algorithm state
    /// have diverged. Useful for
    /// the last node optimization
    fn divergent_seq_state(&self) -> bool {
        self.seq.as_ref().unwrap().len() != self.stored_partials.len()
    }

    fn upstack(&mut self, curr: G::NodeId) {
        self.bases.push(curr);
    }

    fn downstack(&mut self) {
        self.bases.pop();
    }

    fn find_next_ordering(&mut self) -> Result<(), Cycle<G::NodeId>> {
        let mut recurse = self.bases.is_empty();

        while !self.delta.is_empty() && self.partials.len() > 1 {
            let curr = self.delta.pop_back().unwrap();
            if recurse {
                self.upstack(curr)
            }
            self.push_node(curr);
            recurse = true;
        }

        if self.partials.len() == 1 {
            // Small optimization, avoid pushing and popping the
            // last node in an ordering, since it will have no
            // real change to avoid the extra work
            self.push_seq(*self.delta.back().unwrap());
            Ok(())
        } else if self.partials.is_empty() {
            // Exit Condition, we have ran out of nodes to remove
            Ok(())
        } else {
            // Error condition, we ran out of nodes to sequence
            Err(Cycle(*self.partials.iter().next().unwrap().0))
        }
    }

    fn backtrack(&mut self) -> bool {
        if self.divergent_seq_state() {
            self.pop_seq();
        }

        if !self.bases.is_empty() {
            // Retrieve all relations of the form q < j
            let node = self.pop_node();
            self.delta.push_front(node);

            while !self.bases.is_empty()
                && (self.delta.is_empty()
                    || self.delta.back().unwrap() == self.bases.last().unwrap())
            {
                self.downstack();
                if self.bases.is_empty() {
                    break;
                }
                let node = self.pop_node();
                self.delta.push_front(node);
            }
        }

        return !self.bases.is_empty();
    }
}

impl<'a, G> Iterator for TopologicalSortGenerator<G>
where
    G: IntoEdgeReferences + IntoNodeIdentifiers + Visitable + NodeIndexable,
    G::NodeId: Eq + Hash,
{
    type Item = Vec<G::NodeId>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.seq.is_some() && !self.backtrack() {
            // Finished all sorts
            return None;
        }

        if self.seq.is_none() {
            self.seq = Some(Vec::new());
        }

        match self.find_next_ordering() {
            Ok(_) => self.seq.clone(),
            Err(_) => None,
        }
    }
}

/// Create a generator that creates all possible permutations of valid topological sorts of a
/// directed acyclic graph.
///
/// On a graph with no edges, the algorithm with cycle through all V! permutations of nodes.
///
/// On an empty graph, a single empty permutation will return.
///
/// the On a graph with cycles, `TopologicalSortGenerator` will provide an empty list.
///
/// # Arguments
/// * `g`: an acyclic directed graph.
///
/// # Returns
/// * `TopologicalSortGenerator<G>`: A Generator that can be iterated through to create permutations
///
/// # Complexity
/// * Time complexity: **O(|V|! + |E|)**.
/// * Auxiliary space: **O(|V| + |E|)**.
///
/// where **|V|** is the number of nodes and **|E|** is the number of edges.
pub fn all_toposorts<'a, G>(g: G) -> TopologicalSortGenerator<G>
where
    G: IntoEdgeReferences + IntoNodeIdentifiers + Visitable + NodeIndexable,
    G::NodeId: Eq + Hash,
{
    // let mut counts: HashMap<G::NodeId, usize> = HashMap::new();
    let (mut counts, mut partials): (HashMap<_, _>, HashMap<_, _>) = g
        .node_identifiers()
        .map(|n| ((n, 0), (n, HashSet::new())))
        .unzip();

    for e in g.edge_references() {
        counts.entry(e.target()).and_modify(|count| *count += 1);
        partials.entry(e.source()).and_modify(|dests| {
            dests.insert(e.target());
        });
    }

    let delta: VecDeque<G::NodeId> = counts
        .iter()
        .filter(|&(_node, &count)| count == 0)
        .map(|(&node, _count)| node)
        .collect();

    TopologicalSortGenerator::<G> {
        delta,
        counts,
        partials,
        stored_partials: Vec::new(),
        bases: Vec::new(),
        seq: None,
    }
}
