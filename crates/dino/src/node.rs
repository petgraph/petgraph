use core::fmt::{Display, Formatter};

use petgraph_core::{
    attributes::NoValue,
    edge::marker::GraphDirectionality,
    id::{AttributeGraphId, FlaggableGraphId, GraphId, LinearGraphId, ManagedGraphId},
};

use crate::{
    closure::UniqueVec,
    iter::closure::{
        EdgeBetweenIterator, EdgeIdClosureIter, EdgeIntersectionIterator, EdgeIterator,
        NeighbourIterator, NodeIdClosureIter,
    },
    slab::{
        secondary::{SlabAttributeStorage, SlabFlagStorage},
        EntryId, Key, SlabIndexMapper,
    },
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

pub(crate) type NodeSlab<T> = crate::slab::Slab<NodeId, Node<T>>;

#[derive(Debug, Clone)]
pub(crate) struct NodeClosures {
    outgoing_nodes: UniqueVec<NodeId>,
    incoming_nodes: UniqueVec<NodeId>,

    outgoing_edges: UniqueVec<EdgeId>,
    incoming_edges: UniqueVec<EdgeId>,
}

impl NodeClosures {
    const fn new() -> Self {
        Self {
            outgoing_nodes: UniqueVec::new(),
            incoming_nodes: UniqueVec::new(),

            outgoing_edges: UniqueVec::new(),
            incoming_edges: UniqueVec::new(),
        }
    }

    pub(crate) fn insert_outgoing_node(&mut self, node: NodeId) {
        self.outgoing_nodes.insert(node);
    }

    pub(crate) fn remove_outgoing_node(&mut self, node: NodeId) {
        self.outgoing_nodes.remove(&node);
    }

    pub(crate) fn insert_incoming_node(&mut self, node: NodeId) {
        self.incoming_nodes.insert(node);
    }

    pub(crate) fn remove_incoming_node(&mut self, node: NodeId) {
        self.incoming_nodes.remove(&node);
    }

    pub(crate) fn insert_outgoing_edge(&mut self, edge: EdgeId) {
        self.outgoing_edges.insert(edge);
    }

    pub(crate) fn remove_outgoing_edge(&mut self, edge: EdgeId) {
        self.outgoing_edges.remove(&edge);
    }

    pub(crate) fn insert_incoming_edge(&mut self, edge: EdgeId) {
        self.incoming_edges.insert(edge);
    }

    pub(crate) fn remove_incoming_edge(&mut self, edge: EdgeId) {
        self.incoming_edges.remove(&edge);
    }

    pub(crate) fn outgoing_nodes(&self) -> NodeIdClosureIter {
        self.outgoing_nodes.iter().copied()
    }

    pub(crate) fn incoming_nodes(&self) -> NodeIdClosureIter {
        self.incoming_nodes.iter().copied()
    }

    pub(crate) fn neighbours(&self) -> NeighbourIterator {
        NeighbourIterator::new(
            self.outgoing_nodes.iter().copied(),
            self.incoming_nodes.iter().copied(),
        )
    }

    pub(crate) fn outgoing_edges(&self) -> EdgeIdClosureIter {
        self.outgoing_edges.iter().copied()
    }

    pub(crate) fn incoming_edges(&self) -> EdgeIdClosureIter {
        self.incoming_edges.iter().copied()
    }

    pub(crate) fn edges(&self) -> EdgeIterator {
        EdgeIterator::new(
            self.outgoing_edges.iter().copied(),
            self.incoming_edges.iter().copied(),
        )
    }

    pub(crate) fn edges_between_undirected<'a>(
        &'a self,
        other: &'a Self,
    ) -> EdgeBetweenIterator<'a> {
        EdgeBetweenIterator::new(self, other)
    }

    pub(crate) fn edges_between_directed<'a>(
        &'a self,
        other: &'a Self,
    ) -> EdgeIntersectionIterator<'a> {
        EdgeIntersectionIterator::new(
            self.outgoing_edges.iter().copied(),
            other.incoming_edges.iter().copied(),
        )
    }

    pub(crate) fn is_isolated(&self) -> bool {
        self.outgoing_nodes.is_empty() && self.incoming_nodes.is_empty()
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
    pub(crate) const fn new(id: NodeId, weight: T) -> Self {
        Self {
            id,
            weight,

            closures: NodeClosures::new(),
        }
    }
}

impl<T> PartialEq for Node<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        (self.id, &self.weight) == (other.id, &other.weight)
    }
}

impl<T> Eq for Node<T> where T: Eq {}

impl<T> PartialOrd for Node<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        (self.id, &self.weight).partial_cmp(&(other.id, &other.weight))
    }
}

impl<T> Ord for Node<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        (self.id, &self.weight).cmp(&(other.id, &other.weight))
    }
}
