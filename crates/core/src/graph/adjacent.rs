use super::{DirectedGraph, Graph};
use crate::edge::EdgeRef;

pub trait Predecessors: Graph {
    fn predecessors(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId>;

    fn incoming_edges<'graph_ref>(
        &'graph_ref self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'graph_ref, Self>>
    where
        Self: 'graph_ref;
}

pub trait Successors: Graph {
    fn successors(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId>;

    fn outgoing_edges<'graph_ref>(
        &'graph_ref self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'graph_ref, Self>>
    where
        Self: 'graph_ref;
}

impl<G> Predecessors for G
where
    G: DirectedGraph,
{
    #[inline]
    fn predecessors(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        <G as DirectedGraph>::predecessors(self, node)
    }

    #[inline]
    fn incoming_edges<'graph_ref>(
        &'graph_ref self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        <G as DirectedGraph>::incoming_edges(self, node)
    }
}

impl<G> Successors for G
where
    G: DirectedGraph,
{
    #[inline]
    fn successors(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        <G as DirectedGraph>::successors(self, node)
    }

    #[inline]
    fn outgoing_edges<'graph_ref>(
        &'graph_ref self,
        node: Self::NodeId,
    ) -> impl Iterator<Item = EdgeRef<'graph_ref, Self>>
    where
        Self: 'graph_ref,
    {
        <G as DirectedGraph>::outgoing_edges(self, node)
    }
}
