//! A graph coupled with a mapping from arbitrary keys
//! into node indices.

use std::hash::Hash;
use std::collections::HashMap;
use std::ops::{Index, IndexMut};
use std::borrow::Borrow;

use {
    EdgeType,
};

use visit::{
    Graphlike,
};

use graph::{
    Graph,
    IndexType,
    NodeIndex,
    EdgeIndex,
};

/// An input error in the builder
#[derive(Clone, Debug)]
pub struct BuilderError(());

/// `GraphBuilder` is a graph coupled with a mapping
/// from arbitrary keys into node indices.
///
/// `GraphBuilder` can be used to quickly build a graph using hashable identifiers.
#[derive(Clone, Debug)]
pub struct GraphBuilder<Key, G>
    where G: Graphlike,
          Key: Hash + Eq,
{
    graph: G,
    node_map: HashMap<Key, G::NodeId>
}

impl<Key, N, E, Ty, Ix> GraphBuilder<Key, Graph<N, E, Ty, Ix>>
    where Ix: IndexType,
          Ty: EdgeType,
          Key: Eq + Hash,
{
    pub fn from(graph: Graph<N, E, Ty, Ix>) -> Self {
        let (node_cap, _) = graph.capacity();
        GraphBuilder {
            node_map: HashMap::with_capacity(node_cap - graph.node_count()),
            graph: graph,
        }
    }

    pub fn with_capacity(nodes: usize, edges: usize) -> Self {
        GraphBuilder {
            graph: Graph::with_capacity(nodes, edges),
            node_map: HashMap::with_capacity(nodes),
        }
    }

    /// Add a node for `key` with `weight` if it does not exist.
    pub fn ensure_node(&mut self, key: Key, weight: N) -> NodeIndex<Ix> {
        let graph = &mut self.graph;
        *self.node_map.entry(key).or_insert_with(|| {
            graph.add_node(weight)
        })
    }

    /// Add a node for `key` with weight taken from `f()`
    /// if the node does not exist.
    pub fn ensure_node_with<F>(&mut self, key: Key, f: F) -> NodeIndex<Ix> 
        where F: FnOnce() -> N,
    {
        let graph = &mut self.graph;
        *self.node_map.entry(key).or_insert_with(|| {
            graph.add_node(f())
        })
    }

    /// Add an edge from `a` to `b` (parallel edges allowed).
    ///
    /// **Panics** if either of the nodes don't exist.
    pub fn add_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>,
                    weight: E) -> EdgeIndex<Ix>
    {
        self.graph.add_edge(a, b, weight)
    }

    /// Add or update an edge from `a` to `b`.
    ///
    /// **Panics** if either of the nodes don't exist.
    pub fn update_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>,
                       weight: E) -> EdgeIndex<Ix>
    {
        self.graph.update_edge(a, b, weight)
    }

    /// Add or update an edge from `a` to `b`.
    ///
    /// Return an error if either of the nodes don't exist.
    ///
    /// **Panics** if the node map is inconsistent (key in map, but
    /// corresponding node not in graph).
    pub fn update_edge_by_key<Q>(&mut self, a: &Q, b: &Q,
                                 weight: E) -> Result<EdgeIndex<Ix>, BuilderError>
        where Key: Borrow<Q>,
              Q: Eq + Hash,
    {
        if let (Some(&a), Some(&b)) = (self.node_map.get(a), self.node_map.get(b)) {
            Ok(self.graph.update_edge(a, b, weight))
        } else {
            Err(BuilderError(()))
        }
    }

    /// Return a mutable reference to the graph
    ///
    /// **Note:** You should of course not do any operations that shift node
    /// indices on the graph.
    pub fn graph_mut(&mut self) -> &mut Graph<N, E, Ty, Ix> {
        &mut self.graph
    }

    /// Return a mutable reference to the node map
    ///
    pub fn node_map_mut(&mut self) -> &mut HashMap<Key, NodeIndex<Ix>> {
        &mut self.node_map
    }

    /// Split the builder into the node map and the graph
    pub fn into_inner(self) -> (HashMap<Key, NodeIndex<Ix>>, Graph<N, E, Ty, Ix>) {
        (self.node_map, self.graph)
    }

    /// Split the builder into the graph
    pub fn into_graph(self) -> Graph<N, E, Ty, Ix> {
        self.graph
    }
}

/// `GraphBuilder` can be indexed as if it were the underlying graph.
impl<Key, G, I> Index<I> for GraphBuilder<Key, G>
    where G: Index<I> + Graphlike,
          Key: Hash + Eq,
{
    type Output = G::Output;
    fn index(&self, index: I) -> &Self::Output {
        &self.graph[index]
    }
}

/// `GraphBuilder` can be indexed as if it were the underlying graph.
impl<Key, G, I> IndexMut<I> for GraphBuilder<Key, G>
    where G: IndexMut<I> + Graphlike,
          Key: Hash + Eq,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.graph[index]
    }
}
