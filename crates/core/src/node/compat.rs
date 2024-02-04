//! Compatibility implementations for deprecated graph traits.
#![allow(deprecated)]

use crate::{
    deprecated::visit::NodeRef,
    node::{Node, NodeId},
    storage::GraphStorage,
};

impl<S> NodeRef for Node<'_, S>
where
    S: GraphStorage,
{
    type NodeId = NodeId;
    type Weight = S::NodeWeight;

    fn id(&self) -> Self::NodeId {
        self.id
    }

    fn weight(&self) -> &Self::Weight {
        self.weight()
    }
}
