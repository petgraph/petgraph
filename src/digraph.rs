
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
    IterMut,
};
use std::fmt;

/// **DiGraph\<N, E\>** is a directed graph, with generic node values **N** and
/// edge weights **E**.
///
/// It uses an adjacency list representation, i.e. using *O(|N| + |E|)* space.
///
/// The node type must be a simple copyable type and implement **Copy**.
///
/// The node type must be suitable as a hash table key (implementing **Eq
/// + Hash**) as well as being a simple type.
///
#[derive(Clone)]
pub struct DiGraph<N, E>
{
    nodes: HashMap<N, Vec<(N, E)>>,
}

impl<N, E> fmt::Show for DiGraph<N, E> where
    N: Eq + Hash<Hasher> + fmt::Show, E: fmt::Show,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.nodes.fmt(f)
    }
}

impl<N, E> DiGraph<N, E> where N: Copy + Clone + Eq + Hash<Hasher>
{
    /// Create a new **DiGraph**.
    pub fn new() -> DiGraph<N, E>
    {
        DiGraph {
            nodes: HashMap::new(),
        }
    }

    pub fn node_count(&self) -> usize
    {
        self.nodes.len()
    }

    /// Add node **n** to the graph.
    pub fn add_node(&mut self, n: N) -> N {
        self.nodes.insert(n, Vec::new());
        n
    }

    /// Return **true** if node **n** was removed.
    pub fn remove_node(&mut self, n: N) -> bool {
        match self.nodes.remove(&n) {
            None => false,
            Some(..) => {
                for (_, edges) in self.nodes.iter_mut() {
                    match edges.iter().position(|&(elt, _)| elt == n) {
                        // Use swap_remove because order doesn't matter
                        Some(index) => { edges.swap_remove(index); }
                        None => {}
                    }
                }
                true
            }
        }
    }

    /// Return **true** if the node is contained in the graph.
    pub fn contains_node(&self, n: N) -> bool {
        self.nodes.contains_key(&n)
    }

