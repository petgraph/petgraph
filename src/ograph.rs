use std::hash::{Hash};
use std::collections::HashSet;
use std::fmt;
use std::slice;
use std::iter;

use std::collections::BinaryHeap;

use super::{EdgeDirection, Outgoing, Incoming};
use super::MinScored;

use super::unionfind::UnionFind;

#[cfg(test)]
use test;

// FIXME: These aren't stable, so a public wrapper of node/edge indices
// should be lifetimed just like pointers.
#[derive(Copy, Clone, Show, PartialEq, PartialOrd, Eq, Hash)]
pub struct NodeIndex(uint);
#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Hash)]
pub struct EdgeIndex(uint);

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

pub const EDGE_END: EdgeIndex = EdgeIndex(::std::uint::MAX);
//const InvalidNode: NodeIndex = NodeIndex(::std::uint::MAX);

const DIRECTIONS: [EdgeDirection; 2] = [EdgeDirection::Outgoing, EdgeDirection::Incoming];

#[derive(Show, Clone)]
pub struct Node<N> {
    pub data: N,
    /// Next edge in outgoing and incoming edge lists.
    next: [EdgeIndex; 2],
}

impl<N> Node<N>
{
    pub fn next_edge(&self, dir: EdgeDirection) -> EdgeIndex
    {
        self.next[dir as uint]
    }
}

#[derive(Show, Clone)]
pub struct Edge<E> {
    pub data: E,
    /// Next edge in outgoing and incoming edge lists.
    next: [EdgeIndex; 2],
    /// Start and End node index
    node: [NodeIndex; 2],
}

impl<E> Edge<E>
{
    pub fn next_edge(&self, dir: EdgeDirection) -> EdgeIndex
    {
        self.next[dir as uint]
    }

    pub fn source(&self) -> NodeIndex
    {
        self.node[0]
    }

    pub fn target(&self) -> NodeIndex
    {
        self.node[1]
    }
}

/// **OGraph\<N, E\>** is a directed graph using an adjacency list representation.
///
/// The graph maintains unique indices for nodes and edges, so both node and edge
/// data may be accessed mutably.
///
/// Based upon the graph implementation in rustc.
///
/// **NodeIndex** and **EdgeIndex** are types that act as references to nodes and edges,
/// but these are only stable across certain operations. Adding to the graph keeps
/// all indices stable, but removing a node will force another node to shift its index.
///
/// Removing an edge also shifts the index of another edge.
#[derive(Clone)]
pub struct OGraph<N, E> {
    nodes: Vec<Node<N>>,
    edges: Vec<Edge<E>>,
}

impl<N: fmt::Show, E: fmt::Show> fmt::Show for OGraph<N, E>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (index, n) in self.nodes.iter().enumerate() {
            try!(writeln!(f, "{}: {}", index, n));
        }
        for (index, n) in self.edges.iter().enumerate() {
            try!(writeln!(f, "{}: {}", index, n));
        }
        Ok(())
    }
}

enum Pair<T> {
    Both(T, T),
    One(T),
    None,
}

fn index_twice<T>(slc: &mut [T], a: uint, b: uint) -> Pair<&mut T>
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

