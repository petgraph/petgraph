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

    id: &'a S::EdgeId,

    source_id: &'a S::NodeId,
    target_id: &'a S::NodeId,

    weight: &'a S::EdgeWeight,
}

impl<'a, S> Edge<'a, S>
where
    S: GraphStorage,
{
    pub const fn new(
        graph: &'a Graph<S>,

        id: &'a S::EdgeId,

        source_id: &'a S::NodeId,
        target_id: &'a S::NodeId,

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

    #[must_use]
    pub fn graph(&self) -> &'a Graph<S> {
        self.graph
    }

    #[must_use]
    pub fn id(&self) -> &'a S::EdgeId {
        self.id
    }

    #[must_use]
    pub fn source_id(&self) -> &'a S::NodeId {
        self.source_id
    }

    #[must_use]
    pub fn source(&self) -> Option<Node<'a, S>> {
        // self.graph.node(self.source_id)
        todo!()
    }

    #[must_use]
    pub fn target_id(&self) -> &'a S::NodeId {
        self.target_id
    }

    #[must_use]
    pub fn target(&self) -> Option<Node<'a, S>> {
        todo!()
    }

    #[must_use]
    pub fn weight(&self) -> &'a S::EdgeWeight {
        self.weight
    }
}

impl<S> Edge<'_, S>
where
    S: GraphStorage,
    S::NodeId: Clone,
    S::EdgeId: Clone,
    S::EdgeWeight: Clone,
{
    #[must_use]
    pub fn detach(self) -> DetachedEdge<S::EdgeId, S::NodeId, S::EdgeWeight> {
        DetachedEdge::new(
            self.id.clone(),
            self.source_id.clone(),
            self.target_id.clone(),
            self.weight.clone(),
        )
    }
}

pub struct EdgeMut<'a, S>
where
    S: GraphStorage,
{
    // TODO: can this be mut?
    graph: &'a Graph<S>,

    id: &'a S::EdgeId,

    source_id: &'a S::NodeId,
    target_id: &'a S::NodeId,

    weight: &'a mut S::EdgeWeight,
}

impl<'a, S> EdgeMut<'a, S>
where
    S: GraphStorage,
{
    pub fn new(
        graph: &'a Graph<S>,

        id: &'a S::EdgeId,

        source_id: &'a S::NodeId,
        target_id: &'a S::NodeId,

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

    #[must_use]
    pub fn into_ref(self) -> Edge<'a, S> {
        Edge::new(
            self.graph,
            self.id,
            self.source_id,
            self.target_id,
            self.weight,
        )
    }

    #[must_use]
    pub fn id(&self) -> &'a S::EdgeId {
        self.id
    }

    #[must_use]
    pub fn source_id(&self) -> &'a S::NodeId {
        self.source_id
    }

    pub fn source_mut(&mut self) -> Option<NodeMut<'a, S>> {
        todo!()
    }

    #[must_use]
    pub fn source(&self) -> Option<Node<'a, S>> {
        todo!()
    }

    #[must_use]
    pub fn target_id(&self) -> &'a S::NodeId {
        self.target_id
    }

    pub fn target_mut(&mut self) -> Option<NodeMut<'a, S>> {
        todo!()
    }

    #[must_use]
    pub fn target(&self) -> Option<Node<'a, S>> {
        todo!()
    }

    #[must_use]
    pub fn weight(&self) -> &S::EdgeWeight {
        self.weight
    }

    pub fn weight_mut(&mut self) -> &mut S::EdgeWeight {
        self.weight
    }

    pub fn remove(self) -> Result<DetachedEdge<S::EdgeId, S::NodeId, S::EdgeWeight>, S::Error> {
        todo!("remove edge")
    }
}

impl<S> EdgeMut<'_, S>
where
    S: GraphStorage,
    S::NodeId: Clone,
    S::EdgeId: Clone,
    S::EdgeWeight: Clone,
{
    #[must_use]
    pub fn detach(self) -> DetachedEdge<S::EdgeId, S::NodeId, S::EdgeWeight> {
        DetachedEdge::new(
            self.id.clone(),
            self.source_id.clone(),
            self.target_id.clone(),
            self.weight.clone(),
        )
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
