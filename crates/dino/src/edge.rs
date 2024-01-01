use petgraph_core::{edge::EdgeId, node::NodeId};

use crate::slab::{EntryId, Key};

impl Key for EdgeId {
    #[inline]
    fn from_id(id: EntryId) -> Self {
        Self::new(id.into_usize())
    }

    #[inline]
    fn into_id(self) -> EntryId {
        EntryId::new_unchecked(self.into_inner())
    }
}

pub(crate) type EdgeSlab<T> = crate::slab::Slab<EdgeId, Edge<T>>;

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
