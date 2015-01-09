#![feature(slicing_syntax)]
extern crate test;

use std::default::Default;
use std::cmp::Ordering;
use std::cell::Cell;
use std::hash::{Writer, Hash};
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

/// Dijkstra's shortest path algorithm.
pub fn dijkstra<'a, Graph, N, K, F, Edges>(graph: &'a Graph,
                                           start: N,
                                           goal: Option<N>,
                                           mut edges: F) -> HashMap<N, K> where
    Graph: Visitable<N>,
    N: Copy + Clone + Eq + Hash + fmt::Show,
    K: Default + Add<Output=K> + Copy + PartialOrd + fmt::Show,
    F: FnMut(&'a Graph, N) -> Edges,
    Edges: Iterator<Item=(N, K)>,
    <Graph as Visitable<N>>::Map: VisitMap<N>,
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
        write!(f, "Node({})", self.0.get())
    }
}

/// A graph trait for accessing the neighbors iterator **I**.
pub trait IntoNeighbors<N> : Copy {
    type Iter: Iterator<Item=N>;
    fn neighbors(self, n: N) -> Self::Iter;
}

impl<'a, N: 'a, E> IntoNeighbors<N> for &'a Graph<N, E>
where N: Copy + Clone + PartialOrd + Hash + Eq
{
    type Iter = graph::Neighbors<'a, N>;
    fn neighbors(self, n: N) -> graph::Neighbors<'a, N>
    {
        Graph::neighbors(self, n)
    }
}

impl<'a, N: 'a, E: 'a> IntoNeighbors<N> for &'a DiGraph<N, E>
where N: Copy + Clone + Hash + Eq
{
    type Iter = digraph::Neighbors<'a, N, E>;
    fn neighbors(self, n: N) -> digraph::Neighbors<'a, N, E>
    {
        DiGraph::neighbors(self, n)
    }
}

impl<'a, N, E, ETy: EdgeType> IntoNeighbors< ograph::NodeIndex> for &'a OGraph<N, E, ETy>
{
    type Iter = ograph::Neighbors<'a, E>;
    fn neighbors(self, n: ograph::NodeIndex) -> ograph::Neighbors<'a, E>
    {
        OGraph::neighbors(self, n)
    }
}

/// Wrapper type for walking the graph as if it is undirected
pub struct Undirected<G>(pub G);
/// Wrapper type for walking edges the other way
pub struct Reversed<G>(pub G);

impl<'a, 'b, N, E> IntoNeighbors< ograph::NodeIndex> for &'a Undirected<&'b OGraph<N, E>>
{
    type Iter = ograph::Neighbors<'a, E>;
    fn neighbors(self, n: ograph::NodeIndex) -> ograph::Neighbors<'a, E>
    {
        OGraph::neighbors_undirected(self.0, n)
    }
}

impl<'a, 'b, N, E, ETy: EdgeType> IntoNeighbors< ograph::NodeIndex> for &'a Reversed<&'b OGraph<N, E, ETy>>
{
    type Iter = ograph::Neighbors<'a, E>;
    fn neighbors(self, n: ograph::NodeIndex) -> ograph::Neighbors<'a, E>
    {
        OGraph::neighbors_directed(self.0, n, EdgeDirection::Incoming)
    }
}

pub trait VisitMap<N> {
    fn visit(&mut self, N) -> bool;
    fn contains(&self, &N) -> bool;
}

impl VisitMap<ograph::NodeIndex> for BitvSet {
    fn visit(&mut self, x: ograph::NodeIndex) -> bool {
        self.insert(x.0)
    }
    fn contains(&self, x: &ograph::NodeIndex) -> bool {
        self.contains(&x.0)
    }
}

impl<N: Eq + Hash> VisitMap<N> for HashSet<N> {
    fn visit(&mut self, x: N) -> bool {
        self.insert(x)
    }
    fn contains(&self, x: &N) -> bool {
        self.contains(x)
    }
}

