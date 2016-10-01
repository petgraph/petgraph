
use ::{
    EdgeDirection,
    Incoming,
};

use visit::{
    GraphBase,
    GraphRef,
    IntoNodeIdentifiers,
    IntoNeighbors,
    IntoNeighborsDirected,
    IntoExternals,
    Visitable,
};


/// Wrapper type for walking the graph as if all edges are reversed.
#[derive(Copy, Clone)]
pub struct Reversed<G>(pub G);

impl<G: GraphBase> GraphBase for Reversed<G> {
    type NodeId = G::NodeId;
    type EdgeId = G::EdgeId;
}

impl<G: GraphRef> GraphRef for Reversed<G> { }

impl<G> IntoNodeIdentifiers for Reversed<G>
    where G: IntoNodeIdentifiers
{
    type NodeIdentifiers = G::NodeIdentifiers;
    fn node_identifiers(self) -> Self::NodeIdentifiers {
        self.0.node_identifiers()
    }

    fn node_count(&self) -> usize {
        self.0.node_count()
    }
}

impl<G> IntoNeighbors for Reversed<G>
    where G: IntoNeighborsDirected
{
    type Neighbors = G::NeighborsDirected;
    fn neighbors(self, n: G::NodeId) -> G::NeighborsDirected
    {
        self.0.neighbors_directed(n, Incoming)
    }
}

impl<G> IntoNeighborsDirected for Reversed<G>
    where G: IntoNeighborsDirected
{
    type NeighborsDirected = G::NeighborsDirected;
    fn neighbors_directed(self, n: G::NodeId, d: EdgeDirection)
        -> G::NeighborsDirected
    {
        self.0.neighbors_directed(n, d.opposite())
    }
}

impl<G> IntoExternals for Reversed<G>
    where G: IntoExternals,
{
    type Externals = G::Externals;
    fn externals(self, d: EdgeDirection) -> G::Externals {
        self.0.externals(d.opposite())
    }
}

impl<G: Visitable> Visitable for Reversed<G>
{
    type Map = G::Map;
    fn visit_map(&self) -> G::Map {
        self.0.visit_map()
    }
    fn reset_map(&self, map: &mut Self::Map) {
        self.0.reset_map(map);
    }
}

