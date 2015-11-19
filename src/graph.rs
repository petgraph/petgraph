//! `Graph<N, E, Ty, Ix>` is a graph datastructure using an adjacency list representation.

use std::fmt;
use std::slice;
use std::iter;
use std::marker;
use std::ops::{Index, IndexMut};

use super::{
    EdgeDirection, Outgoing, Incoming,
    Undirected,
    Directed,
    EdgeType,
};

/// The default integer type for node and edge indices in `Graph`.
/// `u32` is the default to reduce the size of the graph's data and improve
/// performance in the common case.
pub type DefIndex = u32;

/// Trait for the unsigned integer type used for node and edge indices.
pub trait IndexType : Copy + Ord + fmt::Debug + 'static
{
    fn new(x: usize) -> Self;
    fn index(&self) -> usize;
    fn max() -> Self;
    fn zero() -> Self;
    fn one() -> Self;
}

impl IndexType for usize {
    #[inline(always)]
    fn new(x: usize) -> Self { x }
    #[inline(always)]
    fn index(&self) -> Self { *self }
    #[inline(always)]
    fn max() -> Self { ::std::usize::MAX }
    #[inline(always)]
    fn zero() -> Self { 0 }
    #[inline(always)]
    fn one() -> Self { 1 }
}

impl IndexType for u32 {
    #[inline(always)]
    fn new(x: usize) -> Self { x as u32 }
    #[inline(always)]
    fn index(&self) -> usize { *self as usize }
    #[inline(always)]
    fn max() -> Self { ::std::u32::MAX }
    #[inline(always)]
    fn zero() -> Self { 0 }
    #[inline(always)]
    fn one() -> Self { 1 }
}

impl IndexType for u16 {
    #[inline(always)]
    fn new(x: usize) -> Self { x as u16 }
    #[inline(always)]
    fn index(&self) -> usize { *self as usize }
    #[inline(always)]
    fn max() -> Self { ::std::u16::MAX }
    #[inline(always)]
    fn zero() -> Self { 0 }
    #[inline(always)]
    fn one() -> Self { 1 }
}

impl IndexType for u8 {
    #[inline(always)]
    fn new(x: usize) -> Self { x as u8 }
    #[inline(always)]
    fn index(&self) -> usize { *self as usize }
    #[inline(always)]
    fn max() -> Self { ::std::u8::MAX }
    #[inline(always)]
    fn zero() -> Self { 0 }
    #[inline(always)]
    fn one() -> Self { 1 }
}

// FIXME: These aren't stable, so a public wrapper of node/edge indices
// should be lifetimed just like pointers.
/// Node identifier.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct NodeIndex<Ix=DefIndex>(Ix);

impl<Ix: IndexType = DefIndex> NodeIndex<Ix>
{
    #[inline]
    pub fn new(x: usize) -> Self {
        NodeIndex(IndexType::new(x))
    }

    #[inline]
    pub fn index(self) -> usize
    {
        self.0.index()
    }

    #[inline]
    pub fn end() -> Self
    {
        NodeIndex(IndexType::max())
    }
}

/// Edge identifier.
#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct EdgeIndex<Ix=DefIndex>(Ix);

impl<Ix: IndexType = DefIndex> EdgeIndex<Ix>
{
    #[inline]
    pub fn new(x: usize) -> Self {
        EdgeIndex(IndexType::new(x))
    }

    #[inline]
    pub fn index(self) -> usize
    {
        self.0.index()
    }

    /// An invalid `EdgeIndex` used to denote absence of an edge, for example
    /// to end an adjacency list.
    #[inline]
    pub fn end() -> Self {
        EdgeIndex(IndexType::max())
    }
}

impl<Ix: IndexType> fmt::Debug for EdgeIndex<Ix>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "EdgeIndex("));
        if *self == EdgeIndex::end() {
            try!(write!(f, "End"));
        } else {
            try!(write!(f, "{}", self.index()));
        }
        write!(f, ")")
    }
}

const DIRECTIONS: [EdgeDirection; 2] = [EdgeDirection::Outgoing, EdgeDirection::Incoming];

/// The graph's node type.
#[derive(Debug, Clone)]
pub struct Node<N, Ix: IndexType = DefIndex> {
    /// Associated node data.
    pub weight: N,
    /// Next edge in outgoing and incoming edge lists.
    next: [EdgeIndex<Ix>; 2],
}

impl<N, Ix: IndexType = DefIndex> Node<N, Ix>
{
    /// Accessor for data structure internals: the first edge in the given direction.
    pub fn next_edge(&self, dir: EdgeDirection) -> EdgeIndex<Ix>
    {
        self.next[dir as usize]
    }
}

/// The graph's edge type.
#[derive(Debug, Clone)]
pub struct Edge<E, Ix: IndexType = DefIndex> {
    /// Associated edge data.
    pub weight: E,
    /// Next edge in outgoing and incoming edge lists.
    next: [EdgeIndex<Ix>; 2],
    /// Start and End node index
    node: [NodeIndex<Ix>; 2],
}

