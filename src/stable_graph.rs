//! `StableGraph` keeps indices stable across removals.
//!
//! Depends on `feature = "stable_graph"`.
//!

use std::cmp;
use std::fmt;
use std::iter;
use std::marker::PhantomData;
use std::mem::replace;
use std::mem::size_of;
use std::ops::{Index, IndexMut};
use std::slice;

use {
    Graph,
    EdgeType,
    Directed,
    Undirected,
    Direction,
    Incoming,
    Outgoing,
};

use iter_format::{
    IterFormatExt,
    NoPretty,
    DebugMap,
};

use super::{
    Edge,
    index_twice,
    Node,
    DIRECTIONS,
    Pair,
    Frozen,
};
use IntoWeightedEdge;
use visit::{
    EdgeRef,
    IntoEdges,
    IntoEdgeReferences,
    NodeIndexable,
};

// reexport those things that are shared with Graph
#[doc(no_inline)]
pub use graph::{
    NodeIndex,
    EdgeIndex,
    IndexType,
    DefaultIx,
    node_index,
    edge_index,
};

/// `StableGraph<N, E, Ty, Ix>` is a graph datastructure using an adjacency
/// list representation.
///
/// The graph **does not invalidate** any unrelated node or edge indices when
/// items are removed.
///
/// `StableGraph` is parameterized over:
///
/// - Associated data `N` for nodes and `E` for edges, also called *weights*.
///   The associated data can be of arbitrary type.
/// - Edge type `Ty` that determines whether the graph edges are directed or undirected.
/// - Index type `Ix`, which determines the maximum size of the graph.
///
/// The graph uses **O(|V| + |E|)** space, and allows fast node and edge insert
/// and efficient graph search.
///
/// It implements **O(e')** edge lookup and edge and node removals, where **e'**
/// is some local measure of edge count.
///
/// - Nodes and edges are each numbered in an interval from *0* to some number
/// *m*, but *not all* indices in the range are valid, since gaps are formed
/// by deletions.
///
/// - You can select graph index integer type after the size of the graph. A smaller
/// size may have better performance.
///
/// - Using indices allows mutation while traversing the graph, see `Dfs`.
///
/// - The `StableGraph` is a regular rust collection and is `Send` and `Sync`
/// (as long as associated data `N` and `E` are).
///
/// - Indices don't allow as much compile time checking as references.
///
/// Depends on crate feature `stable_graph` (default). *This is a new feature in
/// petgraph.  You can contribute to help it achieve parity with Graph.*
pub struct StableGraph<N, E, Ty = Directed, Ix = DefaultIx>
{
    g: Graph<Option<N>, Option<E>, Ty, Ix>,
    node_count: usize,
    edge_count: usize,
    free_node: NodeIndex<Ix>,
    free_edge: EdgeIndex<Ix>,
}

/// A `StableGraph` with directed edges.
///
/// For example, an edge from *1* to *2* is distinct from an edge from *2* to
/// *1*.
pub type StableDiGraph<N, E, Ix = DefaultIx> = StableGraph<N, E, Directed, Ix>;

/// A `StableGraph` with undirected edges.
///
/// For example, an edge between *1* and *2* is equivalent to an edge between
/// *2* and *1*.
pub type StableUnGraph<N, E, Ix = DefaultIx> = StableGraph<N, E, Undirected, Ix>;

impl<N, E, Ty, Ix> fmt::Debug for StableGraph<N, E, Ty, Ix>
    where N: fmt::Debug,
          E: fmt::Debug,
          Ty: EdgeType,
          Ix: IndexType,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let etype = if self.is_directed() { "Directed" } else { "Undirected" };
        let mut fmt_struct = f.debug_struct("StableGraph");
        fmt_struct.field("Ty", &etype);
        fmt_struct.field("edges", &self.g.edges.iter()
            .filter(|e| e.weight.is_some())
            .map(|e| NoPretty((e.source().index(), e.target().index())))
            .format(", "));
        // skip weights if they are ZST!
        if size_of::<N>() != 0 {
            fmt_struct.field("node weights",
            &DebugMap(||
                self.g.nodes.iter()
                    .map(|n| n.weight.as_ref())
                    .enumerate()
                    .filter_map(|(i, wo)| wo.map(move |w| (i, w)))
                   ));
        }
        if size_of::<E>() != 0 {
            fmt_struct.field("edge weights",
            &DebugMap(||
                self.g.edges.iter()
                    .map(|n| n.weight.as_ref())
                    .enumerate()
                    .filter_map(|(i, wo)| wo.map(move |w| (i, w)))
                   ));
        }
        fmt_struct.field("free_node", &self.free_node);
        fmt_struct.field("free_edge", &self.free_edge);
        fmt_struct.field("node_count", &self.node_count);
        fmt_struct.field("edge_count", &self.edge_count);
        fmt_struct.finish()
    }
}


