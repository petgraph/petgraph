//! Graph visitor algorithms.
//!

use std::default::Default;
use std::ops::{Add};
use std::collections::{
    BinaryHeap,
    HashSet,
    HashMap,
    Bitv,
    BitvSet,
    RingBuf,
};
use std::collections::hash_map::Hasher;
use std::collections::hash_map::Entry::{
    Occupied,
    Vacant,
};
use std::hash::Hash;

use super::{
    graphmap,
    graph,
    EdgeType,
    EdgeDirection,
    Graph,
    GraphMap,
    MinScored,
};

pub trait Graphlike {
    type NodeId: Clone;
}

/// A graph trait for accessing the neighbors iterator **I**.
pub trait IntoNeighbors<N> : Copy {
    type Iter: Iterator<Item=N>;
    fn neighbors(self, n: N) -> Self::Iter;
}

impl<'a, N: 'a, E> IntoNeighbors<N> for &'a GraphMap<N, E>
where N: Copy + Clone + PartialOrd + Hash<Hasher> + Eq
{
    type Iter = graphmap::Neighbors<'a, N>;
    fn neighbors(self, n: N) -> graphmap::Neighbors<'a, N>
    {
        GraphMap::neighbors(self, n)
    }
}

impl<'a, N, E, Ty: EdgeType> IntoNeighbors< graph::NodeIndex> for &'a Graph<N, E, Ty>
{
    type Iter = graph::Neighbors<'a, E>;
    fn neighbors(self, n: graph::NodeIndex) -> graph::Neighbors<'a, E>
    {
        Graph::neighbors(self, n)
    }
}

/// Wrapper type for walking the graph as if it is undirected
pub struct AsUndirected<G>(pub G);
/// Wrapper type for walking edges the other way
pub struct Reversed<G>(pub G);

impl<'a, 'b, N, E> IntoNeighbors< graph::NodeIndex> for &'a AsUndirected<&'b Graph<N, E>>
{
    type Iter = graph::Neighbors<'a, E>;
    fn neighbors(self, n: graph::NodeIndex) -> graph::Neighbors<'a, E>
    {
        Graph::neighbors_undirected(self.0, n)
    }
}

impl<'a, 'b, N, E, Ty: EdgeType> IntoNeighbors< graph::NodeIndex> for &'a Reversed<&'b Graph<N, E, Ty>>
{
    type Iter = graph::Neighbors<'a, E>;
    fn neighbors(self, n: graph::NodeIndex) -> graph::Neighbors<'a, E>
    {
        Graph::neighbors_directed(self.0, n, EdgeDirection::Incoming)
    }
}

pub trait VisitMap<N> {
    /// Return **true** if the value is not already present.
    fn visit(&mut self, N) -> bool;
    fn contains(&self, &N) -> bool;
}

impl VisitMap<graph::NodeIndex> for BitvSet {
    fn visit(&mut self, x: graph::NodeIndex) -> bool {
        self.insert(x.0)
    }
    fn contains(&self, x: &graph::NodeIndex) -> bool {
        self.contains(&x.0)
    }
}

impl<N: Eq + Hash<Hasher>> VisitMap<N> for HashSet<N> {
    fn visit(&mut self, x: N) -> bool {
        self.insert(x)
    }
    fn contains(&self, x: &N) -> bool {
        self.contains(x)
    }
}

/// Trait for GraphMap that knows which datastructure is the best for its visitor map
pub trait Visitable : Graphlike {
    type Map: VisitMap<<Self as Graphlike>::Item>;
    fn visit_map(&self) -> Self::Map;
}

impl<N, E, Ty> Graphlike for Graph<N, E, Ty> {
    type NodeId = graph::NodeIndex;
}

impl<N, E, Ty> Visitable for Graph<N, E, Ty> where
    Ty: EdgeType,
{
    type Map = BitvSet;
    fn visit_map(&self) -> BitvSet { BitvSet::with_capacity(self.node_count()) }
}

impl<N: Clone, E> Graphlike for GraphMap<N, E>
{
    type NodeId = N;
}

impl<N, E> Visitable for GraphMap<N, E>
    where N: Copy + Clone + Ord + Eq + Hash<Hasher>
{
    type Map = HashSet<N>;
    fn visit_map(&self) -> HashSet<N> { HashSet::with_capacity(self.node_count()) }
}

impl<'a, V: Graphlike> Graphlike for AsUndirected<&'a V>
{
    type NodeId = <V as Graphlike>::NodeId;
}

impl<'a, V: Graphlike> Graphlike for Reversed<&'a V>
{
    type NodeId = <V as Graphlike>::NodeId;
}

impl<'a, V: Visitable> Visitable for AsUndirected<&'a V>
{
    type Map = <V as Visitable>::Map;
    fn visit_map(&self) -> <V as Visitable>::Map {
        self.0.visit_map()
    }
}

