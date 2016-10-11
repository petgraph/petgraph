
use prelude::*;

use visit::{
    GraphBase,
    IntoNeighbors,
    IntoNeighborsDirected,
    NodeIndexable,
    Visitable,
};

/// A graph filter for nodes.
pub trait FilterNode<G>
    where G: GraphBase,
{
    fn include_node(&self, _node: G::NodeId) -> bool { true }
}

impl<G, F> FilterNode<G> for F
    where G: GraphBase,
          F: Fn(G::NodeId) -> bool,
{
    fn include_node(&self, n: G::NodeId) -> bool {
        (*self)(n)
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
          F: FilterNode<G>,
{
    type Neighbors = FilteredNeighbors<'a, G, G::Neighbors, F>;
    fn neighbors(self, n: G::NodeId) -> Self::Neighbors {
        FilteredNeighbors {
            source: n,
            iter: self.0.neighbors(n),
            f: &self.1,
        }
    }
}

/// A filtered neighbors iterator.
pub struct FilteredNeighbors<'a, G, I, F: 'a>
    where G: GraphBase,
{
    source: G::NodeId,
    iter: I,
    f: &'a F,
}

impl<'a, G, I, F> Iterator for FilteredNeighbors<'a, G, I, F>
    where G: GraphBase,
          I: Iterator<Item=G::NodeId>,
          F: FilterNode<G>,
{
    type Item = G::NodeId;
    fn next(&mut self) -> Option<Self::Item> {
        let source = self.source;
        let f = self.f;
        if !f.include_node(source) {
            None
        } else {
            (&mut self.iter).filter(move |&target| f.include_node(target)).next()
        }
    }
}

impl<'a, G, F> IntoNeighborsDirected for &'a Filtered<G, F>
    where G: IntoNeighborsDirected,
          F: FilterNode<G>,
{
    type NeighborsDirected = FilteredNeighbors<'a, G, G::NeighborsDirected, F>;
    fn neighbors_directed(self, n: G::NodeId, dir: Direction)
        -> Self::NeighborsDirected {
        FilteredNeighbors {
            source: n,
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
    fn to_index(n: G::NodeId) -> usize { G::to_index(n) }
}
