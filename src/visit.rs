use std::collections::{
    HashSet,
    BitvSet,
};
use std::collections::hash_map::Hasher;
use std::hash::Hash;

use super::{
    graph,
    digraph,
    ograph,
    EdgeDirection,
    OGraph,
    Graph,
    DiGraph,
};

pub trait Graphlike {
    type NodeId: Clone;
}

/// A graph trait for accessing the neighbors iterator **I**.
pub trait IntoNeighbors<N> : Copy {
    type Iter: Iterator<Item=N>;
    fn neighbors(self, n: N) -> Self::Iter;
}

impl<'a, N: 'a, E> IntoNeighbors<N> for &'a Graph<N, E>
where N: Copy + Clone + PartialOrd + Hash<Hasher> + Eq
{
    type Iter = graph::Neighbors<'a, N>;
    fn neighbors(self, n: N) -> graph::Neighbors<'a, N>
    {
        Graph::neighbors(self, n)
    }
}

impl<'a, N: 'a, E: 'a> IntoNeighbors<N> for &'a DiGraph<N, E>
where N: Copy + Clone + Hash<Hasher> + Eq
{
    type Iter = digraph::Neighbors<'a, N, E>;
    fn neighbors(self, n: N) -> digraph::Neighbors<'a, N, E>
    {
        DiGraph::neighbors(self, n)
    }
}

impl<'a, N, E, Ty: ograph::EdgeType> IntoNeighbors< ograph::NodeIndex> for &'a OGraph<N, E, Ty>
{
    type Iter = ograph::Neighbors<'a, E>;
    fn neighbors(self, n: ograph::NodeIndex) -> ograph::Neighbors<'a, E>
    {
        OGraph::neighbors(self, n)
    }
}

/// Wrapper type for walking the graph as if it is undirected
pub struct Undirected<G>(pub G);
/// Wrapper type for walking edges the other way
pub struct Reversed<G>(pub G);

impl<'a, 'b, N, E> IntoNeighbors< ograph::NodeIndex> for &'a Undirected<&'b OGraph<N, E>>
{
    type Iter = ograph::Neighbors<'a, E>;
    fn neighbors(self, n: ograph::NodeIndex) -> ograph::Neighbors<'a, E>
    {
        OGraph::neighbors_undirected(self.0, n)
    }
}

impl<'a, 'b, N, E, Ty: ograph::EdgeType> IntoNeighbors< ograph::NodeIndex> for &'a Reversed<&'b OGraph<N, E, Ty>>
{
    type Iter = ograph::Neighbors<'a, E>;
    fn neighbors(self, n: ograph::NodeIndex) -> ograph::Neighbors<'a, E>
    {
        OGraph::neighbors_directed(self.0, n, EdgeDirection::Incoming)
    }
}

pub trait VisitMap<N> {
    fn visit(&mut self, N) -> bool;
    fn contains(&self, &N) -> bool;
}

impl VisitMap<ograph::NodeIndex> for BitvSet {
    fn visit(&mut self, x: ograph::NodeIndex) -> bool {
        self.insert(x.0)
    }
    fn contains(&self, x: &ograph::NodeIndex) -> bool {
        self.contains(&x.0)
    }
}

impl<N: Eq + Hash<Hasher>> VisitMap<N> for HashSet<N> {
    fn visit(&mut self, x: N) -> bool {
        self.insert(x)
    }
    fn contains(&self, x: &N) -> bool {
        self.contains(x)
    }
}

/// Trait for Graph that knows which datastructure is the best for its visitor map
pub trait Visitable : Graphlike {
    type Map: VisitMap<<Self as Graphlike>::Item>;
    fn visit_map(&self) -> Self::Map;
}

impl<N, E, Ty> Graphlike for OGraph<N, E, Ty> {
    type NodeId = ograph::NodeIndex;
}

impl<N, E, Ty> Visitable for OGraph<N, E, Ty> where
    Ty: ograph::EdgeType,
{
    type Map = BitvSet;
    fn visit_map(&self) -> BitvSet { BitvSet::with_capacity(self.node_count()) }
}

impl<N: Clone, E> Graphlike for DiGraph<N, E>
{
    type NodeId = N;
}

impl<N, E> Visitable for DiGraph<N, E>
    where N: Copy + Clone + Eq + Hash<Hasher>
{
    type Map = HashSet<N>;
    fn visit_map(&self) -> HashSet<N> { HashSet::with_capacity(self.node_count()) }
}

impl<N: Clone, E> Graphlike for Graph<N, E>
{
    type NodeId = N;
}

impl<N, E> Visitable for Graph<N, E>
    where N: Copy + Clone + Ord + Eq + Hash<Hasher>
{
    type Map = HashSet<N>;
    fn visit_map(&self) -> HashSet<N> { HashSet::with_capacity(self.node_count()) }
}

impl<'a, V: Graphlike> Graphlike for Undirected<&'a V>
{
    type NodeId = <V as Graphlike>::NodeId;
}

impl<'a, V: Graphlike> Graphlike for Reversed<&'a V>
{
    type NodeId = <V as Graphlike>::NodeId;
}

impl<'a, V: Visitable> Visitable for Undirected<&'a V>
{
    type Map = <V as Visitable>::Map;
    fn visit_map(&self) -> <V as Visitable>::Map {
        self.0.visit_map()
    }
}

impl<'a, V: Visitable> Visitable for Reversed<&'a V>
{
    type Map = <V as Visitable>::Map;
    fn visit_map(&self) -> <V as Visitable>::Map {
        self.0.visit_map()
    }
}


/// “Color” of nodes used in a regular depth-first search
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Color {
    /// Unvisited
    White = 0,
    /// Discovered
    Gray = 1,
    /// Visited
    Black = 2,
}

/// Trait for Graph that knows which datastructure is the best for its visitor map
pub trait ColorVisitable : Graphlike {
    type Map;
    fn color_visit_map(&self) -> Self::Map;
}
