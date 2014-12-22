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
};
use std::fmt;

/// **Graph** is a regular graph, with node values and edge weights.
///
/// It uses an adjacency list representation, i.e. using *O(|N| + |E|)* space.
///
/// The node type must be suitable as a hash table key (Implementing **Eq + Hash**)
/// as well as being a simple type.
///
/// The node type must implement **PartialOrd** so that the implementation can
/// properly order the pair (**a**, **b**) for an edge connecting any two nodes **a** and **b**.
#[deriving(Show)]
pub struct Graph<N: Eq + Hash, E> {
    nodes: HashMap<N, Vec<N>>,
    edges: HashMap<(N, N), E>,
}

/*
impl<N: Eq + Hash + fmt::Show, E: fmt::Show> fmt::Show for Graph<N, E>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.gr.fmt(f)
    }
}
*/
#[inline]
fn edge_key<N: Copy + PartialOrd>(a: N, b: N) -> (N, N)
{
    if a <= b { (a, b) } else { (b, a) }
}

impl<N: Copy + PartialOrd + Eq + Hash, E> Graph<N, E>
{
    pub fn new() -> Graph<N, E>
    {
        Graph {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: N) -> N {
        match self.nodes.entry(node) {
            Occupied(_) => {}
            Vacant(ent) => { ent.set(Vec::new()); }
        }
        node
    }

    /// Return **true** if node was removed.
    pub fn remove_node(&mut self, node: N) -> bool {
        // remove node
        let successors = match self.nodes.remove(&node) {
            None => return false,
            Some(sus) => sus,
        };
        for succ in successors.into_iter() {
            // remove all successor links
            self.remove_single_edge(&succ, &node);
            // Remove all edge values
            self.edges.remove(&edge_key(node, succ));
        }
        true
    }

    pub fn contains_node(&self, node: N) -> bool {
        self.nodes.contains_key(&node)
    }

    /// Add an edge connecting **a** and **b**.
    ///
    /// Return **true** if edge was new
    pub fn add_edge(&mut self, a: N, b: N, edge: E) -> bool
    {
        // Use PartialOrd to order the edges
        match self.nodes.entry(a) {
            Occupied(ent) => { ent.into_mut().push(b); }
            Vacant(ent) => { ent.set(vec![b]); }
        }
        match self.nodes.entry(b) {
            Occupied(ent) => { ent.into_mut().push(a); }
            Vacant(ent) => { ent.set(vec![a]); }
        }
        self.edges.insert(edge_key(a, b), edge).is_none()
    }

    /// Remove successor relation from a to b
    fn remove_single_edge(&mut self, a: &N, b: &N) {
        match self.nodes.get_mut(a) {
            None => {}
            Some(sus) => {
                match sus.iter().position(|elt| elt == b) {
                    Some(index) => { sus.swap_remove(index); }
                    None => {}
                }
            }
        }
    }

    /// Remove edge from **a** to **b**.
    ///
    /// Return **None** if the edge didn't exist.
    pub fn remove_edge(&mut self, a: N, b: N) -> Option<E>
    {
        self.remove_single_edge(&a, &b);
        self.remove_single_edge(&b, &a);
        self.edges.remove(&edge_key(a, b))
    }

    pub fn contains_edge(&self, a: N, b: N) -> bool {
        self.edges.contains_key(&edge_key(a, b))
    }

    pub fn nodes<'a>(&'a self) -> Nodes<'a, N>
    {
        Nodes{iter: self.nodes.keys()}
    }

    /// If the node **from** does not exist in the graph, return an empty iterator.
    pub fn neighbors<'a>(&'a self, from: N) -> Neighbors<'a, N>
    {
        Neighbors{iter:
            match self.nodes.get(&from) {
                Some(neigh) => neigh.iter(),
                None => [].iter(),
            }
        }
    }

    /// If the node **from** does not exist in the graph, return an empty iterator.
    pub fn edges<'a>(&'a self, from: N) -> Edges<'a, N, E>
    {
        Edges {
            from: from,
            iter: self.neighbors(from),
            edges: &self.edges,
        }
    }

    pub fn edge_mut<'a>(&'a mut self, a: N, b: N) -> Option<&'a mut E>
    {
        self.edges.get_mut(&edge_key(a, b))
    }
}

macro_rules! iterator_methods(
    ($elt_type:ty) => (
        #[inline]
        fn next(&mut self) -> Option<$elt_type>
        {
            self.iter.next()
        }

        #[inline]
        fn size_hint(&self) -> (uint, Option<uint>)
        {
            self.iter.size_hint()
        }
    )
)

pub struct Nodes<'a, N: 'a> {
    iter: Keys<'a, N, Vec<N>>
}

impl<'a, N: 'a> Iterator<&'a N> for Nodes<'a, N>
{
    iterator_methods!(&'a N)
}

pub struct Neighbors<'a, N: 'a> {
    iter: Items<'a, N>
}

impl<'a, N: 'a> Iterator<&'a N> for Neighbors<'a, N>
{
    iterator_methods!(&'a N)
}

pub struct Edges<'a, N: 'a + Copy + PartialOrd + Eq + Hash, E: 'a> {
    pub from: N,
    pub edges: &'a HashMap<(N, N), E>,
    pub iter: Neighbors<'a, N>,
}

impl<'a, N, E> Iterator<(N, &'a E)> for Edges<'a, N, E>
    where N: 'a + Copy + PartialOrd + Eq + Hash, E: 'a
{
    fn next(&mut self) -> Option<(N, &'a E)>
    {
        match self.iter.next() {
            None => None,
            Some(&b) => {
                let a = self.from;
                match self.edges.get(&edge_key(a, b)) {
                    None => unreachable!(),
                    Some(edge) => {
                        Some((b, edge))
                    }
                }
            }
        }
    }
}