impl<N, E> StableGraph<N, E, Directed> {
    /// Create a new `StableGraph` with directed edges.
    ///
    /// This is a convenience method. See `StableGraph::with_capacity`
    /// or `StableGraph::default` for a constructor that is generic in all the
    /// type parameters of `StableGraph`.
    pub fn new() -> Self {
        Self::with_capacity(0, 0)
    }
}

impl<N, E, Ty, Ix> StableGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    /// Create a new `StableGraph` with estimated capacity.
    pub fn with_capacity(nodes: usize, edges: usize) -> Self {
        StableGraph {
            g: Graph::with_capacity(nodes, edges),
            node_count: 0,
            edge_count: 0,
            free_node: NodeIndex::end(),
            free_edge: EdgeIndex::end(),
        }
    }

    /// Return the current node and edge capacity of the graph.
    pub fn capacity(&self) -> (usize, usize) {
        self.g.capacity()
    }

    /// Remove all nodes and edges
    pub fn clear(&mut self) {
        self.node_count = 0;
        self.edge_count = 0;
        self.free_node = NodeIndex::end();
        self.free_edge = EdgeIndex::end();
        self.g.clear();
    }

    /// Return the number of nodes (vertices) in the graph.
    ///
    /// Computes in **O(1)** time.
    pub fn node_count(&self) -> usize {
        self.node_count
    }

    /// Return the number of edges in the graph.
    ///
    /// Computes in **O(1)** time.
    pub fn edge_count(&self) -> usize {
        self.edge_count
    }

    /// Whether the graph has directed edges or not.
    #[inline]
    pub fn is_directed(&self) -> bool {
        Ty::is_directed()
    }

    /// Add a node (also called vertex) with associated data `weight` to the graph.
    ///
    /// Computes in **O(1)** time.
    ///
    /// Return the index of the new node.
    ///
    /// **Panics** if the Graph is at the maximum number of nodes for its index
    /// type.
    pub fn add_node(&mut self, weight: N) -> NodeIndex<Ix> {
        let index = if self.free_node != NodeIndex::end() {
            let node_idx = self.free_node;
            let node_slot = &mut self.g.nodes[node_idx.index()];
            let _old = replace(&mut node_slot.weight, Some(weight));
            debug_assert!(_old.is_none());
            self.free_node = node_slot.next[0]._into_node();
            node_slot.next[0] = EdgeIndex::end();
            node_idx
        } else {
            self.g.add_node(Some(weight))
        };
        self.node_count += 1;
        index
    }

    /// Remove `a` from the graph if it exists, and return its weight.
    /// If it doesn't exist in the graph, return `None`.
    ///
    /// The node index `a` is invalidated, but none other.
    /// Edge indices are invalidated as they would be following the removal of
    /// each edge with an endpoint in `a`.
    ///
    /// Computes in **O(e')** time, where **e'** is the number of affected
    /// edges, including *n* calls to `.remove_edge()` where *n* is the number
    /// of edges with an endpoint in `a`.
    pub fn remove_node(&mut self, a: NodeIndex<Ix>) -> Option<N> {
        let node_weight = match self.g.nodes.get_mut(a.index()) {
            None => return None,
            Some(n) => n.weight.take(),
        };
        if let None = node_weight {
            return None;
        }
        for d in &DIRECTIONS {
            let k = d.index();

            // Remove all edges from and to this node.
            loop {
                let next = self.g.nodes[a.index()].next[k];
                if next == EdgeIndex::end() {
                    break
                }
                let ret = self.remove_edge(next);
                debug_assert!(ret.is_some());
                let _ = ret;
            }
        }

        let node_slot = &mut self.g.nodes[a.index()];
        //let node_weight = replace(&mut self.g.nodes[a.index()].weight, Entry::Empty(self.free_node));
        //self.g.nodes[a.index()].next = [EdgeIndex::end(), EdgeIndex::end()];
        node_slot.next = [self.free_node._into_edge(), EdgeIndex::end()];
        self.free_node = a;
        self.node_count -= 1;

        node_weight
    }

    /// Retain only nodes from the graph if the `f` returns true and return the weights of the
    /// removed nodes. If no nodes are removed, the Vec is empty.
    ///
    /// The node indecies of the removed nodes are invalidated, but none other.
    /// Edge indices are invalidated as they would be following the removal of
    /// each edge with an endpoint in a removed node.
    ///
    /// Computes in **O(n + e')** time, where **n** is the number of node indices and
    ///  **e'** is the number of affected edges, including *n* calls to `.remove_edge()`
    /// where *n* is the number of edges with an endpoint in a removed node.
    pub fn retain_nodes<F>(&mut self, mut f: F) where F: FnMut(Frozen<Self>, NodeIndex<Ix>) -> bool {
        for i in 0..self.g.node_count() {
            let ix = node_index(i);
            if !f(Frozen(self), ix) {
                self.remove_node(ix);
            }
        }
    }

    pub fn contains_node(&self, a: NodeIndex<Ix>) -> bool {
        self.g.nodes.get(a.index()).map_or(false, |no| no.weight.is_some())
    }

    /// Add an edge from `a` to `b` to the graph, with its associated
    /// data `weight`.
    ///
    /// Return the index of the new edge.
    ///
    /// Computes in **O(1)** time.
    ///
    /// **Panics** if any of the nodes don't exist.<br>
    /// **Panics** if the Graph is at the maximum number of edges for its index
    /// type.
    ///
    /// **Note:** `StableGraph` allows adding parallel (“duplicate”) edges.
    pub fn add_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>, weight: E)
        -> EdgeIndex<Ix>
    {

        if self.free_edge != EdgeIndex::end() {
            let edge_idx = self.free_edge;
            let edge = &mut self.g.edges[edge_idx.index()];
            let _old = replace(&mut edge.weight, Some(weight));
            debug_assert!(_old.is_none());
            self.free_edge = edge.next[0];
            edge.node = [a, b];
            match index_twice(&mut self.g.nodes, a.index(), b.index()) {
                Pair::None => panic!("StableGraph::add_edge: node indices out of bounds"),
                Pair::One(an) => {
                    edge.next = an.next;
                    an.next[0] = edge_idx;
                    an.next[1] = edge_idx;
                }
                Pair::Both(an, bn) => {
                    // a and b are different indices
                    edge.next = [an.next[0], bn.next[1]];
                    an.next[0] = edge_idx;
                    bn.next[1] = edge_idx;
                }
            }
            self.edge_count += 1;
            edge_idx
        } else {
            self.edge_count += 1;
            self.g.add_edge(a, b, Some(weight))
        }
    }

    /// Add or update an edge from `a` to `b`.
    /// If the edge already exists, its weight is updated.
    ///
    /// Return the index of the affected edge.
    ///
    /// Computes in **O(e')** time, where **e'** is the number of edges
    /// connected to `a` (and `b`, if the graph edges are undirected).
    ///
    /// **Panics** if any of the nodes don't exist.
    pub fn update_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>, weight: E)
        -> EdgeIndex<Ix>
    {
        if let Some(ix) = self.find_edge(a, b) {
            self[ix] = weight;
            return ix;
        }
        self.add_edge(a, b, weight)
    }

    /// Remove an edge and return its edge weight, or `None` if it didn't exist.
    ///
    /// Invalidates the edge index `e` but no other.
    ///
    /// Computes in **O(e')** time, where **e'** is the number of edges
    /// conneced to the same endpoints as `e`.
    pub fn remove_edge(&mut self, e: EdgeIndex<Ix>) -> Option<E> {
        // every edge is part of two lists,
        // outgoing and incoming edges.
        // Remove it from both
        let (is_edge, edge_node, edge_next) = match self.g.edges.get(e.index()) {
            None => return None,
            Some(x) => (x.weight.is_some(), x.node, x.next),
        };
        if !is_edge {
            return None;
        }

        // Remove the edge from its in and out lists by replacing it with
        // a link to the next in the list.
        self.g.change_edge_links(edge_node, e, edge_next);

        // Clear the edge and put it in the free list
        let edge = &mut self.g.edges[e.index()];
        edge.next = [self.free_edge, EdgeIndex::end()];
        edge.node = [NodeIndex::end(), NodeIndex::end()];
        self.free_edge = e;
        self.edge_count -= 1;
        edge.weight.take()
    }

    /// Access the weight for node `a`.
    ///
    /// Also available with indexing syntax: `&graph[a]`.
    pub fn node_weight(&self, a: NodeIndex<Ix>) -> Option<&N> {
        match self.g.nodes.get(a.index()) {
            Some(no) => no.weight.as_ref(),
            None => None,
        }
    }

    /// Access the weight for node `a`, mutably.
    ///
    /// Also available with indexing syntax: `&mut graph[a]`.
    pub fn node_weight_mut(&mut self, a: NodeIndex<Ix>) -> Option<&mut N> {
        match self.g.nodes.get_mut(a.index()) {
            Some(no) => no.weight.as_mut(),
            None => None,
        }
    }

    /// Return an iterator over the node indices of the graph
    pub fn node_indices(&self) -> NodeIndices<N, Ix> {
        NodeIndices {
            iter: self.g.nodes.iter().enumerate(),
        }
    }

    /// Access the weight for edge `e`.
    ///
    /// Also available with indexing syntax: `&graph[e]`.
    pub fn edge_weight(&self, e: EdgeIndex<Ix>) -> Option<&E> {
        match self.g.edges.get(e.index()) {
            Some(ed) => ed.weight.as_ref(),
            None => None,
        }
    }

    /// Access the weight for edge `e`, mutably
    ///
    /// Also available with indexing syntax: `&mut graph[e]`.
    pub fn edge_weight_mut(&mut self, e: EdgeIndex<Ix>) -> Option<&mut E> {
        match self.g.edges.get_mut(e.index()) {
            Some(ed) => ed.weight.as_mut(),
            None => None,
        }
    }

    /// Access the source and target nodes for `e`.
    pub fn edge_endpoints(&self, e: EdgeIndex<Ix>)
        -> Option<(NodeIndex<Ix>, NodeIndex<Ix>)>
    {
        match self.g.edges.get(e.index()) {
            Some(ed) if ed.weight.is_some() => Some((ed.source(), ed.target())),
            _otherwise => None,
        }
    }

    /// Lookup an edge from `a` to `b`.
    ///
    /// Computes in **O(e')** time, where **e'** is the number of edges
    /// connected to `a` (and `b`, if the graph edges are undirected).
    pub fn find_edge(&self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> Option<EdgeIndex<Ix>>
    {
        let index = self.g.find_edge(a, b);
        if let Some(i) = index {
            debug_assert!(self.g.edges[i.index()].weight.is_some());
        }
        index
    }

    /// Return an iterator of all nodes with an edge starting from `a`.
    ///
    /// - `Directed`: Outgoing edges from `a`.
    /// - `Undirected`: All edges connected to `a`.
    ///
    /// Produces an empty iterator if the node doesn't exist.<br>
    /// Iterator element type is `NodeIndex<Ix>`.
    ///
    /// Use [`.neighbors(a).detach()`][1] to get a neighbor walker that does
    /// not borrow from the graph.
    ///
    /// [1]: struct.Neighbors.html#method.detach
    pub fn neighbors(&self, a: NodeIndex<Ix>) -> Neighbors<E, Ix> {
        self.neighbors_directed(a, Outgoing)
    }

    /// Return an iterator of all neighbors that have an edge between them and `a`,
    /// in the specified direction.
    /// If the graph's edges are undirected, this is equivalent to *.neighbors(a)*.
    ///
    /// - `Directed`, `Outgoing`: All edges from `a`.
    /// - `Directed`, `Incoming`: All edges to `a`.
    /// - `Undirected`: All edges connected to `a`.
    ///
    /// Produces an empty iterator if the node doesn't exist.<br>
    /// Iterator element type is `NodeIndex<Ix>`.
    ///
    /// Use [`.neighbors_directed(a, dir).detach()`][1] to get a neighbor walker that does
    /// not borrow from the graph.
    ///
    /// [1]: struct.Neighbors.html#method.detach
    pub fn neighbors_directed(&self, a: NodeIndex<Ix>, dir: Direction)
        -> Neighbors<E, Ix>
    {
        let mut iter = self.neighbors_undirected(a);
        if self.is_directed() {
            let k = dir.index();
            iter.next[1 - k] = EdgeIndex::end();
            iter.skip_start = NodeIndex::end();
        }
        iter
    }

    /// Return an iterator of all neighbors that have an edge between them and `a`,
    /// in either direction.
    /// If the graph's edges are undirected, this is equivalent to *.neighbors(a)*.
    ///
    /// - `Directed` and `Undirected`: All edges connected to `a`.
    ///
    /// Produces an empty iterator if the node doesn't exist.<br>
    /// Iterator element type is `NodeIndex<Ix>`.
    ///
    /// Use [`.neighbors_undirected(a).detach()`][1] to get a neighbor walker that does
    /// not borrow from the graph.
    ///
    /// [1]: struct.Neighbors.html#method.detach
    pub fn neighbors_undirected(&self, a: NodeIndex<Ix>) -> Neighbors<E, Ix>
    {
        Neighbors {
            skip_start: a,
            edges: &self.g.edges,
            next: match self.g.nodes.get(a.index()) {
                None => [EdgeIndex::end(), EdgeIndex::end()],
                Some(n) => n.next,
            }
        }
    }

    /// Return an iterator of all edges of `a`.
    ///
    /// - `Directed`: Outgoing edges from `a`.
    /// - `Undirected`: All edges connected to `a`.
    ///
    /// Produces an empty iterator if the node doesn't exist.<br>
    /// Iterator element type is `EdgeReference<E, Ix>`.
    pub fn edges(&self, a: NodeIndex<Ix>) -> Edges<E, Ty, Ix> {
        self.edges_directed(a, Outgoing)
    }

    /// Return an iterator of all edges of `a`, in the specified direction.
    ///
    /// - `Directed`, `Outgoing`: All edges from `a`.
    /// - `Directed`, `Incoming`: All edges to `a`.
    /// - `Undirected`: All edges connected to `a`.
    ///
    /// Produces an empty iterator if the node `a` doesn't exist.<br>
    /// Iterator element type is `EdgeReference<E, Ix>`.
    pub fn edges_directed(&self, a: NodeIndex<Ix>, dir: Direction) -> Edges<E, Ty, Ix>
    {
        let mut iter = self.edges_undirected(a);
        if self.is_directed() {
            iter.direction = Some(dir);
        }
        if self.is_directed() && dir == Incoming {
            iter.next.swap(0, 1);
        }
        iter
    }

    /// Return an iterator over all edges connected to `a`.
    ///
    /// - `Directed` and `Undirected`: All edges connected to `a`.
    ///
    /// Produces an empty iterator if the node `a` doesn't exist.<br>
    /// Iterator element type is `EdgeReference<E, Ix>`.
    fn edges_undirected(&self, a: NodeIndex<Ix>) -> Edges<E, Ty, Ix> {
        Edges {
            skip_start: a,
            edges: &self.g.edges,
            direction: None,
            next: match self.g.nodes.get(a.index()) {
                None => [EdgeIndex::end(), EdgeIndex::end()],
                Some(n) => n.next,
            },
            ty: PhantomData,
        }
    }

    /// Create a new `StableGraph` from an iterable of edges.
    ///
    /// Node weights `N` are set to default values.
    /// Edge weights `E` may either be specified in the list,
    /// or they are filled with default values.
    ///
    /// Nodes are inserted automatically to match the edges.
    ///
    /// ```
    /// use petgraph::stable_graph::StableGraph;
    ///
    /// let gr = StableGraph::<(), i32>::from_edges(&[
    ///     (0, 1), (0, 2), (0, 3),
    ///     (1, 2), (1, 3),
    ///     (2, 3),
    /// ]);
    /// ```
    pub fn from_edges<I>(iterable: I) -> Self
        where I: IntoIterator,
              I::Item: IntoWeightedEdge<E>,
              <I::Item as IntoWeightedEdge<E>>::NodeId: Into<NodeIndex<Ix>>,
              N: Default,
    {
        let mut g = Self::with_capacity(0, 0);
        g.extend_with_edges(iterable);
        g
    }

    /// Extend the graph from an iterable of edges.
    ///
    /// Node weights `N` are set to default values.
    /// Edge weights `E` may either be specified in the list,
    /// or they are filled with default values.
    ///
    /// Nodes are inserted automatically to match the edges.
    pub fn extend_with_edges<I>(&mut self, iterable: I)
        where I: IntoIterator,
              I::Item: IntoWeightedEdge<E>,
              <I::Item as IntoWeightedEdge<E>>::NodeId: Into<NodeIndex<Ix>>,
              N: Default,
    {
        let iter = iterable.into_iter();

        for elt in iter {
            let (source, target, weight) = elt.into_weighted_edge();
            let (source, target) = (source.into(), target.into());
            let nx = cmp::max(source, target);
            while nx.index() >= self.node_count() {
                self.add_node(N::default());
            }
            self.add_edge(source, target, weight);
        }
    }

}