impl<'a, V: Visitable> Visitable for Reversed<&'a V>
{
    type Map = <V as Visitable>::Map;
    fn visit_map(&self) -> <V as Visitable>::Map {
        self.0.visit_map()
    }
}


/// “Color” of nodes used in a regular depth-first search
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Color {
    /// Unvisited
    White = 0,
    /// Discovered
    Gray = 1,
    /// Visited
    Black = 2,
}

/// Trait for GraphMap that knows which datastructure is the best for its visitor map
pub trait ColorVisitable : Graphlike {
    type Map;
    fn color_visit_map(&self) -> Self::Map;
}

impl<N, E> ColorVisitable for GraphMap<N, E> where
    N: Clone + Eq + Hash<Hasher>,
{
    type Map = HashMap<N, Color>;
    fn color_visit_map(&self) -> HashMap<N, Color>
    {
        HashMap::new()
    }
}

impl<N, E, Ty> ColorVisitable for Graph<N, E, Ty> where
    Ty: EdgeType,
{
    type Map = Bitv;
    fn color_visit_map(&self) -> Bitv
    {
        Bitv::from_elem(self.node_count() * 2, false)
    }
}

pub trait ColorMap<K> {
    fn color(&self, &K) -> Color;
    fn visit(&mut self, K, Color);
    fn is_white(&self, k: &K) -> bool {
        self.color(k) == Color::White
    }
}

// Use two bits per node.
// 00 => White
// 10 => Gray
// 11 => Black
impl ColorMap<graph::NodeIndex> for Bitv
{
    fn is_white(&self, k: &graph::NodeIndex) -> bool {
        let ix = k.0;
        self[2*ix]
    }

    fn color(&self, k: &graph::NodeIndex) -> Color {
        let ix = k.0;
        let white_bit = self[2*ix];
        let gray_bit = self[2*ix+1];
        if white_bit {
            Color::White
        } else if gray_bit {
            Color::Gray
        } else {
            Color::Black
        }
    }

    fn visit(&mut self, k: graph::NodeIndex, c: Color) {
        let ix = k.0;
        match c {
            Color::White => {
                self.set(2*ix, false);
                self.set(2*ix + 1, false);
            }
            Color::Gray => {
                self.set(2*ix, true);
                self.set(2*ix + 1, false);
            }
            Color::Black => {
                self.set(2*ix, true);
                self.set(2*ix + 1, true);
            }
        }
    }
}

impl<K: Eq + Hash<Hasher>> ColorMap<K> for HashMap<K, Color>
{
    fn color(&self, k: &K) -> Color {
        *self.get(k).unwrap_or(&Color::White)
    }

    fn visit(&mut self, k: K, c: Color) {
        self.insert(k, c);
    }
}

/// A depth first search (DFS) of a graph.
///
/// Using a **Dfs** you can run a traversal over a graph while still retaining
/// mutable access to it, if you use it like the following example:
///
/// ```
/// use petgraph::{Graph, Dfs};
///
/// let mut graph = Graph::<_,()>::new();
/// let a = graph.add_node(0);
///
/// let mut dfs = Dfs::new(&graph, a);
/// while let Some(nx) = dfs.next(&graph) {
///     // we can access `graph` mutably here still
///     graph[nx] += 1;
/// }
///
/// assert_eq!(graph[a], 1);
/// ```
///
/// **Note:** The algorithm may not behave correctly if nodes are removed
/// during iteration. It may not necessarily visit added nodes or edges.
#[derive(Clone)]
pub struct Dfs<N, VM> {
    pub stack: Vec<N>,
    pub discovered: VM,
}

impl<N, G> Dfs<N, <G as Visitable>::Map> where
    N: Clone,
    G: Visitable<NodeId=N>,
    <G as Visitable>::Map: VisitMap<N>,
{
    /// Create a new **Dfs**, using the graph's visitor map.
    ///
    /// **Note:** Does not borrow the graph.
    pub fn new(graph: &G, start: N) -> Self
    {
        let mut discovered = graph.visit_map();
        discovered.visit(start.clone());
        Dfs {
            stack: vec![start],
            discovered: discovered,
        }
    }
}

impl<N, VM> Dfs<N, VM> where
    N: Clone,
    VM: VisitMap<N>
{
    /// Return the next node in the dfs, or **None** if the traversal is done.
    pub fn next<'a, G>(&mut self, graph: &'a G) -> Option<N> where
        &'a G: IntoNeighbors< N>,
        <&'a G as IntoNeighbors< N>>::Iter: Iterator<Item=N>,
    {
        while let Some(node) = self.stack.pop() {
            for succ in graph.neighbors(node.clone()) {
                if self.discovered.visit(succ.clone()) {
                    self.stack.push(succ);
                }
            }

            return Some(node);
        }
        None
    }

}

