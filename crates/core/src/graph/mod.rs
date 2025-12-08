mod directed;
mod disjoint;
mod undirected;

pub use self::{directed::DirectedGraph, disjoint::DisjointMutGraph, undirected::UndirectedGraph};
use crate::{
    edge::{Edge, EdgeMut},
    id::Id,
    node::{Node, NodeMut},
};

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
    type NodeRef<'graph>: Node<'graph, Id = Self::NodeId>
    where
        Self: 'graph;
    type NodeMut<'graph>: NodeMut<'graph, Id = Self::NodeId>
    where
        Self: 'graph;

    type EdgeId: Id;
    type EdgeRef<'graph>: Edge<'graph, Id = Self::EdgeId, Node = Self::NodeId>
    where
        Self: 'graph;
    type EdgeMut<'graph>: EdgeMut<'graph, Id = Self::EdgeId, Node = Self::NodeId>
    where
        Self: 'graph;
}
