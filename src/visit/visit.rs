//! Graph traits and graph traversals.
//!
//! ### The `Into-` Traits
//!
//! Graph traits like [`IntoNeighbors`][in] create iterators and use the same
//! pattern that `IntoIterator` does: the trait takes a reference to a graph,
//! and produces an iterator. These traits are quite composable, but with the
//! limitation that they only use shared references to graphs.
//!
//! ### Visitors
//!
//! [`Dfs`](struct.Dfs.html), [`Bfs`][bfs], [`DfsPostOrder`][dfspo] and
//! [`Topo`][topo]  are basic visitors and they use ”walker” methods: the
//! visitors don't hold the graph as borrowed during traversal, only for the
//! `.next()` call on the walker.
//!
//! [bfs]: struct.Bfs.html
//! [dfspo]: struct.DfsPostOrder.html
//! [topo]: struct.Topo.html
//!
//! ### Other Graph Traits
//!
//! The traits are rather loosely coupled at the moment (which is intentional,
//! but will develop a bit), and there are traits missing that could be added.
//!
//! Not much is needed to be able to use the visitors on a graph. A graph
//! needs to define [`GraphBase`][gb], [`IntoNeighbors`][in] and
//! [`Visitable`][vis] as a minimum.
//!
//! [gb]: trait.GraphBase.html
//! [in]: trait.IntoNeighbors.html
//! [vis]: trait.Visitable.html
//!

mod filter;
mod reversed;
pub use self::filter::*;
pub use self::reversed::*;

mod dfsvisit;
pub use self::dfsvisit::*;

use fixedbitset::FixedBitSet;
use std::collections::{
    HashSet,
    VecDeque,
};
use std::hash::{Hash, BuildHasher};

use prelude::*;

use super::{
    graph,
    EdgeType,
};

use graph::{
    IndexType,
};
#[cfg(feature = "stable_graph")]
use stable_graph;
use graph::Frozen;

#[cfg(feature = "graphmap")]
use graphmap::{
    self,
    NodeTrait,
};

use data::Data;

/// Base graph trait: defines the associated node identifier and
/// edge identifier types.
pub trait GraphBase {
    /// node identifier
    type NodeId: Copy;
    /// edge identifier
    type EdgeId: Copy;
}

impl<'a, G> GraphBase for &'a G where G: GraphBase {
    type NodeId = G::NodeId;
    type EdgeId = G::EdgeId;
}

/// A copyable reference to a graph.
pub trait GraphRef : Copy + GraphBase { }

impl<'a, G> GraphRef for &'a G where G: GraphBase { }

impl<'a, G> GraphBase for Frozen<'a, G> where G: GraphBase {
    type NodeId = G::NodeId;
    type EdgeId = G::EdgeId;
}

#[cfg(feature = "stable_graph")]
impl<'a, N, E: 'a, Ty, Ix> IntoNeighbors for &'a StableGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type Neighbors = stable_graph::Neighbors<'a, E, Ix>;
    fn neighbors(self, n: Self::NodeId) -> Self::Neighbors {
        (*self).neighbors(n)
    }
}


#[cfg(feature = "graphmap")]
impl<'a, N: 'a, E, Ty> IntoNeighbors for &'a GraphMap<N, E, Ty>
    where N: Copy + Ord + Hash,
          Ty: EdgeType,
{
    type Neighbors = graphmap::Neighbors<'a, N, Ty>;
    fn neighbors(self, n: Self::NodeId) -> Self::Neighbors {
        self.neighbors(n)
    }
}

impl<'a, 'b, G> IntoNeighbors for &'b Frozen<'a, G>
    where &'b G: IntoNeighbors,
          G: GraphBase<NodeId=<&'b G as GraphBase>::NodeId>,
{
    type Neighbors = <&'b G as IntoNeighbors>::Neighbors;
    fn neighbors(self, n: G::NodeId) -> Self::Neighbors {
        (**self).neighbors(n)
    }
}


/// Wrapper type for walking the graph as if it is undirected
#[derive(Copy, Clone)]
pub struct AsUndirected<G>(pub G);

