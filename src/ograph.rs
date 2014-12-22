use std::hash::{Hash};
use std::slice::{
    Items,
};
use std::fmt;
use test;

// FIXME: These aren't stable, so a public wrapper of node/edge indices
// should be lifetimed just like pointers.
#[deriving(Copy, Clone, Show, PartialEq, PartialOrd, Eq, Hash)]
pub struct NodeIndex(uint);
#[deriving(Copy, Clone, Show, PartialEq, PartialOrd, Eq, Hash)]
pub struct EdgeIndex(uint);

const EdgeEnd: EdgeIndex = EdgeIndex(::std::uint::MAX);
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

#[deriving(Show, Copy)]
pub struct Edge<E> {
    pub data: E,
    /// Next edge in outgoing and incoming edge lists.
    next: [EdgeIndex, ..2],
    /// Start and End node index
    node: [NodeIndex, ..2],
}

/// **OGraph\<N, E\>** is a directed graph using an adjacency list representation.
///
/// The graph maintains unique indices for nodes and edges, so both node and edge
/// data may be accessed mutably. Removing nodes or edges from the graph is expensive,
/// while adding is very cheap.
///
/// Based upon the graph implementation in rustc.
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

/// Iterate over an edge list.
fn walk_edge_list<E, F: FnMut(EdgeIndex, &mut Edge<E>) -> bool>(
    fst: EdgeIndex, edges: &mut [Edge<E>], d: Dir, mut f: F)
{
    let k = d as uint;
    let mut cur = fst;
    loop {
        match edges.get_mut(cur.0) {
            None => {
                debug_assert!(cur == EdgeEnd);
                break;
            }
            Some(curedge) => {
                if !f(cur, curedge) {
                    break;
                }
                cur = curedge.next[k];
            }
        }
    }
}

impl<N, E> OGraph<N, E>
//where N: fmt::Show
{
    pub fn new() -> OGraph<N, E>
    {
        OGraph{nodes: Vec::new(), edges: Vec::new()}
    }

    pub fn add_node(&mut self, data: N) -> NodeIndex
    {
        let node = Node{data: data, next: [EdgeEnd, EdgeEnd]};
        let node_idx = NodeIndex(self.nodes.len());
        self.nodes.push(node);
        node_idx
    }

    pub fn node(&self, a: NodeIndex) -> Option<&N>
    {
        self.nodes.get(a.0).map(|n| &n.data)
    }

    pub fn node_mut(&mut self, a: NodeIndex) -> Option<&mut N>
    {
        self.nodes.get_mut(a.0).map(|n| &mut n.data)
    }

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
            walk_edge_list(swap_edges[k], self.edges[mut], d, |_, curedge| {
                debug_assert!(curedge.node[k] == old_index);
                curedge.node[k] = new_index;
                true
            });
        }
        Some(node.data)
    }

    pub fn edge_mut(&mut self, e: EdgeIndex) -> &mut Edge<E>
    {
        &mut self.edges[e.0]
    }

    /// Remove an edge and return its edge weight, or None if it didn't exist.
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
                walk_edge_list(fst, self.edges[mut], d, |eidx, curedge| {
                    if curedge.next[k] == e {
                        curedge.next[k] = edge_next[k];
                        false
                    } else { true }
                });
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
                walk_edge_list(fst, self.edges[mut], d, |eidx, curedge| {
                    if curedge.next[k] == swapped_e {
                        curedge.next[k] = e;
                        false
                    } else { true }
                });
            }
        }
        let edge_data = edge.data;
        Some(edge_data)
    }
}

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