impl<E, Ix: IndexType = DefIndex> Edge<E, Ix>
{
    /// Accessor for data structure internals: the next edge for the given direction.
    pub fn next_edge(&self, dir: EdgeDirection) -> EdgeIndex<Ix>
    {
        self.next[dir as usize]
    }

    /// Return the source node index.
    pub fn source(&self) -> NodeIndex<Ix>
    {
        self.node[0]
    }

    /// Return the target node index.
    pub fn target(&self) -> NodeIndex<Ix>
    {
        self.node[1]
    }
}

/// `Graph<N, E, Ty, Ix>` is a graph datastructure using an adjacency list representation.
///
/// `Graph` is parameterized over the node weight `N`, edge weight `E`,
/// edge type `Ty` that determines whether the graph has directed edges or not,
/// and `Ix` which is the index type used.
///
/// Based on the graph implementation in rustc.
///
/// ### Graph Indices
///
/// The graph maintains unique indices for nodes and edges, and node and edge
/// weights may be accessed mutably.
///
/// `NodeIndex` and `EdgeIndex` are types that act as references to nodes and edges,
/// but these are only stable across certain operations. **Removing nodes or edges may shift
/// other indices**. Adding to the graph keeps
/// all indices stable, but removing a node will force the last node to shift its index to
/// take its place. Similarly, removing an edge shifts the index of the last edge.
///
/// The `Ix` parameter is `u32` by default. The goal is that you can ignore this parameter
/// completely unless you need a very big graph -- then you can use `usize`.
///
/// ### Tradeoffs With Indices
///
/// * The fact that the node and edge indices in the graph are numbered in a compact interval from
/// 0 to *n* - 1 simplifies some graph algorithms.
///
/// * You can select graph index integer type after the size of the graph. A smaller
/// size has better performance due to cache effects.
///
/// * Using indices allows mutation while traversing the graph, see `Dfs`.
///
/// * You can create several graphs using the equal node indices but with
/// differing weights or differing edges.
///
/// * The `Graph` is a regular rust collection and is `Send + Sync` (as long
/// as `N` and `E` are).
///
pub struct Graph<N, E, Ty = Directed, Ix: IndexType = DefIndex> {
    nodes: Vec<Node<N, Ix>>,
    edges: Vec<Edge<E, Ix>>,
    _ty: marker::PhantomData<Ty>,
}

impl<N, E, Ty, Ix: IndexType> Clone for Graph<N, E, Ty, Ix>
    where N: Clone, E: Clone,
{
    fn clone(&self) -> Self {
        Graph {
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
            _ty: self._ty.clone(),
        }
    }
}

impl<N, E, Ty, Ix> fmt::Debug for Graph<N, E, Ty, Ix> where
    N: fmt::Debug, E: fmt::Debug, Ty: EdgeType, Ix: IndexType
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let indent = "    ";
        let etype = if self.is_directed() { "Directed" } else { "Undirected" };
        try!(write!(f, "Graph<{}> {{", etype));
        if self.node_count() == 0 {
            return write!(f, "}}");
        }
        try!(writeln!(f, ""));
        for (index, n) in self.nodes.iter().enumerate() {
            try!(writeln!(f, "{}{}: Node({:?}),", indent, index, n.weight));
        }
        for (index, e) in self.edges.iter().enumerate() {
            try!(writeln!(f, "{}{}: Edge(from={}, to={}, weight={:?}),",
                          indent, index,
                          e.source().index(),
                          e.target().index(),
                          e.weight));
        }
        try!(write!(f, "}}"));
        Ok(())
    }
}

enum Pair<T> {
    Both(T, T),
    One(T),
    None,
}

fn index_twice<T>(slc: &mut [T], a: usize, b: usize) -> Pair<&mut T>
{
    if a == b {
        slc.get_mut(a).map_or(Pair::None, Pair::One)
    } else {
        if a >= slc.len() || b >= slc.len() {
            Pair::None
        } else {
            // safe because a, b are in bounds and distinct
            unsafe {
                let ar = &mut *(slc.get_unchecked_mut(a) as *mut _);
                let br = &mut *(slc.get_unchecked_mut(b) as *mut _);
                Pair::Both(ar, br)
            }
        }
    }
}

impl<N, E> Graph<N, E, Directed>
{
    /// Create a new `Graph` with directed edges.
    pub fn new() -> Self
    {
        Graph{nodes: Vec::new(), edges: Vec::new(),
              _ty: marker::PhantomData}
    }
}

impl<N, E> Graph<N, E, Undirected>
{
    /// Create a new `Graph` with undirected edges.
    pub fn new_undirected() -> Self
    {
        Graph{nodes: Vec::new(), edges: Vec::new(),
              _ty: marker::PhantomData}
    }
}

