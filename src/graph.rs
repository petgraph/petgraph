use std::hash::{Hash};
use std::collections::HashMap;
use std::collections::hash_map::Hasher;
use std::iter::Map;
use std::collections::hash_map::{
    Keys,
};
use std::collections::hash_map::Entry::{
    Occupied,
    Vacant,
};
use std::slice::{
    Iter,
};
use std::fmt;

/// **Graph\<N, E\>** is a regular graph, with generic node values **N** and edge weights **E**.
///
/// It uses an adjacency list representation, i.e. using *O(|N| + |E|)* space.
///
/// The node type must be a simple copyable type and implement **Copy**.
///
/// The node type must be suitable as a hash table key (implementing **Eq + Hash**)
/// as well as being a simple type.
///
/// The node type must implement **PartialOrd** so that the implementation can
/// properly order the pair (**a**, **b**) for an edge connecting any two nodes **a** and **b**.
#[derive(Clone)]
pub struct Graph<N: Eq + Hash<Hasher>, E> {
    nodes: HashMap<N, Vec<N>>,
    edges: HashMap<(N, N), E>,
}

impl<N: Eq + Hash<Hasher> + fmt::Show, E: fmt::Show> fmt::Show for Graph<N, E>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.nodes.fmt(f)
    }
}

#[inline]
fn edge_key<N: Copy + PartialOrd>(a: N, b: N) -> (N, N)
{
    if a <= b { (a, b) } else { (b, a) }
}

#[inline]
fn copy<N: Copy>(n: &N) -> N { *n }

impl<N, E> Graph<N, E> where N: Copy + Clone + PartialOrd + Eq + Hash<Hasher>
{
    /// Create a new **Graph**.
    pub fn new() -> Graph<N, E>
    {
        Graph {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    pub fn node_count(&self) -> usize
    {
        self.nodes.len()
    }

    /// Add node **n** to the graph.
    pub fn add_node(&mut self, n: N) -> N {
        match self.nodes.entry(n) {
            Occupied(_) => {}
            Vacant(ent) => { ent.insert(Vec::new()); }
        }
        n
    }

    /// Return **true** if node **n** was removed.
    pub fn remove_node(&mut self, n: N) -> bool {
        let successors = match self.nodes.remove(&n) {
            None => return false,
            Some(sus) => sus,
        };
        for succ in successors.into_iter() {
            // remove all successor links
            self.remove_single_edge(&succ, &n);
            // Remove all edge values
            self.edges.remove(&edge_key(n, succ));
        }
        true
    }

    /// Return **true** if the node is contained in the graph.
    pub fn contains_node(&self, n: N) -> bool {
        self.nodes.contains_key(&n)
    }

    /// Add an edge connecting **a** and **b** to the graph.
    ///
    /// Return **true** if edge did not previously exist.
    pub fn add_edge(&mut self, a: N, b: N, edge: E) -> bool
    {
        // Use PartialOrd to order the edges
        match self.nodes.entry(a) {
            Occupied(ent) => { ent.into_mut().push(b); }
            Vacant(ent) => { ent.insert(vec![b]); }
        }
        match self.nodes.entry(b) {
            Occupied(ent) => { ent.into_mut().push(a); }
            Vacant(ent) => { ent.insert(vec![a]); }
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

    /// Remove edge from **a** to **b** from the graph.
    ///
    /// Return **None** if the edge didn't exist.
    pub fn remove_edge(&mut self, a: N, b: N) -> Option<E>
    {
        self.remove_single_edge(&a, &b);
        self.remove_single_edge(&b, &a);
        self.edges.remove(&edge_key(a, b))
    }

    /// Return **true** if the edge connecting **a** with **b** is contained in the graph.
    pub fn contains_edge(&self, a: N, b: N) -> bool {
        self.edges.contains_key(&edge_key(a, b))
    }

    /// Return an iterator over the nodes of the graph.
    ///
    /// Iterator element type is **&'a N**.
    pub fn nodes<'a>(&'a self) -> Nodes<'a, N>
    {
        Nodes{iter: self.nodes.keys()}
    }

    /// Return an iterator over the nodes that are connected with **from** by edges.
    ///
    /// If the node **from** does not exist in the graph, return an empty iterator.
    ///
    /// Iterator element type is **N**.
    pub fn neighbors<'a>(&'a self, from: N) -> Neighbors<'a, N>
    {
        Neighbors{iter:
            match self.nodes.get(&from) {
                Some(neigh) => neigh.iter(),
                None => [].iter(),
            }.map(copy as fn(&N) -> N)
        }
    }

    /// Return an iterator over the nodes that are connected with **from** by edges,
    /// paired with the edge weight.
    ///
    /// If the node **from** does not exist in the graph, return an empty iterator.
    ///
    /// Iterator element type is **(N, &'a E)**.
    pub fn edges<'a>(&'a self, from: N) -> Edges<'a, N, E>
    {
        Edges {
            from: from,
            iter: self.neighbors(from),
            edges: &self.edges,
        }
    }

    /// Return a reference to the edge weight connecting **a** with **b**, or
    /// **None** if the edge does not exist in the graph.
    pub fn edge<'a>(&'a self, a: N, b: N) -> Option<&'a E>
    {
        self.edges.get(&edge_key(a, b))
    }

    /// Return a mutable reference to the edge weight connecting **a** with **b**, or
    /// **None** if the edge does not exist in the graph.
    pub fn edge_mut<'a>(&'a mut self, a: N, b: N) -> Option<&'a mut E>
    {
        self.edges.get_mut(&edge_key(a, b))
    }
}

macro_rules! iterator_methods {
    ($elt_type:ty) => (
        #[inline]
        fn next(&mut self) -> Option<$elt_type>
        {
            self.iter.next()
        }

        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>)
        {
            self.iter.size_hint()
        }
    )
}

pub struct Nodes<'a, N: 'a> {
    iter: Keys<'a, N, Vec<N>>
}

impl<'a, N: 'a> Iterator for Nodes<'a, N>
{
    type Item = &'a N;
    iterator_methods!(&'a N);
}

pub struct Neighbors<'a, N: 'a> {
    iter: Map<&'a N, N, Iter<'a, N>, fn(&N) -> N>,
}

impl<'a, N> Iterator for Neighbors<'a, N>
{
    type Item = N;
    iterator_methods!(N);
}

pub struct Edges<'a, N: 'a + Copy + PartialOrd + Eq + Hash<Hasher>, E: 'a> {
    pub from: N,
    pub edges: &'a HashMap<(N, N), E>,
    pub iter: Neighbors<'a, N>,
}

impl<'a, N, E> Iterator for Edges<'a, N, E>
    where N: 'a + Copy + PartialOrd + Eq + Hash<Hasher>, E: 'a
{
    type Item = (N, &'a E);
    fn next(&mut self) -> Option<(N, &'a E)>
    {
        match self.iter.next() {
            None => None,
            Some(b) => {
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
