use alloc::vec::Vec;
use core::fmt::{Display, Formatter};

use bitvec::{boxed::BitBox, vec::BitVec};
use petgraph_core::{
    attributes::NoValue,
    edge::marker::GraphDirectionality,
    id::{FlagStorage, FlaggableGraphId, GraphId, IndexMapper, LinearGraphId, ManagedGraphId},
    GraphStorage,
};

use crate::{
    iter::closure::{EdgeIdClosureIter, NodeIdClosureIter},
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

// remove of mapper gives us ~3s speedup on the benchmark
pub struct FlagStore<'a> {
    vector: BitBox,
    mapper: SlabIndexMapper<'a, NodeId>,
}

impl FlagStorage<NodeId> for FlagStore<'_> {
    fn get(&self, id: &NodeId) -> Option<bool> {
        let index = self.mapper.get(id)?;

        Some(self.vector[index])
    }

    fn set(&mut self, id: &NodeId, flag: bool) -> Option<bool> {
        let index = self.mapper.get(id)?;

        let old = self.vector.replace(index, flag);

        Some(old)
    }
}

impl<N, E, D> FlaggableGraphId<DinoStorage<N, E, D>> for NodeId
where
    D: GraphDirectionality,
{
    type Store<'a> = FlagStore<'a> where
        DinoStorage<N, E, D>: 'a;

    fn flag_store(storage: &DinoStorage<N, E, D>) -> Self::Store<'_> {
        let mapper = Self::index_mapper(storage);

        let vector = BitVec::repeat(false, storage.num_nodes()).into_boxed_bitslice();

        Self::Store { vector, mapper }
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
