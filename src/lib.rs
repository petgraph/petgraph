#![allow(unstable)]

use std::default::Default;
use std::cmp::Ordering;
use std::cell::Cell;
use std::hash::{self, Hash};
use std::collections::hash_map::Hasher;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::RingBuf;
use std::collections::BitvSet;
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
use ograph::{
    EdgeType,
};

pub use self::EdgeDirection::{Outgoing, Incoming};

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
pub fn dijkstra<'a, Graph, N, K, F, Edges>(graph: &'a Graph,
                                           start: N,
                                           goal: Option<N>,
                                           mut edges: F) -> HashMap<N, K> where
    Graph: Visitable<NodeId=N>,
    N: Copy + Clone + Eq + Hash<Hasher> + fmt::Show,
    K: Default + Add<Output=K> + Copy + PartialOrd + fmt::Show,
    F: FnMut(&'a Graph, N) -> Edges,
    Edges: Iterator<Item=(N, K)>,
    <Graph as Visitable>::Map: VisitMap<N>,
{
    let mut visited = graph.visit_map();
    let mut scores = HashMap::new();
    let mut predecessor = HashMap::new();
    let mut visit_next = BinaryHeap::new();
    let zero_score: K = Default::default();
    scores.insert(start, zero_score);
    visit_next.push(MinScored(zero_score, start));
    while let Some(MinScored(node_score, node)) = visit_next.pop() {
        if visited.contains(&node) {
            continue
        }
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
                    ent.insert(next_score);
                    predecessor.insert(next, node);
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
/// A breadth first traversal of a graph.
#[derive(Clone)]
pub struct BreadthFirst<'a, G, N> where
    G: 'a,
    N: Eq + Hash<Hasher>,
{
    pub graph: &'a G,
    pub stack: RingBuf<N>,
    pub visited: HashSet<N>,
}

impl<'a, G, N> BreadthFirst<'a, G, N> where
    G: 'a,
    &'a G: IntoNeighbors< N>,
    N: Copy + Eq + Hash<Hasher>,
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

impl<'a, G: 'a, N> Iterator for BreadthFirst<'a, G, N> where
    &'a G: IntoNeighbors<N>,
    N: Copy + Eq + Hash<Hasher>,
    <&'a G as IntoNeighbors< N>>::Iter: Iterator<Item=N>,
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

/// An iterator for a depth first traversal of a graph.
#[derive(Clone)]
pub struct DepthFirst<'a, G, N> where
    G: 'a,
    N: Eq + Hash<Hasher>,
{
    pub graph: &'a G,
    pub dfs: Dfs<N, HashSet<N>>,
}

/// A depth first traversal of a graph.
#[derive(Clone)]
pub struct Dfs<N, VM> {
    pub stack: Vec<N>,
    pub visited: VM,
}

impl<N> Dfs<N, HashSet<N>> where N: Hash<Hasher> + Eq
{
    /// Create a new **Dfs**.
    fn new_with_hashset(start: N) -> Self
    {
        Dfs {
            stack: vec![start],
            visited: HashSet::new(),
        }
    }
}

impl Dfs<(), ()>
{
    /// Create a new **Dfs**, using the graph's visitor map.
    ///
    /// **Note:** Does not borrow the graph.
    pub fn new<N, G>(graph: &G, start: N)
            -> Dfs<N, <G as Visitable>::Map> where
        G: Visitable<NodeId=N>,
    {
        Dfs {
            stack: vec![start],
            visited: graph.visit_map(),
        }
    }
}

impl<'a, G, N> DepthFirst<'a, G, N> where
    N: Eq + Hash<Hasher>,
{
    pub fn new(graph: &'a G, start: N) -> DepthFirst<'a, G, N>
    {
        DepthFirst {
            graph: graph,
            dfs: Dfs::new_with_hashset(start)
        }
    }
}

impl<N, VM> Dfs<N, VM> where N: Clone, VM: VisitMap<N>
{
    /// Return the next node in the dfs, or **None** if the traversal is done.
    pub fn next<'a, G>(&mut self, graph: &'a G) -> Option<N> where
        &'a G: IntoNeighbors< N>,
        <&'a G as IntoNeighbors< N>>::Iter: Iterator<Item=N>,
    {
        while let Some(node) = self.stack.pop() {
            if !self.visited.visit(node.clone()) {
                continue;
            }

            for succ in graph.neighbors(node.clone()) {
                if !self.visited.contains(&succ) {
                    self.stack.push(succ);
                }
            }

            return Some(node);
        }
        None
    }
}

impl<'a, G, N> Iterator for DepthFirst<'a, G, N> where
    G: 'a,
    &'a G: IntoNeighbors< N>,
    N: Clone + Eq + Hash<Hasher>,
    <&'a G as IntoNeighbors< N>>::Iter: Iterator<Item=N>,
{
    type Item = N;
    fn next(&mut self) -> Option<N>
    {
        self.dfs.next(self.graph)
    }
}

pub fn depth_first_search<'a, G, N, F>(graph: &'a G, start: N, mut f: F) -> bool where
    G: 'a + Visitable<NodeId=N>,
    &'a G: IntoNeighbors<N>,
    N: Clone,
    <&'a G as IntoNeighbors<N>>::Iter: Iterator<Item=N>,
    <G as Visitable>::Map: VisitMap<N>,
    F: FnMut(N) -> bool,
{
    let mut stack = Vec::new();
    let mut visited = graph.visit_map();

    stack.push(start);
    while let Some(node) = stack.pop() {
        if !visited.visit(node.clone()) {
            continue;
        }

        for succ in graph.neighbors(node.clone()) {
            if !visited.contains(&succ) {
                stack.push(succ);
            }
        }

        if !f(node) {
            return false
        }
    }
    true
}

/// Run a DFS over an **OGraph**, passing a mutable ref to the graph to the
/// iteration function on each step.
///
/// **Note:** The algorithm will not behave correctly if nodes are removed
/// during iteration. It will not necessarily visit added nodes or edges.
pub fn depth_first_search_mut<'a, N, E, Ty, F>(graph: &'a mut OGraph<N, E, Ty>,
                                                start: ograph::NodeIndex,
                                                mut f: F) -> bool where
    Ty: ograph::EdgeType,
    F: FnMut(&mut OGraph<N, E, Ty>, ograph::NodeIndex) -> bool,
{
    let mut dfs = Dfs::new(&*graph, start);
    while let Some(node) = dfs.next(&*graph) {
        if !f(graph, node) {
            return false
        }
    }
    true
}


