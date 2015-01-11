#![allow(unstable)]

use std::default::Default;
use std::cmp::Ordering;
use std::cell::Cell;
use std::hash::{self, Hash};
use std::collections::hash_map::Hasher;
use std::collections::HashMap;
use std::collections::BinaryHeap;
use std::collections::hash_map::Entry::{
    Occupied,
    Vacant,
};
use std::fmt;
use std::ops::{Add, Deref};

pub use scored::MinScored;
pub use digraph::DiGraph;
pub use graph::Graph;
pub use ograph::OGraph;

pub use self::EdgeDirection::{Outgoing, Incoming};
pub use visit::{
    Bfs,
    BfsIter,
    Dfs,
    DfsIter,
};
use visit::VisitMap;

mod scored;
pub mod digraph;
pub mod graph;
pub mod ograph;
pub mod visit;

pub mod unionfind;

// Index into the NodeIndex and EdgeIndex arrays
/// Edge direction
#[derive(Copy, Clone, Show, PartialEq)]
pub enum EdgeDirection {
    /// A **Outgoing** edge is an outward edge *from* the current node.
    Outgoing = 0,
    /// An **Incoming** edge is an inbound edge *to* the current node.
    Incoming = 1
}

/// A reference that is hashed and compared by its pointer value.
pub struct Ptr<'b, T: 'b>(pub &'b T);

impl<'b, T> Copy for Ptr<'b, T> {}
impl<'b, T> Clone for Ptr<'b, T>
{
    fn clone(&self) -> Self { *self }
}

fn ptreq<T>(a: &T, b: &T) -> bool {
    a as *const _ == b as *const _
}

impl<'b, T> PartialEq for Ptr<'b, T>
{
    /// Ptr compares by pointer equality, i.e if they point to the same value
    fn eq(&self, other: &Ptr<'b, T>) -> bool {
        ptreq(self.0, other.0)
    }
}

impl<'b, T> PartialOrd for Ptr<'b, T>
{
    fn partial_cmp(&self, other: &Ptr<'b, T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'b, T> Ord for Ptr<'b, T>
{
    /// Ptr is ordered by pointer value, i.e. an arbitrary but stable and total order.
    fn cmp(&self, other: &Ptr<'b, T>) -> Ordering {
        let a = self.0 as *const _;
        let b = other.0 as *const _;
        a.cmp(&b)
    }
}

impl<'b, T> Deref for Ptr<'b, T> {
    type Target = T;
    fn deref<'a>(&'a self) -> &'a T {
        self.0
    }
}

impl<'b, T> Eq for Ptr<'b, T> {}

impl<'b, T, H: hash::Writer + hash::Hasher> Hash<H> for Ptr<'b, T>
{
    fn hash(&self, st: &mut H)
    {
        let ptr = (self.0) as *const _;
        ptr.hash(st)
    }
}

impl<'b, T: fmt::Show> fmt::Show for Ptr<'b, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Dijkstra's shortest path algorithm.
pub fn dijkstra<'a, G, N, K, F, Edges>(graph: &'a G,
                                       start: N,
                                       goal: Option<N>,
                                       mut edges: F) -> HashMap<N, K> where
    G: visit::Visitable<NodeId=N>,
    N: Clone + Eq + Hash<Hasher>,
    K: Default + Add<Output=K> + Copy + PartialOrd,
    F: FnMut(&'a G, N) -> Edges,
    Edges: Iterator<Item=(N, K)>,
    <G as visit::Visitable>::Map: VisitMap<N>,
{
    let mut visited = graph.visit_map();
    let mut scores = HashMap::new();
    let mut predecessor = HashMap::new();
    let mut visit_next = BinaryHeap::new();
    let zero_score: K = Default::default();
    scores.insert(start.clone(), zero_score);
    visit_next.push(MinScored(zero_score, start));
    while let Some(MinScored(node_score, node)) = visit_next.pop() {
        if visited.contains(&node) {
            continue
        }
        for (next, edge) in edges(graph, node.clone()) {
            if visited.contains(&next) {
                continue
            }
            let mut next_score = node_score + edge;
            match scores.entry(next.clone()) {
                Occupied(ent) => if next_score < *ent.get() {
                    *ent.into_mut() = next_score;
                    predecessor.insert(next.clone(), node.clone());
                } else {
                    next_score = *ent.get();
                },
                Vacant(ent) => {
                    ent.insert(next_score);
                    predecessor.insert(next.clone(), node.clone());
                }
            }
            visit_next.push(MinScored(next_score, next));
        }
        if goal.as_ref() == Some(&node) {
            break
        }
        visited.visit(node);
    }
    scores
}

#[derive(Show)]
pub struct Node<T>(pub T);

pub struct NodeCell<T: Copy>(pub Cell<T>);

impl<T: Copy> Deref for NodeCell<T> {
    type Target = Cell<T>;
    #[inline]
    fn deref(&self) -> &Cell<T> {
        &self.0
    }
}

impl<T: Copy + fmt::Show> fmt::Show for NodeCell<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Node({:?})", self.0.get())
    }
}