    /// Add a directed edge from **a** to **b** to the graph.
    ///
    /// Return **true** if edge did not previously exist.
    pub fn add_edge(&mut self, a: N, b: N, edge: E) -> bool
    {
        // We need both lookups anyway to assert sanity, so
        // add nodes if they don't already exist
        //
        // make sure the endpoint exists in the map
        match self.nodes.entry(b) {
            Vacant(ent) => { ent.insert(Vec::new()); }
            _ => {}
        }

        match self.nodes.entry(a) {
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
                ent.insert(vec![(b, edge)]);
                true
            }
        }
    }

    /// Remove edge from **a** to **b** from the graph.
    ///
    /// Return **None** if the edge didn't exist.
    pub fn remove_edge(&mut self, a: N, b: N) -> Option<E>
    {
        match self.nodes.entry(a) {
            Occupied(mut ent) => {
                match ent.get().iter().position(|&(elt, _)| elt == b) {
                    Some(index) => {
                        Some(ent.get_mut().swap_remove(index).1)
                    }
                    None => None,
                }
            }
            Vacant(..) => None,
        }
    }

    /// Return **true** if the directed edge from **a** to **b** is contained in the graph.
    pub fn contains_edge(&mut self, a: N, b: N) -> bool
    {
        match self.nodes.get(&a) {
            None => false,
            Some(sus) => sus.iter().any(|&(elt, _)| elt == b),
        }
    }

    /// Return an iterator over the nodes of the graph.
    ///
    /// Iterator element type is **&'a N**.
    pub fn nodes<'a>(&'a self) -> Nodes<'a, N, E>
    {
        Nodes{iter: self.nodes.keys()}
    }

    /// Return an iterator over the nodes that are connected with **from** by edges.
    ///
    /// If the node **from** does not exist in the graph, return an empty iterator.
    ///
    /// Iterator element type is **N**.
    pub fn neighbors(&self, from: N) -> Neighbors<N, E>
    {
        fn fst<N: Copy, E>(t: &(N, E)) -> N
        {
            t.0
        }

        Neighbors{iter:
            match self.nodes.get(&from) {
                Some(edges) => edges.iter(),
                None => [].iter(),
            }.map(fst as fn(&(N, E)) -> N)
        }
    }

    /// Return an iterator over the nodes that are connected with **from** by edges,
    /// paired with the edge weight.
    ///
    /// If the node **from** does not exist in the graph, return an empty iterator.
    ///
    /// Iterator element type is **(N, &'a E)**.
    pub fn edges<'a>(&'a self, from: N) -> Edges<'a, N, E,>
    {
        fn extract<N: Copy, E>(t: &(N, E)) -> (N, &E)
        {
            let (x, ref e) = *t;
            (x, e)
        }

        Edges{iter:
            match self.nodes.get(&from) {
                Some(edges) => edges.iter(),
                None => [].iter(),
            }.map(extract as fn(&(N, E)) -> (N, &E))
        }
    }

    /// Return an iterator over the nodes that are connected with **from** by edges,
    /// paired with the edge weight.
    ///
    /// If the node **from** does not exist in the graph, return an empty iterator.
    ///
    /// Iterator element type is **&'a mut (N, E)**.
    pub fn edges_mut<'a>(&'a mut self, from: N) -> IterMut<'a, (N, E)>
    {
        match self.nodes.get_mut(&from) {
            Some(edges) => edges.iter_mut(),
            None => [].iter_mut(),
        }
    }

    /// Return a reference to the edge weight connecting **a** with **b**, or
    /// **None** if the edge does not exist in the graph.
    pub fn edge<'a>(&'a self, a: N, b: N) -> Option<&'a E>
    {
        match self.nodes.get(&a) {
            Some(succ) => {
                succ.iter()
                    .find(|&&(ref n, _)| n == &b)
                    .map(|&(_, ref edge)| edge)
            }
            None => None,
        }
    }

    /// Return a mutable reference to the edge weight connecting **a** with **b**, or
    /// **None** if the edge does not exist in the graph.
    pub fn edge_mut<'a>(&'a mut self, a: N, b: N) -> Option<&'a mut E>
    {
        match self.nodes.get_mut(&a) {
            Some(succ) => {
                succ.iter_mut()
                    .find(|&&mut (ref n, _)| n == &b)
                    .map(|&mut (_, ref mut edge)| edge)
            }
            None => None,
        }
    }

}

impl<N, E> DiGraph<N, E> where N: Copy + Clone + Eq + Hash<Hasher>, E: Clone
{
    /// Add a directed edges from **a** to **b** and from **b** to **a** to the
    /// graph.
    ///
    /// Return **true** if at least one of the edges did not previously exist.
    pub fn add_diedge(&mut self, a: N, b: N, edge: E) -> bool
    {
        self.add_edge(a, b, edge.clone()) |
        self.add_edge(b, a, edge)
    }

    /// Return a cloned graph with all edges reversed.
    pub fn reversed(&self) -> DiGraph<N, E>
    {
        let mut g = DiGraph::new();
        for &node in self.nodes() {
            for (other, edge) in self.edges(node) {
                g.add_edge(other, node, edge.clone());
            }
        }
        g
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

pub struct Nodes<'a, N: 'a, E: 'a> {
    iter: Keys<'a, N, Vec<(N, E)>>
}

impl<'a, N: 'a, E: 'a> Iterator for Nodes<'a, N, E>
{
    type Item = &'a N;
    iterator_methods!(&'a N);
}

pub struct Neighbors<'a, N: 'a, E: 'a> {
    iter: Map<&'a (N, E), N, Iter<'a, (N, E)>, fn(&(N, E)) -> N>,
}

impl<'a, N: 'a, E: 'a> Iterator for Neighbors<'a, N, E>
{
    type Item = N;
    iterator_methods!(N);
}

pub struct Edges<'a, N: 'a, E: 'a> {
    iter: Map<&'a (N, E), (N, &'a E), Iter<'a, (N, E)>, fn(&(N, E)) -> (N, &E)>,
}

impl<'a, N: 'a, E: 'a> Iterator for Edges<'a, N, E>
{
    type Item = (N, &'a E);
    iterator_methods!((N, &'a E));
}

