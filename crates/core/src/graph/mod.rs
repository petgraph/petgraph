pub mod adaptors;
mod adjacent;
mod directed;
mod disjoint;
mod undirected;

use core::borrow::{Borrow, BorrowMut};

pub use self::{
    adjacent::{Predecessors, Successors},
    directed::DirectedGraph,
    disjoint::DisjointMutGraph,
    undirected::UndirectedGraph,
};
use crate::id::Id;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Cardinality {
    /// Number of nodes in the graph
    pub order: usize,
    /// Number of edges in the graph
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
    type NodeDataRef<'graph>: Borrow<Self::NodeData<'graph>>
    where
        Self: 'graph;
    type NodeDataMut<'graph>: BorrowMut<Self::NodeData<'graph>>
    where
        Self: 'graph;

    type EdgeId: Id;
    type EdgeData<'graph>
    where
        Self: 'graph;
    type EdgeDataRef<'graph>: Borrow<Self::EdgeData<'graph>>
    where
        Self: 'graph;
    type EdgeDataMut<'graph>: BorrowMut<Self::EdgeData<'graph>>
    where
        Self: 'graph;
}

macro_rules! delegate_impl {
    ($G:ident) => {
        type NodeId = <$G>::NodeId;
        type NodeData<'graph>
            = <$G>::NodeData<'graph>
        where
            Self: 'graph;
        type NodeDataRef<'graph>
            = <$G>::NodeDataRef<'graph>
        where
            Self: 'graph;
        type NodeDataMut<'graph>
            = <$G>::NodeDataMut<'graph>
        where
            Self: 'graph;

        type EdgeId = <$G>::EdgeId;
        type EdgeData<'graph>
            = <$G>::EdgeData<'graph>
        where
            Self: 'graph;
        type EdgeDataRef<'graph>
            = <$G>::EdgeDataRef<'graph>
        where
            Self: 'graph;
        type EdgeDataMut<'graph>
            = <$G>::EdgeDataMut<'graph>
        where
            Self: 'graph;
    };
}

pub(crate) use delegate_impl;

impl<G: Graph> Graph for &G {
    delegate_impl!(G);
}

impl<G: Graph> Graph for &mut G {
    delegate_impl!(G);
}

#[cfg(feature = "alloc")]
impl<G: Graph> Graph for alloc::boxed::Box<G> {
    delegate_impl!(G);
}

#[cfg(feature = "alloc")]
impl<G: Graph> Graph for alloc::rc::Rc<G> {
    delegate_impl!(G);
}

#[cfg(feature = "alloc")]
impl<G: Graph> Graph for alloc::sync::Arc<G> {
    delegate_impl!(G);
}
