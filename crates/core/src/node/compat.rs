//! Compatability implementations for deprecated graph traits.
#![allow(deprecated)]

use crate::{
    deprecated::visit::NodeRef,
    graph::Graph,
    node::{Node, NodeId},
};

impl<S> NodeRef for Node<'_, S>
where
    S: Graph,
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
