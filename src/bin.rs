#![feature(macro_rules)]
#![feature(default_type_params)]
extern crate arena;

use std::default::Default;
use arena::TypedArena;
use std::cell::Cell;
use std::hash::{Writer, Hash};
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::RingBuf;
use std::collections::BinaryHeap;
use std::iter::Map;
use std::collections::hash_map::{
    Keys,
    Occupied,
    Vacant,
};
use std::fmt;

pub use scored::MinScored;
pub use digraph::DiGraph;
pub use graph::Graph;
mod scored;
pub mod digraph;
pub mod graph;


/// A reference that is hashed and compared by its pointer value.
#[deriving(Clone)]
pub struct Ptr<'b, T: 'b>(&'b T);

impl<'b, T> Copy for Ptr<'b, T> {}

fn ptreq<T>(a: &T, b: &T) -> bool {
    a as *const _ == b as *const _
}

impl<'b, T> PartialEq for Ptr<'b, T>
{
    /// Ptr compares by pointer equality, i.e if they point to the same value
    fn eq(&self, other: &Ptr<'b, T>) -> bool {
        ptreq(self.0, other.0)
    }
}

impl<'b, T> PartialOrd for Ptr<'b, T>
{
    fn partial_cmp(&self, other: &Ptr<'b, T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'b, T> Ord for Ptr<'b, T>
{
    /// Ptr is ordered by pointer value, i.e. an arbitrary but stable and total order.
    fn cmp(&self, other: &Ptr<'b, T>) -> Ordering {
        let a = self.0 as *const _;
        let b = other.0 as *const _;
        a.cmp(&b)
    }
}

impl<'b, T> Deref<T> for Ptr<'b, T> {
    fn deref<'a>(&'a self) -> &'a T {
        self.0
    }
}

impl<'b, T> Eq for Ptr<'b, T> {}

impl<'b, T, S: Writer> Hash<S> for Ptr<'b, T>
{
    fn hash(&self, st: &mut S)
    {
        let ptr = (self.0) as *const _;
        ptr.hash(st)
    }
}

impl<'b, T: fmt::Show> fmt::Show for Ptr<'b, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub fn dijkstra<'a,
                Graph, N, K,
                F, Edges>(graph: &'a Graph, start: N, mut edges: F) -> Vec<(N, K)>
where
    N: Copy + Eq + Hash + fmt::Show,
    K: Default + Add<K, K> + Copy + PartialOrd + fmt::Show,
    F: FnMut(&'a Graph, N) -> Edges,
    Edges: Iterator<(N, K)>,
{
    let mut visited = HashSet::new();
    let mut scores = HashMap::new();
    let mut predecessor = HashMap::new();
    let mut visit_next = BinaryHeap::new();
    let zero_score: K = Default::default();
    scores.insert(start, zero_score);
    visit_next.push(MinScored(zero_score, start));
    loop {
        let MinScored(node_score, node) = match visit_next.pop() {
            None => break,
            Some(t) => t,
        };
        if visited.contains(&node) {
            continue
        }
        println!("Visiting {} with score={}", node, node_score);
        for (next, edge) in edges(graph, node) {
            if visited.contains(&next) {
                continue
            }
            let mut next_score = node_score + edge;
            match scores.entry(next) {
                Occupied(ent) => if next_score < *ent.get() {
                    *ent.into_mut() = next_score;
                    predecessor.insert(next, node);
                } else {
                    next_score = *ent.get();
                },
                Vacant(ent) => {
                    ent.set(next_score);
                    predecessor.insert(next, node);
                }
            }
            visit_next.push(MinScored(next_score, next));
        }
        visited.insert(node);
    }
    println!("{}", predecessor);
    scores.into_iter().collect()
}

#[deriving(Show)]
struct Node<T>(pub T);

pub struct NodeCell<T: Copy>(pub Cell<T>);

impl<T: Copy> Deref<Cell<T>> for NodeCell<T> {
    #[inline]
    fn deref(&self) -> &Cell<T> {
        &self.0
    }
}

impl<T: Copy + fmt::Show> fmt::Show for NodeCell<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Node({})", self.0.get())
    }
}


fn make_graph() {
    let root = TypedArena::new();
    let mut g: DiGraph<_, f32> = DiGraph::new();
    let an = g.add_node(Ptr(root.alloc(Node("A"))));
    let bn = g.add_node(Ptr(root.alloc(Node("B"))));
    let cn = g.add_node(Ptr(root.alloc(Node("C"))));
    g.add_edge(an, bn, 1.);
    g.add_edge(an, cn, 2.);
    /*
    println!("{}", g.nodes);

    {
        for node in g.nodes() {
            println!("Node= {}", node);
        }
    }

    for next in g.edges(an) {
        println!("{} is a successor of {}", next, an);
    }

    g.remove_node(bn);
    println!("Removed B, {}", g.nodes);

    g.add_edge(cn, bn, 2.);
    println!("Added edge C to B, {}", g.nodes);
    g.add_edge(bn, an, 1.);
    println!("Added edge B to A, {}", g.nodes);
    g.add_edge(bn, cn, 3.);
    println!("Added edge B to C, {}", g.nodes);
    g.remove_edge(bn, an);
    println!("Removed edge B to A, {}", g.nodes);
    g.remove_edge(bn, an);
    println!("Removed edge B to A, {}", g.nodes);

    println!("Reversed, {}", g.reverse().nodes);

    */
    // Wikipedia example
    let root = TypedArena::<NodeCell<_>>::new();
    let mut g: DiGraph<_, f32> = DiGraph::new();
    let node = |name: &'static str| Ptr(root.alloc(NodeCell(Cell::new(name))));
    let a = g.add_node(node("A"));
    let b = g.add_node(node("B"));
    let c = g.add_node(node("C"));
    let d = g.add_node(node("D"));
    let e = g.add_node(node("E"));
    let f = g.add_node(node("F"));
    g.add_diedge(a, b, 7.);
    g.add_diedge(a, c, 9.);
    g.add_diedge(a, d, 14.);
    g.add_diedge(b, c, 10.);
    g.add_diedge(c, d, 2.);
    g.add_diedge(d, e, 9.);
    g.add_diedge(b, f, 15.);
    g.add_diedge(c, f, 11.);
    g.add_diedge(e, f, 6.);
    println!("{}", g);

    f.set("F'");

    println!("Scores= {}", 
        dijkstra(&g, a, |gr, n| gr.edges(n).map(|&x|x))
    );

    let mut rb = RingBuf::new();
    rb.push_back(a);
    let mut it = graph::BFT{
        graph: &g,
        stack: rb,
        visited: HashSet::new(),
        neighbors: |g, n| g.neighbors(n).map(|&x| x),
    };
    for node in it {
        println!("Visit {}", node);
    }

    let mut g: DiGraph<_, f32> = DiGraph::new();
    let node = |name: &'static str| name;
    let a = g.add_node(node("A"));
    let b = g.add_node(node("B"));
    let c = g.add_node(node("C"));
    let d = g.add_node(node("D"));
    let e = g.add_node(node("E"));
    let f = g.add_node(node("F"));
    g.add_diedge(a, b, 7.);
    g.add_diedge(a, c, 9.);
    g.add_diedge(a, d, 14.);
    g.add_diedge(b, c, 10.);
    g.add_diedge(c, d, 2.);
    g.add_diedge(d, e, 9.);
    g.add_diedge(b, f, 15.);
    g.add_diedge(c, f, 11.);
    g.add_diedge(e, f, 6.);

    println!("{}", g);

    let root = TypedArena::<Node<_>>::new();
    let mut g: Graph<_, f32> = Graph::new();
    let node = |name: &'static str| Ptr(root.alloc(Node(name.to_string())));
    let a = g.add_node(node("A"));
    let b = g.add_node(node("B"));
    let c = g.add_node(node("C"));
    let d = g.add_node(node("D"));
    let e = g.add_node(node("E"));
    let f = g.add_node(node("F"));
    g.add_edge(a, b, 7.);
    g.add_edge(a, c, 9.);
    g.add_edge(a, d, 14.);
    g.add_edge(b, c, 10.);
    g.add_edge(c, d, 2.);
    g.add_edge(d, e, 9.);
    g.add_edge(b, f, 15.);
    g.add_edge(c, f, 11.);
    g.add_edge(e, f, 6.);
    println!("{}", g);
    println!("{}", dijkstra(&g, a, |gr, n| gr.edges(n).map(|(n, &e)| (n, e))));
    for node in g.traverse_depth_first(a) {
        println!("Visit {}", node);
    }
    println!("");
    for node in g.traverse_breadth_first(a) {
        println!("Visit {}", node);
    }

    let mut g: Graph<int, int> = Graph::new();
    g.add_node(1);
    g.add_node(2);
    g.add_edge(1, 2, -1);

    println!("{}", g);
    *g.edge_mut(1, 2).unwrap() = 3;
    for elt in g.edges(1) {
        println!("Edge {} => {}", 1i, elt);
    }
    for elt in g.edges(2) {
        println!("Edge {} => {}", 2i, elt);
    }
    //g.remove_node(2);
    g.remove_edge(2, 1);
    println!("{}", g);
}


fn main() {
    make_graph();
}
