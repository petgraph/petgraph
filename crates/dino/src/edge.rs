use core::fmt::{Display, Formatter};

use petgraph_core::{
    attributes::NoValue,
    edge::marker::GraphDirectionality,
    id::{AssociativeGraphId, FlaggableGraphId, GraphId, LinearGraphId, ManagedGraphId},
};

use crate::{
    node::NodeId,
    slab::{
        secondary::{SlabAttributeStorage, SlabFlagStorage},
        EntryId, Key, SlabIndexMapper,
    },
    DinoStorage,
};

/// Identifier for an edge in [`DinoStorage`].
///
/// [`EdgeId`] is a unique identifier for an edge in a [`DinoStorage`].
/// It is used to reference edges within the graph.
///
/// An [`EdgeId`] is managed, meaning that it is chosen by the graph itself and not by the user.
///
/// [`EdgeId`] implements [`GraphId`], [`ManagedGraphId`] and [`LinearGraphId`].
///
/// # Example
///
/// ```
/// use petgraph_dino::DiDinoGraph;
///
/// let mut graph = DiDinoGraph::new();
///
/// let a = *graph.insert_node("A").id();
/// let b = *graph.insert_node("B").id();
///
/// let ab = *graph.insert_edge("A → B", &a, &b).id();
///
/// println!("Edge A → B: {ab}");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgeId(EntryId);

impl Display for EdgeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

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

impl<N, E, D> LinearGraphId<DinoStorage<N, E, D>> for EdgeId
where
    D: GraphDirectionality,
{
    type Mapper<'a> = SlabIndexMapper<'a, Self> where Self: 'a, N: 'a, E: 'a;

    fn index_mapper(storage: &DinoStorage<N, E, D>) -> Self::Mapper<'_> {
        SlabIndexMapper::new(&storage.edges)
    }
}

impl<N, E, D> FlaggableGraphId<DinoStorage<N, E, D>> for EdgeId
where
    D: GraphDirectionality,
{
    type Store<'a> = SlabFlagStorage<'a> where DinoStorage<N, E, D>: 'a;

    fn flag_store(storage: &DinoStorage<N, E, D>) -> Self::Store<'_> {
        SlabFlagStorage::new(&storage.edges)
    }
}

impl<N, E, D> AssociativeGraphId<DinoStorage<N, E, D>> for EdgeId
where
    D: GraphDirectionality,
{
    type AttributeMapper<'a, V> = SlabAttributeStorage<'a, Self, V> where DinoStorage<N, E, D>: 'a;

    fn attribute_mapper<V>(storage: &DinoStorage<N, E, D>) -> Self::AttributeMapper<'_, V> {
        SlabAttributeStorage::new(&storage.edges)
    }
}

impl ManagedGraphId for EdgeId {}

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
