// Index into the NodeIndex and EdgeIndex arrays
/// Edge direction.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum Direction {
    /// An `Outgoing` edge is an outward edge *from* the current node.
    Outgoing,
    /// An `Incoming` edge is an inbound edge *to* the current node.
    Incoming,
}

impl Direction {
    #[inline]
    fn to_usize(self) -> usize {
        match self {
            Self::Outgoing => 0,
            Self::Incoming => 1,
        }
    }

    /// Return the opposite `Direction`.
    #[inline]
    pub fn opposite(self) -> Direction {
        match self {
            Self::Outgoing => Self::Incoming,
            Self::Incoming => Self::Outgoing,
        }
    }

    /// Return `0` for `Outgoing` and `1` for `Incoming`.
    #[inline]
    pub fn index(self) -> usize {
        self.to_usize() & 0x1
    }
}

#[deprecated(
    since = "0.1.0",
    note = "use `Direction::Incoming` or `Direction::Outgoing` instead"
)]
pub use Direction::{Incoming, Outgoing};

use crate::{
    graph::Graph,
    node::{Node, NodeMut},
    storage::GraphStorage,
};

/// Marker type for a directed graph.
#[derive(Copy, Clone, Debug)]
pub struct Directed;

/// Marker type for an undirected graph.
#[derive(Copy, Clone, Debug)]
pub struct Undirected;

/// A graph's edge type determines whether it has directed edges or not.
pub trait EdgeType {
    fn is_directed() -> bool;
}

impl EdgeType for Directed {
    #[inline]
    fn is_directed() -> bool {
        true
    }
}

impl EdgeType for Undirected {
    #[inline]
    fn is_directed() -> bool {
        false
    }
}

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

    pub fn id(&self) -> &'a S::EdgeIndex {
        self.id
    }

    pub fn source_id(&self) -> &'a S::NodeIndex {
        self.source_id
    }

    pub fn source(&self) -> Node<'a, S> {
        // self.graph.node(self.source_id)
        todo!()
    }

    pub fn target_id(&self) -> &'a S::NodeIndex {
        self.target_id
    }

    pub fn target(&self) -> Node<'a, S> {
        todo!()
    }

    pub fn weight(&self) -> &'a S::NodeWeight {
        self.weight
    }
}

pub struct EdgeMut<'a, S>
where
    S: GraphStorage,
{
    graph: &'a mut Graph<S>,

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
        graph: &'a mut Graph<S>,

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

    pub fn id(&self) -> &'a S::EdgeIndex {
        self.id
    }

    pub fn source_id(&self) -> &'a S::NodeIndex {
        self.source_id
    }

    pub fn source_mut(&mut self) -> NodeMut<'a, S> {
        todo!()
    }

    pub fn source(&self) -> Node<'a, S> {
        todo!()
    }

    pub fn target_id(&self) -> &'a S::NodeIndex {
        self.target_id
    }

    pub fn target_mut(&mut self) -> NodeMut<'a, S> {
        todo!()
    }

    pub fn target(&self) -> Node<'a, S> {
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
