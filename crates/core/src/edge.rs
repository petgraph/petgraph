use core::marker::PhantomData;

use crate::{graph::Graph, id::Id};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Direction {
    Incoming,
    Outgoing,
}

/// Marker type for directed edges.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Directed {}

/// Marker type for undirected edges.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Undirected {}

#[derive(Debug, Copy, Clone, Hash)]
pub struct Edge<I, D, N, Dir> {
    pub id: I,

    pub source: N,
    pub target: N,

    pub data: D,

    direction: PhantomData<Dir>,
}

impl<I: PartialEq, D: PartialEq, N: PartialEq> PartialEq for Edge<I, D, N, Directed> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.source == other.source
            && self.target == other.target
            && self.data == other.data
    }
}
impl<I: PartialEq, D: PartialEq, N: PartialEq> Eq for Edge<I, D, N, Directed> {}

impl<I: PartialEq, D: PartialEq, N: PartialEq> PartialEq for Edge<I, D, N, Undirected> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && ((self.source == other.source && self.target == other.target)
                || (self.source == other.target && self.target == other.source))
            && self.data == other.data
    }
}
impl<I: PartialEq, D: PartialEq, N: PartialEq> Eq for Edge<I, D, N, Undirected> {}

pub type UndirEdge<I, D, N> = Edge<I, D, N, Undirected>;
pub type DirEdge<I, D, N> = Edge<I, D, N, Directed>;

impl<I, D, N, Dir> Edge<I, D, N, Dir> {
    pub const fn opposite_endpoint(&self, direction: Direction) -> N
    where
        N: Id,
    {
        match direction {
            Direction::Incoming => self.source,
            Direction::Outgoing => self.target,
        }
    }
}

pub type DirEdgeRef<'graph, G> =
    Edge<<G as Graph>::EdgeId, <G as Graph>::EdgeDataRef<'graph>, <G as Graph>::NodeId, Directed>;
pub type UndirEdgeRef<'graph, G> =
    Edge<<G as Graph>::EdgeId, <G as Graph>::EdgeDataRef<'graph>, <G as Graph>::NodeId, Undirected>;

pub type DirEdgeMut<'graph, G> =
    Edge<<G as Graph>::EdgeId, <G as Graph>::EdgeDataMut<'graph>, <G as Graph>::NodeId, Directed>;
pub type UndirEdgeMut<'graph, G> =
    Edge<<G as Graph>::EdgeId, <G as Graph>::EdgeDataMut<'graph>, <G as Graph>::NodeId, Undirected>;
