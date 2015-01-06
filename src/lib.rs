#![feature(old_orphan_check)]
#![feature(associated_types)]
#![feature(slicing_syntax)]
#![feature(macro_rules)]
#![feature(default_type_params)]
extern crate test;

use std::default::Default;
use std::cmp::Ordering;
use std::cell::Cell;
use std::hash::{Writer, Hash};
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::RingBuf;
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
mod scored;
pub mod digraph;
pub mod graph;
pub mod ograph;

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
                F, Edges>(graph: &'a Graph, start: N, mut edges: F) -> HashMap<N, K>
where
    N: Copy + Clone + Eq + Hash + fmt::Show,
    K: Default + Add<Output=K> + Copy + PartialOrd + fmt::Show,
    F: FnMut(&'a Graph, N) -> Edges,
    Edges: Iterator<Item=(N, K)>,
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
            match scores.entry(&next) {
                Occupied(ent) => if next_score < *ent.get() {
                    *ent.into_mut() = next_score;
                    predecessor.insert(next, node);
                } else {
                    next_score = *ent.get();
                },
                Vacant(ent) => {
                    ent.insert(next_score);
                    predecessor.insert(next, node);
                }
            }
            visit_next.push(MinScored(next_score, next));
        }
        visited.insert(node);
    }
    println!("{}", predecessor);
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
        write!(f, "Node({})", self.0.get())
    }
}

/// A graph trait for accessing the neighbors iterator **I**.
pub trait GraphNeighbors<'a, N> {
    type Iter: Iterator<Item=N>;
    fn neighbors(&'a self, n: N) -> Self::Iter;
}

impl<'a, N: 'a, E> GraphNeighbors<'a, N> for Graph<N, E>
where N: Copy + Clone + PartialOrd + Hash + Eq
{
    type Iter = graph::Neighbors<'a, N>;
    fn neighbors(&'a self, n: N) -> graph::Neighbors<'a, N>
    {
        Graph::neighbors(self, n)
    }
}

impl<'a, N: 'a, E: 'a> GraphNeighbors<'a, N> for DiGraph<N, E>
where N: Copy + Clone + Hash + Eq
{
    type Iter = digraph::Neighbors<'a, N, E>;
    fn neighbors(&'a self, n: N) -> digraph::Neighbors<'a, N, E>
    {
        DiGraph::neighbors(self, n)
    }
}

impl<'a, N, E> GraphNeighbors<'a, ograph::NodeIndex> for OGraph<N, E>
{
    type Iter = ograph::Neighbors<'a, E>;
    fn neighbors(&'a self, n: ograph::NodeIndex) -> ograph::Neighbors<'a, E>
    {
        OGraph::neighbors(self, n, EdgeDirection::Outgoing)
    }
}

/// Wrapper type for walking the graph as if it is undirected
pub struct Undirected<G>(pub G);

impl<'a, 'b, N, E> GraphNeighbors<'a, ograph::NodeIndex> for Undirected<&'b OGraph<N, E>>
{
    type Iter = ograph::NeighborsBoth<'a, E>;
    fn neighbors(&'a self, n: ograph::NodeIndex) -> ograph::NeighborsBoth<'a, E>
    {
        OGraph::neighbors_both(self.0, n)
    }
}

/// A breadth first traversal of a graph.
#[derive(Clone)]
pub struct BreadthFirst<'a, G, N>
    where
        G: 'a,
        N: Eq + Hash,
{
    pub graph: &'a G,
    pub stack: RingBuf<N>,
    pub visited: HashSet<N>,
}

impl<'a, G, N> BreadthFirst<'a, G, N>
    where
        G: 'a + GraphNeighbors<'a, N>,
        N: Copy + Eq + Hash,
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

impl<'a, G, N> Iterator for BreadthFirst<'a, G, N>
    where
        G: 'a + GraphNeighbors<'a, N>,
        N: Copy + Eq + Hash,
        <G as GraphNeighbors<'a, N>>::Iter: Iterator<Item=N>,
{
    type Item = N;
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
#[derive(Clone)]
pub struct DepthFirst<'a, G, N>
    where
        G: 'a,
        N: Eq + Hash,
{
    pub graph: &'a G,
    pub stack: Vec<N>,
    pub visited: HashSet<N>,
}

impl<'a, G, N> DepthFirst<'a, G, N>
    where
        G: 'a + GraphNeighbors<'a, N>,
        N: Copy + Eq + Hash,
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

impl<'a, G, N> Iterator for DepthFirst<'a, G, N>
    where
        G: 'a + GraphNeighbors<'a, N>,
        N: Copy + Eq + Hash,
        <G as GraphNeighbors<'a, N>>::Iter: Iterator<Item=N>,
{
    type Item = N;
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

