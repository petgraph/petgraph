//! **Graph\<N, E, Ty\>** is a graph datastructure using an adjacency list representation.

use std::fmt;
use std::slice;
use std::iter;
use std::ops::{Index, IndexMut};

use std::collections::BinaryHeap;
use std::collections::BitvSet;

use super::{
    EdgeDirection, Outgoing, Incoming,
    Undirected,
    Directed,
    EdgeType,
};
use super::MinScored;

use super::unionfind::UnionFind;
use super::visit::{
    Reversed,
    Dfs,
    VisitMap,
};

// FIXME: These aren't stable, so a public wrapper of node/edge indices
// should be lifetimed just like pointers.
/// Node identifier.
#[derive(Copy, Clone, Show, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct NodeIndex(pub usize);
/// Edge identifier.
#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct EdgeIndex(pub usize);

impl fmt::Show for EdgeIndex
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "EdgeIndex("));
        if *self == EDGE_END {
            try!(write!(f, "End"));
        } else {
            try!(write!(f, "{}", self.0));
        }
        write!(f, ")")
    }
}

/// An invalid **EdgeIndex** used to denote absence of an edge, for example
/// to end an adjacency list.
pub const EDGE_END: EdgeIndex = EdgeIndex(::std::usize::MAX);

const DIRECTIONS: [EdgeDirection; 2] = [EdgeDirection::Outgoing, EdgeDirection::Incoming];

/// The graph's node type.
#[derive(Show, Clone)]
pub struct Node<N> {
    /// Associated node data.
    pub weight: N,
    /// Next edge in outgoing and incoming edge lists.
    next: [EdgeIndex; 2],
}

impl<N> Node<N>
{
    /// Accessor for data structure internals: the first edge in the given direction.
    pub fn next_edge(&self, dir: EdgeDirection) -> EdgeIndex
    {
        self.next[dir as usize]
    }
}

/// The graph's edge type.
#[derive(Show, Clone)]
pub struct Edge<E> {
    /// Associated edge data.
    pub weight: E,
    /// Next edge in outgoing and incoming edge lists.
    next: [EdgeIndex; 2],
    /// Start and End node index
    node: [NodeIndex; 2],
}

impl<E> Edge<E>
{
    /// Accessor for data structure internals: the next edge for the given direction.
    pub fn next_edge(&self, dir: EdgeDirection) -> EdgeIndex
    {
        self.next[dir as usize]
    }

    /// Return the source node index.
    pub fn source(&self) -> NodeIndex
    {
        self.node[0]
    }

    /// Return the target node index.
    pub fn target(&self) -> NodeIndex
    {
        self.node[1]
    }
}

/// **Graph\<N, E, Ty\>** is a graph datastructure using an adjacency list representation.
///
/// **Graph** is parameterized over the node weight **N**, edge weight **E** and
/// the parameter **Ty** that determines whether the graph has directed edges or not.
///
/// Based on the graph implementation in rustc.
///
/// The graph maintains unique indices for nodes and edges, and node and edge
/// weights may be accessed mutably.
///
/// **NodeIndex** and **EdgeIndex** are types that act as references to nodes and edges,
/// but these are only stable across certain operations. **Removing nodes or edges may shift
/// other indices**. Adding to the graph keeps
/// all indices stable, but removing a node will force the last node to shift its index to
/// take its place. Similarly, removing an edge shifts the index of the last edge.
#[derive(Clone)]
pub struct Graph<N, E, Ty=Directed> {
    nodes: Vec<Node<N>>,
    edges: Vec<Edge<E>>,
}

impl<N: fmt::Show, E: fmt::Show, Ty: EdgeType> fmt::Show for Graph<N, E, Ty>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (index, n) in self.nodes.iter().enumerate() {
            try!(writeln!(f, "{}: {:?}", index, n));
        }
        for (index, n) in self.edges.iter().enumerate() {
            try!(writeln!(f, "{}: {:?}", index, n));
        }
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
    /// Create a new **Graph** with directed edges.
    pub fn new() -> Self
    {
        Graph{nodes: Vec::new(), edges: Vec::new()}
    }
}

