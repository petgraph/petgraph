use petgraph_core::{
    attributes::NoValue,
    id::{GraphId, LinearGraphId, ManagedGraphId},
};

use crate::{
    node::NodeId,
    slab::{EntryId, Key, SlabLinearIndexLookup},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgeId(EntryId);

impl Key for EdgeId {
    fn from_id(id: EntryId) -> Self {
        Self(id)
    }

    fn into_id(self) -> EntryId {
        self.0
    }
}

impl GraphId for EdgeId {
    type AttributeIndex = NoValue;
}

impl LinearGraphId for EdgeId {}

impl ManagedGraphId for EdgeId {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Edge<T> {
    pub(crate) id: EdgeId,
    pub(crate) weight: T,

    pub(crate) source: NodeId,
    pub(crate) target: NodeId,
}

impl<T> Edge<T> {
    pub(crate) const fn new(id: EdgeId, weight: T, source: NodeId, target: NodeId) -> Self {
        Self {
            id,
            weight,
            source,
            target,
        }
    }
}
