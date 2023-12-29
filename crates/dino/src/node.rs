use core::fmt::{Display, Formatter};

use petgraph_core::{
    attributes::NoValue,
    edge::marker::GraphDirectionality,
    id::{AttributeGraphId, FlaggableGraphId, GraphId, LinearGraphId, ManagedGraphId},
};

use crate::{
    slab::{
        secondary::{SlabAttributeStorage, SlabFlagStorage},
        EntryId, Key, SlabIndexMapper,
    },
    DinoStorage,
};

/// Identifier for a node in [`DinoStorage`].
///
/// [`NodeId`] is a unique identifier for a node in a [`DinoStorage`].
/// It is used to reference nodes within the graph.
///
/// A [`NodeId`] is managed, meaning that it is chosen by the graph itself and not by the user.
///
/// [`NodeId`] implements [`GraphId`], [`ManagedGraphId`] and [`LinearGraphId`].
///
/// # Example
///
/// ```
/// use petgraph_dino::DiDinoGraph;
///
/// let mut graph = DiDinoGraph::<_, u8>::new();
///
/// let a = *graph.insert_node("A").id();
///
/// println!("Node A: {a}");
/// ```
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

impl<N, E, D> LinearGraphId<DinoStorage<N, E, D>> for NodeId
where
    D: GraphDirectionality,
{
    type Mapper<'a> = SlabIndexMapper<'a, Self> where Self: 'a, N: 'a, E: 'a;

    fn index_mapper(storage: &DinoStorage<N, E, D>) -> Self::Mapper<'_> {
        SlabIndexMapper::new(&storage.nodes)
    }
}

impl<N, E, D> FlaggableGraphId<DinoStorage<N, E, D>> for NodeId
where
    D: GraphDirectionality,
{
    type Store<'a> = SlabFlagStorage<'a> where DinoStorage<N, E, D>: 'a;

    fn flag_store(storage: &DinoStorage<N, E, D>) -> Self::Store<'_> {
        SlabFlagStorage::new(&storage.nodes)
    }
}

impl<N, E, D> AttributeGraphId<DinoStorage<N, E, D>> for NodeId
where
    D: GraphDirectionality,
{
    type Store<'a, V> = SlabAttributeStorage<'a, Self, V> where DinoStorage<N, E, D>: 'a;

    fn attribute_store<V>(storage: &DinoStorage<N, E, D>) -> Self::Store<'_, V> {
        SlabAttributeStorage::new(&storage.nodes)
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
