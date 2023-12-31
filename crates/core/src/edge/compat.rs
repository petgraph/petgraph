//! Compatability implementations for deprecated graph traits.
#![allow(deprecated)]

use crate::{
    deprecated::visit::EdgeRef,
    edge::{Edge, EdgeId},
    node::NodeId,
    storage::GraphStorage,
};

impl<S> EdgeRef for Edge<'_, S>
where
    S: GraphStorage,
{
    type EdgeId = EdgeId;
    type NodeId = NodeId;
    type Weight = S::EdgeWeight;

    fn source(&self) -> Self::NodeId {
        self.u
    }

    fn target(&self) -> Self::NodeId {
        self.v
    }

    fn weight(&self) -> &Self::Weight {
        self.weight()
    }

    fn id(&self) -> Self::EdgeId {
        self.id
    }
}
