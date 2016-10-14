//! Graph traits for associated data and graph construction.


use Graph;
use stable_graph::StableGraph;
use ::{
    EdgeType,
};
use graph::IndexType;
use visit::{
    Data,
    NodeCount,
    NodeIndexable,
    Reversed,
};

trait_template!{
    /// Access node and edge weights (associated data).
pub trait DataMap : Data {
    @section self_ref
    fn node_weight(&self, id: Self::NodeId) -> Option<&Self::NodeWeight>;
    fn edge_weight(&self, id: Self::EdgeId) -> Option<&Self::EdgeWeight>;
}
}

macro_rules! access0 {
    ($e:expr) => ($e.0);
}

DataMap!{delegate_impl []}
DataMap!{delegate_impl [['a, G], G, &'a mut G, deref_twice]}
DataMap!{delegate_impl [[G], G, Reversed<G>, access0]}

trait_template! {
    /// Access node and edge weights mutably.
pub trait DataMapMut : DataMap {
    @section self_mut
    fn node_weight_mut(&mut self, id: Self::NodeId) -> Option<&mut Self::NodeWeight>;
    fn edge_weight_mut(&mut self, id: Self::EdgeId) -> Option<&mut Self::EdgeWeight>;
}
}

DataMapMut!{delegate_impl [['a, G], G, &'a mut G, deref_twice]}
DataMapMut!{delegate_impl [[G], G, Reversed<G>, access0]}

/// A graph that can be extended with further nodes and edges
pub trait Build : Data + NodeCount {
    fn add_node(&mut self, weight: Self::NodeWeight) -> Self::NodeId;
    /// Add a new edge. If parallel edges (duplicate) are not allowed and
    /// the edge already exists, return `None`.
    fn add_edge(&mut self,
                a: Self::NodeId,
                b: Self::NodeId,
                weight: Self::EdgeWeight) -> Option<Self::EdgeId> {
        Some(self.update_edge(a, b, weight))
    }
    /// Add or update the edge from `a` to `b`. Return the id of the affected
    /// edge.
    fn update_edge(&mut self,
                   a: Self::NodeId,
                   b: Self::NodeId,
                   weight: Self::EdgeWeight) -> Self::EdgeId;
}

/// A graph that can be created
pub trait Create : Build + DataMapMut + Default {
    fn with_capacity(nodes: usize, edges: usize) -> Self;
}

impl<N, E, Ty, Ix> Data for Graph<N, E, Ty, Ix>
    where Ix: IndexType
{
    type NodeWeight = N;
    type EdgeWeight = E;
}

impl<N, E, Ty, Ix> DataMap for Graph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType
{
    fn node_weight(&self, id: Self::NodeId) -> Option<&Self::NodeWeight> {
        self.node_weight(id)
    }
    fn edge_weight(&self, id: Self::EdgeId) -> Option<&Self::EdgeWeight> {
        self.edge_weight(id)
    }
}

impl<N, E, Ty, Ix> DataMapMut for Graph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType
{
    fn node_weight_mut(&mut self, id: Self::NodeId) -> Option<&mut Self::NodeWeight> {
        self.node_weight_mut(id)
    }
    fn edge_weight_mut(&mut self, id: Self::EdgeId) -> Option<&mut Self::EdgeWeight> {
        self.edge_weight_mut(id)
    }
}

impl<N, E, Ty, Ix> DataMap for StableGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType
{
    fn node_weight(&self, id: Self::NodeId) -> Option<&Self::NodeWeight> {
        self.node_weight(id)
    }
    fn edge_weight(&self, id: Self::EdgeId) -> Option<&Self::EdgeWeight> {
        self.edge_weight(id)
    }
}

impl<N, E, Ty, Ix> DataMapMut for StableGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType
{
    fn node_weight_mut(&mut self, id: Self::NodeId) -> Option<&mut Self::NodeWeight> {
        self.node_weight_mut(id)
    }
    fn edge_weight_mut(&mut self, id: Self::EdgeId) -> Option<&mut Self::EdgeWeight> {
        self.edge_weight_mut(id)
    }
}

impl<N, E, Ty, Ix> Build for Graph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    fn add_node(&mut self, weight: Self::NodeWeight) -> Self::NodeId {
        self.add_node(weight)
    }
    fn add_edge(&mut self,
                a: Self::NodeId,
                b: Self::NodeId,
                weight: Self::EdgeWeight) -> Option<Self::EdgeId>
    {
        Some(self.add_edge(a, b, weight))
    }
    fn update_edge(&mut self,
                   a: Self::NodeId,
                   b: Self::NodeId,
                   weight: Self::EdgeWeight) -> Self::EdgeId
    {
        self.update_edge(a, b, weight)
    }
}

impl<N, E, Ty, Ix> Build for StableGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    fn add_node(&mut self, weight: Self::NodeWeight) -> Self::NodeId {
        self.add_node(weight)
    }
    fn add_edge(&mut self,
                a: Self::NodeId,
                b: Self::NodeId,
                weight: Self::EdgeWeight) -> Option<Self::EdgeId>
    {
        Some(self.add_edge(a, b, weight))
    }
    fn update_edge(&mut self,
                   a: Self::NodeId,
                   b: Self::NodeId,
                   weight: Self::EdgeWeight) -> Self::EdgeId
    {
        self.update_edge(a, b, weight)
    }
}

impl<N, E, Ty, Ix> Create for Graph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    fn with_capacity(nodes: usize, edges: usize) -> Self {
        Self::with_capacity(nodes, edges)
    }
}

impl<N, E, Ty, Ix> Create for StableGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    fn with_capacity(nodes: usize, edges: usize) -> Self {
        Self::with_capacity(nodes, edges)
    }
}

/// A graph element
pub enum Element<N, E> {
    Node(N),
    Edge(usize, usize, E),
}

/// Create a graph from an iterator of elements.
pub trait FromElements : Create {
    fn from_elements<I>(iterable: I) -> Self
        where Self: Sized,
              I: IntoIterator<Item=Element<Self::NodeWeight, Self::EdgeWeight>>,
    {
        let mut gr = Self::with_capacity(0, 0);
        // usize -> NodeId map
        let mut map = Vec::new();
        for element in iterable {
            match element {
                Element::Node(w) => {
                    map.push(gr.add_node(w));
                }
                Element::Edge(a, b, w) => {
                    gr.add_edge(map[a], map[b], w);
                }
            }
        }
        gr
    }
        
}

fn from_elements_indexable<G, I>(iterable: I) -> G
    where G: Create + NodeIndexable,
          I: IntoIterator<Item=Element<G::NodeWeight, G::EdgeWeight>>,
{
    let mut gr = G::with_capacity(0, 0);
    let map = |gr: &G, i| gr.from_index(i);
    for element in iterable {
        match element {
            Element::Node(w) => {
                gr.add_node(w);
            }
            Element::Edge(a, b, w) => {
                let from = map(&gr, a);
                let to = map(&gr, b);
                gr.add_edge(from, to, w);
            }
        }
    }
    gr
}

impl<N, E, Ty, Ix> FromElements for Graph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    fn from_elements<I>(iterable: I) -> Self
        where Self: Sized,
              I: IntoIterator<Item=Element<Self::NodeWeight, Self::EdgeWeight>>,
    {
        from_elements_indexable(iterable)
    }
}

impl<N, E, Ty, Ix> FromElements for StableGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    fn from_elements<I>(iterable: I) -> Self
        where Self: Sized,
              I: IntoIterator<Item=Element<Self::NodeWeight, Self::EdgeWeight>>,
    {
        from_elements_indexable(iterable)
    }
}