impl<'b, N, E, Ty, Ix> IntoNeighbors for AsUndirected<&'b Graph<N, E, Ty, Ix>>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type Neighbors = graph::Neighbors<'b, E, Ix>;

    fn neighbors(self, n: graph::NodeIndex<Ix>) -> graph::Neighbors<'b, E, Ix>
    {
        Graph::neighbors_undirected(self.0, n)
    }
}

/// Access to the neighbors of each node
///
/// The neighbors are, depending on the graph’s edge type:
///
/// - `Directed`: All targets of edges from `a`.
/// - `Undirected`: All other endpoints of edges connected to `a`.
pub trait IntoNeighbors : GraphRef {
    type Neighbors: Iterator<Item=Self::NodeId>;
    /// Return an iterator of the neighbors of node `a`.
    fn neighbors(self, a: Self::NodeId) -> Self::Neighbors;
}

/// Access to the neighbors of each node, through incoming or outgoing edges.
///
/// Depending on the graph’s edge type, the neighbors of a given directionality
/// are:
///
/// - `Directed`, `Outgoing`: All targets of edges from `a`.
/// - `Directed`, `Incoming`: All sources of edges to `a`.
/// - `Undirected`: All other endpoints of edges connected to `a`.
pub trait IntoNeighborsDirected : IntoNeighbors {
    type NeighborsDirected: Iterator<Item=Self::NodeId>;
    fn neighbors_directed(self, n: Self::NodeId, d: Direction)
        -> Self::NeighborsDirected;
}

impl<'a, N, E: 'a, Ty, Ix> IntoNeighbors for &'a Graph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type Neighbors = graph::Neighbors<'a, E, Ix>;
    fn neighbors(self, n: graph::NodeIndex<Ix>)
        -> graph::Neighbors<'a, E, Ix>
    {
        Graph::neighbors(self, n)
    }
}

impl<'a, N, E: 'a, Ty, Ix> IntoNeighborsDirected for &'a Graph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type NeighborsDirected = graph::Neighbors<'a, E, Ix>;
    fn neighbors_directed(self, n: graph::NodeIndex<Ix>, d: Direction)
        -> graph::Neighbors<'a, E, Ix>
    {
        Graph::neighbors_directed(self, n, d)
    }
}

#[cfg(feature = "stable_graph")]
impl<'a, N, E: 'a, Ty, Ix> IntoNeighborsDirected for &'a StableGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type NeighborsDirected = stable_graph::Neighbors<'a, E, Ix>;
    fn neighbors_directed(self, n: graph::NodeIndex<Ix>, d: Direction)
        -> Self::NeighborsDirected
    {
        StableGraph::neighbors_directed(self, n, d)
    }
}

#[cfg(feature = "graphmap")]
impl<'a, N: 'a, E, Ty> IntoNeighborsDirected for &'a GraphMap<N, E, Ty>
    where N: Copy + Ord + Hash,
          Ty: EdgeType,
{
    type NeighborsDirected = graphmap::NeighborsDirected<'a, N, Ty>;
    fn neighbors_directed(self, n: N, dir: Direction)
        -> Self::NeighborsDirected
    {
        self.neighbors_directed(n, dir)
    }
}

pub trait IntoEdges : IntoEdgeReferences {
    type Edges: Iterator<Item=Self::EdgeRef>;
    fn edges(self, a: Self::NodeId) -> Self::Edges;
}

/// A graph with a known node count.
pub trait NodeCount : GraphBase {
    fn node_count(&self) -> usize;
}

impl<'a, G> NodeCount for &'a G
    where G: NodeCount,
{
    fn node_count(&self) -> usize { (*self).node_count() }
}

/// Access to the sequence of the graph’s `NodeId`s.
pub trait IntoNodeIdentifiers : GraphRef {
    type NodeIdentifiers: Iterator<Item=Self::NodeId>;
    fn node_identifiers(self) -> Self::NodeIdentifiers;
    fn node_count(&self) -> usize;
}

impl<'a, G> IntoNodeIdentifiers for &'a G
    where G: IntoNodeIdentifiers,
{
    type NodeIdentifiers = G::NodeIdentifiers;
    fn node_identifiers(self) -> Self::NodeIdentifiers {
        (*self).node_identifiers()
    }
    fn node_count(&self) -> usize {
        (*self).node_count()
    }
}

