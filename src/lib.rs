#![feature(macro_rules)]
#![feature(default_type_params)]
extern crate arena;

use std::default::Default;
use arena::TypedArena;
use std::cell::Cell;
use std::hash::{Writer, Hash};
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::RingBuf;
use std::collections::BinaryHeap;
use std::iter::Map;
use std::collections::hash_map::{
    Keys,
    Occupied,
    Vacant,
};
use std::fmt;

pub use scored::MinScored;
pub use digraph::DiGraph;
pub use graph::Graph;
mod scored;
pub mod digraph;
pub mod graph;


/// A reference that is hashed and compared by its pointer value.
#[deriving(Clone)]
pub struct Ptr<'b, T: 'b>(pub &'b T);

impl<'b, T> Copy for Ptr<'b, T> {}

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

impl<'b, T> Deref<T> for Ptr<'b, T> {
    fn deref<'a>(&'a self) -> &'a T {
        self.0
    }
}

impl<'b, T> Eq for Ptr<'b, T> {}

impl<'b, T, S: Writer> Hash<S> for Ptr<'b, T>
{
    fn hash(&self, st: &mut S)
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

pub fn dijkstra<'a,
                Graph, N, K,
                F, Edges>(graph: &'a Graph, start: N, mut edges: F) -> Vec<(N, K)>
where
    N: Copy + Eq + Hash + fmt::Show,
    K: Default + Add<K, K> + Copy + PartialOrd + fmt::Show,
    F: FnMut(&'a Graph, N) -> Edges,
    Edges: Iterator<(N, K)>,
{
    let mut visited = HashSet::new();
    let mut scores = HashMap::new();
    let mut predecessor = HashMap::new();
    let mut visit_next = BinaryHeap::new();
    let zero_score: K = Default::default();
    scores.insert(start, zero_score);
    visit_next.push(MinScored(zero_score, start));
    loop {
        let MinScored(node_score, node) = match visit_next.pop() {
            None => break,
            Some(t) => t,
        };
        if visited.contains(&node) {
            continue
        }
        println!("Visiting {} with score={}", node, node_score);
        for (next, edge) in edges(graph, node) {
            if visited.contains(&next) {
                continue
            }
            let mut next_score = node_score + edge;
            match scores.entry(next) {
                Occupied(ent) => if next_score < *ent.get() {
                    *ent.into_mut() = next_score;
                    predecessor.insert(next, node);
                } else {
                    next_score = *ent.get();
                },
                Vacant(ent) => {
                    ent.set(next_score);
                    predecessor.insert(next, node);
                }
            }
            visit_next.push(MinScored(next_score, next));
        }
        visited.insert(node);
    }
    println!("{}", predecessor);
    scores.into_iter().collect()
}

#[deriving(Show)]
pub struct Node<T>(pub T);

pub struct NodeCell<T: Copy>(pub Cell<T>);

impl<T: Copy> Deref<Cell<T>> for NodeCell<T> {
    #[inline]
    fn deref(&self) -> &Cell<T> {
        &self.0
    }
}

impl<T: Copy + fmt::Show> fmt::Show for NodeCell<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Node({})", self.0.get())
    }
}