/// An iterator for a depth first traversal of a graph.
#[derive(Clone)]
pub struct DfsIter<'a, G, N, VM> where
    G: 'a,
{
    graph: &'a G,
    dfs: Dfs<N, VM>,
}

impl<'a, G, N> DfsIter<'a, G, N, <G as Visitable>::Map> where
    N: Clone,
    G: Visitable<NodeId=N>,
    <G as Visitable>::Map: VisitMap<N>,
{
    pub fn new(graph: &'a G, start: N) -> DfsIter<'a, G, N, <G as Visitable>::Map>
    {
        DfsIter {
            graph: graph,
            dfs: Dfs::new(graph, start)
        }
    }
}

impl<'a, G, N, VM> Iterator for DfsIter<'a, G, N, VM> where
    G: 'a,
    N: Clone,
    VM: VisitMap<N>,
    &'a G: IntoNeighbors< N>,
    <&'a G as IntoNeighbors<N>>::Iter: Iterator<Item=N>,
{
    type Item = N;
    fn next(&mut self) -> Option<N>
    {
        self.dfs.next(self.graph)
    }
}

/// A breadth first search (BFS) of a graph.
///
/// Using a **Bfs** you can run a traversal over a graph while still retaining
/// mutable access to it, if you use it like the following example:
///
/// ```
/// use petgraph::{Graph, Bfs};
///
/// let mut graph = Graph::<_,()>::new();
/// let a = graph.add_node(0);
///
/// let mut bfs = Bfs::new(&graph, a);
/// while let Some(nx) = bfs.next(&graph) {
///     // we can access `graph` mutably here still
///     graph[nx] += 1;
/// }
///
/// assert_eq!(graph[a], 1);
/// ```
///
/// **Note:** The algorithm may not behave correctly if nodes are removed
/// during iteration. It may not necessarily visit added nodes or edges.
#[derive(Clone)]
pub struct Bfs<N, VM> {
    pub stack: RingBuf<N>,
    pub discovered: VM,
}

impl<N, G> Bfs<N, <G as Visitable>::Map> where
    N: Clone,
    G: Visitable<NodeId=N>,
    <G as Visitable>::Map: VisitMap<N>,
{
    /// Create a new **Bfs**, using the graph's visitor map.
    ///
    /// **Note:** Does not borrow the graph.
    pub fn new(graph: &G, start: N) -> Self
    {
        let mut discovered = graph.visit_map();
        discovered.visit(start.clone());
        let mut stack = RingBuf::new();
        stack.push_front(start.clone());
        Bfs {
            stack: stack,
            discovered: discovered,
        }
    }
}


impl<N, VM> Bfs<N, VM> where
    N: Clone,
    VM: VisitMap<N>
{
    /// Return the next node in the dfs, or **None** if the traversal is done.
    pub fn next<'a, G>(&mut self, graph: &'a G) -> Option<N> where
        &'a G: IntoNeighbors< N>,
        <&'a G as IntoNeighbors< N>>::Iter: Iterator<Item=N>,
    {
        while let Some(node) = self.stack.pop_front() {
            for succ in graph.neighbors(node.clone()) {
                if self.discovered.visit(succ.clone()) {
                    self.stack.push_back(succ);
                }
            }

            return Some(node);
        }
        None
    }

}

/// An iterator for a breadth first traversal of a graph.
#[derive(Clone)]
pub struct BfsIter<'a, G, N, VM> where
    G: 'a,
{
    graph: &'a G,
    bfs: Bfs<N, VM>,
}

impl<'a, G, N> BfsIter<'a, G, N, <G as Visitable>::Map> where
    N: Clone,
    G: Visitable<NodeId=N>,
    <G as Visitable>::Map: VisitMap<N>,
{
    pub fn new(graph: &'a G, start: N) -> BfsIter<'a, G, N, <G as Visitable>::Map>
    {
        BfsIter {
            graph: graph,
            bfs: Bfs::new(graph, start)
        }
    }
}

impl<'a, G, N, VM> Iterator for BfsIter<'a, G, N, VM> where
    G: 'a,
    N: Clone,
    VM: VisitMap<N>,
    &'a G: IntoNeighbors< N>,
    <&'a G as IntoNeighbors< N>>::Iter: Iterator<Item=N>,
{
    type Item = N;
    fn next(&mut self) -> Option<N>
    {
        self.bfs.next(self.graph)
    }
}

/// Dijkstra's shortest path algorithm.
pub fn dijkstra<'a, G, N, K, F, Edges>(graph: &'a G,
                                       start: N,
                                       goal: Option<N>,
                                       mut edges: F) -> HashMap<N, K> where
    G: Visitable<NodeId=N>,
    N: Clone + Eq + Hash<Hasher>,
    K: Default + Add<Output=K> + Copy + PartialOrd,
    F: FnMut(&'a G, N) -> Edges,
    Edges: Iterator<Item=(N, K)>,
    <G as Visitable>::Map: VisitMap<N>,
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