impl<'a, N, E: 'a, Ty, Ix> IntoNodeIdentifiers for &'a Graph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type NodeIdentifiers = graph::NodeIndices<Ix>;
    fn node_identifiers(self) -> graph::NodeIndices<Ix> {
        Graph::node_indices(self)
    }

    fn node_count(&self) -> usize {
        Graph::node_count(self)
    }
}

#[cfg(feature = "stable_graph")]
impl<'a, N, E: 'a, Ty, Ix> IntoNodeIdentifiers for &'a StableGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type NodeIdentifiers = stable_graph::NodeIndices<'a, N, Ix>;
    fn node_identifiers(self) -> Self::NodeIdentifiers {
        StableGraph::node_indices(self)
    }

    fn node_count(&self) -> usize {
        StableGraph::node_count(self)
    }
}

impl<'a, G> IntoNeighbors for &'a G
    where G: IntoNeighbors
{
    type Neighbors = G::Neighbors;
    fn neighbors(self, n: G::NodeId) -> G::Neighbors {
        (*self).neighbors(n)
    }
}

impl<'a, G> IntoNeighborsDirected for &'a G
    where G: IntoNeighborsDirected
{
    type NeighborsDirected = G::NeighborsDirected;
    fn neighbors_directed(self, n: G::NodeId, d: Direction)
        -> G::NeighborsDirected
    {
        (*self).neighbors_directed(n, d)
    }
}

/// Access to the graph’s nodes without edges to them (`Incoming`) or from them
/// (`Outgoing`).
pub trait IntoExternals : GraphRef {
    type Externals: Iterator<Item=Self::NodeId>;

    /// Return an iterator of all nodes with no edges in the given direction
    fn externals(self, d: Direction) -> Self::Externals;
}

impl<'a, N: 'a, E, Ty, Ix> IntoExternals for &'a Graph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type Externals = graph::Externals<'a, N, Ty, Ix>;
    fn externals(self, d: Direction) -> graph::Externals<'a, N, Ty, Ix> {
        Graph::externals(self, d)
    }
}

/// An edge reference
pub trait EdgeRef : Copy {
    type NodeId;
    type EdgeId;
    type Weight;
    fn source(&self) -> Self::NodeId;
    fn target(&self) -> Self::NodeId;
    fn weight(&self) -> &Self::Weight;
    fn id(&self) -> Self::EdgeId;
}

impl<'a, N, E> EdgeRef for (N, N, &'a E)
    where N: Copy
{
    type NodeId = N;
    type EdgeId = (N, N);
    type Weight = E;

    fn source(&self) -> N { self.0 }
    fn target(&self) -> N { self.1 }
    fn weight(&self) -> &E { self.2 }
    fn id(&self) -> (N, N) { (self.0, self.1) }
}

/// A node reference.
pub trait NodeRef : Copy {
    type NodeId;
    type Weight;
    fn id(&self) -> Self::NodeId;
    fn weight(&self) -> &Self::Weight;
}

/// Access to the sequence of the graph’s nodes
pub trait IntoNodeReferences : Data + IntoNodeIdentifiers {
    type NodeRef: NodeRef<Weight=Self::NodeWeight>;
    type NodeReferences: Iterator<Item=Self::NodeRef>;
    fn node_references(self) -> Self::NodeReferences;
}

impl<'a, G> IntoNodeReferences for &'a G
    where G: IntoNodeReferences
{
    type NodeRef = G::NodeRef;
    type NodeReferences = G::NodeReferences;
    fn node_references(self) -> Self::NodeReferences {
        (*self).node_references()
    }
}

impl<Id> NodeRef for (Id, ())
    where Id: Copy,
{
    type NodeId = Id;
    type Weight = ();
    fn id(&self) -> Self::NodeId { self.0 }
    fn weight(&self) -> &Self::Weight {
        static DUMMY: () = ();
        &DUMMY
    }
}

impl<'a, Id, W> NodeRef for (Id, &'a W)
    where Id: Copy,
{
    type NodeId = Id;
    type Weight = W;
    fn id(&self) -> Self::NodeId { self.0 }
    fn weight(&self) -> &Self::Weight { self.1 }
}


