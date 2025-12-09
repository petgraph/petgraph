use crate::{graph::Graph, id::Id};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Direction {
    Incoming,
    Outgoing,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Edge<I, D, N> {
    pub id: I,

    pub source: N,
    pub target: N,

    pub data: D,
}

impl<I, D, N> Edge<I, D, N> {
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

pub type EdgeRef<'graph, G> =
    Edge<<G as Graph>::EdgeId, <G as Graph>::EdgeDataRef<'graph>, <G as Graph>::NodeId>;
pub type EdgeMut<'graph, G> =
    Edge<<G as Graph>::EdgeId, <G as Graph>::EdgeDataMut<'graph>, <G as Graph>::NodeId>;
