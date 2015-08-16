//! Graph visitor algorithms.
//!

use std::collections::{
    HashSet,
    VecDeque,
};
use std::hash::Hash;
use fb::FixedBitSet;

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

/// Base trait for graphs that defines the node identifier.
pub trait Graphlike {
    type NodeId: Clone;
}

/// A graph trait for accessing the neighbors iterator
pub trait NeighborIter<'a> : Graphlike {
    type Iter: Iterator<Item=Self::NodeId>;

    /// Return an iterator that visits all neighbors of the node **n**.
    fn neighbors(&'a self, n: Self::NodeId) -> Self::Iter;
}

impl<'a, N, E: 'a, Ty, Ix> NeighborIter<'a> for Graph<N, E, Ty, Ix> where
    Ty: EdgeType,
    Ix: IndexType,
{
    type Iter = graph::Neighbors<'a, E, Ix>;
    fn neighbors(&'a self, n: graph::NodeIndex<Ix>) -> graph::Neighbors<'a, E, Ix>
    {
        Graph::neighbors(self, n)
    }
}

impl<'a, N: 'a, E> NeighborIter<'a> for GraphMap<N, E>
where N: Copy + Ord + Hash
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

impl<'a, 'b, N, E: 'a, Ty, Ix> NeighborIter<'a> for AsUndirected<&'b Graph<N, E, Ty, Ix>> where
    Ty: EdgeType,
    Ix: IndexType,
{
    type Iter = graph::Neighbors<'a, E, Ix>;

    fn neighbors(&'a self, n: graph::NodeIndex<Ix>) -> graph::Neighbors<'a, E, Ix>
    {
        Graph::neighbors_undirected(self.0, n)
    }
}

impl<'a, 'b, N, E: 'a, Ty, Ix> NeighborIter<'a> for Reversed<&'b Graph<N, E, Ty, Ix>> where
    Ty: EdgeType,
    Ix: IndexType,
{
    type Iter = graph::Neighbors<'a, E, Ix>;
    fn neighbors(&'a self, n: graph::NodeIndex<Ix>) -> graph::Neighbors<'a, E, Ix>
    {
        Graph::neighbors_directed(self.0, n, EdgeDirection::Incoming)
    }
}

/// A mapping from node → is_visited.
pub trait VisitMap<N> {
    /// Return **true** if the value is not already present.
    fn visit(&mut self, N) -> bool;
    fn is_visited(&self, &N) -> bool;
}

impl<Ix> VisitMap<graph::NodeIndex<Ix>> for FixedBitSet where
    Ix: IndexType,
{
    fn visit(&mut self, x: graph::NodeIndex<Ix>) -> bool {
        let present = self.contains(x.index());
        self.insert(x.index());
        !present
    }
    fn is_visited(&self, x: &graph::NodeIndex<Ix>) -> bool {
        self.contains(x.index())
    }
}

impl<Ix> VisitMap<graph::EdgeIndex<Ix>> for FixedBitSet where
    Ix: IndexType,
{
    fn visit(&mut self, x: graph::EdgeIndex<Ix>) -> bool {
        let present = self.contains(x.index());
        self.insert(x.index());
        !present
    }
    fn is_visited(&self, x: &graph::EdgeIndex<Ix>) -> bool {
        self.contains(x.index())
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

/// Trait for which datastructure to use for a graph’s visitor map
pub trait Visitable : Graphlike {
    type Map: VisitMap<Self::NodeId>;
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
    type Map = FixedBitSet;
    fn visit_map(&self) -> FixedBitSet { FixedBitSet::with_capacity(self.node_count()) }
}

impl<N: Clone, E> Graphlike for GraphMap<N, E>
{
    type NodeId = N;
}

impl<N, E> Visitable for GraphMap<N, E>
    where N: Copy + Ord + Hash
{
    type Map = HashSet<N>;
    fn visit_map(&self) -> HashSet<N> { HashSet::with_capacity(self.node_count()) }
}

impl<'a, V: Graphlike> Graphlike for AsUndirected<&'a V>
{
    type NodeId = V::NodeId;
}

impl<'a, V: Graphlike> Graphlike for Reversed<&'a V>
{
    type NodeId = V::NodeId;
}

impl<'a, V: Visitable> Visitable for AsUndirected<&'a V>
{
    type Map = V::Map;
    fn visit_map(&self) -> V::Map {
        self.0.visit_map()
    }
}

impl<'a, V: Visitable> Visitable for Reversed<&'a V>
{
    type Map = V::Map;
    fn visit_map(&self) -> V::Map {
        self.0.visit_map()
    }
}

/// Create or access the adjacency matrix of a graph
pub trait GetAdjacencyMatrix : Graphlike {
    type AdjMatrix;
    fn adjacency_matrix(&self) -> Self::AdjMatrix;
    fn is_adjacent(&self, matrix: &Self::AdjMatrix, a: Self::NodeId, b: Self::NodeId) -> bool;
}

/// The **GraphMap** keeps an adjacency matrix internally.
impl<N, E> GetAdjacencyMatrix for GraphMap<N, E>
    where N: Copy + Ord + Hash
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
    /// The stack of nodes to visit
    pub stack: Vec<N>,
    /// The map of discovered nodes
    pub discovered: VM,
}

impl<N, VM> Dfs<N, VM>
    where N: Clone,
          VM: VisitMap<N>,
{
    /// Create a new **Dfs**, using the graph's visitor map, and put **start**
    /// in the stack of nodes to visit.
    pub fn new<G>(graph: &G, start: N) -> Self
        where G: Visitable<NodeId=N, Map=VM>
    {
        let mut dfs = Dfs::empty(graph);
        dfs.move_to(start);
        dfs
    }

    /// Create a new **Dfs** using the graph's visitor map, and no stack.
    pub fn empty<G>(graph: &G) -> Self
        where G: Visitable<NodeId=N, Map=VM>
    {
        Dfs {
            stack: Vec::new(),
            discovered: graph.visit_map(),
        }
    }

    /// Keep the discovered map, but clear the visit stack and restart
    /// the dfs from a particular node.
    pub fn move_to(&mut self, start: N)
    {
        self.discovered.visit(start.clone());
        self.stack.clear();
        self.stack.push(start);
    }

    /// Return the next node in the dfs, or **None** if the traversal is done.
    pub fn next<'a, G>(&mut self, graph: &'a G) -> Option<N> where
        G: Graphlike<NodeId=N>,
        G: NeighborIter<'a>,
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
pub struct DfsIter<'a, G: 'a + Visitable>
{
    graph: &'a G,
    dfs: Dfs<G::NodeId, G::Map>,
}

impl<'a, G: Visitable> DfsIter<'a, G>
{
    pub fn new(graph: &'a G, start: G::NodeId) -> Self
    {
        // Inline the code from Dfs::new to
        // work around rust bug #22841
        let mut dfs = Dfs::empty(graph);
        dfs.move_to(start);
        DfsIter {
            graph: graph,
            dfs: dfs,
        }
    }

    /// Keep the discovered map, but clear the visit stack and restart
    /// the DFS traversal from a particular node.
    pub fn move_to(&mut self, start: G::NodeId)
    {
        self.dfs.move_to(start)
    }
}

impl<'a, G: 'a + Visitable> Iterator for DfsIter<'a, G> where
    G: NeighborIter<'a>,
{
    type Item = G::NodeId;

    #[inline]
    fn next(&mut self) -> Option<G::NodeId>
    {
        self.dfs.next(self.graph)
    }

    fn size_hint(&self) -> (usize, Option<usize>)
    {
        // Very vauge info about size of traversal
        (self.dfs.stack.len(), None)
    }
}

impl<'a, G: Visitable> Clone for DfsIter<'a, G> where Dfs<G::NodeId, G::Map>: Clone
{
    fn clone(&self) -> Self {
        DfsIter {
            graph: self.graph,
            dfs: self.dfs.clone(),
        }
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
    /// The queue of nodes to visit
    pub stack: VecDeque<N>,
    /// The map of discovered nodes
    pub discovered: VM,
}

impl<N, VM> Bfs<N, VM>
    where N: Clone,
          VM: VisitMap<N>,
{
    /// Create a new **Bfs**, using the graph's visitor map, and put **start**
    /// in the stack of nodes to visit.
    pub fn new<G>(graph: &G, start: N) -> Self
        where G: Visitable<NodeId=N, Map=VM>
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

    /// Return the next node in the dfs, or **None** if the traversal is done.
    pub fn next<'a, G>(&mut self, graph: &'a G) -> Option<N> where
        G: Graphlike<NodeId=N>,
        G: NeighborIter<'a>,
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
pub struct BfsIter<'a, G: 'a + Visitable> {
    graph: &'a G,
    bfs: Bfs<G::NodeId, G::Map>,
}

impl<'a, G: Visitable> BfsIter<'a, G> where
    G::NodeId: Clone,
{
    pub fn new(graph: &'a G, start: G::NodeId) -> Self
    {
        // Inline the code from Bfs::new to
        // work around rust bug #22841
        let mut discovered = graph.visit_map();
        discovered.visit(start.clone());
        let mut stack = VecDeque::new();
        stack.push_front(start.clone());
        let bfs = Bfs {
            stack: stack,
            discovered: discovered,
        };
        BfsIter {
            graph: graph,
            bfs: bfs,
        }
    }
}

impl<'a, G: 'a + Visitable> Iterator for BfsIter<'a, G> where
    G::NodeId: Clone,
    G: NeighborIter<'a>,
{
    type Item = G::NodeId;
    fn next(&mut self) -> Option<G::NodeId>
    {
        self.bfs.next(self.graph)
    }

    fn size_hint(&self) -> (usize, Option<usize>)
    {
        (self.bfs.stack.len(), None)
    }
}

impl<'a, G: Visitable> Clone for BfsIter<'a, G> where Bfs<G::NodeId, G::Map>: Clone
{
    fn clone(&self) -> Self {
        BfsIter {
            graph: self.graph,
            bfs: self.bfs.clone(),
        }
    }
}
