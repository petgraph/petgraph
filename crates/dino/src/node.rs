use core::fmt::{Display, Formatter};

use petgraph_core::{
    attributes::NoValue,
    edge::marker::GraphDirectionality,
    id::{GraphId, LinearGraphId, ManagedGraphId},
};
use roaring::RoaringBitmap;

use crate::{
    closure::{UnionIntoIterator, UnionIterator},
    slab::{EntryId, Key, SlabIndexMapper},
    DinoStorage, EdgeId,
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

impl ManagedGraphId for NodeId {}

pub(crate) type NodeSlab<T> = crate::slab::Slab<NodeId, Node<T>>;

#[derive(Debug, Clone)]
pub(crate) struct NodeClosures {
    pub(crate) source_to_targets: RoaringBitmap,
    pub(crate) target_to_sources: RoaringBitmap,

    pub(crate) source_to_edges: RoaringBitmap,
    pub(crate) target_to_edges: RoaringBitmap,
}

impl NodeClosures {
    fn new() -> Self {
        Self {
            source_to_targets: RoaringBitmap::new(),
            target_to_sources: RoaringBitmap::new(),

            source_to_edges: RoaringBitmap::new(),
            target_to_edges: RoaringBitmap::new(),
        }
    }

    pub(crate) fn outgoing_neighbours(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.source_to_targets
            .iter()
            .map(|value| NodeId::from_id(EntryId::new_unchecked(value)))
    }

    pub(crate) fn incoming_neighbours(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.target_to_sources
            .iter()
            .map(|value| NodeId::from_id(EntryId::new_unchecked(value)))
    }

    pub(crate) fn neighbours(&self) -> impl Iterator<Item = NodeId> + '_ {
        UnionIterator::new(&self.source_to_targets, &self.target_to_sources)
            .map(|value| NodeId::from_id(EntryId::new_unchecked(value)))
    }

    pub(crate) fn clear(&mut self) {
        self.source_to_targets.clear();
        self.target_to_sources.clear();

        self.source_to_edges.clear();
        self.target_to_edges.clear();
    }

    pub(crate) fn into_edges(self) -> impl Iterator<Item = EdgeId> {
        UnionIntoIterator::new(self.source_to_edges, self.target_to_edges)
            .map(|value| EdgeId::from_id(EntryId::new_unchecked(value)))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Node<T> {
    pub(crate) id: NodeId,
    pub(crate) weight: T,

    pub(crate) closures: NodeClosures,
}

impl<T> Node<T> {
    pub(crate) fn new(id: NodeId, weight: T) -> Self {
        Self {
            id,
            weight,
            closures: NodeClosures::new(),
        }
    }

    pub(crate) fn outgoing_neighbours(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.closures.outgoing_neighbours()
    }

    pub(crate) fn incoming_neighbours(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.closures.incoming_neighbours()
    }

    pub(crate) fn neighbours(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.closures.neighbours()
    }

    pub(crate) fn outgoing_edges(&self) -> impl Iterator<Item = EdgeId> + '_ {
        self.closures
            .source_to_edges
            .iter()
            .map(|value| EdgeId::from_id(EntryId::new_unchecked(value)))
    }

    pub(crate) fn incoming_edges(&self) -> impl Iterator<Item = EdgeId> + '_ {
        self.closures
            .target_to_edges
            .iter()
            .map(|value| EdgeId::from_id(EntryId::new_unchecked(value)))
    }

    pub(crate) fn edges(&self) -> impl Iterator<Item = EdgeId> + '_ {
        UnionIterator::new(
            &self.closures.source_to_edges,
            &self.closures.target_to_edges,
        )
        .map(|value| EdgeId::from_id(EntryId::new_unchecked(value)))
    }

    pub(crate) fn is_isolated(&self) -> bool {
        self.closures.source_to_targets.is_empty() && self.closures.target_to_sources.is_empty()
    }
}

impl<T> PartialEq for Node<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        (&self.id, &self.weight) == (&other.id, &other.weight)
    }
}

impl<T> Eq for Node<T> where T: Eq {}

impl<T> PartialOrd for Node<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        (&self.id, &self.weight).partial_cmp(&(&other.id, &other.weight))
    }
}

impl<T> Ord for Node<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        (&self.id, &self.weight).cmp(&(&other.id, &other.weight))
    }
}
