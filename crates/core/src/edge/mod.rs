mod direction;

pub use direction::Direction;

use crate::{
    node::{Node, NodeMut},
    storage::GraphStorage,
};

type DetachedStorageEdge<S> = DetachedEdge<
    <S as GraphStorage>::EdgeId,
    <S as GraphStorage>::NodeId,
    <S as GraphStorage>::EdgeWeight,
>;

pub struct Edge<'a, S>
where
    S: GraphStorage,
{
    // TODO: should this be `graph` instead of `storage`?
    storage: &'a S,

    pub id: &'a S::EdgeId,

    source_id: &'a S::NodeId,
    target_id: &'a S::NodeId,

    pub weight: &'a S::EdgeWeight,
}

impl<'a, S> Edge<'a, S>
where
    S: GraphStorage,
{
    pub const fn new(
        storage: &'a S,

        id: &'a S::EdgeId,

        source_id: &'a S::NodeId,
        target_id: &'a S::NodeId,

        weight: &'a S::EdgeWeight,
    ) -> Self {
        Self {
            storage,

            id,

            source_id,
            target_id,

            weight,
        }
    }

    #[must_use]
    pub const fn id(&self) -> &'a S::EdgeId {
        self.id
    }

    #[must_use]
    pub const fn source_id(&self) -> &'a S::NodeId {
        self.source_id
    }

    #[must_use]
    pub fn source(&self) -> Option<Node<'a, S>> {
        self.storage.node(self.source_id)
    }

    #[must_use]
    pub const fn target_id(&self) -> &'a S::NodeId {
        self.target_id
    }

    #[must_use]
    pub fn target(&self) -> Option<Node<'a, S>> {
        self.storage.node(self.target_id)
    }

    #[must_use]
    pub const fn weight(&self) -> &'a S::EdgeWeight {
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
    pub fn detach(self) -> DetachedStorageEdge<S> {
        DetachedEdge::new(
            self.id.clone(),
            self.source_id.clone(),
            self.target_id.clone(),
            self.weight.clone(),
        )
    }
}

pub struct UnboundEdge<'a, S>
where
    S: GraphStorage,
{
    id: &'a S::EdgeId,

    source_id: &'a S::NodeId,
    target_id: &'a S::NodeId,

    weight: &'a S::EdgeWeight,
}

impl<'a, S> UnboundEdge<'a, S>
where
    S: GraphStorage,
{
    fn new(
        id: &'a S::EdgeId,

        source_id: &'a S::NodeId,
        target_id: &'a S::NodeId,

        weight: &'a S::EdgeWeight,
    ) -> Self {
        Self {
            id,

            source_id,
            target_id,

            weight,
        }
    }

    pub fn bind(self, storage: &'a S) -> Edge<'a, S> {
        Edge::new(
            storage,
            self.id,
            self.source_id,
            self.target_id,
            self.weight,
        )
    }
}

pub struct EdgeMut<'a, S>
where
    S: GraphStorage,
{
    // TODO: can we include the graph here?
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
        id: &'a S::EdgeId,

        source_id: &'a S::NodeId,
        target_id: &'a S::NodeId,

        weight: &'a mut S::EdgeWeight,
    ) -> Self {
        Self {
            id,

            source_id,
            target_id,

            weight,
        }
    }

    #[must_use]
    pub fn downgrade(self) -> UnboundEdge<'a, S> {
        UnboundEdge::new(self.id, self.source_id, self.target_id, &*self.weight)
    }

    #[must_use]
    pub const fn id(&self) -> &'a S::EdgeId {
        self.id
    }

    #[must_use]
    pub const fn source_id(&self) -> &'a S::NodeId {
        self.source_id
    }

    #[must_use]
    pub const fn target_id(&self) -> &'a S::NodeId {
        self.target_id
    }

    #[must_use]
    pub fn weight(&self) -> &S::EdgeWeight {
        self.weight
    }

    pub fn weight_mut(&mut self) -> &mut S::EdgeWeight {
        self.weight
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
    pub fn detach(self) -> DetachedStorageEdge<S> {
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
    pub const fn new(id: E, source: N, target: N, weight: W) -> Self {
        Self {
            id,
            source,
            target,
            weight,
        }
    }
}
