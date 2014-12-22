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

/// Index into the EdgeIndex arrays
enum Dir {
    Out = 0,
    In = 1
}

#[deriving(Show)]
pub struct Node<N> {
    pub data: N,
    next: [EdgeIndex, ..2],
}

#[deriving(Show, Copy)]
pub struct Edge<E> {
    pub data: E,
    next: [EdgeIndex, ..2],
    a: NodeIndex,
    b: NodeIndex,
}

/// **OGraph\<N, E\>** is a directed graph using an adjacency list representation.
///
/// The graph maintains unique indices for nodes and edges, so both node and edge
/// data may be accessed mutably. Removing nodes or edges from the graph is expensive,
/// while adding is very cheap.
///
/// Based upon the graph implementation in rustc.
//#[deriving(Show)]
pub struct OGraph<N> {
    nodes: Vec<Node<N>>,
    edges: Vec<Edge<()>>,
}

impl<N: fmt::Show> fmt::Show for OGraph<N>
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

impl<N> OGraph<N>
//where N: fmt::Show
{
    pub fn new() -> OGraph<N>
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

    pub fn neighbors(&self, a: NodeIndex) -> Neighbors<N>
    {
        Neighbors{
            graph: self,
            next: match self.nodes.get(a.0) {
                None => EdgeEnd,
                Some(n) => n.next[0],
            }
        }
    }

    pub fn add_edge(&mut self, a: NodeIndex, b: NodeIndex) -> EdgeIndex
    {
        let edge_idx = EdgeIndex(self.edges.len());
        match index_twice(self.nodes[mut], a.0, b.0) {
            Pair::None => panic!("NodeIndices out of bounds"),
            Pair::One(an) => {
                let edge = Edge {
                    data: (),
                    a: a,
                    b: b,
                    next: an.next,
                };
                an.next[0] = edge_idx;
                an.next[1] = edge_idx;
                self.edges.push(edge);
            }
            Pair::Both(an, bn) => {
                // a and b are different indices
                let edge = Edge {
                    data: (),
                    a: a,
                    b: b,
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
        // Remove all edges from and to this node.
        loop {
            let next = self.nodes[a.0].next[0];
            if next == EdgeEnd {
                break
            }
            //println!("Rmove edge {}", next);
            self.remove_edge(next);
        }

        loop {
            let next = self.nodes[a.0].next[1];
            if next == EdgeEnd {
                break
            }
            //println!("Rmove edge {}", next);
            self.remove_edge(next);
        }

        //println!("REMOVED EDGES: {}", self);

        // Adjust all node indices affected
        for edge in self.edges.iter_mut() {
            debug_assert!(edge.a != a);
            debug_assert!(edge.b != a);
            if edge.a > a {
                edge.a.0 -= 1;
            }
            if edge.b > a {
                edge.b.0 -= 1;
            }
        }
        self.nodes.remove(a.0).map(|n|n.data)
    }

    pub fn edge_mut(&mut self, e: EdgeIndex) -> &mut Edge<()>
    {
        &mut self.edges[e.0]
    }

    pub fn remove_edge(&mut self, e: EdgeIndex)
    {
        fn update_edge_list(replace: EdgeIndex, fst: EdgeIndex, edges: &mut [Edge<()>], d: Dir) {
            debug_assert!(fst != replace);
            let k = d as uint;
            let edge_next = edges[replace.0].next[k];
            // walk through edge list
            let mut cur = fst;
            while cur != EdgeEnd {
                let curedge = &mut edges[cur.0];
                if curedge.next[k] == replace {
                    //println!("Have to replace link in {}", curedge);
                    curedge.next[k] = edge_next;
                    break
                } else {
                    cur = curedge.next[k];
                }
            }
        }
        // every edge is part of two lists,
        // outgoing and incoming edges.
        // Remove it from both
        let (edge_a, edge_b, edge_next) = match self.edges.get(e.0) {
            None => return,
            Some(x) => (x.a, x.b, x.next),
        };
        {
            // List out from A
            let node = &mut self.nodes[edge_a.0];
            let fst = node.next[0];
            if fst == e {
                node.next[0] = edge_next[0];
            } else {
                update_edge_list(e, fst, self.edges[mut], Dir::Out);
            }
        }
        {
            // List in to B
            let node = &mut self.nodes[edge_b.0];
            let fst = node.next[1];
            if fst == e {
                node.next[1] = edge_next[1];
            } else {
                update_edge_list(e, fst, self.edges[mut], Dir::In);
            }
        }
        self.remove_edge_adjust_indices(e);
    }

    fn remove_edge_adjust_indices(&mut self, e: EdgeIndex) -> Option<()>
    {
        // swap_remove the edge -- only the removed edge
        // and the edge swapped into place are affected and need updating
        // indices.
        let edge = self.edges.swap_remove(e.0).unwrap();
        let (swap_a, swap_b) = match self.edges.get(e.0) {
            // no elment needed to be swapped.
            None => return Some(edge.data),
            Some(ed) => (ed.a, ed.b),
        };
        let swapped_e = EdgeIndex(self.edges.len());

        fn update_edge_list(replace: EdgeIndex, fst: EdgeIndex, new: EdgeIndex, edges: &mut [Edge<()>], d: Dir) {
            debug_assert!(fst != replace);
            let k = d as uint;
            // walk through edge list
            let mut cur = fst;
            while cur != EdgeEnd {
                let curedge = &mut edges[cur.0];
                if curedge.next[k] == replace {
                    //println!("Have to replace link in {}", curedge);
                    curedge.next[k] = new;
                    break
                } else {
                    cur = curedge.next[k];
                }
            }
        }
        {
            // List out from A
            let node = &mut self.nodes[swap_a.0];
            let fst = node.next[0];
            if fst == swapped_e {
                node.next[0] = e;
            } else {
                update_edge_list(swapped_e, fst, e, self.edges[mut], Dir::Out);
            }
        }
        {
            // List in to B
            let node = &mut self.nodes[swap_b.0];
            let fst = node.next[1];
            if fst == swapped_e {
                node.next[1] = e;
            } else {
                update_edge_list(swapped_e, fst, e, self.edges[mut], Dir::In);
            }
        }
        // All that refer to the swapped edge need to update.
        //
        /*
        // Edge lists are fine, so remove the edge and adjust all edge indices in nodes
        let edge_data = self.edges.remove(e.0).unwrap().data;
        for &k in [0u, 1].iter() {
            for node in self.nodes.iter_mut() {
                debug_assert!(node.next[k] != e);
                if node.next[k] != EdgeEnd && node.next[k] > e {
                    node.next[k].0 -= 1;
                }
            }
            for edge in self.edges.iter_mut() {
                debug_assert!(edge.next[k] != e);
                if edge.next[k] != EdgeEnd && edge.next[k] > e {
                    edge.next[k].0 -= 1;
                }
            }
        }
        */
        let edge_data = edge.data;
        Some(edge_data)
    }
}

pub struct Neighbors<'a, N: 'a> {
    graph: &'a OGraph<N>,
    next: EdgeIndex,
}

impl<'a, N> Iterator<NodeIndex> for Neighbors<'a, N>
{
    fn next(&mut self) -> Option<NodeIndex>
    {
        match self.graph.edges.get(self.next.0) {
            None => None,
            Some(edge) => {
                self.next = edge.next[0];
                Some(edge.b)
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
        og.add_edge(fst, n);
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
        og.add_edge(prev, n);
        prev = n;
    }
    //println!("{}", og);
    b.iter(|| {
        for _ in range(0, 100i) {
            og.remove_node(fst);
        }
    })
}
