mod compat;
mod direction;
pub mod marker;

use core::fmt::{Debug, Formatter};

pub use direction::Direction;

use crate::{node::Node, storage::GraphStorage};

type DetachedStorageEdge<S> = DetachedEdge<
    <S as GraphStorage>::EdgeId,
    <S as GraphStorage>::NodeId,
    <S as GraphStorage>::EdgeWeight,
>;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Edge<'a, S>
where
    S: GraphStorage,
{
    storage: &'a S,

    id: &'a S::EdgeId,

    source_id: &'a S::NodeId,
    target_id: &'a S::NodeId,

    weight: &'a S::EdgeWeight,
}

impl<S> Clone for Edge<'_, S>
where
    S: GraphStorage,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> Copy for Edge<'_, S> where S: GraphStorage {}

impl<S> Debug for Edge<'_, S>
where
    S: GraphStorage,
    S::EdgeId: Debug,
    S::NodeId: Debug,
    S::EdgeWeight: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Edge")
            .field("id", &self.id)
            .field("source_id", &self.source_id)
            .field("target_id", &self.target_id)
            .field("weight", &self.weight)
            .finish()
    }
}

impl<'a, S> Edge<'a, S>
where
    S: GraphStorage,
{
    pub const fn new(
        storage: &'a S,

        id: &'a S::EdgeId,
        weight: &'a S::EdgeWeight,

        source_id: &'a S::NodeId,
        target_id: &'a S::NodeId,
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
            self.weight.clone(),
            self.source_id.clone(),
            self.target_id.clone(),
        )
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgeMut<'a, S>
where
    S: GraphStorage,
{
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
        weight: &'a mut S::EdgeWeight,

        source_id: &'a S::NodeId,
        target_id: &'a S::NodeId,
    ) -> Self {
        Self {
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
            self.weight.clone(),
            self.source_id.clone(),
            self.target_id.clone(),
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DetachedEdge<E, N, W> {
    pub id: E,

    pub source: N,
    pub target: N,

    pub weight: W,
}

impl<E, N, W> DetachedEdge<E, N, W> {
    pub const fn new(id: E, weight: W, source: N, target: N) -> Self {
        Self {
            id,
            source,
            target,
            weight,
        }
    }
}