/// Access to the sequence of the graph’s edges
pub trait IntoEdgeReferences : Data + GraphRef {
    type EdgeRef: EdgeRef<NodeId=Self::NodeId, EdgeId=Self::EdgeId,
                          Weight=Self::EdgeWeight>;
    type EdgeReferences: Iterator<Item=Self::EdgeRef>;
    fn edge_references(self) -> Self::EdgeReferences;
}

impl<'a, G> IntoEdgeReferences for &'a G
    where G: IntoEdgeReferences
{
    type EdgeRef = G::EdgeRef;
    type EdgeReferences = G::EdgeReferences;
    fn edge_references(self) -> Self::EdgeReferences {
        (*self).edge_references()
    }
}

#[cfg(feature = "graphmap")]
impl<N, E, Ty> Data for GraphMap<N, E, Ty>
    where N: Copy,
          Ty: EdgeType,
{
    type NodeWeight = N;
    type EdgeWeight = E;
}

pub trait Prop : GraphBase {
    /// The kind edges in the graph.
    type EdgeType: EdgeType;

    fn is_directed(&self) -> bool {
        <Self::EdgeType>::is_directed()
    }
}

impl<'a, G> Prop for &'a G
    where G: Prop
{
    type EdgeType = G::EdgeType;
    fn is_directed(&self) -> bool { (*self).is_directed() }
}

impl<N, E, Ty, Ix> Prop for Graph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type EdgeType = Ty;
}

#[cfg(feature = "graphmap")]
impl<N, E, Ty> Prop for GraphMap<N, E, Ty>
    where N: NodeTrait,
          Ty: EdgeType,
{
    type EdgeType = Ty;
}


#[cfg(feature = "graphmap")]
impl<'a, N: 'a, E: 'a, Ty> IntoEdgeReferences for &'a GraphMap<N, E, Ty>
    where N: NodeTrait,
          Ty: EdgeType,
{
    type EdgeRef = (N, N, &'a E);
    type EdgeReferences = graphmap::AllEdges<'a, N, E, Ty>;
    fn edge_references(self) -> Self::EdgeReferences {
        self.all_edges()
    }
}

impl<'a, N: 'a, E: 'a, Ty, Ix> IntoEdgeReferences for &'a Graph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type EdgeRef = graph::EdgeReference<'a, E, Ix>;
    type EdgeReferences = graph::EdgeReferences<'a, E, Ix>;
    fn edge_references(self) -> Self::EdgeReferences {
        (*self).edge_references()
    }
}

/// The graph’s `NodeId`s map to indices
pub trait NodeIndexable : GraphBase {
    /// Return an upper bound of the node indices in the graph
    /// (suitable for the size of a bitmap).
    fn node_bound(&self) -> usize;
    /// Convert `a` to an integer index.
    fn to_index(a: Self::NodeId) -> usize;
    fn from_index(i: usize) -> Self::NodeId;
}

/// The graph’s `NodeId`s map to indices, in a range without holes.
///
/// The graph's node identifiers correspond to exactly the indices
/// `0..self.node_bound()`.
pub trait NodeCompactIndexable : NodeIndexable { }

impl<'a, G> NodeIndexable for &'a G
    where G: NodeIndexable
{
    fn node_bound(&self) -> usize { (**self).node_bound() }
    fn to_index(ix: Self::NodeId) -> usize { G::to_index(ix) }
    fn from_index(ix: usize) -> Self::NodeId { G::from_index(ix) }
}

impl<'a, G> NodeCompactIndexable for &'a G
    where G: NodeCompactIndexable
{ }

impl<N, E, Ty, Ix> NodeIndexable for Graph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    fn node_bound(&self) -> usize { self.node_count() }
    fn to_index(ix: NodeIndex<Ix>) -> usize { ix.index() }
    fn from_index(ix: usize) -> Self::NodeId { NodeIndex::new(ix) }
}

impl<N, E, Ty, Ix> NodeCompactIndexable for Graph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{ }