impl<N, E, Ty=Directed, Ix=DefIndex> Graph<N, E, Ty, Ix> where
    Ty: EdgeType,
    Ix: IndexType,
{
    /// Create a new `Graph` with estimated capacity.
    pub fn with_capacity(nodes: usize, edges: usize) -> Self
    {
        Graph{nodes: Vec::with_capacity(nodes), edges: Vec::with_capacity(edges),
              _ty: marker::PhantomData}
    }

    /// Return the number of nodes (vertices) in the graph.
    pub fn node_count(&self) -> usize
    {
        self.nodes.len()
    }

    /// Return the number of edges in the graph.
    ///
    /// Computes in **O(1)** time.
    pub fn edge_count(&self) -> usize
    {
        self.edges.len()
    }

    /// Remove all nodes and edges
    pub fn clear(&mut self)
    {
        self.nodes.clear();
        self.edges.clear();
    }

    /// Return whether the graph has directed edges or not.
    #[inline]
    pub fn is_directed(&self) -> bool
    {
        Ty::is_directed()
    }

    /// Cast the graph as either undirected or directed. No edge adjustments
    /// are done.
    ///
    /// Computes in **O(1)** time.
    pub fn into_edge_type<NewTy>(self) -> Graph<N, E, NewTy, Ix> where
        NewTy: EdgeType
    {
        Graph{nodes: self.nodes, edges: self.edges,
              _ty: marker::PhantomData}
    }

    /// Add a node (also called vertex) with weight `w` to the graph.
    ///
    /// Computes in **O(1)** time.
    ///
    /// Return the index of the new node.
    ///
    /// **Panics** if the Graph is at the maximum number of nodes for its index
    /// type.
    pub fn add_node(&mut self, w: N) -> NodeIndex<Ix>
    {
        let node = Node{weight: w, next: [EdgeIndex::end(), EdgeIndex::end()]};
        let node_idx = NodeIndex::new(self.nodes.len());
        assert!(NodeIndex::end() != node_idx);
        self.nodes.push(node);
        node_idx
    }

    /// Access node weight for node `a`.
    pub fn node_weight(&self, a: NodeIndex<Ix>) -> Option<&N>
    {
        self.nodes.get(a.index()).map(|n| &n.weight)
    }

    /// Access node weight for node `a`.
    pub fn node_weight_mut(&mut self, a: NodeIndex<Ix>) -> Option<&mut N>
    {
        self.nodes.get_mut(a.index()).map(|n| &mut n.weight)
    }

    /// Return an iterator of all nodes with an edge starting from `a`.
    ///
    /// Produces an empty iterator if the node doesn't exist.
    ///
    /// Iterator element type is `NodeIndex<Ix>`.
    pub fn neighbors(&self, a: NodeIndex<Ix>) -> Neighbors<E, Ix>
    {
        if self.is_directed() {
            self.neighbors_directed(a, Outgoing)
        } else {
            self.neighbors_undirected(a)
        }
    }

    /// Return an iterator of all neighbors that have an edge between them and `a`,
    /// in the specified direction.
    /// If the graph is undirected, this is equivalent to *.neighbors(a)*.
    ///
    /// Produces an empty iterator if the node doesn't exist.
    ///
    /// Iterator element type is `NodeIndex<Ix>`.
    pub fn neighbors_directed(&self, a: NodeIndex<Ix>, dir: EdgeDirection) -> Neighbors<E, Ix>
    {
        let mut iter = self.neighbors_undirected(a);
        if self.is_directed() {
            // remove the other edges not wanted.
            let k = dir as usize;
            iter.next[1 - k] = EdgeIndex::end();
        }
        iter
    }

    /// Return an iterator of all neighbors that have an edge between them and `a`,
    /// in either direction.
    /// If the graph is undirected, this is equivalent to *.neighbors(a)*.
    ///
    /// Produces an empty iterator if the node doesn't exist.
    ///
    /// Iterator element type is `NodeIndex<Ix>`.
    pub fn neighbors_undirected(&self, a: NodeIndex<Ix>) -> Neighbors<E, Ix>
    {
        Neighbors {
            edges: &self.edges,
            next: match self.nodes.get(a.index()) {
                None => [EdgeIndex::end(), EdgeIndex::end()],
                Some(n) => n.next,
            }
        }
    }

    /// Return an iterator over the neighbors of node `a`, paired with their respective edge
    /// weights.
    ///
    /// Produces an empty iterator if the node doesn't exist.
    ///
    /// Iterator element type is `(NodeIndex<Ix>, &'a E)`.
    pub fn edges(&self, a: NodeIndex<Ix>) -> Edges<E, Ix>
    {
        self.edges_directed(a, EdgeDirection::Outgoing)
    }

    /// Return an iterator of all neighbors that have an edge between them and `a`,
    /// in the specified direction, paired with the respective edge weights.
    ///
    /// If the graph is undirected, this is equivalent to *.edges(a)*.
    ///
    /// Produces an empty iterator if the node doesn't exist.
    ///
    /// Iterator element type is `(NodeIndex<Ix>, &'a E)`.
    pub fn edges_directed(&self, a: NodeIndex<Ix>, dir: EdgeDirection) -> Edges<E, Ix>
    {
        let mut iter = self.edges_both(a);
        if self.is_directed() {
            iter.next[1 - dir as usize] = EdgeIndex::end();
        }
        iter
    }

    /// Return an iterator over the edgs from `a` to its neighbors, then *to* `a` from its
    /// neighbors.
    ///
    /// Produces an empty iterator if the node doesn't exist.
    ///
    /// Iterator element type is `(NodeIndex<Ix>, &'a E)`.
    pub fn edges_both(&self, a: NodeIndex<Ix>) -> Edges<E, Ix>
    {
        Edges {
            edges: &self.edges,
            next: match self.nodes.get(a.index()) {
                None => [EdgeIndex::end(), EdgeIndex::end()],
                Some(n) => n.next,
            }
        }
    }

    /// Add an edge from `a` to `b` to the graph, with its edge weight.
    ///
    /// **Note:** `Graph` allows adding parallel (“duplicate”) edges. If you want
    /// to avoid this, use [*.update_edge(a, b, weight)*](#method.update_edge) instead.
    ///
    /// Computes in **O(1)** time.
    ///
    /// Return the index of the new edge.
    ///
    /// **Panics** if any of the nodes don't exist.
    ///
    /// **Panics** if the Graph is at the maximum number of edges for its index
    /// type.
    pub fn add_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>, weight: E) -> EdgeIndex<Ix>
    {
        let edge_idx = EdgeIndex::new(self.edges.len());
        assert!(edge_idx != EdgeIndex::end());
        let mut edge = Edge {
            weight: weight,
            node: [a, b],
            next: [EdgeIndex::end(); 2],
        };
        match index_twice(&mut self.nodes, a.index(), b.index()) {
            Pair::None => panic!("Graph::add_edge: node indices out of bounds"),
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
        self.edges.push(edge);
        edge_idx
    }

    /// Add or update an edge from `a` to `b`.
    ///
    /// If the edge already exists, its weight is updated.
    ///
    /// Computes in **O(e')** time, where **e'** is the number of edges
    /// connected to the vertices `a` (and `b`).
    ///
    /// Return the index of the affected edge.
    ///
    /// **Panics** if any of the nodes don't exist.
    pub fn update_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>, weight: E) -> EdgeIndex<Ix>
    {
        if let Some(ix) = self.find_edge(a, b) {
            match self.edge_weight_mut(ix) {
                Some(ed) => {
                    *ed = weight;
                    return ix;
                }
                None => {}
            }
        }
        self.add_edge(a, b, weight)
    }

    /// Access the edge weight for `e`.
    pub fn edge_weight(&self, e: EdgeIndex<Ix>) -> Option<&E>
    {
        self.edges.get(e.index()).map(|ed| &ed.weight)
    }

    /// Access the edge weight for `e` mutably.
    pub fn edge_weight_mut(&mut self, e: EdgeIndex<Ix>) -> Option<&mut E>
    {
        self.edges.get_mut(e.index()).map(|ed| &mut ed.weight)
    }

    /// Remove `a` from the graph if it exists, and return its weight.
    /// If it doesn't exist in the graph, return `None`.
    pub fn remove_node(&mut self, a: NodeIndex<Ix>) -> Option<N>
    {
        match self.nodes.get(a.index()) {
            None => return None,
            _ => {}
        }
        for d in DIRECTIONS.iter() {
            let k = *d as usize;

            // Remove all edges from and to this node.
            loop {
                let next = self.nodes[a.index()].next[k];
                if next == EdgeIndex::end() {
                    break
                }
                let ret = self.remove_edge(next);
                debug_assert!(ret.is_some());
                let _ = ret;
            }
        }

        // Use swap_remove -- only the swapped-in node is going to change
        // NodeIndex<Ix>, so we only have to walk its edges and update them.

        let node = self.nodes.swap_remove(a.index());

        // Find the edge lists of the node that had to relocate.
        // It may be that no node had to relocate, then we are done already.
        let swap_edges = match self.nodes.get(a.index()) {
            None => return Some(node.weight),
            Some(ed) => ed.next,
        };

        // The swapped element's old index
        let old_index = NodeIndex::new(self.nodes.len());
        let new_index = a;

        // Adjust the starts of the out edges, and ends of the in edges.
        for &d in DIRECTIONS.iter() {
            let k = d as usize;
            for (_, curedge) in EdgesMut::new(&mut self.edges, swap_edges[k], d) {
                debug_assert!(curedge.node[k] == old_index);
                curedge.node[k] = new_index;
            }
        }
        Some(node.weight)
    }

    /// For edge `e` with endpoints `edge_node`, replace links to it,
    /// with links to `edge_next`.
    fn change_edge_links(&mut self, edge_node: [NodeIndex<Ix>; 2], e: EdgeIndex<Ix>,
                         edge_next: [EdgeIndex<Ix>; 2])
    {
        for &d in DIRECTIONS.iter() {
            let k = d as usize;
            let node = match self.nodes.get_mut(edge_node[k].index()) {
                Some(r) => r,
                None => {
                    debug_assert!(false, "Edge's endpoint dir={:?} index={:?} not found",
                                  d, edge_node[k]);
                    return
                }
            };
            let fst = node.next[k];
            if fst == e {
                //println!("Updating first edge 0 for node {}, set to {}", edge_node[0], edge_next[0]);
                node.next[k] = edge_next[k];
            } else {
                for (_i, curedge) in EdgesMut::new(&mut self.edges, fst, d) {
                    if curedge.next[k] == e {
                        curedge.next[k] = edge_next[k];
                        break; // the edge can only be present once in the list.
                    }
                }
            }
        }
    }

    /// Remove an edge and return its edge weight, or `None` if it didn't exist.
    ///
    /// Computes in **O(e')** time, where **e'** is the size of four particular edge lists, for
    /// the vertices of `e` and the vertices of another affected edge.
    pub fn remove_edge(&mut self, e: EdgeIndex<Ix>) -> Option<E>
    {
        // every edge is part of two lists,
        // outgoing and incoming edges.
        // Remove it from both
        let (edge_node, edge_next) = match self.edges.get(e.index()) {
            None => return None,
            Some(x) => (x.node, x.next),
        };
        // Remove the edge from its in and out lists by replacing it with
        // a link to the next in the list.
        self.change_edge_links(edge_node, e, edge_next);
        self.remove_edge_adjust_indices(e)
    }

    fn remove_edge_adjust_indices(&mut self, e: EdgeIndex<Ix>) -> Option<E>
    {
        // swap_remove the edge -- only the removed edge
        // and the edge swapped into place are affected and need updating
        // indices.
        let edge = self.edges.swap_remove(e.index());
        let swap = match self.edges.get(e.index()) {
            // no elment needed to be swapped.
            None => return Some(edge.weight),
            Some(ed) => ed.node,
        };
        let swapped_e = EdgeIndex::new(self.edges.len());

        // Update the edge lists by replacing links to the old index by references to the new
        // edge index.
        self.change_edge_links(swap, swapped_e, [e, e]);
        Some(edge.weight)
    }

    /// Lookup an edge from `a` to `b`.
    ///
    /// Computes in **O(e')** time, where **e'** is the number of edges
    /// connected to the vertices `a` (and `b`).
    pub fn find_edge(&self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> Option<EdgeIndex<Ix>>
    {
        if !self.is_directed() {
            self.find_edge_undirected(a, b).map(|(ix, _)| ix)
        } else {
            match self.nodes.get(a.index()) {
                None => None,
                Some(node) => {
                    let mut edix = node.next[0];
                    while let Some(edge) = self.edges.get(edix.index()) {
                        if edge.node[1] == b {
                            return Some(edix)
                        }
                        edix = edge.next[0];
                    }
                    None
                }
            }
        }
    }

    /// Lookup an edge between `a` and `b`, in either direction.
    ///
    /// If the graph is undirected, then this is equivalent to *.find_edge()*.
    ///
    /// Return the edge index and its directionality, with *Outgoing* meaning
    /// from `a` to `b` and *Incoming* the reverse,
    /// or `None` if the edge does not exist.
    pub fn find_edge_undirected(&self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> Option<(EdgeIndex<Ix>, EdgeDirection)>
    {
        match self.nodes.get(a.index()) {
            None => None,
            Some(node) => {
                for &d in DIRECTIONS.iter() {
                    let k = d as usize;
                    let mut edix = node.next[k];
                    while let Some(edge) = self.edges.get(edix.index()) {
                        if edge.node[1 - k] == b {
                            return Some((edix, d))
                        }
                        edix = edge.next[k];
                    }
                }
                None
            }
        }
    }

    /// Reverse the direction of all edges
    pub fn reverse(&mut self)
    {
        for edge in self.edges.iter_mut() {
            edge.node.swap(0, 1)
        }
    }

    /* Removed: Easy to implement externally with iterate in reverse
     *
    /// Retain only nodes that return **true** from the predicate.
    pub fn retain_nodes<F>(&mut self, mut visit: F) where
        F: FnMut(&Self, NodeIndex<Ix>, &Node<N>) -> bool
    {
        for index in (0..self.node_count()).rev() {
            let nix = NodeIndex<Ix>(index);
            if !visit(&self, nix, &self.nodes[nix.index()]) {
                let ret = self.remove_node(nix);
                debug_assert!(ret.is_some());
                let _ = ret;
            }
        }
    }

    /// Retain only edges that return **true** from the predicate.
    pub fn retain_edges<F>(&mut self, mut visit: F) where
        F: FnMut(&Self, EdgeIndex, &Edge<E>) -> bool
    {
        for index in (0..self.edge_count()).rev() {
            let eix = EdgeIndex::new(index);
            if !visit(&self, eix, &self.edges[eix.index()]) {
                let ret = self.remove_edge(EdgeIndex::new(index));
                debug_assert!(ret.is_some());
                let _ = ret;
            }
        }
    }
    */

    /// Return an iterator over either the nodes without edges to them or from them.
    ///
    /// The nodes in *.without_edges(Incoming)* are the source nodes and
    /// *.without_edges(Outgoing)* are the sinks.
    ///
    /// For an undirected graph, the sinks/sources are just the vertices without edges.
    ///
    /// The whole iteration computes in **O(|V|)** time.
    pub fn without_edges(&self, dir: EdgeDirection) -> WithoutEdges<N, Ty, Ix>
    {
        WithoutEdges{iter: self.nodes.iter().enumerate(), dir: dir,
                     _ty: marker::PhantomData}
    }

    /// Return an iterator yielding mutable access to all node weights.
    ///
    /// The order in which weights are yielded matches the order of their
    /// node indices.
    pub fn node_weights_mut<'a>(&'a mut self) -> NodeWeightsMut<'a, N, Ix>
    {
        NodeWeightsMut { nodes: self.nodes.iter_mut() }
    }

    /// Return an iterator yielding mutable access to all edge weights.
    ///
    /// The order in which weights are yielded matches the order of their
    /// edge indices.
    pub fn edge_weights_mut<'a>(&'a mut self) -> EdgeWeightsMut<'a, E, Ix>
    {
        EdgeWeightsMut { edges: self.edges.iter_mut() }
    }

    // Remaining methods are of the more internal flavour, read-only access to
    // the data structure's internals.

    /// Access the internal node array.
    pub fn raw_nodes(&self) -> &[Node<N, Ix>]
    {
        &self.nodes
    }

    /// Access the internal edge array.
    pub fn raw_edges(&self) -> &[Edge<E, Ix>]
    {
        &self.edges
    }

    /// Accessor for data structure internals: the first edge in the given direction.
    pub fn first_edge(&self, a: NodeIndex<Ix>, dir: EdgeDirection) -> Option<EdgeIndex<Ix>>
    {
        match self.nodes.get(a.index()) {
            None => None,
            Some(node) => {
                let edix = node.next[dir as usize];
                if edix == EdgeIndex::end() {
                    None
                } else { Some(edix) }
            }
        }
    }

    /// Accessor for data structure internals: the next edge for the given direction.
    pub fn next_edge(&self, e: EdgeIndex<Ix>, dir: EdgeDirection) -> Option<EdgeIndex<Ix>>
    {
        match self.edges.get(e.index()) {
            None => None,
            Some(node) => {
                let edix = node.next[dir as usize];
                if edix == EdgeIndex::end() {
                    None
                } else { Some(edix) }
            }
        }
    }

    /// Return a “walker” object that can be used to step through the edges
    /// of the node `a` in direction `dir`.
    ///
    /// Note: The walker does not borrow from the graph, this is to allow mixing
    /// edge walking with mutating the graph's weights.
    pub fn walk_edges_directed(&self, a: NodeIndex<Ix>, dir: EdgeDirection) -> WalkEdges<Ix>
    {
        let first_edge = match self.nodes.get(a.index()) {
            None => EdgeIndex::end(),
            Some(node) => node.next[dir as usize],
        };
        WalkEdges { next: first_edge, direction: dir }
    }

    /// Index the `Graph` by two indices, any combination of
    /// node or edge indices is fine.
    ///
    /// **Panics** if the indices are equal or if they are out of bounds.
    ///
    /// ```
    /// use petgraph::{Graph, Dfs, Incoming};
    ///
    /// let mut gr = Graph::new();
    /// let a = gr.add_node(0.);
    /// let b = gr.add_node(0.);
    /// let c = gr.add_node(0.);
    /// gr.add_edge(a, b, 3.);
    /// gr.add_edge(b, c, 2.);
    /// gr.add_edge(c, b, 1.);
    ///
    /// // walk the graph and sum incoming edges into the node weight
    /// let mut dfs = Dfs::new(&gr, a);
    /// while let Some(node) = dfs.next(&gr) {
    ///     let mut edges = gr.walk_edges_directed(node, Incoming);
    ///     while let Some(edge) = edges.next(&gr) {
    ///         let (nw, ew) = gr.index_twice_mut(node, edge);
    ///         *nw += *ew;
    ///     }
    /// }
    ///
    /// // check the result
    /// assert_eq!(gr[a], 0.);
    /// assert_eq!(gr[b], 4.);
    /// assert_eq!(gr[c], 2.);
    /// ```
    pub fn index_twice_mut<T, U>(&mut self, i: T, j: U)
        -> (&mut <Self as Index<T>>::Output,
            &mut <Self as Index<U>>::Output)
        where Self: IndexMut<T> + IndexMut<U>,
              T: GraphIndex,
              U: GraphIndex,
    {
        assert!(T::is_node_index() != U::is_node_index() ||
                i.index() != j.index());

        // Allow two mutable indexes here -- they are nonoverlapping
        unsafe {
            let self_mut = self as *mut _;
            (<Self as IndexMut<T>>::index_mut(&mut *self_mut, i),
             <Self as IndexMut<U>>::index_mut(&mut *self_mut, j))
        }
    }
}

