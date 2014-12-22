#![feature(macro_rules)]
#![feature(default_type_params)]
extern crate arena;

use std::default::Default;
use std::cell::Cell;
use std::hash::{Writer, Hash};
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::RingBuf;
use std::collections::BinaryHeap;
use std::collections::hash_map::{
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

/// A graph trait for accessing the neighbors iterator **I**.
pub trait GraphNeighbors<'a, N, I> {
    fn neighbors(&'a self, n: N) -> I;
}

impl<'a, N, E> GraphNeighbors<'a, N, graph::Neighbors<'a, N>> for Graph<N, E>
where N: Copy + PartialOrd + Hash + Eq
{
    fn neighbors(&'a self, n: N) -> graph::Neighbors<'a, N>
    {
        Graph::neighbors(self, n)
    }
}

impl<'a, N, E> GraphNeighbors<'a, N, digraph::Neighbors<'a, N, E>> for DiGraph<N, E>
where N: Copy + Hash + Eq
{
    fn neighbors(&'a self, n: N) -> digraph::Neighbors<'a, N, E>
    {
        DiGraph::neighbors(self, n)
    }
}

/// A breadth first traversal of a graph.
#[deriving(Clone)]
pub struct BreadthFirst<'a, G, N>
    where
        G: 'a,
        N: Eq + Hash,
{
    pub graph: &'a G,
    pub stack: RingBuf<N>,
    pub visited: HashSet<N>,
}

impl<'a, G, N, NIter> BreadthFirst<'a, G, N>
    where
        G: 'a + GraphNeighbors<'a, N, NIter>,
        N: Copy + Eq + Hash,
        NIter: Iterator<N>,
{
    pub fn new(graph: &'a G, start: N) -> BreadthFirst<'a, G, N>
    {
        let mut rb = RingBuf::new();
        rb.push_back(start);
        BreadthFirst{
            graph: graph,
            stack: rb,
            visited: HashSet::new(),
        }
    }
}

impl<'a, G, N, NIter> Iterator<N> for BreadthFirst<'a, G, N>
    where
        G: 'a + GraphNeighbors<'a, N, NIter>,
        N: Copy + Eq + Hash,
        NIter: Iterator<N>,
{
    fn next(&mut self) -> Option<N>
    {
        while let Some(node) = self.stack.pop_front() {
            if !self.visited.insert(node) {
                continue;
            }

            for succ in self.graph.neighbors(node) {
                if !self.visited.contains(&succ) {
                    self.stack.push_back(succ);
                }
            }

            return Some(node);
        }
        None
    }
}

/// A depth first traversal of a graph.
#[deriving(Clone)]
pub struct DepthFirst<'a, G, N>
    where
        G: 'a,
        N: Eq + Hash,
{
    pub graph: &'a G,
    pub stack: Vec<N>,
    pub visited: HashSet<N>,
}

impl<'a, G, N, NIter> DepthFirst<'a, G, N>
    where
        G: 'a + GraphNeighbors<'a, N, NIter>,
        N: Copy + Eq + Hash,
        NIter: Iterator<N>,
{
    pub fn new(graph: &'a G, start: N) -> DepthFirst<'a, G, N>
    {
        DepthFirst{
            graph: graph,
            stack: vec![start],
            visited: HashSet::new(),
        }
    }
}

impl<'a, G, N, NIter> Iterator<N> for DepthFirst<'a, G, N>
    where
        G: 'a + GraphNeighbors<'a, N, NIter>,
        N: Copy + Eq + Hash,
        NIter: Iterator<N>,
{
    fn next(&mut self) -> Option<N>
    {
        while let Some(node) = self.stack.pop() {
            if !self.visited.insert(node) {
                continue;
            }

            for succ in self.graph.neighbors(node) {
                if !self.visited.contains(&succ) {
                    self.stack.push(succ);
                }
            }

            return Some(node);
        }
        None
    }
}

