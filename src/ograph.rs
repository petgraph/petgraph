use std::hash::{Hash};
use std::collections::HashSet;
use std::fmt;
use std::slice;
use std::iter;
use test;

// FIXME: These aren't stable, so a public wrapper of node/edge indices
// should be lifetimed just like pointers.
#[deriving(Copy, Clone, Show, PartialEq, PartialOrd, Eq, Hash)]
pub struct NodeIndex(uint);
#[deriving(Copy, Clone, Show, PartialEq, PartialOrd, Eq, Hash)]
pub struct EdgeIndex(uint);

pub const EdgeEnd: EdgeIndex = EdgeIndex(::std::uint::MAX);
//const InvalidNode: NodeIndex = NodeIndex(::std::uint::MAX);

/// Index into the NodeIndex and EdgeIndex arrays
#[deriving(Copy, Clone, Show, PartialEq)]
enum Dir {
    Out = 0,
    In = 1
}

const DIRECTIONS: [Dir, ..2] = [Dir::Out, Dir::In];

#[deriving(Show)]
pub struct Node<N> {
    pub data: N,
    /// Next edge in outgoing and incoming edge lists.
    next: [EdgeIndex, ..2],
}

impl<N> Node<N>
{
    pub fn next_edges(&self) -> [EdgeIndex, ..2]
    {
        self.next
    }
}

#[deriving(Show, Copy)]
pub struct Edge<E> {
    pub data: E,
    /// Next edge in outgoing and incoming edge lists.
    next: [EdgeIndex, ..2],
    /// Start and End node index
    node: [NodeIndex, ..2],
}

impl<E> Edge<E>
{
    pub fn next_edges(&self) -> [EdgeIndex, ..2]
    {
        self.next
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
//#[deriving(Show)]
pub struct OGraph<N, E> {
    nodes: Vec<Node<N>>,
    edges: Vec<Edge<E>>,
}

impl<N: fmt::Show, E: fmt::Show> fmt::Show for OGraph<N, E>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for n in self.nodes.iter() {
            try!(writeln!(f, "{}", n));
        }
        for n in self.edges.iter() {
            try!(writeln!(f, "{}", n));
        }
        Ok(())
    }
}

