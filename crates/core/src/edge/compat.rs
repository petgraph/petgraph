//! Compatability implementations for deprecated graph traits.
#![allow(deprecated)]

use crate::{deprecated::visit::EdgeRef, edge::Edge, storage::GraphStorage};

impl<S> EdgeRef for Edge<'_, S>
where
    S: GraphStorage,
    S::NodeId: Clone,
    S::EdgeId: Clone,
{
    type EdgeId = S::EdgeId;
    type NodeId = S::NodeId;
    type Weight = S::EdgeWeight;

    fn source(&self) -> Self::NodeId {
        self.source_id.clone()
    }

    fn target(&self) -> Self::NodeId {
        self.target_id.clone()
    }

    fn weight(&self) -> &Self::Weight {
        self.weight()
    }

    fn id(&self) -> Self::EdgeId {
        self.id.clone()
    }
}
