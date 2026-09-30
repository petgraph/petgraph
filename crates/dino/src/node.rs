use petgraph_core::id::Id;

use crate::{
    closure::UniqueVec,
    edge::DinoEdgeId,
    iter::closure::{
        EdgeBetweenIterator, EdgeIdClosureIter, EdgeIntersectionIterator, EdgeIterator,
        NeighbourIterator, NodeIdClosureIter,
    },
    slab::{EntryId, Key},
};

/// TODO: Change
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DinoNodeId(u32);

impl Key for DinoNodeId {
    fn from_id(id: EntryId) -> Self {
        Self(id.raw())
    }

    fn into_id(self) -> EntryId {
        EntryId::new_unchecked(self.0 as usize)
    }
}

impl core::fmt::Display for DinoNodeId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "DinoNodeId({})", self.0)
    }
}

impl Id for DinoNodeId {}

pub(crate) type NodeSlab<T> = crate::slab::Slab<DinoNodeId, Node<T>>;

#[derive(Debug, Clone)]
pub(crate) struct NodeClosures {
    outgoing_nodes: UniqueVec<DinoNodeId>,
    incoming_nodes: UniqueVec<DinoNodeId>,

    outgoing_edges: UniqueVec<DinoEdgeId>,
    incoming_edges: UniqueVec<DinoEdgeId>,
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

    pub(crate) fn insert_outgoing_node(&mut self, node: DinoNodeId) {
        self.outgoing_nodes.insert(node);
    }

    pub(crate) fn remove_outgoing_node(&mut self, node: DinoNodeId) {
        self.outgoing_nodes.remove(&node);
    }

    pub(crate) fn insert_incoming_node(&mut self, node: DinoNodeId) {
        self.incoming_nodes.insert(node);
    }

    pub(crate) fn remove_incoming_node(&mut self, node: DinoNodeId) {
        self.incoming_nodes.remove(&node);
    }

    pub(crate) fn insert_outgoing_edge(&mut self, edge: DinoEdgeId) {
        self.outgoing_edges.insert(edge);
    }

    pub(crate) fn remove_outgoing_edge(&mut self, edge: DinoEdgeId) {
        self.outgoing_edges.remove(&edge);
    }

    pub(crate) fn insert_incoming_edge(&mut self, edge: DinoEdgeId) {
        self.incoming_edges.insert(edge);
    }

    pub(crate) fn remove_incoming_edge(&mut self, edge: DinoEdgeId) {
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

    pub(crate) fn incident_edges(&self) -> EdgeIterator {
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
    pub(crate) id: DinoNodeId,
    pub(crate) weight: T,

    pub(crate) closures: NodeClosures,
}

impl<T> Node<T> {
    pub(crate) const fn new(id: DinoNodeId, weight: T) -> Self {
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