/// Trait for Graph that knows which datastructure is the best for its visitor map
pub trait Visitable<N> {
    type Map: VisitMap<N>;
    fn visit_map(&self) -> Self::Map;
}

impl<N, E, ETy> Visitable<ograph::NodeIndex> for OGraph<N, E, ETy> where
    ETy: ograph::EdgeType,
{
    type Map = BitvSet;
    fn visit_map(&self) -> BitvSet { BitvSet::with_capacity(self.node_count()) }
}

impl<N, E> Visitable<N> for DiGraph<N, E>
    where N: Copy + Clone + Eq + Hash
{
    type Map = HashSet<N>;
    fn visit_map(&self) -> HashSet<N> { HashSet::with_capacity(self.node_count()) }
}

impl<N, E> Visitable<N> for Graph<N, E>
    where N: Copy + Clone + Ord + Eq + Hash
{
    type Map = HashSet<N>;
    fn visit_map(&self) -> HashSet<N> { HashSet::with_capacity(self.node_count()) }
}

impl<'a, N, V: Visitable<N>> Visitable<N> for Undirected<&'a V>
{
    type Map = <V as Visitable<N>>::Map;
    fn visit_map(&self) -> <V as Visitable<N>>::Map {
        self.0.visit_map()
    }
}

impl<'a, N, V: Visitable<N>> Visitable<N> for Reversed<&'a V>
{
    type Map = <V as Visitable<N>>::Map;
    fn visit_map(&self) -> <V as Visitable<N>>::Map {
        self.0.visit_map()
    }
}

/// A breadth first traversal of a graph.
#[derive(Clone)]
pub struct BreadthFirst<'a, G, N> where
    G: 'a,
    N: Eq + Hash,
{
    pub graph: &'a G,
    pub stack: RingBuf<N>,
    pub visited: HashSet<N>,
}

impl<'a, G, N> BreadthFirst<'a, G, N> where
    G: 'a,
    &'a G: IntoNeighbors< N>,
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

impl<'a, G: 'a, N> Iterator for BreadthFirst<'a, G, N> where
    &'a G: IntoNeighbors<N>,
    N: Copy + Eq + Hash,
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
    N: Eq + Hash,
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

impl<N> Dfs<N, HashSet<N>> where N: Hash + Eq
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
            -> Dfs<N, <G as Visitable<N>>::Map> where
        G: Visitable<N>,
    {
        Dfs {
            stack: vec![start],
            visited: graph.visit_map(),
        }
    }
}

impl<'a, G, N> DepthFirst<'a, G, N> where
    N: Eq + Hash,
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
    pub fn next_node<'a, G>(&mut self, graph: &'a G) -> Option<N> where
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
    N: Clone + Eq + Hash,
    <&'a G as IntoNeighbors< N>>::Iter: Iterator<Item=N>,
{
    type Item = N;
    fn next(&mut self) -> Option<N>
    {
        self.dfs.next_node(self.graph)
    }
}

pub fn depth_first_search<'a, G, N, F>(graph: &'a G, start: N, mut f: F) -> bool where
    G: 'a + Visitable<N>,
    &'a G: IntoNeighbors<N>,
    N: Clone,
    <&'a G as IntoNeighbors<N>>::Iter: Iterator<Item=N>,
    <G as Visitable<N>>::Map: VisitMap<N>,
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
pub fn depth_first_search_mut<'a, N, E, ETy, F>(graph: &'a mut OGraph<N, E, ETy>,
                                                start: ograph::NodeIndex,
                                                mut f: F) -> bool where
    ETy: ograph::EdgeType,
    F: FnMut(&mut OGraph<N, E, ETy>, ograph::NodeIndex) -> bool,
{
    let mut dfs = Dfs::new(&*graph, start);
    while let Some(node) = dfs.next_node(&*graph) {
        if !f(graph, node) {
            return false
        }
    }
    true
}


