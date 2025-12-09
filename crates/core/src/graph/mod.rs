mod directed;
mod disjoint;
mod undirected;

pub use self::{directed::DirectedGraph, disjoint::DisjointMutGraph, undirected::UndirectedGraph};
use crate::id::Id;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Cardinality {
    pub order: usize,
    pub size: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DensityHint {
    Sparse,
    Dense,
}

pub trait Graph {
    type NodeId: Id;
    type NodeData<'graph>
    where
        Self: 'graph;
    type NodeDataRef<'graph>: AsRef<Self::NodeData<'graph>>
    where
        Self: 'graph;
    type NodeDataMut<'graph>: AsMut<Self::NodeData<'graph>>
    where
        Self: 'graph;

    type EdgeId: Id;
    type EdgeData<'graph>
    where
        Self: 'graph;
    type EdgeDataRef<'graph>: AsRef<Self::EdgeData<'graph>>
    where
        Self: 'graph;
    type EdgeDataMut<'graph>: AsMut<Self::EdgeData<'graph>>
    where
        Self: 'graph;
}