/// The resulting cloned graph has the same graph indices as `self`.
impl<N, E, Ty, Ix: IndexType> Clone for StableGraph<N, E, Ty, Ix>
    where N: Clone, E: Clone,
{
    fn clone(&self) -> Self {
        StableGraph {
            g: self.g.clone(),
            node_count: self.node_count,
            edge_count: self.edge_count,
            free_node: self.free_node,
            free_edge: self.free_edge,
        }
    }

    fn clone_from(&mut self, rhs: &Self) {
        self.g.clone_from(&rhs.g);
        self.node_count = rhs.node_count;
        self.edge_count = rhs.edge_count;
        self.free_node = rhs.free_node;
        self.free_edge = rhs.free_edge;
    }
}

/// Index the `StableGraph` by `NodeIndex` to access node weights.
///
/// **Panics** if the node doesn't exist.
impl<N, E, Ty, Ix> Index<NodeIndex<Ix>> for StableGraph<N, E, Ty, Ix> where
    Ty: EdgeType,
    Ix: IndexType,
{
    type Output = N;
    fn index(&self, index: NodeIndex<Ix>) -> &N {
        self.node_weight(index).unwrap()
    }
}

/// Index the `StableGraph` by `NodeIndex` to access node weights.
///
/// **Panics** if the node doesn't exist.
impl<N, E, Ty, Ix> IndexMut<NodeIndex<Ix>> for StableGraph<N, E, Ty, Ix> where
    Ty: EdgeType,
    Ix: IndexType,
{
    fn index_mut(&mut self, index: NodeIndex<Ix>) -> &mut N {
        self.node_weight_mut(index).unwrap()
    }

}

