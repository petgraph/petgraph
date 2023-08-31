use error_stack::Result;

use crate::{
    edge::{Direction, Edge},
    graph::Graph,
    storage::{DirectedGraphStorage, GraphStorage},
};

pub struct Node<'a, S>
where
    S: GraphStorage,
{
    storage: &'a S,

    id: &'a S::NodeId,

    weight: &'a S::NodeWeight,
}

impl<'a, S> Node<'a, S>
where
    S: GraphStorage,
{
    pub const fn new(storage: &'a S, id: &'a S::NodeId, weight: &'a S::NodeWeight) -> Self {
        Self {
            storage,
            id,
            weight,
        }
    }

    #[must_use]
    pub const fn id(&self) -> &'a S::NodeId {
        self.id
    }

    #[must_use]
    pub const fn weight(&self) -> &'a S::NodeWeight {
        self.weight
    }
}

impl<'a, S> Node<'a, S>
where
    S: GraphStorage,
{
    pub fn neighbours(&self) -> impl Iterator<Item = Node<'_, S>> {
        self.storage.node_neighbours(self.id)
    }
}

impl<'a, S> Node<'a, S>
where
    S: DirectedGraphStorage,
{
    // TODO: rename?! think about if they are actually possible in a mutable context?!
    pub fn outgoing(&self) -> impl Iterator<Item = Edge<'_, S>> {
        self.storage
            .node_directed_connections(self.id, Direction::Outgoing)
    }

    pub fn incoming(&self) -> impl Iterator<Item = Edge<'_, S>> {
        self.storage
            .node_directed_connections(self.id, Direction::Incoming)
    }

    pub fn neighbours_directed(&self, direction: Direction) -> impl Iterator<Item = Node<'_, S>> {
        self.storage.node_directed_neighbours(self.id, direction)
    }
}

impl<S> Node<'_, S>
where
    S: GraphStorage,
    S::NodeId: Clone,
    S::NodeWeight: Clone,
{
    #[must_use]
    pub fn detach(&self) -> DetachedNode<S::NodeId, S::NodeWeight> {
        DetachedNode::new(self.id.clone(), self.weight.clone())
    }
}

pub struct NodeMut<'a, S>
where
    S: GraphStorage,
{
    // TODO: can we include the graph here?
    id: &'a S::NodeId,

    weight: &'a mut S::NodeWeight,
}

impl<'a, S> NodeMut<'a, S>
where
    S: GraphStorage,
{
    pub fn new(id: &'a S::NodeId, weight: &'a mut S::NodeWeight) -> Self {
        Self { id, weight }
    }

    #[must_use]
    pub fn into_ref(self, storage: &'a S) -> Node<'a, S> {
        Node::new(storage, self.id, self.weight)
    }

    #[must_use]
    pub const fn id(&self) -> &'a S::NodeId {
        self.id
    }

    #[must_use]
    pub fn weight(&self) -> &S::NodeWeight {
        self.weight
    }

    pub fn weight_mut(&mut self) -> &mut S::NodeWeight {
        self.weight
    }
}

impl<S> NodeMut<'_, S>
where
    S: GraphStorage,
    S::NodeId: Clone,
    S::NodeWeight: Clone,
{
    #[must_use]
    pub fn detach(&self) -> DetachedNode<S::NodeId, S::NodeWeight> {
        DetachedNode::new(self.id.clone(), self.weight.clone())
    }
}

// TODO: methods to get the neighbour, outgoing and incoming connections, etc.

pub struct DetachedNode<N, W> {
    pub id: N,

    pub weight: W,
}

impl<N, W> DetachedNode<N, W> {
    pub const fn new(id: N, weight: W) -> Self {
        Self { id, weight }
    }
}
