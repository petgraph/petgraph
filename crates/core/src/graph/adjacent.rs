use super::{DirectedGraph, Graph};
use crate::edge::EdgeRef;

pub trait Predecessors: Graph {
    fn predecessors(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId>;

    fn incoming_edges(&self, node: Self::NodeId) -> impl Iterator<Item = EdgeRef<'_, Self>>;
}

pub trait Successors: Graph {
    fn successors(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId>;

    fn outgoing_edges(&self, node: Self::NodeId) -> impl Iterator<Item = EdgeRef<'_, Self>>;
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
    fn incoming_edges(&self, node: Self::NodeId) -> impl Iterator<Item = EdgeRef<'_, Self>> {
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
    fn outgoing_edges(&self, node: Self::NodeId) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        <G as DirectedGraph>::outgoing_edges(self, node)
    }
}