/// Index the `StableGraph` by `EdgeIndex` to access edge weights.
///
/// **Panics** if the edge doesn't exist.
impl<N, E, Ty, Ix> Index<EdgeIndex<Ix>> for StableGraph<N, E, Ty, Ix> where
    Ty: EdgeType,
    Ix: IndexType,
{
    type Output = E;
    fn index(&self, index: EdgeIndex<Ix>) -> &E {
        self.edge_weight(index).unwrap()
    }
}

/// Index the `StableGraph` by `EdgeIndex` to access edge weights.
///
/// **Panics** if the edge doesn't exist.
impl<N, E, Ty, Ix> IndexMut<EdgeIndex<Ix>> for StableGraph<N, E, Ty, Ix> where
    Ty: EdgeType,
    Ix: IndexType,
{
    fn index_mut(&mut self, index: EdgeIndex<Ix>) -> &mut E {
        self.edge_weight_mut(index).unwrap()
    }
}

/// Create a new empty `StableGraph`.
impl<N, E, Ty, Ix> Default for StableGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    fn default() -> Self { Self::with_capacity(0, 0) }
}

/// Reference to a `StableGraph` edge.
#[derive(Debug)]
pub struct EdgeReference<'a, E: 'a, Ix = DefaultIx> {
    index: EdgeIndex<Ix>,
    node: [NodeIndex<Ix>; 2],
    weight: &'a E,
}