/// A mapping for storing the visited status for NodeId `N`.
pub trait VisitMap<N> {
    /// Mark `a` as visited.
    ///
    /// Return **true** if this is the first visit, false otherwise.
    fn visit(&mut self, a: N) -> bool;

    /// Return whether `a` has been visited before.
    fn is_visited(&self, a: &N) -> bool;
}

impl<Ix> VisitMap<graph::NodeIndex<Ix>> for FixedBitSet
    where Ix: IndexType,
{
    fn visit(&mut self, x: graph::NodeIndex<Ix>) -> bool {
        !self.put(x.index())
    }
    fn is_visited(&self, x: &graph::NodeIndex<Ix>) -> bool {
        self.contains(x.index())
    }
}

impl<Ix> VisitMap<graph::EdgeIndex<Ix>> for FixedBitSet
    where Ix: IndexType,
{
    fn visit(&mut self, x: graph::EdgeIndex<Ix>) -> bool {
        !self.put(x.index())
    }
    fn is_visited(&self, x: &graph::EdgeIndex<Ix>) -> bool {
        self.contains(x.index())
    }
}

impl<Ix> VisitMap<Ix> for FixedBitSet
    where Ix: IndexType,
{
    fn visit(&mut self, x: Ix) -> bool {
        !self.put(x.index())
    }
    fn is_visited(&self, x: &Ix) -> bool {
        self.contains(x.index())
    }
}

impl<N, S> VisitMap<N> for HashSet<N, S>
    where N: Hash + Eq,
          S: BuildHasher,
{
    fn visit(&mut self, x: N) -> bool {
        self.insert(x)
    }
    fn is_visited(&self, x: &N) -> bool {
        self.contains(x)
    }
}

/// A graph that can create a map that tracks the visited status of its nodes.
pub trait Visitable : GraphBase {
    /// The associated map type
    type Map: VisitMap<Self::NodeId>;
    /// Create a new visitor map
    fn visit_map(&self) -> Self::Map;
    /// Reset the visitor map (and resize to new size of graph if needed)
    fn reset_map(&self, &mut Self::Map);
}

impl<N, E, Ty, Ix> GraphBase for Graph<N, E, Ty, Ix>
    where Ix: IndexType,
{
    type NodeId = graph::NodeIndex<Ix>;
    type EdgeId = graph::EdgeIndex<Ix>;
}

impl<'a, G> Visitable for &'a G where G: Visitable {
    type Map = G::Map;
    fn visit_map(&self) -> Self::Map { (**self).visit_map() }
    fn reset_map(&self, map: &mut Self::Map) {
        (**self).reset_map(map)
    }
}

impl<N, E, Ty, Ix> Visitable for Graph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type Map = FixedBitSet;
    fn visit_map(&self) -> FixedBitSet { FixedBitSet::with_capacity(self.node_count()) }

    fn reset_map(&self, map: &mut Self::Map) {
        map.clear();
        map.grow(self.node_count());
    }
}

#[cfg(feature = "stable_graph")]
impl<N, E, Ty, Ix> GraphBase for StableGraph<N, E, Ty, Ix>
    where Ix: IndexType,
{
    type NodeId = graph::NodeIndex<Ix>;
    type EdgeId = graph::EdgeIndex<Ix>;
}

#[cfg(feature = "stable_graph")]
impl<N, E, Ty, Ix> Visitable for StableGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type Map = FixedBitSet;
    fn visit_map(&self) -> FixedBitSet {
        FixedBitSet::with_capacity(self.node_bound())
    }
    fn reset_map(&self, map: &mut Self::Map) {
        map.clear();
        map.grow(self.node_bound());
    }
}

#[cfg(feature = "stable_graph")]
impl<N, E, Ty, Ix> Data for StableGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type NodeWeight = N;
    type EdgeWeight = E;
}

impl<'a, G> Visitable for Frozen<'a, G> where G: Visitable {
    type Map = G::Map;
    fn visit_map(&self) -> Self::Map { (**self).visit_map() }
    fn reset_map(&self, map: &mut Self::Map) {
        (**self).reset_map(map)
    }
}

#[cfg(feature = "graphmap")]
impl<N: Copy, E, Ty> GraphBase for GraphMap<N, E, Ty>
{
    type NodeId = N;
    type EdgeId = (N, N);
}

