use core::hash::Hash;

use petgraph_core::{
    DirectedGraph, Edge, EdgeMut, Node, NodeMut,
    edge::{Direction, marker::Directed},
    node::NodeId,
};

use crate::EntryStorage;

impl<NK, NV, EK, EV> DirectedGraph for EntryStorage<NK, NV, EK, EV, Directed>
where
    NK: Hash,
    EK: Hash,
{
    fn directed_edges_between(
        &self,
        source: NodeId,
        target: NodeId,
    ) -> impl Iterator<Item = Edge<'_, Self>> {
        self.inner
            .directed_edges_between(source, target)
            .map(|edge| edge.change_storage_unchecked(self))
    }

    fn directed_edges_between_mut(
        &mut self,
        source: NodeId,
        target: NodeId,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.inner
            .directed_edges_between_mut(source, target)
            .map(EdgeMut::change_storage_unchecked)
    }

    fn node_directed_connections(
        &self,
        id: NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = Edge<'_, Self>> {
        self.inner
            .node_directed_connections(id, direction)
            .map(|edge| edge.change_storage_unchecked(self))
    }

    fn node_directed_connections_mut(
        &mut self,
        id: NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.inner
            .node_directed_connections_mut(id, direction)
            .map(EdgeMut::change_storage_unchecked)
    }

    fn node_directed_degree(&self, id: NodeId, direction: Direction) -> usize {
        self.inner.node_directed_degree(id, direction)
    }

    fn node_directed_neighbours(
        &self,
        id: NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = Node<'_, Self>> {
        self.inner
            .node_directed_neighbours(id, direction)
            .map(|node| node.change_storage_unchecked(self))
    }

    fn node_directed_neighbours_mut(
        &mut self,
        id: NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = NodeMut<'_, Self>> {
        self.inner
            .node_directed_neighbours_mut(id, direction)
            .map(NodeMut::change_storage_unchecked)
    }
}
