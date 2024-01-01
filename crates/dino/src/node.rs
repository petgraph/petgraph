use petgraph_core::{edge::EdgeId, node::NodeId};

use crate::{
    closure::UniqueVec,
    iter::closure::{
        EdgeBetweenIterator, EdgeIdClosureIter, EdgeIntersectionIterator, EdgeIterator,
        NeighbourIterator, NodeIdClosureIter,
    },
    slab::{EntryId, Key},
};

impl Key for NodeId {
    #[inline]
    fn from_id(id: EntryId) -> Self {
        Self::new(id.into_usize())
    }

    #[inline]
    fn into_id(self) -> EntryId {
        EntryId::new_unchecked(self.into_inner())
    }
}

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