#[cfg(feature = "graphmap")]
impl<N, E, Ty> Visitable for GraphMap<N, E, Ty>
    where N: Copy + Ord + Hash,
          Ty: EdgeType,
{
    type Map = HashSet<N>;
    fn visit_map(&self) -> HashSet<N> { HashSet::with_capacity(self.node_count()) }
    fn reset_map(&self, map: &mut Self::Map) {
        map.clear();
    }
}

impl<G: GraphBase> GraphBase for AsUndirected<G>
{
    type NodeId = G::NodeId;
    type EdgeId = G::EdgeId;
}

impl<G: GraphRef> GraphRef for AsUndirected<G> { }


impl<G: Visitable> Visitable for AsUndirected<G>
{
    type Map = G::Map;
    fn visit_map(&self) -> G::Map {
        self.0.visit_map()
    }
    fn reset_map(&self, map: &mut Self::Map) {
        self.0.reset_map(map);
    }
}

/// Create or access the adjacency matrix of a graph.
///
/// The implementor can either create an adjacency matrix, or it can return
/// a placeholder if it has the needed representation internally.
pub trait GetAdjacencyMatrix : GraphBase {
    /// The associated adjacency matrix type
    type AdjMatrix;
    /// Create the adjacency matrix
    fn adjacency_matrix(&self) -> Self::AdjMatrix;
    /// Return true if there is an edge from `a` to `b`, false otherwise.
    ///
    /// Computes in O(1) time.
    fn is_adjacent(&self, matrix: &Self::AdjMatrix, a: Self::NodeId, b: Self::NodeId) -> bool;
}

impl<'a, G> GetAdjacencyMatrix for &'a G
    where G: GetAdjacencyMatrix,
{
    type AdjMatrix = G::AdjMatrix;
    fn adjacency_matrix(&self) -> Self::AdjMatrix {
        (*self).adjacency_matrix()
    }
    fn is_adjacent(&self, matrix: &Self::AdjMatrix, a: Self::NodeId, b: Self::NodeId) -> bool {
        (*self).is_adjacent(matrix, a, b)
    }
}

#[cfg(feature = "graphmap")]
/// The `GraphMap` keeps an adjacency matrix internally.
impl<N, E, Ty> GetAdjacencyMatrix for GraphMap<N, E, Ty>
    where N: Copy + Ord + Hash,
          Ty: EdgeType,
{
    type AdjMatrix = ();
    #[inline]
    fn adjacency_matrix(&self) { }
    #[inline]
    fn is_adjacent(&self, _: &(), a: N, b: N) -> bool {
        self.contains_edge(a, b)
    }
}

