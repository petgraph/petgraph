use alloc::vec::Vec;
use core::{
    fmt::{Display, Formatter},
    iter::Enumerate,
};

use bitvec::{boxed::BitBox, vec::BitVec};
use petgraph_core::{
    attributes::NoValue,
    base::MaybeOwned,
    edge::marker::GraphDirectionality,
    id::{
        AttributeGraphId, AttributeStorage, FlagStorage, FlaggableGraphId, GraphId, IndexMapper,
        LinearGraphId, ManagedGraphId,
    },
    GraphStorage,
};

use crate::{
    iter::closure::{EdgeIdClosureIter, NodeIdClosureIter},
    slab::{EntryId, Generation, Key, SlabIndexMapper},
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

    #[inline]
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

// TODO: potential matter to reduce memory usage
pub struct FlagStore {
    vector: BitBox,
}

impl FlagStorage<NodeId> for FlagStore {
    #[inline]
    fn get(&self, id: &NodeId) -> Option<bool> {
        self.vector.get(id.into_id().index()).map(|flag| *flag)
    }

    #[inline]

    fn index(&self, id: &NodeId) -> bool {
        self.vector[id.into_id().index()]
    }

    fn set(&mut self, id: &NodeId, flag: bool) -> Option<bool> {
        let old = self.vector.replace(id.into_id().index(), flag);

        Some(old)
    }
}

impl<N, E, D> FlaggableGraphId<DinoStorage<N, E, D>> for NodeId
where
    D: GraphDirectionality,
{
    type Store<'a> = FlagStore where
        DinoStorage<N, E, D>: 'a;

    fn flag_store(storage: &DinoStorage<N, E, D>) -> Self::Store<'_> {
        let vector = BitVec::repeat(false, storage.num_nodes()).into_boxed_bitslice();

        Self::Store { vector }
    }
}

pub struct AttributeStoreIter<'a, T> {
    iter: Enumerate<core::slice::Iter<'a, Option<(Generation, T)>>>,
}

impl<'a, T> Iterator for AttributeStoreIter<'a, T> {
    type Item = (MaybeOwned<'a, NodeId>, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (index, item) = self.iter.next()?;

            if let Some((generation, item)) = item {
                let id = EntryId::new(*generation, index)?;
                let id = NodeId::from_id(id);

                return Some((MaybeOwned::Owned(id), item));
            }
        }
    }
}

pub struct AttributeStore<T> {
    items: Vec<Option<(Generation, T)>>,
}

impl<T> AttributeStorage<NodeId, T> for AttributeStore<T> {
    type Iter<'a> = AttributeStoreIter<'a, T> where NodeId: 'a, T: 'a, Self: 'a;

    fn get(&self, id: &NodeId) -> Option<&T> {
        if id.into_id().index() >= self.items.len() {
            return None;
        }

        self.items[id.into_id().index()]
            .as_ref()
            .map(|(_, item)| item)
    }

    fn get_mut(&mut self, id: &NodeId) -> Option<&mut T> {
        if id.into_id().index() >= self.items.len() {
            return None;
        }

        self.items[id.into_id().index()]
            .as_mut()
            .map(|(_, item)| item)
    }

    fn index(&self, id: &NodeId) -> &T {
        self.items[id.into_id().index()]
            .as_ref()
            .map(|(_, item)| item)
            .expect("index called on empty node")
    }

    fn index_mut(&mut self, id: &NodeId) -> &mut T {
        self.items[id.into_id().index()]
            .as_mut()
            .map(|(_, item)| item)
            .expect("index_mut called on empty node")
    }

    fn set(&mut self, id: &NodeId, value: T) -> Option<T> {
        let index = id.into_id().index();
        let generation = id.into_id().generation();

        if index >= self.items.len() {
            return None;
        }

        core::mem::replace(&mut self.items[index], Some((generation, value))).map(|(_, item)| item)
    }

    fn remove(&mut self, id: &NodeId) -> Option<T> {
        let index = id.into_id().index();

        if index >= self.items.len() {
            return None;
        }

        self.items[index].take().map(|(_, item)| item)
    }

    fn iter(&self) -> Self::Iter<'_> {
        Self::Iter {
            iter: self.items.iter().enumerate(),
        }
    }
}

impl<N, E, D> AttributeGraphId<DinoStorage<N, E, D>> for NodeId
where
    D: GraphDirectionality,
{
    type Store<'a, T> = AttributeStore<T> where DinoStorage<N, E, D>: 'a;

    fn attribute_store<T>(storage: &DinoStorage<N, E, D>) -> Self::Store<'_, T> {
        let mut items = Vec::with_capacity(storage.nodes.total_len());
        items.resize_with(storage.nodes.total_len(), || None);

        Self::Store { items }
    }
}

impl ManagedGraphId for NodeId {}

pub(crate) type NodeSlab<T> = crate::slab::Slab<NodeId, Node<T>>;

#[derive(Debug, Clone)]
pub(crate) struct NodeClosures {
    pub(crate) outgoing_nodes: Vec<NodeId>,
    pub(crate) incoming_nodes: Vec<NodeId>,

    pub(crate) outgoing_edges: Vec<EdgeId>,
    pub(crate) incoming_edges: Vec<EdgeId>,
}

impl NodeClosures {
    fn new() -> Self {
        Self {
            outgoing_nodes: Vec::new(),
            incoming_nodes: Vec::new(),

            outgoing_edges: Vec::new(),
            incoming_edges: Vec::new(),
        }
    }

    pub(crate) fn outgoing_neighbours(&self) -> NodeIdClosureIter {
        self.outgoing_nodes.iter().copied()
    }

    pub(crate) fn incoming_neighbours(&self) -> NodeIdClosureIter {
        self.incoming_nodes.iter().copied()
    }

    pub(crate) fn neighbours(&self) -> impl Iterator<Item = NodeId> + '_ {
        todo!();
        core::iter::empty()
    }

    pub(crate) fn outgoing_edges(&self) -> EdgeIdClosureIter {
        self.outgoing_edges.iter().copied()
    }

    pub(crate) fn incoming_edges(&self) -> EdgeIdClosureIter {
        self.incoming_edges.iter().copied()
    }

    pub(crate) fn edges(&self) -> impl Iterator<Item = EdgeId> + '_ {
        todo!();
        core::iter::empty()
    }

    pub(crate) fn clear(&mut self) {
        self.outgoing_nodes.clear();
        self.incoming_nodes.clear();

        self.outgoing_edges.clear();
        self.incoming_edges.clear();
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
        self.closures.outgoing_edges()
    }

    pub(crate) fn incoming_edges(&self) -> impl Iterator<Item = EdgeId> + '_ {
        self.closures.incoming_edges()
    }

    pub(crate) fn edges(&self) -> impl Iterator<Item = EdgeId> + '_ {
        self.closures.edges()
    }

    pub(crate) fn is_isolated(&self) -> bool {
        self.closures.outgoing_nodes.is_empty() && self.closures.incoming_nodes.is_empty()
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
