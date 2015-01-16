//! Graph visitor algorithms.
//!

use std::default::Default;
use std::ops::{Add};
use std::collections::{
    BinaryHeap,
    HashSet,
    HashMap,
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

/// A graph trait for accessing the neighbors iterator
pub trait NeighborIter<'a, N> {
    type Iter: Iterator<Item=N>;
    fn neighbors(&'a self, n: N) -> Self::Iter;
}

impl<'a, N, E, Ty: EdgeType> NeighborIter<'a, graph::NodeIndex> for Graph<N, E, Ty>
{
    type Iter = graph::Neighbors<'a, E>;
    fn neighbors(&'a self, n: graph::NodeIndex) -> graph::Neighbors<'a, E>
    {
        Graph::neighbors(self, n)
    }
}

impl<'a, N, E> NeighborIter<'a, N> for GraphMap<N, E>
where N: Copy + Clone + PartialOrd + Hash<Hasher> + Eq
{
    type Iter = graphmap::Neighbors<'a, N>;
    fn neighbors(&'a self, n: N) -> graphmap::Neighbors<'a, N>
    {
        GraphMap::neighbors(self, n)
    }
}

/// Wrapper type for walking the graph as if it is undirected
pub struct AsUndirected<G>(pub G);
/// Wrapper type for walking edges the other way
pub struct Reversed<G>(pub G);

impl<'a, 'b, N, E, Ty: EdgeType> NeighborIter<'a, graph::NodeIndex> for AsUndirected<&'b Graph<N, E, Ty>>
{
    type Iter = graph::Neighbors<'a, E>;
    fn neighbors(&'a self, n: graph::NodeIndex) -> graph::Neighbors<'a, E>
    {
        Graph::neighbors_undirected(self.0, n)
    }
}

impl<'a, 'b, N, E, Ty: EdgeType> NeighborIter<'a, graph::NodeIndex> for Reversed<&'b Graph<N, E, Ty>>
{
    type Iter = graph::Neighbors<'a, E>;
    fn neighbors(&'a self, n: graph::NodeIndex) -> graph::Neighbors<'a, E>
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
    type Map: VisitMap<<Self as Graphlike>::NodeId>;
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

impl<G: Visitable> Dfs<G::NodeId, <G as Visitable>::Map>
{
    /// Create a new **Dfs**, using the graph's visitor map, and put **start**
    /// in the stack of nodes to visit.
    pub fn new(graph: &G, start: G::NodeId) -> Self
    {
        let mut dfs = Dfs::empty(graph);
        dfs.move_to(start);
        dfs
    }

    /// Create a new **Dfs** using the graph's visitor map, and no stack.
    pub fn empty(graph: &G) -> Self
    {
        Dfs {
            stack: Vec::new(),
            discovered: graph.visit_map(),
        }
    }
}

impl<N, VM> Dfs<N, VM> where
    N: Clone,
    VM: VisitMap<N>
{
    /// Keep the discovered map, but clear the visit stack and restart
    /// the dfs from a particular node.
    pub fn move_to(&mut self, start: N)
    {
        self.discovered.visit(start.clone());
        self.stack.clear();
        self.stack.push(start);
    }
}

impl<N, VM> Dfs<N, VM> where
    N: Clone,
    VM: VisitMap<N>
{
    /// Return the next node in the dfs, or **None** if the traversal is done.
    pub fn next<'a, G>(&mut self, graph: &'a G) -> Option<N> where
        G: for<'b> NeighborIter<'b, N>,
        <G as NeighborIter<'a, N>>::Iter: Iterator<Item=N>,
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

impl<'a, G: Visitable> DfsIter<'a, G, G::NodeId, <G as Visitable>::Map>
{
    pub fn new(graph: &'a G, start: G::NodeId) -> DfsIter<'a, G, G::NodeId, <G as Visitable>::Map>
    {
        DfsIter {
            graph: graph,
            dfs: Dfs::new(graph, start)
        }
    }
}

impl<'a, G: Visitable, VM> Iterator for DfsIter<'a, G, G::NodeId, VM> where
    G: 'a,
    VM: VisitMap<G::NodeId>,
    G: for<'b> NeighborIter<'b, G::NodeId>,
    <G as NeighborIter<'a, G::NodeId>>::Iter: Iterator<Item=G::NodeId>,
{
    type Item = G::NodeId;
    fn next(&mut self) -> Option<G::NodeId>
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
    /// Create a new **Bfs**, using the graph's visitor map, and put **start**
    /// in the stack of nodes to visit.
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
        G: for<'b> NeighborIter<'b, N>,
        <G as NeighborIter<'a, N>>::Iter: Iterator<Item=N>,
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

/*
pub struct Visitor<G> where
    G: Visitable,
{
    stack: Vec<<G as Graphlike>::NodeId>,
    discovered: <G as Visitable>::Map,
}

pub fn visitor<G>(graph: &G, start: <G as Graphlike>::NodeId) -> Visitor<G> where
    G: Visitable
{
    Visitor{
        stack: vec![start],
        discovered: graph.visit_map(),
    }
}
*/

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
    G: for<'b> NeighborIter<'b, N>,
    <G as NeighborIter<'a, N>>::Iter: Iterator<Item=N>,
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