/// An iterator over either the nodes without edges to them or from them.
pub struct WithoutEdges<'a, N: 'a, Ty, Ix: IndexType = DefIndex> {
    iter: iter::Enumerate<slice::Iter<'a, Node<N, Ix>>>,
    dir: EdgeDirection,
    _ty: marker::PhantomData<Ty>,
}

impl<'a, N: 'a, Ty, Ix> Iterator for WithoutEdges<'a, N, Ty, Ix> where
    Ty: EdgeType,
    Ix: IndexType,
{
    type Item = NodeIndex<Ix>;
    fn next(&mut self) -> Option<NodeIndex<Ix>>
    {
        let k = self.dir as usize;
        loop {
            match self.iter.next() {
                None => return None,
                Some((index, node)) => {
                    if node.next[k] == EdgeIndex::end() &&
                        (Ty::is_directed() ||
                         node.next[1-k] == EdgeIndex::end()) {
                        return Some(NodeIndex::new(index))
                    } else {
                        continue
                    }
                },
            }
        }
    }
}

/// Iterator over the neighbors of a node.
///
/// Iterator element type is `NodeIndex`.
pub struct Neighbors<'a, E: 'a, Ix: 'a = DefIndex> where
    Ix: IndexType,
{
    edges: &'a [Edge<E, Ix>],
    next: [EdgeIndex<Ix>; 2],
}

