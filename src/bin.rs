#![feature(default_type_params)]
extern crate arena;

use std::default::Default;
use arena::TypedArena;
use std::cell::Cell;
use std::hash::{Writer, Hash};
use std::kinds;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BinaryHeap;
use std::iter::Map;
use std::collections::hash_map::{
    Keys,
    Occupied,
    Vacant,
};
use std::slice::{
    Items,
    MutItems,
};
use std::fmt;

/// A reference that is hashed and compared by its raw pointer value.
#[deriving(Clone)]
pub struct Ptr<'b, T: 'b>(&'b T);

impl<'b, T> Copy for Ptr<'b, T> {}

fn ptreq<T>(a: &T, b: &T) -> bool {
    a as *const _ == b as *const _
}

impl<'b, T> PartialEq for Ptr<'b, T>
{
    fn eq(&self, other: &Ptr<'b, T>) -> bool {
        ptreq(self.0, other.0)
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

/// **DiGraph** is a *Directed Graph*
pub struct DiGraph<N: Eq + Hash, E> {
    nodes: HashMap<N, Vec<(N, E)>>,
}

impl<N: Eq + Hash + fmt::Show, E: fmt::Show> fmt::Show for DiGraph<N, E>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.nodes.fmt(f)
    }
}

pub type Nodes<'a, N, E> = Keys<'a, N, Vec<(N, E)>>;

impl<N: Copy + Eq + Hash, E> DiGraph<N, E>
{
    pub fn new() -> DiGraph<N, E>
    {
        DiGraph {
            nodes: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: N) -> N {
        self.nodes.insert(node, Vec::new());
        node
    }

    pub fn remove_node(&mut self, node: N) {
        match self.nodes.remove(&node) {
            None => {}
            Some(..) => {
                for (_, edges) in self.nodes.iter_mut() {
                    match edges.iter().position(|&(elt, _)| elt == node) {
                        Some(index) => { edges.remove(index); }
                        None => {}
                    }
                }
            }
        }
    }

    /// Add directed edge from `a` to `b`.
    pub fn add_edge(&mut self, a: N, b: N, edge: E) -> bool
    {
        let mut did_insert = false;
        // We need both lookups anyway to assert sanity, so
        // add nodes if they don't already exist
        match self.nodes.entry(a) {
            Occupied(ent) => {
                // Add edge only if it isn't already there
                let edges = ent.into_mut();
                if edges.iter().position(|&(elt, _)| elt == b).is_none() {
                    edges.push((b, edge));
                    did_insert = true;
                }
            }
            Vacant(ent) => {
                ent.set(vec![(b, edge)]);
                did_insert = true;
            }
        }
        // make sure both endpoints exist in the map
        match self.nodes.entry(b) {
            Vacant(ent) => { ent.set(Vec::new()); }
            _ => {}
        }
        did_insert
    }

    /// Remove edge from `a` to `b`.
    ///
    /// Return None if the edge didn't exist.
    pub fn remove_edge(&mut self, a: N, b: N) -> Option<E>
    {
        match self.nodes.entry(a) {
            Occupied(mut ent) => {
                match ent.get().iter().position(|&(elt, _)| elt == b) {
                    Some(index) => {
                        ent.get_mut().remove(index).map(|(_, edge)| edge)
                    }
                    None => None,
                }
            }
            Vacant(..) => None,
        }
    }

    pub fn nodes<'a>(&'a self) -> Nodes<'a, N, E>
    {
        self.nodes.keys()
    }

    pub fn edges<'a>(&'a self, n: N) -> Items<'a, (N, E)>
    {
        self.nodes[n].iter()
    }

    pub fn edges_mut<'a>(&'a mut self, n: N) -> MutItems<'a, (N, E)>
    {
        self.nodes[n].iter_mut()
    }
}

impl<N: Copy + Eq + Hash, E: Clone> DiGraph<N, E>
{
    /// Add edge from `a` to `b`.
    pub fn add_diedge(&mut self, a: N, b: N, edge: E) -> bool
    {
        self.add_edge(a, b, edge.clone()) |
        self.add_edge(b, a, edge)
    }

    /// Return a reverse graph.
    pub fn reverse(&self) -> DiGraph<N, E>
    {
        let mut g = DiGraph::new();
        for &node in self.nodes() {
            for &(other, ref edge) in self.edges(node) {
                g.add_edge(other, node, edge.clone());
            }
        }
        g
    }
}

/// **Graph**
pub struct Graph<N: Eq + Hash, E> {
    gr: DiGraph<N, Option<E>>
}

impl<N: Eq + Hash + fmt::Show, E: fmt::Show> fmt::Show for Graph<N, E>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.gr.fmt(f)
    }
}