/// Visit nodes of a graph in a depth-first-search (DFS) emitting nodes in
/// preorder (when they are first discovered).
///
/// The traversal starts at a given node and only traverses nodes reachable
/// from it.
///
/// `Dfs` is not recursive.
///
/// `Dfs` does not itself borrow the graph, and because of this you can run
/// a traversal over a graph while still retaining mutable access to it, if you
/// use it like the following example:
///
/// ```
/// use petgraph::Graph;
/// use petgraph::visit::Dfs;
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
    where N: Copy,
          VM: VisitMap<N>,
{
    /// Create a new **Dfs**, using the graph's visitor map, and put **start**
    /// in the stack of nodes to visit.
    pub fn new<G>(graph: G, start: N) -> Self
        where G: GraphRef + Visitable<NodeId=N, Map=VM>
    {
        let mut dfs = Dfs::empty(graph);
        dfs.move_to(start);
        dfs
    }

    /// Create a `Dfs` from a vector and a visit map
    pub fn from_parts(stack: Vec<N>, discovered: VM) -> Self {
        Dfs {
            stack: stack,
            discovered: discovered,
        }
    }

    /// Clear the visit state
    pub fn reset<G>(&mut self, graph: G)
        where G: GraphRef + Visitable<NodeId=N, Map=VM>
    {
        graph.reset_map(&mut self.discovered);
        self.stack.clear();
    }

    /// Create a new **Dfs** using the graph's visitor map, and no stack.
    pub fn empty<G>(graph: G) -> Self
        where G: GraphRef + Visitable<NodeId=N, Map=VM>
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
    pub fn next<G>(&mut self, graph: G) -> Option<N>
        where G: IntoNeighbors<NodeId=N>,
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
pub struct DfsIter<G>
    where G: GraphRef + Visitable,
{
    graph: G,
    dfs: Dfs<G::NodeId, G::Map>,
}

impl<G> DfsIter<G>
    where G: GraphRef + Visitable
{
    pub fn new(graph: G, start: G::NodeId) -> Self {
        DfsIter {
            graph: graph,
            dfs: Dfs::new(graph, start),
        }
    }

    /// Keep the discovered map, but clear the visit stack and restart
    /// the DFS traversal from a particular node.
    pub fn move_to(&mut self, start: G::NodeId) {
        self.dfs.move_to(start)
    }
}

impl<G> Iterator for DfsIter<G>
    where G: GraphRef + Visitable + IntoNeighbors
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

impl<G> Clone for DfsIter<G>
    where G: GraphRef + Visitable,
          Dfs<G::NodeId, G::Map>: Clone
{
    fn clone(&self) -> Self {
        DfsIter {
            graph: self.graph,
            dfs: self.dfs.clone(),
        }
    }
}

/// Visit nodes in a depth-first-search (DFS) emitting nodes in postorder
/// (each node after all its decendants have been emitted).
///
/// `DfsPostOrder` is not recursive.
///
/// The traversal starts at a given node and only traverses nodes reachable
/// from it.
#[derive(Clone, Debug)]
pub struct DfsPostOrder<N, VM> {
    /// The stack of nodes to visit
    pub stack: Vec<N>,
    /// The map of discovered nodes
    pub discovered: VM,
    /// The map of finished nodes
    pub finished: VM,
}

impl<N, VM> DfsPostOrder<N, VM>
    where N: Copy,
          VM: VisitMap<N>,
{
    /// Create a new `DfsPostOrder` using the graph's visitor map, and put
    /// `start` in the stack of nodes to visit.
    pub fn new<G>(graph: G, start: N) -> Self
        where G: GraphRef + Visitable<NodeId=N, Map=VM>
    {
        let mut dfs = Self::empty(graph);
        dfs.move_to(start);
        dfs
    }

    /// Create a new `DfsPostOrder` using the graph's visitor map, and no stack.
    pub fn empty<G>(graph: G) -> Self
        where G: GraphRef + Visitable<NodeId=N, Map=VM>
    {
        DfsPostOrder {
            stack: Vec::new(),
            discovered: graph.visit_map(),
            finished: graph.visit_map(),
        }
    }

    /// Clear the visit state
    pub fn reset<G>(&mut self, graph: G)
        where G: GraphRef + Visitable<NodeId=N, Map=VM>
    {
        graph.reset_map(&mut self.discovered);
        graph.reset_map(&mut self.finished);
        self.stack.clear();
    }

    /// Keep the discovered and finished map, but clear the visit stack and restart
    /// the dfs from a particular node.
    pub fn move_to(&mut self, start: N)
    {
        self.stack.clear();
        self.stack.push(start);
    }

    /// Return the next node in the traversal, or `None` if the traversal is done.
    pub fn next<G>(&mut self, graph: G) -> Option<N>
        where G: IntoNeighbors<NodeId=N>,
    {
        while let Some(&nx) = self.stack.last() {
            if self.discovered.visit(nx) {
                // First time visiting `nx`: Push neighbors, don't pop `nx`
                for succ in graph.neighbors(nx) {
                    if !self.discovered.is_visited(&succ) {
                        self.stack.push(succ);
                    }
                }
            } else {
                self.stack.pop();
                if self.finished.visit(nx) {
                    // Second time: All reachable nodes must have been finished
                    return Some(nx);
                }
            }
        }
        None
    }
}

/// A breadth first search (BFS) of a graph.
///
/// The traversal starts at a given node and only traverses nodes reachable
/// from it.
///
/// `Bfs` is not recursive.
///
/// `Bfs` does not itself borrow the graph, and because of this you can run
/// a traversal over a graph while still retaining mutable access to it, if you
/// use it like the following example:
///
/// ```
/// use petgraph::Graph;
/// use petgraph::visit::Bfs;
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
    where N: Copy,
          VM: VisitMap<N>,
{
    /// Create a new **Bfs**, using the graph's visitor map, and put **start**
    /// in the stack of nodes to visit.
    pub fn new<G>(graph: G, start: N) -> Self
        where G: GraphRef + Visitable<NodeId=N, Map=VM>
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
    pub fn next<G>(&mut self, graph: G) -> Option<N>
        where G: IntoNeighbors<NodeId=N>
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
pub struct BfsIter<G: Visitable> {
    graph: G,
    bfs: Bfs<G::NodeId, G::Map>,
}

impl<G: Visitable> BfsIter<G>
    where G::NodeId: Copy,
          G: GraphRef,
{
    pub fn new(graph: G, start: G::NodeId) -> Self {
        BfsIter {
            graph: graph,
            bfs: Bfs::new(graph, start),
        }
    }
}

impl< G: Visitable> Iterator for BfsIter<G>
    where G: IntoNeighbors,
{
    type Item = G::NodeId;
    fn next(&mut self) -> Option<G::NodeId> {
        self.bfs.next(self.graph)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.bfs.stack.len(), None)
    }
}

impl<G: Visitable> Clone for BfsIter<G>
    where Bfs<G::NodeId, G::Map>: Clone,
          G: GraphRef
{
    fn clone(&self) -> Self {
        BfsIter {
            graph: self.graph,
            bfs: self.bfs.clone(),
        }
    }
}


/// A topological order traversal for a graph.
///
/// **Note** that `Topo` only visits nodes that are not part of cycles,
/// i.e. nodes in a true DAG. Use other visitors like `DfsPostOrder` or
/// algorithms like scc to handle graphs with possible cycles.
#[derive(Clone)]
pub struct Topo<N, VM> {
    tovisit: Vec<N>,
    ordered: VM,
}

impl<N, VM> Topo<N, VM>
    where N: Copy,
          VM: VisitMap<N>,
{
    /// Create a new `Topo`, using the graph's visitor map, and put all
    /// initial nodes in the to visit list.
    pub fn new<G>(graph: G) -> Self
        where G: IntoExternals + Visitable<NodeId=N, Map=VM>,
    {
        let mut topo = Self::empty(graph);
        topo.tovisit.extend(graph.externals(Incoming));
        topo
    }

    /* Private until it has a use */
    /// Create a new `Topo`, using the graph's visitor map with *no* starting
    /// index specified.
    fn empty<G>(graph: G) -> Self
        where G: GraphRef + Visitable<NodeId=N, Map=VM>
    {
        Topo {
            ordered: graph.visit_map(),
            tovisit: Vec::new(),
        }
    }

    /// Clear visited state, and put all initial nodes in the to visit list.
    pub fn reset<G>(&mut self, graph: G)
        where G: IntoExternals + Visitable<NodeId=N, Map=VM>,
    {
        graph.reset_map(&mut self.ordered);
        self.tovisit.clear();
        self.tovisit.extend(graph.externals(Incoming));
    }

    /// Return the next node in the current topological order traversal, or
    /// `None` if the traversal is at the end.
    ///
    /// *Note:* The graph may not have a complete topological order, and the only
    /// way to know is to run the whole traversal and make sure it visits every node.
    pub fn next<G>(&mut self, g: G) -> Option<N>
        where G: IntoNeighborsDirected + Visitable<NodeId=N, Map=VM>,
    {
        // Take an unvisited element and find which of its neighbors are next
        while let Some(nix) = self.tovisit.pop() {
            if self.ordered.is_visited(&nix) {
                continue;
            }
            self.ordered.visit(nix.clone());
            for neigh in g.neighbors(nix) {
                // Look at each neighbor, and those that only have incoming edges
                // from the already ordered list, they are the next to visit.
                if Reversed(g).neighbors(neigh).all(|b| self.ordered.is_visited(&b)) {
                    self.tovisit.push(neigh);
                }
            }
            return Some(nix);
        }
        None
    }
}

