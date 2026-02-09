#[cfg(feature = "alloc")]
use alloc::borrow::ToOwned;

use crate::graph::Graph;

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

impl<I, D, N, DNew> Edge<I, D, N>
where
    I: Copy,
    N: Copy,
    D: ToOwned<Owned = DNew>,
    DNew: Copy,
{
    #[cfg(feature = "alloc")]
    pub fn to_owned_edge(&self) -> Edge<I, DNew, N> {
        Edge {
            id: self.id,
            source: self.source,
            target: self.target,
            data: self.data.to_owned(),
        }
    }
}

pub type EdgeRef<'graph, G> =
    Edge<<G as Graph>::EdgeId, <G as Graph>::EdgeDataRef<'graph>, <G as Graph>::NodeId>;
pub type EdgeMut<'graph, G> =
    Edge<<G as Graph>::EdgeId, <G as Graph>::EdgeDataMut<'graph>, <G as Graph>::NodeId>;