//pub type Nodes<'a, N, E> = Keys<'a, N, Vec<(N, E)>>;

impl<N: Copy + Eq + Hash, E> Graph<N, E>
{
    pub fn new() -> Graph<N, E>
    {
        Graph {
            gr: DiGraph::new()
        }
    }

    pub fn add_node(&mut self, node: N) -> N {
        self.gr.add_node(node)
    }

    pub fn remove_node(&mut self, node: N) {
        // FIXME: Use digraph property
        match self.gr.nodes.remove(&node) {
            None => {}
            Some(..) => {
                for (_, edges) in self.gr.nodes.iter_mut() {
                    match edges.iter().position(|&(elt, _)| elt == node) {
                        Some(index) => { edges.remove(index); }
                        None => {}
                    }
                }
            }
        }
    }

    /// Add an edge connecting `a` and `b`.
    pub fn add_edge(&mut self, a: N, b: N, edge: E) -> bool
    {
        // Only one edge can own the edge data --
        // we arbitrarily pick the first here to own it.
        let insert = |a: N, b: N, edge: Option<E>| -> bool {
            match self.gr.nodes.entry(a) {
                Occupied(ent) => {
                    // Add edge only if it isn't already there
                    let edges = ent.into_mut();
                    if edges.iter().position(|&(elt, _)| elt == b).is_none() {
                        edges.push((b, edge));
                        true
                    } else {
                        false
                    }
                }
                Vacant(ent) => {
                    ent.set(vec![(b, edge)]);
                    true
                }
            }
        };
        insert(a, b, Some(edge)) |
        insert(b, a, None)
    }

    /// Remove edge from `a` to `b`.
    ///
    /// Return None if the edge didn't exist.
    pub fn remove_edge(&mut self, a: N, b: N) -> Option<E>
    {
        /*
        match self.nodes.entry(a) {
            Occupied(mut ent) => {
                match ent.get().iter().position(|&(elt, _)| elt == b) {
                    Some(index) => {
                        ent.get_mut().remove(index).map(|(_, edge)| edge)
                    }
                    None => None,
                }
            }
            Vacant(..) => None,
        }
        */
        panic!()
    }

    pub fn nodes<'a>(&'a self) -> Nodes<'a, N, Option<E>>
    {
        self.gr.nodes.keys()
    }

    pub fn edges<'a>(&'a self, n: N) -> Edges<'a, N, E>
    {
        Edges {
            from: n,
            graph: self,
            iter: self.gr.nodes[n].iter()
        }
    }
    /*
        // Here we need to patch in edges
        // one of (a, b, Some(E))
        //        (b, a, None)
        //
        // will have the edge.
        // Also: Unwrap :-(
        fn map_edge<N: Copy + Eq + Hash, E>(elt: &(N, Option<E>)) -> (N, &E)
        {
            let &(node, ref edge_opt) = elt;
            let edge_ref = match *edge_opt {
                Some(ref xyz) => xyz,
                None => {
                    // Find the edge from the reverse
                    panic!()
                }
            };
            (node, edge_ref)
        }
        self.gr.nodes[n].iter().map(map_edge)
    }
    */

    pub fn edges_mut<'a>(&'a mut self, n: N) -> MutItems<'a, (N, E)>
    {
        panic!()
        //self.nodes[n].iter_mut()
    }
}

pub struct Edges<'a, N: 'a + Copy + Eq + Hash, E: 'a> {
    from: N,
    graph: &'a Graph<N, E>,
    iter: Items<'a, (N, Option<E>)>,
}