impl<'a, E, Ix> Iterator for Neighbors<'a, E, Ix> where
    Ix: IndexType,
{
    type Item = NodeIndex<Ix>;
    fn next(&mut self) -> Option<NodeIndex<Ix>>
    {
        match self.edges.get(self.next[0].index()) {
            None => {}
            Some(edge) => {
                self.next[0] = edge.next[0];
                return Some(edge.node[1])
            }
        }
        match self.edges.get(self.next[1].index()) {
            None => None,
            Some(edge) => {
                self.next[1] = edge.next[1];
                Some(edge.node[0])
            }
        }
    }
}

struct EdgesMut<'a, E: 'a, Ix: IndexType = DefIndex> {
    edges: &'a mut [Edge<E, Ix>],
    next: EdgeIndex<Ix>,
    dir: EdgeDirection,
}

impl<'a, E, Ix> EdgesMut<'a, E, Ix> where
    Ix: IndexType,
{
    fn new(edges: &'a mut [Edge<E, Ix>], next: EdgeIndex<Ix>, dir: EdgeDirection) -> Self
    {
        EdgesMut{
            edges: edges,
            next: next,
            dir: dir
        }
    }
}

impl<'a, E, Ix> Iterator for EdgesMut<'a, E, Ix> where
    Ix: IndexType,
{
    type Item = (EdgeIndex<Ix>, &'a mut Edge<E, Ix>);
    fn next(&mut self) -> Option<(EdgeIndex<Ix>, &'a mut Edge<E, Ix>)>
    {
        let this_index = self.next;
        let k = self.dir as usize;
        match self.edges.get_mut(self.next.index()) {
            None => None,
            Some(edge) => {
                self.next = edge.next[k];
                // We cannot in safe rust, derive a &'a mut from &mut self,
                // when the life of &mut self is shorter than 'a.
                //
                // We guarantee that this will not allow two pointers to the same
                // edge, and use unsafe to extend the life.
                //
                // See http://stackoverflow.com/a/25748645/3616050
                let long_life_edge = unsafe {
                    &mut *(edge as *mut _)
                };
                Some((this_index, long_life_edge))
            }
        }
    }
}

/// Iterator over the edges of a node.
pub struct Edges<'a, E: 'a, Ix: IndexType = DefIndex> {
    edges: &'a [Edge<E, Ix>],
    next: [EdgeIndex<Ix>; 2],
}

impl<'a, E, Ix> Iterator for Edges<'a, E, Ix> where
    Ix: IndexType,
{
    type Item = (NodeIndex<Ix>, &'a E);
    fn next(&mut self) -> Option<(NodeIndex<Ix>, &'a E)>
    {
        // First any outgoing edges
        match self.edges.get(self.next[0].index()) {
            None => {}
            Some(edge) => {
                self.next[0] = edge.next[0];
                return Some((edge.node[1], &edge.weight))
            }
        }
        // Then incoming edges
        match self.edges.get(self.next[1].index()) {
            None => None,
            Some(edge) => {
                self.next[1] = edge.next[1];
                Some((edge.node[0], &edge.weight))
            }
        }
    }
}

/// Iterator yielding mutable access to all node weights.
pub struct NodeWeightsMut<'a, N: 'a, Ix: IndexType = DefIndex> {
    nodes: ::std::slice::IterMut<'a, Node<N, Ix>>,
}

impl<'a, N, Ix> Iterator for NodeWeightsMut<'a, N, Ix> where
    Ix: IndexType,
{
    type Item = &'a mut N;

    fn next(&mut self) -> Option<&'a mut N> {
        self.nodes.next().map(|node| &mut node.weight)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.nodes.size_hint()
    }
}

/// Iterator yielding mutable access to all edge weights.
pub struct EdgeWeightsMut<'a, E: 'a, Ix: IndexType = DefIndex> {
    edges: ::std::slice::IterMut<'a, Edge<E, Ix>>,
}

impl<'a, E, Ix> Iterator for EdgeWeightsMut<'a, E, Ix> where
    Ix: IndexType,
{
    type Item = &'a mut E;

    fn next(&mut self) -> Option<&'a mut E> {
        self.edges.next().map(|edge| &mut edge.weight)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.edges.size_hint()
    }
}

/// Index the `Graph` by `NodeIndex` to access node weights.
///
/// **Panics** if the node doesn't exist.
impl<N, E, Ty, Ix> Index<NodeIndex<Ix>> for Graph<N, E, Ty, Ix> where
    Ty: EdgeType,
    Ix: IndexType,
{
    type Output = N;
    fn index(&self, index: NodeIndex<Ix>) -> &N {
        &self.nodes[index.index()].weight
    }
}

/// Index the `Graph` by `NodeIndex` to access node weights.
///
/// **Panics** if the node doesn't exist.
impl<N, E, Ty, Ix> IndexMut<NodeIndex<Ix>> for Graph<N, E, Ty, Ix> where
    Ty: EdgeType,
    Ix: IndexType,
{
    fn index_mut(&mut self, index: NodeIndex<Ix>) -> &mut N {
        &mut self.nodes[index.index()].weight
    }

}

/// Index the `Graph` by `EdgeIndex` to access edge weights.
///
/// **Panics** if the edge doesn't exist.
impl<N, E, Ty, Ix> Index<EdgeIndex<Ix>> for Graph<N, E, Ty, Ix> where
    Ty: EdgeType,
    Ix: IndexType,
{
    type Output = E;
    fn index(&self, index: EdgeIndex<Ix>) -> &E {
        &self.edges[index.index()].weight
    }
}

/// Index the `Graph` by `EdgeIndex` to access edge weights.
///
/// **Panics** if the edge doesn't exist.
impl<N, E, Ty, Ix> IndexMut<EdgeIndex<Ix>> for Graph<N, E, Ty, Ix> where
    Ty: EdgeType,
    Ix: IndexType,
{
    fn index_mut(&mut self, index: EdgeIndex<Ix>) -> &mut E {
        &mut self.edges[index.index()].weight
    }
}

/// A  `GraphIndex` is a node or edge index.
pub trait GraphIndex : Copy {
    #[doc(hidden)]
    fn index(&self) -> usize;
    #[doc(hidden)]
    fn is_node_index() -> bool;
}

impl<Ix: IndexType> GraphIndex for NodeIndex<Ix> {
    #[inline]
    fn index(&self) -> usize { NodeIndex::index(*self) }
    #[inline]
    fn is_node_index() -> bool { true }
}

impl<Ix: IndexType> GraphIndex for EdgeIndex<Ix> {
    #[inline]
    fn index(&self) -> usize { EdgeIndex::index(*self) }
    #[inline]
    fn is_node_index() -> bool { false }
}

/// A “walker” object that can be used to step through the edge list of a node.
///
/// See [*.walk_edges_directed()*](struct.Graph.html#method.walk_edges_directed)
/// for more information.
#[derive(Clone, Debug)]
pub struct WalkEdges<Ix: IndexType> {
    next: EdgeIndex<Ix>, // a valid index or EdgeIndex::max()
    direction: EdgeDirection,
}

impl<Ix: IndexType> WalkEdges<Ix> {
    /// Fetch the next edge index in the walk for graph `g`.
    pub fn next<N, E, Ty: EdgeType>(&mut self, g: &Graph<N, E, Ty, Ix>) -> Option<EdgeIndex<Ix>> {
        self.next_neighbor(g).map(|(e, _)| e)
    }

    /// Fetch the next edge index and the next node index in the walk for graph `g`.
    ///
    /// The next node indices are always the others than the starting point
    /// where the `WalkEdges` value was created.
    /// For an `Outgoing` walk, the target nodes,
    /// for an `Incoming` walk, the source nodes of the edge.
    pub fn next_neighbor<N, E, Ty: EdgeType>(&mut self, g: &Graph<N, E, Ty, Ix>)
        -> Option<(EdgeIndex<Ix>, NodeIndex<Ix>)> {
        match g.edges.get(self.next.index()) {
            None => None,
            Some(edge) => {
                let edge_index = self.next;
                self.next = edge.next[self.direction as usize];
                Some((edge_index, edge.node[1 - self.direction as usize]))
            }
        }
    }
}