impl<'a, E, Ix: IndexType> Clone for EdgeReference<'a, E, Ix> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, E, Ix: IndexType> Copy for EdgeReference<'a, E, Ix> { }

impl<'a, E, Ix: IndexType> PartialEq for EdgeReference<'a, E, Ix>
    where E: PartialEq,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.index == rhs.index && self.weight == rhs.weight
    }
}

impl<'a, Ix, E> EdgeReference<'a, E, Ix>
    where Ix: IndexType,
{
    /// Access the edge’s weight.
    ///
    /// **NOTE** that this method offers a longer lifetime
    /// than the trait (unfortunately they don't match yet).
    pub fn weight(&self) -> &'a E { self.weight }
}

impl<'a, Ix, E> EdgeRef for EdgeReference<'a, E, Ix>
    where Ix: IndexType,
{
    type NodeId = NodeIndex<Ix>;
    type EdgeId = EdgeIndex<Ix>;
    type Weight = E;

    fn source(&self) -> Self::NodeId { self.node[0] }
    fn target(&self) -> Self::NodeId { self.node[1] }
    fn weight(&self) -> &E { self.weight }
    fn id(&self) -> Self::EdgeId { self.index }
}

impl<'a, N, E, Ty, Ix> IntoEdges for &'a StableGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type Edges = Edges<'a, E, Ty, Ix>;
    fn edges(self, a: Self::NodeId) -> Self::Edges {
        self.edges(a)
    }
}



