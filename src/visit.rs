use std::collections::{
    HashSet,
    BitvSet,
    RingBuf,
};
use std::collections::hash_map::Hasher;
use std::hash::Hash;

use super::{
    graph,
    digraph,
    ograph,
    EdgeDirection,
    OGraph,
    Graph,
    DiGraph,
};

pub trait Graphlike {
    type NodeId: Clone;
}

/// A graph trait for accessing the neighbors iterator **I**.
pub trait IntoNeighbors<N> : Copy {
    type Iter: Iterator<Item=N>;
    fn neighbors(self, n: N) -> Self::Iter;
}

impl<'a, N: 'a, E> IntoNeighbors<N> for &'a Graph<N, E>
where N: Copy + Clone + PartialOrd + Hash<Hasher> + Eq
{
    type Iter = graph::Neighbors<'a, N>;
    fn neighbors(self, n: N) -> graph::Neighbors<'a, N>
    {
        Graph::neighbors(self, n)
    }
}

impl<'a, N: 'a, E: 'a> IntoNeighbors<N> for &'a DiGraph<N, E>
where N: Copy + Clone + Hash<Hasher> + Eq
{
    type Iter = digraph::Neighbors<'a, N, E>;
    fn neighbors(self, n: N) -> digraph::Neighbors<'a, N, E>
    {
        DiGraph::neighbors(self, n)
    }
}

impl<'a, N, E, Ty: ograph::EdgeType> IntoNeighbors< ograph::NodeIndex> for &'a OGraph<N, E, Ty>
{
    type Iter = ograph::Neighbors<'a, E>;
    fn neighbors(self, n: ograph::NodeIndex) -> ograph::Neighbors<'a, E>
    {
        OGraph::neighbors(self, n)
    }
}

/// Wrapper type for walking the graph as if it is undirected
pub struct AsUndirected<G>(pub G);
/// Wrapper type for walking edges the other way
pub struct Reversed<G>(pub G);

impl<'a, 'b, N, E> IntoNeighbors< ograph::NodeIndex> for &'a AsUndirected<&'b OGraph<N, E>>
{
    type Iter = ograph::Neighbors<'a, E>;
    fn neighbors(self, n: ograph::NodeIndex) -> ograph::Neighbors<'a, E>
    {
        OGraph::neighbors_undirected(self.0, n)
    }
}

impl<'a, 'b, N, E, Ty: ograph::EdgeType> IntoNeighbors< ograph::NodeIndex> for &'a Reversed<&'b OGraph<N, E, Ty>>
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

impl<N: Eq + Hash<Hasher>> VisitMap<N> for HashSet<N> {
    fn visit(&mut self, x: N) -> bool {
        self.insert(x)
    }
    fn contains(&self, x: &N) -> bool {
        self.contains(x)
    }
}

/// Trait for Graph that knows which datastructure is the best for its visitor map
pub trait Visitable : Graphlike {
    type Map: VisitMap<<Self as Graphlike>::Item>;
    fn visit_map(&self) -> Self::Map;
}

impl<N, E, Ty> Graphlike for OGraph<N, E, Ty> {
    type NodeId = ograph::NodeIndex;
}

impl<N, E, Ty> Visitable for OGraph<N, E, Ty> where
    Ty: ograph::EdgeType,
{
    type Map = BitvSet;
    fn visit_map(&self) -> BitvSet { BitvSet::with_capacity(self.node_count()) }
}

impl<N: Clone, E> Graphlike for DiGraph<N, E>
{
    type NodeId = N;
}

impl<N, E> Visitable for DiGraph<N, E>
    where N: Copy + Clone + Eq + Hash<Hasher>
{
    type Map = HashSet<N>;
    fn visit_map(&self) -> HashSet<N> { HashSet::with_capacity(self.node_count()) }
}

impl<N: Clone, E> Graphlike for Graph<N, E>
{
    type NodeId = N;
}

impl<N, E> Visitable for Graph<N, E>
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

/// Trait for Graph that knows which datastructure is the best for its visitor map
pub trait ColorVisitable : Graphlike {
    type Map;
    fn color_visit_map(&self) -> Self::Map;
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

/// A depth first search (DFS) of a graph.
///
/// Using a **Dfs** you can run a traversal over a graph while still retaining
/// mutable access to it, if you use it like the following example:
///
/// ```
/// use petgraph::{OGraph, Dfs};
///
/// let mut graph = OGraph::<_,()>::new();
/// let a = graph.add_node(0);
///
/// let mut dfs = Dfs::new(&graph, a);
/// while let Some(nx) = dfs.next(&graph) {
///     // we can access parts of `graph` mutably here still
///     graph[nx] += 1;
/// }
///
/// assert_eq!(graph.node_weight(a), Some(&1));
/// ```
///
/// **Note:** The algorithm will not behave correctly if nodes are removed
/// during iteration. It will also not necessarily visit added nodes or edges.
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

impl<G> Dfs<<G as Graphlike>::NodeId, <G as Visitable>::Map> where
    G: Visitable,
{
    /// Create a new **Dfs**, using the graph's visitor map.
    ///
    /// **Note:** Does not borrow the graph.
    pub fn new(graph: &G, start: <G as Graphlike>::NodeId) -> Self
    {
        Dfs {
            stack: vec![start],
            visited: graph.visit_map(),
        }
    }
}


impl<N, G> Dfs<N, <G as Visitable>::Map> where
    G: Visitable<NodeId=N>,
{
    /// Run a DFS over a graph.
    pub fn search<'a, F>(graph: &'a G, start: N, mut f: F) -> bool where
        N: Clone,
        &'a G: IntoNeighbors<N>,
        <&'a G as IntoNeighbors<N>>::Iter: Iterator<Item=N>,
        <G as Visitable>::Map: VisitMap<N>,
        F: FnMut(N) -> bool,
    {
        let mut dfs = Dfs::new(graph, start);
        while let Some(node) = dfs.next(graph) {
            if !f(node) {
                return false
            }
        }
        true
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

/// An iterator for a depth first traversal of a graph.
#[derive(Clone)]
pub struct DfsIter<'a, G, N, VM> where
    G: 'a,
    //N: Clone,
    //VM: VisitMap<N>,
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
    <&'a G as IntoNeighbors< N>>::Iter: Iterator<Item=N>,
{
    type Item = N;
    fn next(&mut self) -> Option<N>
    {
        self.dfs.next(self.graph)
    }
}