impl<N, E> Graph<N, E, Undirected>
{
    /// Create a new **Graph** with undirected edges.
    pub fn new_undirected() -> Self
    {
        Graph{nodes: Vec::new(), edges: Vec::new()}
    }
}

impl<N, E, Ty=Directed> Graph<N, E, Ty> where Ty: EdgeType
{
    /// Create a new **Graph** with estimated capacity.
    pub fn with_capacity(nodes: usize, edges: usize) -> Self
    {
        Graph{nodes: Vec::with_capacity(nodes), edges: Vec::with_capacity(edges)}
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

    /// Return whether the graph has directed edges or not.
    #[inline]
    pub fn is_directed(&self) -> bool
    {
        EdgeType::is_directed(None::<Ty>)
    }

    /// Cast the graph as either undirected or directed. No edge adjustments
    /// are done.
    ///
    /// Computes in **O(1)** time.
    pub fn into_edge_type<NewTy>(self) -> Graph<N, E, NewTy> where
        NewTy: EdgeType
    {
        Graph{nodes: self.nodes, edges: self.edges}
    }

    /// Add a node (also called vertex) with weight **w** to the graph.
    ///
    /// Computes in **O(1)** time.
    ///
    /// Return the index of the new node.
    pub fn add_node(&mut self, w: N) -> NodeIndex
    {
        let node = Node{weight: w, next: [EDGE_END, EDGE_END]};
        let node_idx = NodeIndex(self.nodes.len());
        self.nodes.push(node);
        node_idx
    }

    /// Access node weight for node **a**.
    pub fn node_weight(&self, a: NodeIndex) -> Option<&N>
    {
        self.nodes.get(a.0).map(|n| &n.weight)
    }

    /// Access node weight for node **a**.
    pub fn node_weight_mut(&mut self, a: NodeIndex) -> Option<&mut N>
    {
        self.nodes.get_mut(a.0).map(|n| &mut n.weight)
    }

    /// Return an iterator of all nodes with an edge starting from **a**.
    ///
    /// Produces an empty iterator if the node doesn't exist.
    ///
    /// Iterator element type is **NodeIndex**.
    pub fn neighbors(&self, a: NodeIndex) -> Neighbors<E>
    {
        if self.is_directed() {
            self.neighbors_directed(a, Outgoing)
        } else {
            self.neighbors_undirected(a)
        }
    }

    /// Return an iterator of all neighbors that have an edge between them and **a**,
    /// in the specified direction.
    /// If the graph is undirected, this is equivalent to *.neighbors(a)*.
    ///
    /// Produces an empty iterator if the node doesn't exist.
    ///
    /// Iterator element type is **NodeIndex**.
    pub fn neighbors_directed(&self, a: NodeIndex, dir: EdgeDirection) -> Neighbors<E>
    {
        let mut iter = self.neighbors_undirected(a);
        if self.is_directed() {
            // remove the other edges not wanted.
            let k = dir as usize;
            iter.next[1 - k] = EDGE_END;
        }
        iter
    }

    /// Return an iterator of all neighbors that have an edge between them and **a**,
    /// in either direction.
    /// If the graph is undirected, this is equivalent to *.neighbors(a)*.
    ///
    /// Produces an empty iterator if the node doesn't exist.
    ///
    /// Iterator element type is **NodeIndex**.
    pub fn neighbors_undirected(&self, a: NodeIndex) -> Neighbors<E>
    {
        Neighbors{
            edges: &*self.edges,
            next: match self.nodes.get(a.0) {
                None => [EDGE_END, EDGE_END],
                Some(n) => n.next,
            }
        }
    }

    /// Return an iterator over the neighbors of node **a**, paired with their respective edge
    /// weights.
    ///
    /// Produces an empty iterator if the node doesn't exist.
    ///
    /// Iterator element type is **(NodeIndex, &'a E)**.
    pub fn edges(&self, a: NodeIndex) -> Edges<E>
    {
        let mut iter = self.edges_both(a);
        if self.is_directed() {
            iter.next[Incoming as usize] = EDGE_END;
        }
        iter
    }

    /// Return an iterator over the edgs from **a** to its neighbors, then *to* **a** from its
    /// neighbors.
    ///
    /// Produces an empty iterator if the node doesn't exist.
    ///
    /// Iterator element type is **(NodeIndex, &'a E)**.
    pub fn edges_both(&self, a: NodeIndex) -> Edges<E>
    {
        Edges{
            edges: &*self.edges,
            next: match self.nodes.get(a.0) {
                None => [EDGE_END, EDGE_END],
                Some(n) => n.next,
            }
        }
    }
    
    /// Add an edge from **a** to **b** to the graph, with its edge weight.
    ///
    /// **Note:** **Graph** allows adding parallel (“duplicate”) edges. If you want
    /// to avoid this, use [*.update_edge(a, b, weight)*](#method.update_edge) instead.
    ///
    /// Computes in **O(1)** time.
    ///
    /// Return the index of the new edge.
    ///
    /// **Panics** if any of the nodes don't exist.
    pub fn add_edge(&mut self, a: NodeIndex, b: NodeIndex, weight: E) -> EdgeIndex
    {
        let edge_idx = EdgeIndex(self.edges.len());
        match index_twice(self.nodes.as_mut_slice(), a.0, b.0) {
            Pair::None => panic!("NodeIndices out of bounds"),
            Pair::One(an) => {
                let edge = Edge {
                    weight: weight,
                    node: [a, b],
                    next: an.next,
                };
                an.next[0] = edge_idx;
                an.next[1] = edge_idx;
                self.edges.push(edge);
            }
            Pair::Both(an, bn) => {
                // a and b are different indices
                let edge = Edge {
                    weight: weight,
                    node: [a, b],
                    next: [an.next[0], bn.next[1]],
                };
                an.next[0] = edge_idx;
                bn.next[1] = edge_idx;
                self.edges.push(edge);
            }
        }
        edge_idx
    }

    /// Add or update an edge from **a** to **b**.
    ///
    /// If the edge already exists, its weight is updated.
    ///
    /// Computes in **O(e')** time, where **e'** is the number of edges
    /// connected to the vertices **a** (and **b**).
    ///
    /// Return the index of the affected edge.
    ///
    /// **Panics** if any of the nodes don't exist.
    pub fn update_edge(&mut self, a: NodeIndex, b: NodeIndex, weight: E) -> EdgeIndex
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

    /// Access the edge weight for **e**.
    pub fn edge_weight(&self, e: EdgeIndex) -> Option<&E>
    {
        self.edges.get(e.0).map(|ed| &ed.weight)
    }

    /// Access the edge weight for **e** mutably.
    pub fn edge_weight_mut(&mut self, e: EdgeIndex) -> Option<&mut E>
    {
        self.edges.get_mut(e.0).map(|ed| &mut ed.weight)
    }

    /// Remove **a** from the graph if it exists, and return its weight.
    /// If it doesn't exist in the graph, return **None**.
    pub fn remove_node(&mut self, a: NodeIndex) -> Option<N>
    {
        match self.nodes.get(a.0) {
            None => return None,
            _ => {}
        }
        for d in DIRECTIONS.iter() { 
            let k = *d as usize;
            /*
            println!("Starting edge removal for k={}, node={}", k, a);
            for (i, n) in self.nodes.iter().enumerate() {
                println!("Node {}: Edges={}", i, n.next);
            }
            for (i, ed) in self.edges.iter().enumerate() {
                println!("Edge {}: {}", i, ed);
            }
            */
            // Remove all edges from and to this node.
            loop {
                let next = self.nodes[a.0].next[k];
                if next == EDGE_END {
                    break
                }
                let ret = self.remove_edge(next);
                debug_assert!(ret.is_some());
                let _ = ret;
            }
        }

        // Use swap_remove -- only the swapped-in node is going to change
        // NodeIndex, so we only have to walk its edges and update them.

        let node = self.nodes.swap_remove(a.0);

        // Find the edge lists of the node that had to relocate.
        // It may be that no node had to relocate, then we are done already.
        let swap_edges = match self.nodes.get(a.0) {
            None => return Some(node.weight),
            Some(ed) => ed.next,
        };

        // The swapped element's old index
        let old_index = NodeIndex(self.nodes.len());
        let new_index = a;

        // Adjust the starts of the out edges, and ends of the in edges.
        for &d in DIRECTIONS.iter() {
            let k = d as usize;
            for (_, curedge) in EdgesMut::new(self.edges.as_mut_slice(), swap_edges[k], d) {
                debug_assert!(curedge.node[k] == old_index);
                curedge.node[k] = new_index;
            }
        }
        Some(node.weight)
    }

    /// For edge **e** with endpoints **edge_node**, replace links to it,
    /// with links to **edge_next**.
    fn change_edge_links(&mut self, edge_node: [NodeIndex; 2], e: EdgeIndex,
                         edge_next: [EdgeIndex; 2])
    {
        for &d in DIRECTIONS.iter() {
            let k = d as usize;
            let node = match self.nodes.get_mut(edge_node[k].0) {
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
                for (_i, curedge) in EdgesMut::new(self.edges.as_mut_slice(), fst, d) {
                    if curedge.next[k] == e {
                        curedge.next[k] = edge_next[k];
                        break; // the edge can only be present once in the list.
                    }
                }
            }
        }
    }

    /// Remove an edge and return its edge weight, or **None** if it didn't exist.
    ///
    /// Computes in **O(e')** time, where **e'** is the size of four particular edge lists, for
    /// the vertices of **e** and the vertices of another affected edge.
    pub fn remove_edge(&mut self, e: EdgeIndex) -> Option<E>
    {
        // every edge is part of two lists,
        // outgoing and incoming edges.
        // Remove it from both
        let (edge_node, edge_next) = match self.edges.get(e.0) {
            None => return None,
            Some(x) => (x.node, x.next),
        };
        // Remove the edge from its in and out lists by replacing it with
        // a link to the next in the list.
        self.change_edge_links(edge_node, e, edge_next);
        self.remove_edge_adjust_indices(e)
    }

    fn remove_edge_adjust_indices(&mut self, e: EdgeIndex) -> Option<E>
    {
        // swap_remove the edge -- only the removed edge
        // and the edge swapped into place are affected and need updating
        // indices.
        let edge = self.edges.swap_remove(e.0);
        let swap = match self.edges.get(e.0) {
            // no elment needed to be swapped.
            None => return Some(edge.weight),
            Some(ed) => ed.node,
        };
        let swapped_e = EdgeIndex(self.edges.len());

        // Update the edge lists by replacing links to the old index by references to the new
        // edge index.
        self.change_edge_links(swap, swapped_e, [e, e]);
        Some(edge.weight)
    }

    /// Lookup an edge from **a** to **b**.
    ///
    /// Computes in **O(e')** time, where **e'** is the number of edges
    /// connected to the vertices **a** (and **b**).
    pub fn find_edge(&self, a: NodeIndex, b: NodeIndex) -> Option<EdgeIndex>
    {
        if !self.is_directed() {
            self.find_edge_undirected(a, b).map(|(ix, _)| ix)
        } else {
            match self.nodes.get(a.0) {
                None => None,
                Some(node) => {
                    let mut edix = node.next[0];
                    while let Some(edge) = self.edges.get(edix.0) {
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

    /// Lookup an edge between **a** and **b**, in either direction.
    ///
    /// If the graph is undirected, then this is equivalent to *.find_edge()*.
    pub fn find_edge_undirected(&self, a: NodeIndex, b: NodeIndex) -> Option<(EdgeIndex, EdgeDirection)>
    {
        match self.nodes.get(a.0) {
            None => None,
            Some(node) => {
                for &d in DIRECTIONS.iter() {
                    let k = d as usize;
                    let mut edix = node.next[k];
                    while let Some(edge) = self.edges.get(edix.0) {
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
        F: FnMut(&Self, NodeIndex, &Node<N>) -> bool
    {
        for index in (0..self.node_count()).rev() {
            let nix = NodeIndex(index);
            if !visit(&*self, nix, &self.nodes[nix.0]) {
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
            let eix = EdgeIndex(index);
            if !visit(&*self, eix, &self.edges[eix.0]) {
                let ret = self.remove_edge(EdgeIndex(index));
                debug_assert!(ret.is_some());
                let _ = ret;
            }
        }
    }
    */

    /// Access the internal node array
    pub fn raw_nodes(&self) -> &[Node<N>]
    {
        &*self.nodes
    }

    /// Access the internal edge array
    pub fn raw_edges(&self) -> &[Edge<E>]
    {
        &*self.edges
    }

    /// Accessor for data structure internals: the first edge in the given direction.
    pub fn first_edge(&self, a: NodeIndex, dir: EdgeDirection) -> Option<EdgeIndex>
    {
        match self.nodes.get(a.0) {
            None => None,
            Some(node) => {
                let edix = node.next[dir as usize];
                if edix == EDGE_END {
                    None
                } else { Some(edix) }
            }
        }
    }

    /// Accessor for data structure internals: the next edge for the given direction.
    pub fn next_edge(&self, e: EdgeIndex, dir: EdgeDirection) -> Option<EdgeIndex>
    {
        match self.edges.get(e.0) {
            None => None,
            Some(node) => {
                let edix = node.next[dir as usize];
                if edix == EDGE_END {
                    None
                } else { Some(edix) }
            }
        }
    }

    /// Return an iterator over either the nodes without edges to them or from them.
    ///
    /// The nodes in *.without_edges(Incoming)* are the initial nodes and 
    /// *.without_edges(Outgoing)* are the terminals.
    ///
    /// For an undirected graph, the initials/terminals are just the vertices without edges.
    ///
    /// The whole iteration computes in **O(|V|)** time.
    pub fn without_edges(&self, dir: EdgeDirection) -> WithoutEdges<N, Ty>
    {
        WithoutEdges{iter: self.nodes.iter().enumerate(), dir: dir}
    }
}

/// An iterator over either the nodes without edges to them or from them.
pub struct WithoutEdges<'a, N: 'a, Ty> {
    iter: iter::Enumerate<slice::Iter<'a, Node<N>>>,
    dir: EdgeDirection,
}

impl<'a, N: 'a, Ty> Iterator for WithoutEdges<'a, N, Ty> where
    Ty: EdgeType
{
    type Item = NodeIndex;
    fn next(&mut self) -> Option<NodeIndex>
    {
        let k = self.dir as usize;
        loop {
            match self.iter.next() {
                None => return None,
                Some((index, node)) => {
                    if node.next[k] == EDGE_END &&
                        (EdgeType::is_directed(None::<Ty>) ||
                         node.next[1-k] == EDGE_END) {
                        return Some(NodeIndex(index))
                    } else {
                        continue
                    }
                },
            }
        }
    }
}

/// Perform a topological sort of a directed graph.
///
/// Return a vector of nodes in topological order: each node is ordered
/// before its successors.
///
/// If the returned vec contains less than all the nodes of the graph, then
/// the graph was cyclic.
pub fn toposort<N, E>(g: &Graph<N, E, Directed>) -> Vec<NodeIndex>
{
    let mut order = Vec::with_capacity(g.node_count());
    let mut ordered = BitvSet::with_capacity(g.node_count());
    let mut tovisit = Vec::new();

    // find all initial nodes
    tovisit.extend(g.without_edges(Incoming));

    // Take an unvisited element and 
    while let Some(nix) = tovisit.pop() {
        if ordered.contains(&nix.0) {
            continue;
        }
        order.push(nix);
        ordered.insert(nix.0);
        for neigh in g.neighbors_directed(nix, Outgoing) {
            // Look at each neighbor, and those that only have incoming edges
            // from the already ordered list, they are the next to visit.
            if g.neighbors_directed(neigh, Incoming).all(|b| ordered.contains(&b.0)) {
                tovisit.push(neigh);
            }
        }
    }

    order
}

/// Compute *Strongly connected components* using Kosaraju's algorithm.
///
/// Return a vector where each element is an scc.
///
/// For an undirected graph, the sccs are simply the connected components.
pub fn scc<N, E, Ty>(g: &Graph<N, E, Ty>) -> Vec<Vec<NodeIndex>> where
    Ty: EdgeType
{
    let mut dfs = Dfs::empty(g);

    // First phase, reverse dfs pass, compute finishing times.
    let mut finish_order = Vec::new();
    for index in (0..g.node_count()) {
        if dfs.discovered.contains(&index) {
            continue
        }
        // We want to order the vertices by finishing time --
        // so record when we see them, then reverse all we have seen in this DFS pass.
        let pass_start = finish_order.len();
        dfs.move_to(NodeIndex(index));
        while let Some(nx) = dfs.next(&Reversed(g)) {
            finish_order.push(nx);
        }
        finish_order[pass_start..].reverse();
    }

    dfs.discovered.clear();
    let mut sccs = Vec::new();

    // Second phase
    // Process in decreasing finishing time order
    for &nindex in finish_order.iter().rev() {
        if dfs.discovered.contains(&nindex.0) {
            continue;
        }
        // Move to the leader node.
        dfs.move_to(nindex);
        //let leader = nindex;
        let mut scc = Vec::new();
        while let Some(nx) = dfs.next(g) {
            scc.push(nx);
        }
        sccs.push(scc);
    }
    sccs
}

/// Return **true** if the input graph contains a cycle.
///
/// Treat the input graph as undirected.
pub fn is_cyclic<N, E, Ty>(g: &Graph<N, E, Ty>) -> bool where Ty: EdgeType
{
    let mut edge_sets = UnionFind::new(g.node_count());
    for edge in g.edges.iter() {
        let (a, b) = (edge.source(), edge.target());

        // union the two vertices of the edge
        //  -- if they were already the same, then we have a cycle
        if !edge_sets.union(a.0, b.0) {
            return true
        }
    }
    false
}

/// Return the number of connected components of the graph.
///
/// For a directed graph, this is the *weakly* connected components.
pub fn connected_components<N, E, Ty>(g: &Graph<N, E, Ty>) -> usize where Ty: EdgeType
{
    let mut vertex_sets = UnionFind::new(g.node_count());
    for edge in g.edges.iter() {
        let (a, b) = (edge.source(), edge.target());

        // union the two vertices of the edge
        vertex_sets.union(a.0, b.0);
    }
    let mut labels = vertex_sets.into_labeling();
    labels.sort();
    labels.dedup();
    labels.len()
}


/// Return a *Minimum Spanning Tree* of a graph.
///
/// Treat the input graph as undirected.
///
/// Using Kruskal's algorithm with runtime **O(|E| log |E|)**. We actually
/// return a minimum spanning forest, i.e. a minimum spanning tree for each connected
/// component of the graph.
///
/// The resulting graph has all the vertices of the input graph (with identical node indices),
/// and **|V| - c** edges, where **c** is the number of connected components in **g**.
pub fn min_spanning_tree<N, E, Ty>(g: &Graph<N, E, Ty>) -> Graph<N, E, Undirected> where
    N: Clone,
    E: Clone + PartialOrd,
    Ty: EdgeType,
{
    if g.node_count() == 0 {
        return Graph::new_undirected()
    }

    // Create a mst skeleton by copying all nodes
    let mut mst = Graph::with_capacity(g.node_count(), g.node_count() - 1);
    for node in g.nodes.iter() {
        mst.add_node(node.weight.clone());
    }

    // Initially each vertex is its own disjoint subgraph, track the connectedness
    // of the pre-MST with a union & find datastructure.
    let mut subgraphs = UnionFind::new(g.node_count());

    let mut sort_edges = BinaryHeap::with_capacity(g.edge_count());
    for edge in g.edges.iter() {
        sort_edges.push(MinScored(edge.weight.clone(), (edge.source(), edge.target())));
    }

    // Kruskal's algorithm.
    // Algorithm is this:
    //
    // 1. Create a pre-MST with all the vertices and no edges.
    // 2. Repeat:
    //
    //  a. Remove the shortest edge from the original graph.
    //  b. If the edge connects two disjoint trees in the pre-MST,
    //     add the edge.
    while let Some(MinScored(score, (a, b))) = sort_edges.pop() {
        // check if the edge would connect two disjoint parts
        if subgraphs.union(a.0, b.0) {
            mst.add_edge(a, b, score);
        }
    }

    debug_assert!(mst.node_count() == g.node_count());
    debug_assert!(mst.edge_count() < g.node_count());
    mst
}

/*
/// Iterator over the neighbors of a node.
///
/// Iterator element type is **NodeIndex**.
pub struct DiNeighbors<'a, E: 'a> {
    edges: &'a [Edge<E>],
    next: EdgeIndex,
    dir: EdgeDirection,
}

impl<'a, E> Iterator for DiNeighbors<'a, E>
{
    type Item = NodeIndex;
    fn next(&mut self) -> Option<NodeIndex>
    {
        let k = self.dir as usize;
        match self.edges.get(self.next.0) {
            None => None,
            Some(edge) => {
                self.next = edge.next[k];
                Some(edge.node[1-k])
            }
        }
    }
}
*/

/// Iterator over the neighbors of a node.
///
/// Iterator element type is **NodeIndex**.
pub struct Neighbors<'a, E: 'a> {
    edges: &'a [Edge<E>],
    next: [EdgeIndex; 2],
}

impl<'a, E> Iterator for Neighbors<'a, E>
{
    type Item = NodeIndex;
    fn next(&mut self) -> Option<NodeIndex>
    {
        match self.edges.get(self.next[0].0) {
            None => {}
            Some(edge) => {
                self.next[0] = edge.next[0];
                return Some(edge.node[1])
            }
        }
        match self.edges.get(self.next[1].0) {
            None => None,
            Some(edge) => {
                self.next[1] = edge.next[1];
                Some(edge.node[0])
            }
        }
    }
}

struct EdgesMut<'a, E: 'a> {
    edges: &'a mut [Edge<E>],
    next: EdgeIndex,
    dir: EdgeDirection,
}

impl<'a, E> EdgesMut<'a, E>
{
    fn new(edges: &'a mut [Edge<E>], next: EdgeIndex, dir: EdgeDirection) -> Self
    {
        EdgesMut{
            edges: edges,
            next: next,
            dir: dir
        }
    }
}

impl<'a, E> Iterator for EdgesMut<'a, E>
{
    type Item = (EdgeIndex, &'a mut Edge<E>);
    fn next(&mut self) -> Option<(EdgeIndex, &'a mut Edge<E>)>
    {
        let this_index = self.next;
        let k = self.dir as usize;
        match self.edges.get_mut(self.next.0) {
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
pub struct Edges<'a, E: 'a> {
    edges: &'a [Edge<E>],
    next: [EdgeIndex; 2],
}

impl<'a, E> Iterator for Edges<'a, E>
{
    type Item = (NodeIndex, &'a E);
    fn next(&mut self) -> Option<(NodeIndex, &'a E)>
    {
        // First any outgoing edges
        match self.edges.get(self.next[0].0) {
            None => {}
            Some(edge) => {
                self.next[0] = edge.next[0];
                return Some((edge.node[1], &edge.weight))
            }
        }
        // Then incoming edges
        match self.edges.get(self.next[1].0) {
            None => None,
            Some(edge) => {
                self.next[1] = edge.next[1];
                Some((edge.node[0], &edge.weight))
            }
        }
    }
}

impl<N, E, Ty> Index<NodeIndex> for Graph<N, E, Ty> where
    Ty: EdgeType
{
    type Output = N;
    /// Index the **Graph** by **NodeIndex** to access node weights.
    fn index(&self, index: &NodeIndex) -> &N {
        self.node_weight(*index).unwrap()
    }
}

impl<N, E, Ty> IndexMut<NodeIndex> for Graph<N, E, Ty> where
    Ty: EdgeType
{
    type Output = N;
    /// Index the **Graph** by **NodeIndex** to access node weights.
    fn index_mut(&mut self, index: &NodeIndex) -> &mut N {
        self.node_weight_mut(*index).unwrap()
    }

}
impl<N, E, Ty> Index<EdgeIndex> for Graph<N, E, Ty> where
    Ty: EdgeType
{
    type Output = E;
    /// Index the **Graph** by **EdgeIndex** to access edge weights.
    fn index(&self, index: &EdgeIndex) -> &E {
        self.edge_weight(*index).unwrap()
    }
}

impl<N, E, Ty> IndexMut<EdgeIndex> for Graph<N, E, Ty> where
    Ty: EdgeType
{
    type Output = E;
    /// Index the **Graph** by **EdgeIndex** to access edge weights.
    fn index_mut(&mut self, index: &EdgeIndex) -> &mut E {
        self.edge_weight_mut(*index).unwrap()
    }
}