/// Iterator over the edges of from or to a node
pub struct Edges<'a, E: 'a, Ty, Ix: 'a = DefaultIx>
    where Ty: EdgeType,
          Ix: IndexType,
{
    /// starting node to skip over
    skip_start: NodeIndex<Ix>,
    edges: &'a [Edge<Option<E>, Ix>],

    /// Next edge to visit.
    /// If we are only following one direction, we only use next[0] regardless.
    next: [EdgeIndex<Ix>; 2],

    /// Which direction to follow
    /// None: Both,
    /// Some(d): d if Directed, Both if Undirected
    direction: Option<Direction>,
    ty: PhantomData<Ty>,
}

impl<'a, E, Ty, Ix> Iterator for Edges<'a, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type Item = EdgeReference<'a, E, Ix>;

    fn next(&mut self) -> Option<Self::Item> {
        // First the outgoing or incoming edges (directionality)
        let k = self.direction.unwrap_or(Outgoing).index();
        let i = self.next[0].index();
        match self.edges.get(i) {
            None => {}
            Some(&Edge { ref node, weight: Some(ref weight), ref next }) => {
                self.next[0] = next[k];
                return Some(EdgeReference {
                    index: edge_index(i),
                    node: *node,
                    weight: weight,
                });
            }
            Some(_otherwise) => unreachable!(),
        }
        // Stop here if we only follow one direction
        if self.direction.is_some() {
            return None;
        }
        // Then incoming edges
        // For an "undirected" iterator (traverse both incoming
        // and outgoing edge lists), make sure we don't double
        // count selfloops by skipping them in the incoming list.

        // We reach here if self.direction was None or Outgoing.
        debug_assert_eq!(k, 0);
        while let Some(edge) = self.edges.get(self.next[1].index()) {
            debug_assert!(edge.weight.is_some());
            let i = self.next[1].index();
            self.next[1] = edge.next[1];
            if edge.node[0] != self.skip_start {
                return Some(EdgeReference {
                    index: edge_index(i),
                    node: swap_pair(edge.node),
                    weight: edge.weight.as_ref().unwrap(),
                });
            }
        }
        None
    }
}

fn swap_pair<T>(mut x: [T; 2]) -> [T; 2] {
    x.swap(0, 1);
    x
}

