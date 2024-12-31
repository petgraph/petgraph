//! Compatability implementations for deprecated graph traits.
#![allow(deprecated)]

use crate::{
    deprecated::visit::EdgeRef,
    edge::{Edge, EdgeId},
    graph::Graph,
    node::NodeId,
};

impl<S> EdgeRef for Edge<'_, S>
where
    S: Graph,
{
    type EdgeId = EdgeId;
    type NodeId = NodeId;
    type Weight = S::EdgeWeight;

    fn source(&self) -> Self::NodeId {
        self.source
    }

    fn target(&self) -> Self::NodeId {
        self.target
    }

    fn weight(&self) -> &Self::Weight {
        self.weight()
    }

    fn id(&self) -> Self::EdgeId {
        self.id
    }
}
