//! Compatability implementations for deprecated graph traits.
#![allow(deprecated)]

use crate::{deprecated::visit::NodeRef, node::Node, storage::GraphStorage};

impl<S> NodeRef for Node<'_, S>
where
    S: GraphStorage,
{
    type NodeId = S::NodeId;
    type Weight = S::NodeWeight;

    fn id(&self) -> Self::NodeId {
        self.id
    }

    fn weight(&self) -> &Self::Weight {
        self.weight()
    }
}
