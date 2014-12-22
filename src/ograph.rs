use std::hash::{Hash};
use std::slice::{
    Items,
};
use std::fmt;

#[deriving(Copy, Clone, Show, PartialEq, PartialOrd)]
pub struct NodeIndex(uint);
#[deriving(Copy, Clone, Show, PartialEq)]
pub struct EdgeIndex(uint);

const InvalidEdge: EdgeIndex = EdgeIndex(::std::uint::MAX);
const InvalidNode: NodeIndex = NodeIndex(::std::uint::MAX);


#[deriving(Show)]
pub struct Node<N> {
    pub data: N,
    first_out: EdgeIndex,
    first_in: EdgeIndex,
}

#[deriving(Show)]
pub struct Edge<E> {
    pub data: E,
    next_out: EdgeIndex,
    next_in: EdgeIndex,
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
        let node = Node{data: data, first_out: InvalidEdge, first_in: InvalidEdge};
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
                    next_out: an.first_out,
                    next_in: an.first_in,
                };
                an.first_out = edge_idx;
                an.first_in = edge_idx;
                self.edges.push(edge);
            }
            Pair::Both(an, bn) => {
                // a and b are different indices
                let edge = Edge {
                    data: (),
                    a: a,
                    b: b,
                    next_out: an.first_out,
                    next_in: bn.first_in,
                };
                an.first_out = edge_idx;
                bn.first_in = edge_idx;
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
            let mut fst = node.first_out;
            loop {
                println!("Examining {} for node {}", fst, node);
                match self.edges.get_mut(fst.0) {
                    None => break,
                    Some(edge) => {
                        if edge.a == InvalidNode || edge.b == InvalidNode {
                            println!("Edge to SKIP: {}", edge);
                        }
                        fst = edge.next_out;
                    }
                }
            }

            // "in" chain
            let mut fst = node.first_in;
            loop {
                println!("Examining {} for node {}", fst, node);
                match self.edges.get_mut(fst.0) {
                    None => break,
                    Some(edge) => {
                        if edge.a == InvalidNode || edge.b == InvalidNode {
                            println!("Edge to SKIP: {}", edge);
                        }
                        fst = edge.next_in;
                    }
                }
            }

        }

        Some(remove_node.data)
    }

    pub fn remove_edge(&mut self, e: EdgeIndex)
    {
        let edge = match self.edges.remove(e.0) {
            None => return,
            Some(x) => x,
        };
    }
}
