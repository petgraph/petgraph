use crate::{graph::Graph, storage::GraphStorage};

pub struct Node<'a, S>
where
    S: GraphStorage,
{
    graph: &'a Graph<S>,

    id: &'a S::NodeIndex,

    weight: &'a S::NodeWeight,
}

impl<'a, S> Node<'a, S>
where
    S: GraphStorage,
{
    pub fn new(graph: &'a Graph<S>, id: &'a S::NodeIndex, weight: &'a S::NodeWeight) -> Self {
        Self { graph, id, weight }
    }

    pub fn id(&self) -> &'a S::NodeIndex {
        self.id
    }

    pub fn weight(&self) -> &'a S::NodeWeight {
        self.weight
    }
}

pub struct NodeMut<'a, S>
where
    S: GraphStorage,
{
    graph: &'a mut Graph<S>,

    id: &'a S::NodeIndex,

    weight: &'a mut S::NodeWeight,
}

impl<'a, S> NodeMut<'a, S>
where
    S: GraphStorage,
{
    pub fn new(
        graph: &'a mut Graph<S>,
        id: &'a S::NodeIndex,
        weight: &'a mut S::NodeWeight,
    ) -> Self {
        Self { graph, id, weight }
    }

    pub fn as_ref(&self) -> Node<'_, S> {
        Node::new(self.graph, self.id, self.weight)
    }

    pub fn id(&self) -> &'a S::NodeIndex {
        self.id
    }

    pub fn weight(&self) -> &'a S::NodeWeight {
        self.weight
    }

    pub fn weight_mut(&mut self) -> &'a mut S::NodeWeight {
        self.weight
    }
}

pub struct DetachedNode<N, W> {
    pub id: N,

    pub weight: W,
}

impl<N, W> DetachedNode<N, W> {
    pub fn new(id: N, weight: W) -> Self {
        Self { id, weight }
    }
}
