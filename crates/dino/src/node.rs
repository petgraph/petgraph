use petgraph_core::{
    attributes::Never,
    id::{GraphId, ManagedGraphId},
};

use crate::slab::{EntryId, Key};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(EntryId);

impl Key for NodeId {
    fn from_id(id: EntryId) -> Self {
        Self(id)
    }

    fn into_id(self) -> EntryId {
        self.0
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
