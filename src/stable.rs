//! ***Unstable.*** `StableGraph` keeps indices stable across removals.
//!
//! ***Unstable: API may change at any time.*** Depends on `feature = "stable_graph"`.
//!

use std::fmt;
use std::iter;
use std::mem::replace;
use std::ops::{Index, IndexMut};
use std::slice;

use {
    EdgeType,
    Directed,
    Outgoing,
    EdgeDirection,
};
use super::{
    DefIndex,
    Edge,
    EdgeIndex,
    Graph,
    index_twice,
    IndexType,
    Node,
    NodeIndex,
    node_index,
    DIRECTIONS,
    Pair,
};

/// `StableGraph<N, E, Ty, Ix>` is a graph datastructure using an adjacency
/// list representation.
///
/// ***Unstable: API may change at any time.*** Depends on `feature = "stable_graph"`.
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
///
pub struct StableGraph<N, E, Ty = Directed, Ix = DefIndex>
    where Ix: IndexType
{
    g: Graph<Option<N>, Option<E>, Ty, Ix>,
    node_count: usize,
    edge_count: usize,
    free_node: NodeIndex<Ix>,
    free_edge: EdgeIndex<Ix>,
}

impl<N, E, Ty, Ix> fmt::Debug for StableGraph<N, E, Ty, Ix> where
    N: fmt::Debug, E: fmt::Debug, Ty: EdgeType, Ix: IndexType
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(writeln!(f, "{:?}", self.g));
        try!(writeln!(f, "free_node={:?}", self.free_node));
        try!(writeln!(f, "free_edge={:?}", self.free_edge));
        try!(writeln!(f, "node_count={:?}", self.node_count));
        try!(writeln!(f, "edge_count={:?}", self.edge_count));
        Ok(())
    }
}

impl<N, E> StableGraph<N, E, Directed> {
    /// Create a new `StableGraph` with directed edges.
    pub fn new() -> Self {
        Self::with_capacity(0, 0)
    }
}

impl<N, E, Ty=Directed, Ix=DefIndex> StableGraph<N, E, Ty, Ix>
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
    /// of edges with an endpoint in `a`, and including the edges with an
    /// endpoint in the displaced node.
    pub fn remove_node(&mut self, a: NodeIndex<Ix>) -> Option<N> {
        let node_weight = match self.g.nodes.get_mut(a.index()) {
            None => return None,
            Some(n) => n.weight.take(),
        };
        if let None = node_weight {
            return None;
        }
        for d in DIRECTIONS.iter() {
            let k = *d as usize;

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
    /// - `Undirected`: All edges from or to `a`.
    /// - `Directed`: Outgoing edges from `a`.
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
    /// - `Undirected`: All edges from or to `a`.
    /// - `Directed`, `Outgoing`: All edges from `a`.
    /// - `Directed`, `Incoming`: All edges to `a`.
    ///
    /// Produces an empty iterator if the node doesn't exist.<br>
    /// Iterator element type is `NodeIndex<Ix>`.
    ///
    /// Use [`.neighbors_directed(a, dir).detach()`][1] to get a neighbor walker that does
    /// not borrow from the graph.
    ///
    /// [1]: struct.Neighbors.html#method.detach
    pub fn neighbors_directed(&self, a: NodeIndex<Ix>, dir: EdgeDirection)
        -> Neighbors<E, Ix>
    {
        let mut iter = self.neighbors_undirected(a);
        if self.is_directed() {
            let k = dir as usize;
            iter.next[1 - k] = EdgeIndex::end();
            iter.skip_start = NodeIndex::end();
        }
        iter
    }

    /// Return an iterator of all neighbors that have an edge between them and `a`,
    /// in either direction.
    /// If the graph's edges are undirected, this is equivalent to *.neighbors(a)*.
    ///
    /// - `Undirected` and `Directed`: All edges from or to `a`.
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

/// Iterator over the neighbors of a node.
///
/// Iterator element type is `NodeIndex`.
pub struct Neighbors<'a, E: 'a, Ix: 'a = DefIndex> where
    Ix: IndexType,
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
/// use petgraph::{Dfs, Incoming};
/// use petgraph::graph::stable::StableGraph;
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
pub struct NodeIndices<'a, N: 'a, Ix: IndexType = DefIndex> {
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
    use Dfs;

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
