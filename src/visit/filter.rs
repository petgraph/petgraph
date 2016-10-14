
use prelude::*;

use fixedbitset::FixedBitSet;
use std::collections::HashSet;

use visit::{
    GraphBase,
    IntoNeighbors,
    IntoNeighborsDirected,
    NodeIndexable,
    Visitable,
    VisitMap,
};

/// A graph filter for nodes.
pub trait FilterNode<N>
{
    fn include_node(&self, node: N) -> bool { let _ = node; true }
}

impl<F, N> FilterNode<N> for F
    where F: Fn(N) -> bool,
{
    fn include_node(&self, n: N) -> bool {
        (*self)(n)
    }
}

/// This filter includes the nodes that are contained in the set.
impl<N> FilterNode<N> for FixedBitSet
    where FixedBitSet: VisitMap<N>,
{
    fn include_node(&self, n: N) -> bool {
        self.is_visited(&n)
    }
}

/// This filter includes the nodes that are contained in the set.
impl<N, S> FilterNode<N> for HashSet<N, S>
    where HashSet<N, S>: VisitMap<N>,
{
    fn include_node(&self, n: N) -> bool {
        self.is_visited(&n)
    }
}

/// A filtered adaptor of a graph.
#[derive(Copy, Clone, Debug)]
pub struct Filtered<G, F>(pub G, pub F);

impl<G, F> GraphBase for Filtered<G, F> where G: GraphBase {
    type NodeId = G::NodeId;
    type EdgeId = G::EdgeId;
}

impl<'a, G, F> IntoNeighbors for &'a Filtered<G, F>
    where G: IntoNeighbors,
          F: FilterNode<G::NodeId>,
{
    type Neighbors = FilteredNeighbors<'a, G::Neighbors, F>;
    fn neighbors(self, n: G::NodeId) -> Self::Neighbors {
        FilteredNeighbors {
            include_source: self.1.include_node(n),
            iter: self.0.neighbors(n),
            f: &self.1,
        }
    }
}

/// A filtered neighbors iterator.
pub struct FilteredNeighbors<'a, I, F: 'a>
{
    include_source: bool,
    iter: I,
    f: &'a F,
}

impl<'a, I, F> Iterator for FilteredNeighbors<'a, I, F>
    where I: Iterator,
          I::Item: Copy,
          F: FilterNode<I::Item>,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        let f = self.f;
        if !self.include_source {
            None
        } else {
            (&mut self.iter).filter(move |&target| f.include_node(target)).next()
        }
    }
}

impl<'a, G, F> IntoNeighborsDirected for &'a Filtered<G, F>
    where G: IntoNeighborsDirected,
          F: FilterNode<G::NodeId>,
{
    type NeighborsDirected = FilteredNeighbors<'a, G::NeighborsDirected, F>;
    fn neighbors_directed(self, n: G::NodeId, dir: Direction)
        -> Self::NeighborsDirected {
        FilteredNeighbors {
            include_source: self.1.include_node(n),
            iter: self.0.neighbors_directed(n, dir),
            f: &self.1,
        }
    }
}

impl<G, F> Visitable for Filtered<G, F>
    where G: Visitable,
{
    type Map = G::Map;
    fn visit_map(&self) -> G::Map {
        self.0.visit_map()
    }
    fn reset_map(&self, map: &mut Self::Map) {
        self.0.reset_map(map);
    }
}

impl<G, F> NodeIndexable for Filtered<G, F>
    where G: NodeIndexable,
{
    fn node_bound(&self) -> usize { self.0.node_bound() }
    fn to_index(&self, n: G::NodeId) -> usize { self.0.to_index(n) }
    fn from_index(&self, ix: usize) -> Self::NodeId { self.0.from_index(ix) }
}
