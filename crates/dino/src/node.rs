use core::fmt::{Display, Formatter};

use petgraph_core::{
    attributes::NoValue,
    edge::marker::GraphDirection,
    id::{GraphId, IndexMapper, LinearGraphId, ManagedGraphId},
};

use crate::{
    slab::{EntryId, Key, SlabIndexMapper},
    DinosaurStorage,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(EntryId);

impl Display for NodeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Key for NodeId {
    fn from_id(id: EntryId) -> Self {
        Self(id)
    }

    fn into_id(self) -> EntryId {
        self.0
    }
}

impl GraphId for NodeId {
    type AttributeIndex = NoValue;
}

impl<N, E, D> LinearGraphId<DinosaurStorage<N, E, D>> for NodeId
where
    D: GraphDirection,
{
    type Mapper<'a> = SlabIndexMapper<'a, NodeId> where Self: 'a, N: 'a, E: 'a;

    fn index_mapper(storage: &DinosaurStorage<N, E, D>) -> Self::Mapper<'_> {
        SlabIndexMapper::new(&storage.nodes)
    }
}

impl ManagedGraphId for NodeId {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Node<T> {
    pub(crate) id: NodeId,
    pub(crate) weight: T,
}

impl<T> Node<T> {
    pub(crate) const fn new(id: NodeId, weight: T) -> Self {
        Self { id, weight }
    }
}
