use std::hash::{Hash};
use std::slice::{
    Items,
};
use std::fmt;

#[deriving(Copy, Clone, Show, PartialEq, PartialOrd)]
pub struct NodeIndex(uint);
#[deriving(Copy, Clone, Show, PartialEq, PartialOrd)]
pub struct EdgeIndex(uint);

const InvalidEdge: EdgeIndex = EdgeIndex(::std::uint::MAX);
const InvalidNode: NodeIndex = NodeIndex(::std::uint::MAX);

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

/// **OGraph\<N, E\>** is a graph.
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
where N: fmt::Show
{
    pub fn new() -> OGraph<N>
    {
        OGraph{nodes: Vec::new(), edges: Vec::new()}
    }

    pub fn add_node(&mut self, data: N) -> NodeIndex
    {
        let node = Node{data: data, next: [InvalidEdge, InvalidEdge]};
        let node_idx = NodeIndex(self.nodes.len());
        self.nodes.push(node);
        node_idx
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
        let remove_node = match self.nodes.remove(a.0) {
            None => return None,
            Some(n) => n,
        };

        // Adjust all node indices affected
        // Mark edges to be removed with InvalidNode links
        for edge in self.edges.iter_mut() {
            if edge.a == a {
                edge.a = InvalidNode;
            } else if edge.a > a {
                edge.a = NodeIndex(edge.a.0 - 1);
            }
            if edge.b == a {
                edge.b = InvalidNode;
            } else if edge.b > a {
                edge.b = NodeIndex(edge.b.0 - 1);
            }
        }

        // Rewrite edge chains to skip edges to be removed
        for node in self.nodes.iter_mut() {
            let mut fst = node.next[0];
            loop {
                println!("Examining {} for node {}", fst, node);
                match self.edges.get_mut(fst.0) {
                    None => break,
                    Some(edge) => {
                        if edge.a == InvalidNode || edge.b == InvalidNode {
                            println!("Edge to SKIP: {}", edge);
                        }
                        fst = edge.next[0];
                    }
                }
            }

            // "in" chain
            let mut fst = node.next[1];
            loop {
                println!("Examining {} for node {}", fst, node);
                match self.edges.get_mut(fst.0) {
                    None => break,
                    Some(edge) => {
                        if edge.a == InvalidNode || edge.b == InvalidNode {
                            println!("Edge to SKIP: {}", edge);
                        }
                        fst = edge.next[1];
                    }
                }
            }

        }

        Some(remove_node.data)
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
            while cur != InvalidEdge {
                let curedge = &mut edges[cur.0];
                if curedge.next[k] == replace {
                    println!("Have to replace link in {}", curedge);
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

        // Edge lists are fine, so remove the edge and adjust all edge indices
        let edge_data = self.edges.remove(e.0).unwrap().data;
        for node in self.nodes.iter_mut() {
            for &k in [0u, 1].iter() {
                if node.next[k] != InvalidEdge && node.next[k] > e {
                    node.next[k].0 -= 1;
                }
            }
        }
        for edge in self.edges.iter_mut() {
            for &k in [0u, 1].iter() {
                if edge.next[k] != InvalidEdge && edge.next[k] > e {
                    edge.next[k].0 -= 1;
                }
            }
        }
        //Some(edge_data)
    }
}