impl<'a, N: 'a, E: 'a, Ty, Ix> IntoEdgeReferences for &'a StableGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type EdgeRef = EdgeReference<'a, E, Ix>;
    type EdgeReferences = EdgeReferences<'a, E, Ix>;

    /// Create an iterator over all edges in the graph, in indexed order.
    ///
    /// Iterator element type is `EdgeReference<E, Ix>`.
    fn edge_references(self) -> Self::EdgeReferences {
        EdgeReferences {
            iter: self.g.edges.iter().enumerate()
        }
    }

}

/// Iterator over all edges of a graph.
pub struct EdgeReferences<'a, E: 'a, Ix: 'a = DefaultIx> {
    iter: iter::Enumerate<slice::Iter<'a, Edge<Option<E>, Ix>>>,
}

impl<'a, E, Ix> Iterator for EdgeReferences<'a, E, Ix>
    where Ix: IndexType
{
    type Item = EdgeReference<'a, E, Ix>;

    fn next(&mut self) -> Option<Self::Item> {
        (&mut self.iter).filter_map(|(i, edge)|
            edge.weight.as_ref().map(move |weight| {
                EdgeReference {
                    index: edge_index(i),
                    node: edge.node,
                    weight: weight,
                }
            }))
            .next()
    }
}


/// Iterator over the neighbors of a node.
///
/// Iterator element type is `NodeIndex`.
pub struct Neighbors<'a, E: 'a, Ix: 'a = DefaultIx>
{
    /// starting node to skip over
    skip_start: NodeIndex<Ix>,
    edges: &'a [Edge<Option<E>, Ix>],
    next: [EdgeIndex<Ix>; 2],
}

impl<'a, E, Ix> Neighbors<'a, E, Ix>
    where Ix: IndexType,
{
    /// Return a “walker” object that can be used to step through the
    /// neighbors and edges from the origin node.
    ///
    /// Note: The walker does not borrow from the graph, this is to allow mixing
    /// edge walking with mutating the graph's weights.
    pub fn detach(&self) -> WalkNeighbors<Ix> {
        WalkNeighbors {
            inner: super::WalkNeighbors {
                skip_start: self.skip_start,
                next: self.next
            },
        }
    }
}

impl<'a, E, Ix> Iterator for Neighbors<'a, E, Ix> where
    Ix: IndexType,
{
    type Item = NodeIndex<Ix>;

    fn next(&mut self) -> Option<NodeIndex<Ix>> {
        // First any outgoing edges
        match self.edges.get(self.next[0].index()) {
            None => {}
            Some(edge) => {
                debug_assert!(edge.weight.is_some());
                self.next[0] = edge.next[0];
                return Some((edge.node[1]));
            }
        }
        // Then incoming edges
        // For an "undirected" iterator (traverse both incoming
        // and outgoing edge lists), make sure we don't double
        // count selfloops by skipping them in the incoming list.
        while let Some(edge) = self.edges.get(self.next[1].index()) {
            debug_assert!(edge.weight.is_some());
            self.next[1] = edge.next[1];
            if edge.node[0] != self.skip_start {
                return Some((edge.node[0]));
            }
        }
        None
    }
}

/// A “walker” object that can be used to step through the edge list of a node.
///
/// See [*.detach()*](struct.Neighbors.html#method.detach) for more information.
///
/// The walker does not borrow from the graph, so it lets you step through
/// neighbors or incident edges while also mutating graph weights, as
/// in the following example:
///
/// ```
/// use petgraph::visit::Dfs;
/// use petgraph::Incoming;
/// use petgraph::stable_graph::StableGraph;
///
/// let mut gr = StableGraph::new();
/// let a = gr.add_node(0.);
/// let b = gr.add_node(0.);
/// let c = gr.add_node(0.);
/// gr.add_edge(a, b, 3.);
/// gr.add_edge(b, c, 2.);
/// gr.add_edge(c, b, 1.);
///
/// // step through the graph and sum incoming edges into the node weight
/// let mut dfs = Dfs::new(&gr, a);
/// while let Some(node) = dfs.next(&gr) {
///     // use a detached neighbors walker
///     let mut edges = gr.neighbors_directed(node, Incoming).detach();
///     while let Some(edge) = edges.next_edge(&gr) {
///         gr[node] += gr[edge];
///     }
/// }
///
/// // check the result
/// assert_eq!(gr[a], 0.);
/// assert_eq!(gr[b], 4.);
/// assert_eq!(gr[c], 2.);
/// ```
pub struct WalkNeighbors<Ix> {
    inner: super::WalkNeighbors<Ix>,
}

impl<Ix: IndexType> Clone for WalkNeighbors<Ix> {
    clone_fields!(WalkNeighbors, inner);
}

