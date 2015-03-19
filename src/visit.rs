//! Graph visitor algorithms.
//!

use std::marker;
use std::collections::{
    HashSet,
    BitSet,
    VecDeque,
};
use std::hash::Hash;

use super::{
    graphmap,
    graph,
    EdgeType,
    EdgeDirection,
    Graph,
    GraphMap,
};

use graph::{
    IndexType,
};

pub trait Graphlike : marker::MarkerTrait {
    type NodeId: Clone;
}

/// A graph trait for accessing the neighbors iterator
pub trait NeighborIter<'a> : Graphlike{
    type Iter: Iterator<Item=Self::NodeId>;
    fn neighbors(&'a self, n: Self::NodeId) -> Self::Iter;
}

impl<'a, N, E, Ty, Ix> NeighborIter<'a> for Graph<N, E, Ty, Ix> where
    Ty: EdgeType,
    Ix: IndexType,
{
    type Iter = graph::Neighbors<'a, E, Ix>;
    fn neighbors(&'a self, n: graph::NodeIndex<Ix>) -> graph::Neighbors<'a, E, Ix>
    {
        Graph::neighbors(self, n)
    }
}

impl<'a, N, E> NeighborIter<'a> for GraphMap<N, E>
where N: Copy + Clone + Ord + Hash + Eq
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

impl<'a, 'b, N, E, Ty, Ix> NeighborIter<'a> for AsUndirected<&'b Graph<N, E, Ty, Ix>> where
    Ty: EdgeType,
    Ix: IndexType,
{
    type Iter = graph::Neighbors<'a, E, Ix>;

    fn neighbors(&'a self, n: graph::NodeIndex<Ix>) -> graph::Neighbors<'a, E, Ix>
    {
        Graph::neighbors_undirected(self.0, n)
    }
}

impl<'a, 'b, N, E, Ty, Ix> NeighborIter<'a> for Reversed<&'b Graph<N, E, Ty, Ix>> where
    Ty: EdgeType,
    Ix: IndexType,
{
    type Iter = graph::Neighbors<'a, E, Ix>;
    fn neighbors(&'a self, n: graph::NodeIndex<Ix>) -> graph::Neighbors<'a, E, Ix>
    {
        Graph::neighbors_directed(self.0, n, EdgeDirection::Incoming)
    }
}

pub trait VisitMap<N> {
    /// Return **true** if the value is not already present.
    fn visit(&mut self, N) -> bool;
    fn is_visited(&self, &N) -> bool;
}

impl<Ix> VisitMap<graph::NodeIndex<Ix>> for BitSet where
    Ix: IndexType,
{
    fn visit(&mut self, x: graph::NodeIndex<Ix>) -> bool {
        self.insert(x.index())
    }
    fn is_visited(&self, x: &graph::NodeIndex<Ix>) -> bool {
        self.contains(&x.index())
    }
}

impl<N: Eq + Hash> VisitMap<N> for HashSet<N> {
    fn visit(&mut self, x: N) -> bool {
        self.insert(x)
    }
    fn is_visited(&self, x: &N) -> bool {
        self.contains(x)
    }
}

/// Trait for GraphMap that knows which datastructure is the best for its visitor map
pub trait Visitable : Graphlike {
    type Map: VisitMap<<Self as Graphlike>::NodeId>;
    fn visit_map(&self) -> Self::Map;
}

impl<N, E, Ty, Ix> Graphlike for Graph<N, E, Ty, Ix> where
    Ix: IndexType,
{
    type NodeId = graph::NodeIndex<Ix>;
}

impl<N, E, Ty, Ix> Visitable for Graph<N, E, Ty, Ix> where
    Ty: EdgeType,
    Ix: IndexType,
{
    type Map = BitSet;
    fn visit_map(&self) -> BitSet { BitSet::with_capacity(self.node_count()) }
}

impl<N: Clone, E> Graphlike for GraphMap<N, E>
{
    type NodeId = N;
}

impl<N, E> Visitable for GraphMap<N, E>
    where N: Copy + Clone + Ord + Eq + Hash
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

/// Create or access the adjacency matrix of a graph
pub trait GetAdjacencyMatrix : Graphlike {
    type AdjMatrix;
    fn adjacency_matrix(&self) -> Self::AdjMatrix;
    fn is_adjacent(&self, matrix: &Self::AdjMatrix, a: Self::NodeId, b: Self::NodeId) -> bool;
}

impl<N, E> GetAdjacencyMatrix for GraphMap<N, E>
    where N: Copy + Clone + Ord + Eq + Hash
{
    type AdjMatrix = ();
    #[inline]
    fn adjacency_matrix(&self) { }
    #[inline]
    fn is_adjacent(&self, _: &(), a: N, b: N) -> bool {
        self.contains_edge(a, b)
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
#[derive(Clone, Debug)]
pub struct Dfs<N, VM> {
    pub stack: Vec<N>,
    pub discovered: VM,
}

impl<G: Visitable> Dfs<G::NodeId, G::Map>
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
        G: Graphlike<NodeId=N>,
        G: for<'b> NeighborIter<'b>,
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
    pub stack: VecDeque<N>,
    pub discovered: VM,
}

impl<G: Visitable> Bfs<G::NodeId, <G as Visitable>::Map> where
    G::NodeId: Clone,
{
    /// Create a new **Bfs**, using the graph's visitor map, and put **start**
    /// in the stack of nodes to visit.
    pub fn new(graph: &G, start: G::NodeId) -> Self
    {
        let mut discovered = graph.visit_map();
        discovered.visit(start.clone());
        let mut stack = VecDeque::new();
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
        G: Graphlike<NodeId=N>,
        G: for<'b> NeighborIter<'b>,
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