impl<N, E> OGraph<N, E>
//where N: fmt::Show
{
    /// Create a new OGraph.
    pub fn new() -> OGraph<N, E>
    {
        OGraph{nodes: Vec::new(), edges: Vec::new()}
    }

    /// Create a new OGraph with estimated capacity
    pub fn with_capacity(nodes: uint, edges: uint) -> Self
    {
        OGraph{nodes: Vec::with_capacity(nodes), edges: Vec::with_capacity(edges)}
    }

    /// Return the number of nodes (vertices) in the graph.
    pub fn node_count(&self) -> uint
    {
        self.nodes.len()
    }

    /// Return the number of edges in the graph.
    ///
    /// This will compute in O(1) time.
    pub fn edge_count(&self) -> uint
    {
        self.edges.len()
    }

    /// Add a node with weight **data** to the graph.
    pub fn add_node(&mut self, data: N) -> NodeIndex
    {
        let node = Node{data: data, next: [EDGE_END, EDGE_END]};
        let node_idx = NodeIndex(self.nodes.len());
        self.nodes.push(node);
        node_idx
    }

    /// Access node data for node **a**.
    pub fn node(&self, a: NodeIndex) -> Option<&N>
    {
        self.nodes.get(a.0).map(|n| &n.data)
    }

    /// Access node data for node **a**.
    pub fn node_mut(&mut self, a: NodeIndex) -> Option<&mut N>
    {
        self.nodes.get_mut(a.0).map(|n| &mut n.data)
    }

    /// Return an iterator of all neighbors that have an edge from **a** to them.
    ///
    /// Produces an empty iterator if the node doesn't exist.
    ///
    /// Iterator element type is **NodeIndex**.
    pub fn neighbors(&self, a: NodeIndex, dir: EdgeDirection) -> Neighbors<E>
    {
        Neighbors{
            edges: &*self.edges,
            dir: dir,
            next: match self.nodes.get(a.0) {
                None => EDGE_END,
                Some(n) => n.next[dir as uint],
            }
        }
    }

    /// Return an iterator of all neighbors that have an edge from **a** to them.
    ///
    /// Produces an empty iterator if the node doesn't exist.
    ///
    /// Iterator element type is **NodeIndex**.
    pub fn neighbors_both(&self, a: NodeIndex) -> NeighborsBoth<E>
    {
        NeighborsBoth{
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
    pub fn edges(&self, a: NodeIndex, dir: EdgeDirection) -> Edges<E>
    {
        Edges{
            edges: &*self.edges,
            dir: dir,
            next: match self.nodes.get(a.0) {
                None => EDGE_END,
                Some(n) => n.next[dir as uint],
            }
        }
    }

    /// Return an iterator over the edgs from **a** to its neighbors, then *to* **a** from its
    /// neighbors.
    ///
    /// Produces an empty iterator if the node doesn't exist.
    ///
    /// Iterator element type is **(NodeIndex, &'a E)**.
    pub fn edges_both(&self, a: NodeIndex) -> EdgesBoth<N, E>
    {
        EdgesBoth{
            graph: self,
            next: match self.nodes.get(a.0) {
                None => [EDGE_END, EDGE_END],
                Some(n) => n.next,
            }
        }
    }
    
    /// Add an edge from **a** to **b** to the graph, with its edge weight.
    ///
    /// **Panics** if any of the nodes don't exist.
    pub fn add_edge(&mut self, a: NodeIndex, b: NodeIndex, data: E) -> EdgeIndex
    {
        let edge_idx = EdgeIndex(self.edges.len());
        match index_twice(self.nodes.as_mut_slice(), a.0, b.0) {
            Pair::None => panic!("NodeIndices out of bounds"),
            Pair::One(an) => {
                let edge = Edge {
                    data: data,
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
                    data: data,
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

    /// Remove **a** from the graph if it exists, and return its data value.
    /// If it doesn't exist in the graph, return **None**.
    pub fn remove_node(&mut self, a: NodeIndex) -> Option<N>
    {
        match self.nodes.get(a.0) {
            None => return None,
            _ => {}
        }
        for d in DIRECTIONS.iter() { 
            let k = *d as uint;
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
            None => return Some(node.data),
            Some(ed) => ed.next,
        };

        // The swapped element's old index
        let old_index = NodeIndex(self.nodes.len());
        let new_index = a;

        // Adjust the starts of the out edges, and ends of the in edges.
        for &d in DIRECTIONS.iter() {
            let k = d as uint;
            for (_, curedge) in EdgesMut::new(self.edges.as_mut_slice(), swap_edges[k], d) {
                debug_assert!(curedge.node[k] == old_index);
                curedge.node[k] = new_index;
            }
        }
        Some(node.data)
    }

    pub fn edge_mut(&mut self, e: EdgeIndex) -> &mut Edge<E>
    {
        &mut self.edges[e.0]
    }

    /// For edge **e** with endpoints **edge_node**, replace links to it,
    /// with links to **edge_next**.
    fn change_edge_links(&mut self, edge_node: [NodeIndex; 2], e: EdgeIndex,
                         edge_next: [EdgeIndex; 2])
    {
        for &d in DIRECTIONS.iter() {
            let k = d as uint;
            let node = match self.nodes.get_mut(edge_node[k].0) {
                Some(r) => r,
                None => {
                    debug_assert!(false, "Edge's endpoint dir={} index={} not found",
                                  k, edge_node[k]);
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
                    }
                }
            }
        }
    }

    /// Remove an edge and return its edge weight, or **None** if it didn't exist.
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
            None => return Some(edge.data),
            Some(ed) => ed.node,
        };
        let swapped_e = EdgeIndex(self.edges.len());

        // Update the edge lists by replacing links to the old index by references to the new
        // edge index.
        self.change_edge_links(swap, swapped_e, [e, e]);
        Some(edge.data)
    }

    /// Lookup an edge from **a** to **b**.
    pub fn find_edge(&self, a: NodeIndex, b: NodeIndex) -> Option<EdgeIndex>
    {
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

    /// Lookup an edge between **a** and **b**.
    pub fn find_any_edge(&self, a: NodeIndex, b: NodeIndex) -> Option<(EdgeIndex, EdgeDirection)>
    {
        match self.nodes.get(a.0) {
            None => None,
            Some(node) => {
                for &d in DIRECTIONS.iter() {
                    let k = d as uint;
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

    pub fn first_edge(&self, a: NodeIndex, dir: EdgeDirection) -> Option<EdgeIndex>
    {
        match self.nodes.get(a.0) {
            None => None,
            Some(node) => {
                let edix = node.next[dir as uint];
                if edix == EDGE_END {
                    None
                } else { Some(edix) }
            }
        }
    }

    pub fn next_edge(&self, e: EdgeIndex, dir: EdgeDirection) -> Option<EdgeIndex>
    {
        match self.edges.get(e.0) {
            None => None,
            Some(node) => {
                let edix = node.next[dir as uint];
                if edix == EDGE_END {
                    None
                } else { Some(edix) }
            }
        }
    }

    /// Return an iterator over either the nodes without edges to them or from them.
    ///
    /// The nodes in **.without_edges(Incoming)** are the initial nodes and 
    /// **.without_edges(Outgoing)** are the terminals.
    pub fn without_edges(&self, dir: EdgeDirection) -> WithoutEdges<N>
    {
        WithoutEdges{iter: self.nodes.iter().enumerate(), dir: dir}
    }
}

/// An iterator over either the nodes without edges to them or from them.
pub struct WithoutEdges<'a, N: 'a> {
    iter: iter::Enumerate<slice::Iter<'a, Node<N>>>,
    dir: EdgeDirection,
}

impl<'a, N: 'a> Iterator for WithoutEdges<'a, N>
{
    type Item = NodeIndex;
    fn next(&mut self) -> Option<NodeIndex>
    {
        let k = self.dir as uint;
        loop {
            match self.iter.next() {
                None => return None,
                Some((index, node)) if node.next[k] == EDGE_END => {
                    return Some(NodeIndex(index))
                },
                _ => continue,
            }
        }
    }
}

/// Perform a topological sort of the graph.
///
/// Return a vector of nodes in topological order: each node is ordered
/// before its successors.
///
/// If the returned vec contains less than all the nodes of the graph, then
/// the graph was cyclic.
pub fn toposort<N, E>(g: &OGraph<N, E>) -> Vec<NodeIndex>
{
    let mut order = Vec::with_capacity(g.node_count());
    let mut ordered = HashSet::with_capacity(g.node_count());
    let mut tovisit = Vec::new();

    // find all initial nodes
    tovisit.extend(g.without_edges(Incoming));

    // Take an unvisited element and 
    while let Some(nix) = tovisit.pop() {
        if ordered.contains(&nix) {
            continue;
        }
        order.push(nix);
        ordered.insert(nix);
        for neigh in g.neighbors(nix, EdgeDirection::Outgoing) {
            // Look at each neighbor, and those that only have incoming edges
            // from the already ordered list, they are the next to visit.
            if g.neighbors(neigh, EdgeDirection::Incoming).all(|b| ordered.contains(&b)) {
                tovisit.push(neigh);
            }
        }
    }

    order
}

/// Return a *Minimum Spanning Tree* of a graph.
///
/// Treat the input graph as undirected.
pub fn min_spanning_tree<N, E>(g: &OGraph<N, E>) -> OGraph<N, E>
    where N: Clone, E: Clone + PartialOrd
{
    if g.node_count() == 0 {
        return OGraph::new()
    }

    // Create a mst skeleton by copying all nodes
    let mut mst = OGraph::with_capacity(g.node_count(), g.node_count() - 1);
    for node in g.nodes.iter() {
        mst.add_node(node.data.clone());
    }

    // Initially each vertex is its own disjoint subgraph, track the connectedness
    // of the pre-MST with a union & find datastructure.
    let mut subgraphs = UnionFind::new(g.node_count());

    let mut sort_edges = BinaryHeap::with_capacity(g.edge_count());
    for edge in g.edges.iter() {
        sort_edges.push(MinScored(edge.data.clone(), (edge.source(), edge.target())));
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
    // If the graph is connected, |E| will be |V| - 1,
    // otherwise this applies instead to the disjoint parts,
    // so |E| - |disjoint parts|
    debug_assert!(mst.edge_count() < g.node_count());
    mst
}

/// Iterator over the neighbors of a node.
///
/// Iterator element type is **NodeIndex**.
pub struct Neighbors<'a, E: 'a> {
    edges: &'a [Edge<E>],
    next: EdgeIndex,
    dir: EdgeDirection,
}

impl<'a, E> Iterator for Neighbors<'a, E>
{
    type Item = NodeIndex;
    fn next(&mut self) -> Option<NodeIndex>
    {
        let k = self.dir as uint;
        match self.edges.get(self.next.0) {
            None => None,
            Some(edge) => {
                self.next = edge.next[k];
                Some(edge.node[1-k])
            }
        }
    }
}

/// Iterator over the neighbors of a node.
///
/// Iterator element type is **NodeIndex**.
pub struct NeighborsBoth<'a, E: 'a> {
    edges: &'a [Edge<E>],
    next: [EdgeIndex; 2],
}

impl<'a, E> Iterator for NeighborsBoth<'a, E>
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

pub struct Edges<'a, E: 'a> {
    edges: &'a [Edge<E>],
    next: EdgeIndex,
    dir: EdgeDirection,
}

impl<'a, E> Iterator for Edges<'a, E>
{
    type Item = (NodeIndex, &'a E);
    fn next(&mut self) -> Option<(NodeIndex, &'a E)>
    {
        let k = self.dir as uint;
        match self.edges.get(self.next.0) {
            None => None,
            Some(edge) => {
                self.next = edge.next[k];
                Some((edge.node[1-k], &edge.data))
            }
        }
    }

    fn size_hint(&self) -> (uint, Option<uint>)
    {
        let low = (self.next != EDGE_END) as uint;
        let hi = low * self.edges.len();
        (low, Some(hi))
    }
}

pub struct EdgesMut<'a, E: 'a> {
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
        let k = self.dir as uint;
        match self.edges.get_mut(self.next.0) {
            None => None,
            Some(edge) => {
                self.next = edge.next[k];
                // We cannot in safe rust, derive a &'a mut from &self,
                // because the life of &self is shorter than 'a.
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

pub struct EdgesBoth<'a, N: 'a, E: 'a> {
    graph: &'a OGraph<N, E>,
    next: [EdgeIndex; 2],
}

impl<'a, N, E> Iterator for EdgesBoth<'a, N, E>
{
    type Item = (NodeIndex, &'a E);
    fn next(&mut self) -> Option<(NodeIndex, &'a E)>
    {
        // First any outgoing edges
        match self.graph.edges.get(self.next[0].0) {
            None => {}
            Some(edge) => {
                self.next[0] = edge.next[0];
                return Some((edge.node[1], &edge.data))
            }
        }
        // Then incoming edges
        match self.graph.edges.get(self.next[1].0) {
            None => None,
            Some(edge) => {
                self.next[1] = edge.next[1];
                Some((edge.node[0], &edge.data))
            }
        }
    }
}

#[bench]
fn bench_inser(b: &mut test::Bencher) {
    let mut og = OGraph::new();
    let fst = og.add_node(0i);
    for x in range(1, 125) {
        let n = og.add_node(x);
        og.add_edge(fst, n, ());
    }
    b.iter(|| {
        og.add_node(1)
    })
}

#[bench]
fn bench_remove(b: &mut test::Bencher) {
    // removal is very slow in a big graph.
    // and this one doesn't even have many nodes.
    let mut og = OGraph::new();
    let fst = og.add_node(0i);
    let mut prev = fst;
    for x in range(1, 1250) {
        let n = og.add_node(x);
        og.add_edge(prev, n, ());
        prev = n;
    }
    //println!("{}", og);
    b.iter(|| {
        for _ in range(0, 100i) {
            og.remove_node(fst);
        }
    })
}
