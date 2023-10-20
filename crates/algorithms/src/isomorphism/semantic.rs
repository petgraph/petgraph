use petgraph_core::{
    deprecated::{
        data::DataMap,
        visit::{EdgeRef, GraphBase, IntoEdgesDirected},
    },
    edge::Direction,
};

use super::*;

pub struct NoSemanticMatch;

pub trait NodeMatcher<G0: GraphBase, G1: GraphBase> {
    fn enabled() -> bool;
    fn eq(&mut self, _g0: &G0, _g1: &G1, _n0: G0::NodeId, _n1: G1::NodeId) -> bool;
}

impl<G0: GraphBase, G1: GraphBase> NodeMatcher<G0, G1> for NoSemanticMatch {
    #[inline]
    fn enabled() -> bool {
        false
    }

    #[inline]
    fn eq(&mut self, _g0: &G0, _g1: &G1, _n0: G0::NodeId, _n1: G1::NodeId) -> bool {
        true
    }
}

impl<G0, G1, F> NodeMatcher<G0, G1> for F
where
    G0: GraphBase + DataMap,
    G1: GraphBase + DataMap,
    F: FnMut(&G0::NodeWeight, &G1::NodeWeight) -> bool,
{
    #[inline]
    fn enabled() -> bool {
        true
    }

    #[inline]
    fn eq(&mut self, g0: &G0, g1: &G1, n0: G0::NodeId, n1: G1::NodeId) -> bool {
        if let (Some(x), Some(y)) = (g0.node_weight(n0), g1.node_weight(n1)) {
            self(x, y)
        } else {
            false
        }
    }
}

pub trait EdgeMatcher<G0: GraphBase, G1: GraphBase> {
    fn enabled() -> bool;
    fn eq(
        &mut self,
        _g0: &G0,
        _g1: &G1,
        e0: (G0::NodeId, G0::NodeId),
        e1: (G1::NodeId, G1::NodeId),
    ) -> bool;
}

impl<G0: GraphBase, G1: GraphBase> EdgeMatcher<G0, G1> for NoSemanticMatch {
    #[inline]
    fn enabled() -> bool {
        false
    }

    #[inline]
    fn eq(
        &mut self,
        _g0: &G0,
        _g1: &G1,
        _e0: (G0::NodeId, G0::NodeId),
        _e1: (G1::NodeId, G1::NodeId),
    ) -> bool {
        true
    }
}

impl<G0, G1, F> EdgeMatcher<G0, G1> for F
where
    G0: GraphBase + DataMap + IntoEdgesDirected,
    G1: GraphBase + DataMap + IntoEdgesDirected,
    F: FnMut(&G0::EdgeWeight, &G1::EdgeWeight) -> bool,
{
    #[inline]
    fn enabled() -> bool {
        true
    }

    #[inline]
    fn eq(
        &mut self,
        g0: &G0,
        g1: &G1,
        e0: (G0::NodeId, G0::NodeId),
        e1: (G1::NodeId, G1::NodeId),
    ) -> bool {
        let w0 = g0
            .edges_directed(e0.0, Direction::Outgoing)
            .find(|edge| edge.target() == e0.1)
            .and_then(|edge| g0.edge_weight(edge.id()));
        let w1 = g1
            .edges_directed(e1.0, Direction::Outgoing)
            .find(|edge| edge.target() == e1.1)
            .and_then(|edge| g1.edge_weight(edge.id()));
        if let (Some(x), Some(y)) = (w0, w1) {
            self(x, y)
        } else {
            false
        }
    }
}