impl<Ix: IndexType> WalkNeighbors<Ix> {
    /// Step to the next edge and its endpoint node in the walk for graph `g`.
    ///
    /// The next node indices are always the others than the starting point
    /// where the `WalkNeighbors` value was created.
    /// For an `Outgoing` walk, the target nodes,
    /// for an `Incoming` walk, the source nodes of the edge.
    pub fn next<N, E, Ty: EdgeType>(&mut self, g: &StableGraph<N, E, Ty, Ix>)
        -> Option<(EdgeIndex<Ix>, NodeIndex<Ix>)> {
        self.inner.next(&g.g)
    }

    pub fn next_node<N, E, Ty: EdgeType>(&mut self, g: &StableGraph<N, E, Ty, Ix>)
        -> Option<NodeIndex<Ix>>
    {
        self.next(g).map(|t| t.1)
    }

    pub fn next_edge<N, E, Ty: EdgeType>(&mut self, g: &StableGraph<N, E, Ty, Ix>)
        -> Option<EdgeIndex<Ix>>
    {
        self.next(g).map(|t| t.0)
    }
}

/// Iterator over the node indices of a graph.
pub struct NodeIndices<'a, N: 'a, Ix: 'a = DefaultIx> {
    iter: iter::Enumerate<slice::Iter<'a, Node<Option<N>, Ix>>>,
}

impl<'a, N, Ix: IndexType> Iterator for NodeIndices<'a, N, Ix> {
    type Item = NodeIndex<Ix>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.by_ref().filter_map(|(i, node)| {
            if node.weight.is_some() {
                Some(node_index(i))
            } else { None }
        }).next()
    }
}

impl<'a, N, Ix: IndexType> DoubleEndedIterator for NodeIndices<'a, N, Ix> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.by_ref().filter_map(|(i, node)| {
            if node.weight.is_some() {
                Some(node_index(i))
            } else { None }
        }).next_back()
    }
}

impl<N, E, Ty, Ix> NodeIndexable for StableGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    /// Return an upper bound of the node indices in the graph
    fn node_bound(&self) -> usize {
        self.g.nodes.iter().rposition(|elt| elt.weight.is_some()).unwrap_or(0) + 1
    }
    fn to_index(&self, ix: NodeIndex<Ix>) -> usize { ix.index() }
    fn from_index(&self, ix: usize) -> Self::NodeId { NodeIndex::new(ix) }
}


#[test]
fn stable_graph() {
    let mut gr = StableGraph::<_, _>::with_capacity(0, 0);
    let a = gr.add_node(0);
    let b = gr.add_node(1);
    let c = gr.add_node(2);
    let _ed = gr.add_edge(a, b, 1);
    println!("{:?}", gr);
    gr.remove_node(b);
    println!("{:?}", gr);
    let d = gr.add_node(3);
    println!("{:?}", gr);
    gr.remove_node(a);
    gr.remove_node(c);
    println!("{:?}", gr);
    gr.add_edge(d, d, 2);
    println!("{:?}", gr);

    let e = gr.add_node(4);
    gr.add_edge(d, e, 3);
    println!("{:?}", gr);
    for neigh in gr.neighbors(d) {
        println!("edge {:?} -> {:?}", d, neigh);
    }
}

#[test]
fn dfs() {
    use visit::Dfs;

    let mut gr = StableGraph::<_, _>::with_capacity(0, 0);
    let a = gr.add_node("a");
    let b = gr.add_node("b");
    let c = gr.add_node("c");
    let d = gr.add_node("d");
    gr.add_edge(a, b, 1);
    gr.add_edge(a, c, 2);
    gr.add_edge(b, c, 3);
    gr.add_edge(b, d, 4);
    gr.add_edge(c, d, 5);
    gr.add_edge(d, b, 6);
    gr.add_edge(c, b, 7);
    println!("{:?}", gr);

    let mut dfs = Dfs::new(&gr, a);
    while let Some(next) = dfs.next(&gr) {
        println!("dfs visit => {:?}, weight={:?}", next, &gr[next]);
    }
}

#[test]
fn test_retain_nodes() {
    let mut gr = StableGraph::<_, _>::with_capacity(6, 6);
    let a = gr.add_node("a");
    let b = gr.add_node("b");
    let c = gr.add_node("c");
    let d = gr.add_node("d");
    let e = gr.add_node("e");
    gr.add_edge(a, b, 1);
    gr.add_edge(a, c, 2);
    gr.add_edge(b, c, 3);
    gr.add_edge(b, d, 4);
    gr.add_edge(c, d, 5);
    gr.add_edge(d, b, 6);
    gr.add_edge(c, b, 7);
    gr.add_edge(d, e, 8);

    assert_eq!(gr.node_count(), 5);
    assert_eq!(gr.edge_count(), 8);
    gr.retain_nodes(|frozen_gr, ix| {frozen_gr[ix] >= "c"});
    assert_eq!(gr.node_count(), 3);
    assert_eq!(gr.edge_count(), 2);
}