pub enum Pair<'a, T: 'a> {
    Both(&'a mut T, &'a mut T),
    One(&'a mut T),
    None,
}

pub fn index_twice<T>(slc: &mut [T], a: uint, b: uint) -> Pair<T>
{
    if a == b {
        slc.get_mut(a).map_or(Pair::None, Pair::One)
    } else {
        if a >= slc.len() || b >= slc.len() {
            Pair::None
        } else {
            // safe because a, b are in bounds and distinct
            unsafe {
                let ar = &mut *(slc.unsafe_mut(a) as *mut _);
                let br = &mut *(slc.unsafe_mut(b) as *mut _);
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

    /// Return the number of nodes (vertices) in the graph.
    pub fn node_count(&self) -> uint
    {
        self.nodes.len()
    }

    /// Add a node with weight **data** to the graph.
    pub fn add_node(&mut self, data: N) -> NodeIndex
    {
        let node = Node{data: data, next: [EdgeEnd, EdgeEnd]};
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
    pub fn neighbors(&self, a: NodeIndex) -> Neighbors<N, E>
    {
        Neighbors{
            graph: self,
            next: match self.nodes.get(a.0) {
                None => EdgeEnd,
                Some(n) => n.next[0],
            }
        }
    }

    /// Return an iterator over the neighbors of node **a**, paired with their respective edge
    /// weights.
    ///
    /// Produces an empty iterator if the node doesn't exist.
    ///
    /// Iterator element type is **(NodeIndex, &'a E)**.
    pub fn edges(&self, a: NodeIndex) -> Edges<N, E>
    {
        Edges{
            graph: self,
            next: match self.nodes.get(a.0) {
                None => EdgeEnd,
                Some(n) => n.next[0],
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
                None => [EdgeEnd, EdgeEnd],
                Some(n) => n.next,
            }
        }
    }
    
    /// Return an iterator over nodes that have an edge to **a**.
    ///
    /// Produces an empty iterator if the node doesn't exist.
    ///
    /// Iterator element type is **(NodeIndex, &'a E)**.
    pub fn in_edges(&self, a: NodeIndex) -> EdgesIn<N, E>
    {
        EdgesIn{
            graph: self,
            next: match self.nodes.get(a.0) {
                None => EdgeEnd,
                Some(n) => n.next[1],
            }
        }
    }

    /// Add an edge from **a** to **b** to the graph, with its edge weight.
    ///
    /// **Panics** if any of the nodes don't exist.
    pub fn add_edge(&mut self, a: NodeIndex, b: NodeIndex, data: E) -> EdgeIndex
    {
        let edge_idx = EdgeIndex(self.edges.len());
        match index_twice(self.nodes[mut], a.0, b.0) {
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
                if next == EdgeEnd {
                    break
                }
                let ret = self.remove_edge(next);
                debug_assert!(ret.is_some());
                let _ = ret;
            }
        }

        // Use swap_remove -- only the swapped-in node is going to change
        // NodeIndex, so we only have to walk its edges and update them.

        let node = match self.nodes.swap_remove(a.0) {
            None => return None,
            Some(node) => node,
        };

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
            for (_, curedge) in EdgesMut::new(self.edges[mut], swap_edges[k], d) {
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

    /// Remove an edge and return its edge weight, or **None** if it didn't exist.
    pub fn remove_edge(&mut self, e: EdgeIndex) -> Option<E>
    {
        // every edge is part of two lists,
        // outgoing and incoming edges.
        // Remove it from both
        //debug_assert!(self.edges.get(e.0).is_some(), "No such edge: {}", e);
        let (edge_node, edge_next) = match self.edges.get(e.0) {
            None => return None,
            Some(x) => (x.node, x.next),
        };
        // List out from A
        // List in from B
        for &d in DIRECTIONS.iter() {
            let k = d as uint;
            let node = match self.nodes.get_mut(edge_node[k].0) {
                Some(r) => r,
                None => {
                    debug_assert!(false, "Edge's endpoint dir={} index={} not found",
                                  k, edge_node[k]);
                    return None
                }
            };
            let fst = node.next[k];
            if fst == e {
                //println!("Updating first edge 0 for node {}, set to {}", edge_node[0], edge_next[0]);
                node.next[k] = edge_next[k];
            } else {
                for (_i, curedge) in EdgesMut::new(self.edges[mut], fst, d) {
                    if curedge.next[k] == e {
                        curedge.next[k] = edge_next[k];
                    }
                }
            }
        }
        self.remove_edge_adjust_indices(e)
    }

    fn remove_edge_adjust_indices(&mut self, e: EdgeIndex) -> Option<E>
    {
        // swap_remove the edge -- only the removed edge
        // and the edge swapped into place are affected and need updating
        // indices.
        let edge = self.edges.swap_remove(e.0).unwrap();
        let swap = match self.edges.get(e.0) {
            // no elment needed to be swapped.
            None => return Some(edge.data),
            Some(ed) => ed.node,
        };
        let swapped_e = EdgeIndex(self.edges.len());

        // List out from A
        // List in to B
        for &d in DIRECTIONS.iter() {
            let k = d as uint;
            let node = &mut self.nodes[swap[k].0];
            let fst = node.next[k];
            if fst == swapped_e {
                node.next[k] = e;
            } else {
                for (_i, curedge) in EdgesMut::new(self.edges[mut], fst, d) {
                    if curedge.next[k] == swapped_e {
                        curedge.next[k] = e;
                    }
                }
            }
        }
        let edge_data = edge.data;
        Some(edge_data)
    }

    /// Lookup an edge from **a** to **b**.
    pub fn find_edge(&self, a: NodeIndex, b: NodeIndex) -> Option<EdgeIndex>
    {
        match self.nodes.get(a.0) {
            None => None,
            Some(node) => {
                let edix = node.next[0];
                while edix != EdgeEnd {
                    let edge = &self.edges[edix.0];
                    if edge.node[1] == b {
                        return Some(edix)
                    }
                }
                None
            }
        }
    }

    pub fn first_out_edge(&self, a: NodeIndex) -> Option<EdgeIndex>
    {
        match self.nodes.get(a.0) {
            None => None,
            Some(node) => {
                let edix = node.next[0];
                if edix == EdgeEnd {
                    None
                } else { Some(edix) }
            }
        }
    }

    pub fn next_out_edge(&self, e: EdgeIndex) -> Option<EdgeIndex>
    {
        match self.edges.get(e.0) {
            None => None,
            Some(node) => {
                let edix = node.next[0];
                if edix == EdgeEnd {
                    None
                } else { Some(edix) }
            }
        }
    }

    pub fn first_in_edge(&self, a: NodeIndex) -> Option<EdgeIndex>
    {
        match self.nodes.get(a.0) {
            None => None,
            Some(node) => {
                let edix = node.next[1];
                if edix == EdgeEnd {
                    None
                } else { Some(edix) }
            }
        }
    }

    pub fn next_in_edge(&self, e: EdgeIndex) -> Option<EdgeIndex>
    {
        match self.edges.get(e.0) {
            None => None,
            Some(node) => {
                let edix = node.next[1];
                if edix == EdgeEnd {
                    None
                } else { Some(edix) }
            }
        }
    }

    /// Return an iterator over the nodes without incoming edges
    pub fn initials(&self) -> Initials<N>
    {
        Initials{iter: self.nodes.iter().enumerate()}
    }
}

pub struct Initials<'a, N: 'a> {
    iter: iter::Enumerate<slice::Iter<'a, Node<N>>>,
}

impl<'a, N: 'a> Iterator<NodeIndex> for Initials<'a, N>
{
    fn next(&mut self) -> Option<NodeIndex>
    {
        loop {
            match self.iter.next() {
                None => return None,
                Some((index, node)) if node.next[1] == EdgeEnd => {
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
    let mut tovisit = HashSet::new();
    let mut ordered = HashSet::new();

    // find all initial nodes
    tovisit.extend(g.initials());

    // Take an unvisited element and 
    while let Some(&nix) = tovisit.iter().next() {
        tovisit.remove(&nix);
        order.push(nix);
        ordered.insert(nix);
        for neigh in g.neighbors(nix) {
            // Look at each neighbor, and those that only have incoming edges
            // from the already ordered list, they are the next to visit.
            if g.in_edges(neigh).all(|(b, _)| ordered.contains(&b)) {
                tovisit.insert(neigh);
            }
        }
    }

    order
}

/// Iterator over the neighbors of a node.
///
/// Iterator element type is **NodeIndex**.
pub struct Neighbors<'a, N: 'a, E: 'a> {
    graph: &'a OGraph<N, E>,
    next: EdgeIndex,
}

impl<'a, N, E> Iterator<NodeIndex> for Neighbors<'a, N, E>
{
    fn next(&mut self) -> Option<NodeIndex>
    {
        match self.graph.edges.get(self.next.0) {
            None => None,
            Some(edge) => {
                self.next = edge.next[0];
                Some(edge.node[1])
            }
        }
    }
}

pub struct Edges<'a, N: 'a, E: 'a> {
    graph: &'a OGraph<N, E>,
    next: EdgeIndex,
}

impl<'a, N, E> Iterator<(NodeIndex, &'a E)> for Edges<'a, N, E>
{
    fn next(&mut self) -> Option<(NodeIndex, &'a E)>
    {
        match self.graph.edges.get(self.next.0) {
            None => None,
            Some(edge) => {
                self.next = edge.next[0];
                Some((edge.node[1], &edge.data))
            }
        }
    }
}

pub struct EdgesMut<'a, E: 'a> {
    edges: &'a mut [Edge<E>],
    next: EdgeIndex,
    dir: Dir,
}

impl<'a, E> EdgesMut<'a, E>
{
    fn new(edges: &mut [Edge<E>], next: EdgeIndex, dir: Dir) -> EdgesMut<E>
    {
        EdgesMut{
            edges: edges,
            next: next,
            dir: dir
        }
    }
}

impl<'a, E> Iterator<(EdgeIndex, &'a mut Edge<E>)> for EdgesMut<'a, E>
{
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
    next: [EdgeIndex, ..2],
}

impl<'a, N, E> Iterator<(NodeIndex, &'a E)> for EdgesBoth<'a, N, E>
{
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

pub struct EdgesIn<'a, N: 'a, E: 'a> {
    graph: &'a OGraph<N, E>,
    next: EdgeIndex,
}

impl<'a, N, E> Iterator<(NodeIndex, &'a E)> for EdgesIn<'a, N, E>
{
    fn next(&mut self) -> Option<(NodeIndex, &'a E)>
    {
        match self.graph.edges.get(self.next.0) {
            None => None,
            Some(edge) => {
                self.next = edge.next[1];
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