impl<'a, N: 'a + Copy + Eq + Hash, E: 'a> Iterator<(N, &'a E)> for Edges<'a, N, E>
{
    fn next(&mut self) -> Option<(N, &'a E)>
    {
        match self.iter.next() {
            None => None,
            Some(&(node, ref edge_opt)) => {
                match *edge_opt {
                    Some(ref edge) => Some((node, edge)),
                    None => {
                        // dig out the other reference
                        for &(n, ref edge_opt) in self.graph.gr.edges(node) {
                            if n == self.from {
                                let edge_ref = edge_opt.as_ref().unwrap();
                                return Some((node, edge_ref));
                            }
                        }
                        unreachable!()
                    }
                }
            }
        }
    }
}

/// Hold a score and a scored object in a pair for use with a BinaryHeap.
///
/// MinScore compares in reverse order compared to the score, so that we can
/// use BinaryHeap as a "min-heap" to extract the score, value pair with the
/// lowest score.
///
/// NOTE: MinScore implements a total order (Eq + Ord), so that it is possible
/// to use float types as scores.
pub struct MinScore<K, T>(pub K, pub T);

impl<K: PartialEq, T> PartialEq for MinScore<K, T> {
    #[inline]
    fn eq(&self, other: &MinScore<K, T>) -> bool {
        self.0 == other.0
    }
}

impl<K: PartialEq, T> Eq for MinScore<K, T> {}

impl<K: PartialOrd, T> PartialOrd for MinScore<K, T> {
    #[inline]
    fn partial_cmp(&self, other: &MinScore<K, T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K: PartialOrd + PartialEq, T> Ord for MinScore<K, T> {
    #[inline]
    fn cmp(&self, other: &MinScore<K, T>) -> Ordering {
        // Order NaN first, (NaN is equal to itself and largest)
        let selfnan = self.0 != self.0;
        let othernan = other.0 != other.0;
        if selfnan && othernan {
            Equal
        } else if selfnan {
            Less
        } else if othernan {
            Greater
        // Then order in reverse order
        } else if self.0 < other.0 {
            Greater
        } else if self.0 > other.0 {
            Less
        } else {
            Equal
        }
    }
}

impl<N: Copy + Eq + Hash,
     K: Default + Add<K, K> + Copy + PartialOrd> DiGraph<N, K>
{
    pub fn dijkstra(&self, start: N) -> Vec<(N, K)> {
        let mut visited = HashSet::new();
        let mut scores = HashMap::new();
        let mut visit_next = BinaryHeap::new();
        let zero_score: K = Default::default();
        scores.insert(start, zero_score);
        visit_next.push(MinScore(zero_score, start));
        loop {
            let MinScore(node_score, node) = match visit_next.pop() {
                None => break,
                Some(t) => t,
            };
            if visited.contains(&node) {
                continue
            }
            for &(next, edge) in self.edges(node) {
                if visited.contains(&next) {
                    continue
                }
                let mut next_score = node_score + edge;
                match scores.entry(next) {
                    Occupied(ent) => if next_score < *ent.get() {
                        *ent.into_mut() = next_score
                    } else {
                        next_score = *ent.get();
                    },
                    Vacant(ent) => { ent.set(next_score); }
                }
                visit_next.push(MinScore(next_score, next));
            }
            visited.insert(node);
        }
        scores.into_iter().collect()
    }
}

#[deriving(Show)]
struct Node<T>(pub T);

pub struct NodeCell<T: Copy>(pub Cell<T>);

impl<'a, T: Copy> Deref<Cell<T>> for NodeCell<T> {
    #[inline]
    fn deref<'a>(&'a self) -> &'a Cell<T> {
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
    g.add_diedge(a, c, 14.);
    g.add_diedge(b, c, 10.);
    g.add_diedge(c, d, 2.);
    g.add_diedge(d, e, 9.);
    g.add_diedge(b, f, 15.);
    g.add_diedge(c, f, 11.);
    g.add_diedge(e, f, 6.);
    println!("{}", g.nodes);

    f.set("F'");

    println!("Scores= {}", 
        g.dijkstra(a)
    );

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
    g.add_diedge(a, c, 14.);
    g.add_diedge(b, c, 10.);
    g.add_diedge(c, d, 2.);
    g.add_diedge(d, e, 9.);
    g.add_diedge(b, f, 15.);
    g.add_diedge(c, f, 11.);
    g.add_diedge(e, f, 6.);
    println!("{}", g);

    let root = TypedArena::<Node<_>>::new();
    let mut g: DiGraph<_, f32> = DiGraph::new();
    let node = |name: &'static str| Ptr(root.alloc(Node(name.to_string())));
    let a = g.add_node(node("A"));
    let b = g.add_node(node("B"));
    let c = g.add_node(node("C"));
    let d = g.add_node(node("D"));
    let e = g.add_node(node("E"));
    let f = g.add_node(node("F"));
    g.add_diedge(a, b, 7.);
    g.add_diedge(a, c, 9.);
    g.add_diedge(a, d, 14.);
    g.add_diedge(a, c, 14.);
    g.add_diedge(b, c, 10.);
    g.add_diedge(c, d, 2.);
    g.add_diedge(d, e, 9.);
    g.add_diedge(b, f, 15.);
    g.add_diedge(c, f, 11.);
    g.add_diedge(e, f, 6.);
    println!("{}", g);

    let mut g: Graph<int, int> = Graph::new();
    g.add_node(1);
    g.add_node(2);
    g.add_edge(1, 2, -1);

    println!("{}", g);
    for elt in g.edges(2) {
        println!("Edge => {}", elt);
    }
}


fn main() {
    make_graph();
}
