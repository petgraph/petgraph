use error_stack::Result;

use crate::{
    edge::{Direction, Edge, EdgeMut},
    graph::Graph,
    node::{Node, NodeMut},
    storage::DirectedGraphStorage,
};

impl<S> Graph<S>
where
    S: DirectedGraphStorage,
{
    #[inline]
    pub fn neighbors_directed<'a: 'b, 'b>(
        &'a self,
        id: &'b S::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = Node<'a, S>> + 'b {
        self.neighbours_directed(id, direction)
    }

    pub fn neighbours_directed<'a: 'b, 'b>(
        &'a self,
        id: &'b S::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = Node<'a, S>> + 'b {
        self.storage.node_directed_neighbours(id, direction)
    }

    #[inline]
    pub fn neighbors_directed_mut<'a: 'b, 'b>(
        &'a mut self,
        id: &'b S::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = NodeMut<'a, S>> + 'b {
        self.neighbours_directed_mut(id, direction)
    }

    pub fn neighbours_directed_mut<'a: 'b, 'b>(
        &'a mut self,
        id: &'b S::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = NodeMut<'a, S>> + 'b {
        self.storage.node_directed_neighbours_mut(id, direction)
    }

    pub fn connections_directed<'a: 'b, 'b>(
        &'a self,
        id: &'b S::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = Edge<'a, S>> + 'b {
        self.storage.node_directed_connections(id, direction)
    }

    pub fn connections_directed_mut<'a: 'b, 'b>(
        &'a mut self,
        id: &'b S::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = EdgeMut<'a, S>> + 'b {
        self.storage.node_directed_connections_mut(id, direction)
    }

    pub fn find_directed_edges<'a: 'b, 'b>(
        &'a self,
        source: &'b S::NodeId,
        target: &'b S::NodeId,
    ) -> impl Iterator<Item = Edge<'a, S>> + 'b {
        self.storage.find_directed_edges(source, target)
    }

    // TODO: should this be part of `GraphIterator`?
    pub fn reverse(self) -> Result<Self, S::Error> {
        let (nodes, edges) = self.storage.into_parts();

        let edges = edges.map(|mut edge| {
            let source = edge.source;
            let target = edge.target;

            edge.source = target;
            edge.target = source;

            edge
        });

        Self::from_parts(nodes, edges)
    }

    // These should go into extensions:
    // into_undirected, into_directed, from_edges, extend_with_edges
}
