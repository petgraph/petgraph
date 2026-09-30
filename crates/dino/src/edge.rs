use petgraph_core::id::Id;

use crate::{
    node::DinoNodeId,
    slab::{EntryId, Key},
};

/// TODO: Change
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DinoEdgeId(u32);

impl Key for DinoEdgeId {
    fn from_id(id: EntryId) -> Self {
        Self(id.raw())
    }

    fn into_id(self) -> EntryId {
        EntryId::new_unchecked(self.0 as usize)
    }
}

impl core::fmt::Display for DinoEdgeId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "DinoEdgeId({})", self.0)
    }
}

impl Id for DinoEdgeId {}

pub(crate) type EdgeSlab<T> = crate::slab::Slab<DinoEdgeId, Edge<T>>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Edge<T> {
    pub(crate) id: DinoEdgeId,
    pub(crate) weight: T,

    pub(crate) source: DinoNodeId,
    pub(crate) target: DinoNodeId,
}

impl<T> Edge<T> {
    pub(crate) const fn new(
        id: DinoEdgeId,
        weight: T,
        source: DinoNodeId,
        target: DinoNodeId,
    ) -> Self {
        Self {
            id,
            weight,
            source,
            target,
        }
    }
}
