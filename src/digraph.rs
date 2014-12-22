
use std::hash::{Hash};
use std::collections::HashMap;
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

/// **DiGraph** is a directed graph, with node values and edge weights.
///
/// It uses an adjacency list representation, i.e. using *O(|N| + |E|)* space.
pub struct DiGraph<N: Eq + Hash, E> {
    nodes: HashMap<N, Vec<(N, E)>>,
}

impl<N: Eq + Hash + fmt::Show, E: fmt::Show> fmt::Show for DiGraph<N, E>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.nodes.fmt(f)
    }
}

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

    /// Return true if node was removed.
    pub fn remove_node(&mut self, node: N) -> bool {
        match self.nodes.remove(&node) {
            None => false,
            Some(..) => {
                for (_, edges) in self.nodes.iter_mut() {
                    match edges.iter().position(|&(elt, _)| elt == node) {
                        // Use swap_remove because order doesn't matter
                        Some(index) => { edges.swap_remove(index); }
                        None => {}
                    }
                }
                true
            }
        }
    }

    pub fn contains_node(&self, node: N) -> bool {
        self.nodes.contains_key(&node)
    }

    /// Add directed edge from `a` to `b`.
    ///
    /// Return `true` if an edge was inserted.
    pub fn add_edge(&mut self, a: N, b: N, edge: E) -> bool
    {
        // We need both lookups anyway to assert sanity, so
        // add nodes if they don't already exist
        //
        // make sure the endpoint exists in the map
        match self.nodes.entry(b) {
            Vacant(ent) => { ent.set(Vec::new()); }
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
                ent.set(vec![(b, edge)]);
                true
            }
        }
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
                        ent.get_mut().swap_remove(index).map(|(_, edge)| edge)
                    }
                    None => None,
                }
            }
            Vacant(..) => None,
        }
    }

    /// Return true if the directed edge from `a` to `b` exists
    pub fn contains_edge(&mut self, a: N, b: N) -> bool
    {
        match self.nodes.get(&a) {
            None => false,
            Some(sus) => sus.iter().any(|&(elt, _)| elt == b),
        }
    }

    pub fn nodes<'a>(&'a self) -> Nodes<'a, N, E>
    {
        Nodes{iter: self.nodes.keys()}
    }

    pub fn neighbors(&self, n: N) -> Neighbors<N, E>
    {
        fn fst<'a, N: Copy, E>(t: &'a (N, E)) -> &'a N
        {
            &t.0
        }

        Neighbors{iter: self.edges(n).map(fst)}
    }

    /// If the node **n** does not exist in the graph, return an empty iterator.
    pub fn edges<'a>(&'a self, n: N) -> Items<'a, (N, E)>
    {
        match self.nodes.get(&n) {
            Some(edges) => edges.iter(),
            None => [].iter(),
        }
    }

    /// If the node **n** does not exist in the graph, return an empty iterator.
    pub fn edges_mut<'a>(&'a mut self, n: N) -> MutItems<'a, (N, E)>
    {
        match self.nodes.get_mut(&n) {
            Some(edges) => edges.iter_mut(),
            None => [].iter_mut(),
        }
    }

    pub fn edge_mut<'a>(&'a mut self, a: N, b: N) -> Option<&'a mut E>
    {
        match self.nodes.get_mut(&a) {
            Some(succ) => {
                succ.iter_mut()
                    .find(|&&(ref n, _)| n == &b)
                    .map(|&(_, ref mut edge)| edge)
            }
            None => None,
        }
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
    pub fn reversed(&self) -> DiGraph<N, E>
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

pub struct Nodes<'a, N: 'a, E: 'a> {
    iter: Keys<'a, N, Vec<(N, E)>>
}

impl<'a, N: 'a, E: 'a> Iterator<&'a N> for Nodes<'a, N, E>
{
    iterator_methods!(&'a N)
}

type MapPtr<'a, From, To, Iter> = Map<&'a From, &'a To, Iter, for<'b> fn(&'b From) -> &'b To>;

pub struct Neighbors<'a, N: 'a, E: 'a> {
    iter: MapPtr<'a, (N, E), N, Items<'a, (N, E)>>,
}

impl<'a, N: 'a, E: 'a> Iterator<&'a N> for Neighbors<'a, N, E>
{
    iterator_methods!(&'a N)
}

