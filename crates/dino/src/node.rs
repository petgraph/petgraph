use petgraph_core::{
    attributes::Never,
    id::{GraphId, ManagedGraphId},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(usize);

impl NodeId {
    pub(crate) fn new(id: usize) -> Self {
        Self(id)
    }

    pub(crate) fn increment(self) -> Self {
        Self(self.0 + 1)
    }
}

impl GraphId for NodeId {
    type AttributeIndex = Never;
}

impl ManagedGraphId for NodeId {}

pub struct Node<T> {
    pub(crate) id: NodeId,
    pub(crate) weight: T,
}

impl<T> Node<T> {
    pub fn new(id: NodeId, weight: T) -> Self {
        Self { id, weight }
    }
}
