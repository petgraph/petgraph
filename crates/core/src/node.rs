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
    graph: &'a Graph<S>,

    id: &'a S::NodeId,

    weight: &'a S::NodeWeight,
}

impl<'a, S> Node<'a, S>
where
    S: GraphStorage,
{
    pub fn new(graph: &'a Graph<S>, id: &'a S::NodeId, weight: &'a S::NodeWeight) -> Self {
        Self { graph, id, weight }
    }

    #[must_use] pub fn id(&self) -> &'a S::NodeId {
        self.id
    }

    #[must_use] pub fn weight(&self) -> &'a S::NodeWeight {
        self.weight
    }
}

impl<S> Node<'_, S>
where
    S: GraphStorage,
    S::NodeId: Clone,
    S::NodeWeight: Clone,
{
    #[must_use] pub fn detach(&self) -> DetachedNode<S::NodeId, S::NodeWeight> {
        DetachedNode::new(self.id.clone(), self.weight.clone())
    }
}

pub struct NodeMut<'a, S>
where
    S: GraphStorage,
{
    // TODO: can this be mut?
    graph: &'a Graph<S>,

    id: &'a S::NodeId,

    weight: &'a mut S::NodeWeight,
}

impl<'a, S> NodeMut<'a, S>
where
    S: GraphStorage,
{
    pub fn new(graph: &'a Graph<S>, id: &'a S::NodeId, weight: &'a mut S::NodeWeight) -> Self {
        Self { graph, id, weight }
    }

    #[must_use] pub fn into_ref(self) -> Node<'a, S> {
        Node::new(self.graph, self.id, self.weight)
    }

    #[must_use] pub fn id(&self) -> &'a S::NodeId {
        self.id
    }

    #[must_use] pub fn weight(&self) -> &S::NodeWeight {
        self.weight
    }

    pub fn weight_mut(&mut self) -> &mut S::NodeWeight {
        self.weight
    }

    pub fn neighbours(&self) -> impl Iterator<Item = Node<'_, S>> {
        self.graph.neighbours(self.id)
    }

    pub fn remove(self) -> Result<DetachedNode<S::NodeId, S::NodeWeight>, S::Error> {
        todo!("remove node")
    }
}

impl<'a, S> NodeMut<'a, S>
where
    S: DirectedGraphStorage,
{
    // TODO: rename?! think about if they are actually possible in a mutable context?!
    pub fn outgoing(&self) -> impl Iterator<Item = Edge<'_, S>> {
        self.graph
            .connections_directed(self.id, Direction::Outgoing)
    }

    pub fn incoming(&self) -> impl Iterator<Item = Edge<'_, S>> {
        self.graph
            .connections_directed(self.id, Direction::Incoming)
    }
}

impl<S> NodeMut<'_, S>
where
    S: GraphStorage,
    S::NodeId: Clone,
    S::NodeWeight: Clone,
{
    #[must_use] pub fn detach(&self) -> DetachedNode<S::NodeId, S::NodeWeight> {
        DetachedNode::new(self.id.clone(), self.weight.clone())
    }
}

// TODO: methods to get the neighbour, outgoing and incoming connections, etc.

pub struct DetachedNode<N, W> {
    pub id: N,

    pub weight: W,
}

impl<N, W> DetachedNode<N, W> {
    pub fn new(id: N, weight: W) -> Self {
        Self { id, weight }
    }
}
