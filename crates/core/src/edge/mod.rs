mod direction;

pub use direction::Direction;

use crate::{
    graph::Graph,
    node::{Node, NodeMut},
    storage::GraphStorage,
};

pub struct Edge<'a, S>
where
    S: GraphStorage,
{
    graph: &'a Graph<S>,

    id: &'a S::EdgeIndex,

    source_id: &'a S::NodeIndex,
    target_id: &'a S::NodeIndex,

    weight: &'a S::EdgeWeight,
}

impl<'a, S> Edge<'a, S>
where
    S: GraphStorage,
{
    pub fn new(
        graph: &'a Graph<S>,

        id: &'a S::EdgeIndex,

        source_id: &'a S::NodeIndex,
        target_id: &'a S::NodeIndex,

        weight: &'a S::EdgeWeight,
    ) -> Self {
        Self {
            graph,

            id,

            source_id,
            target_id,

            weight,
        }
    }

    pub fn graph(&self) -> &'a Graph<S> {
        self.graph
    }

    pub fn id(&self) -> &'a S::EdgeIndex {
        self.id
    }

    pub fn source_id(&self) -> &'a S::NodeIndex {
        self.source_id
    }

    pub fn source(&self) -> Option<Node<'a, S>> {
        // self.graph.node(self.source_id)
        todo!()
    }

    pub fn target_id(&self) -> &'a S::NodeIndex {
        self.target_id
    }

    pub fn target(&self) -> Option<Node<'a, S>> {
        todo!()
    }

    pub fn weight(&self) -> &'a S::EdgeWeight {
        self.weight
    }
}

pub struct EdgeMut<'a, S>
where
    S: GraphStorage,
{
    // TODO: can this be mut?
    graph: &'a Graph<S>,

    id: &'a S::EdgeIndex,

    source_id: &'a S::NodeIndex,
    target_id: &'a S::NodeIndex,

    weight: &'a mut S::EdgeWeight,
}

impl<'a, S> EdgeMut<'a, S>
where
    S: GraphStorage,
{
    pub fn new(
        graph: &'a Graph<S>,

        id: &'a S::EdgeIndex,

        source_id: &'a S::NodeIndex,
        target_id: &'a S::NodeIndex,

        weight: &'a mut S::EdgeWeight,
    ) -> Self {
        Self {
            graph,

            id,

            source_id,
            target_id,

            weight,
        }
    }

    pub fn as_ref(&self) -> Edge<'_, S> {
        Edge::new(
            self.graph,
            self.id,
            self.source_id,
            self.target_id,
            self.weight,
        )
    }

    pub fn id(&self) -> &'a S::EdgeIndex {
        self.id
    }

    pub fn source_id(&self) -> &'a S::NodeIndex {
        self.source_id
    }

    pub fn source_mut(&mut self) -> Option<NodeMut<'a, S>> {
        todo!()
    }

    pub fn source(&self) -> Option<Node<'a, S>> {
        todo!()
    }

    pub fn target_id(&self) -> &'a S::NodeIndex {
        self.target_id
    }

    pub fn target_mut(&mut self) -> Option<NodeMut<'a, S>> {
        todo!()
    }

    pub fn target(&self) -> Option<Node<'a, S>> {
        todo!()
    }

    pub fn weight(&self) -> &'a S::EdgeWeight {
        self.weight
    }

    pub fn weight_mut(&mut self) -> &'a mut S::EdgeWeight {
        self.weight
    }
}

pub struct DetachedEdge<E, N, W> {
    pub id: E,

    pub source: N,
    pub target: N,

    pub weight: W,
}

impl<E, N, W> DetachedEdge<E, N, W> {
    pub fn new(id: E, source: N, target: N, weight: W) -> Self {
        Self {
            id,
            source,
            target,
            weight,
        }
    }
}
