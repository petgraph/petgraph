use crate::{
    edge::{Direction, Edge, EdgeMut},
    node::{Node, NodeMut},
    storage::GraphStorage,
};

pub trait DirectedGraphStorage: GraphStorage {
    fn find_directed_edges<'a: 'b, 'b>(
        &'a self,
        source: &'b Self::NodeId,
        target: &'b Self::NodeId,
    ) -> impl Iterator<Item = Edge<'a, Self>> + 'b {
        self.node_directed_connections(source, Direction::Outgoing)
            .filter(move |edge| edge.target_id() == target)
    }

    fn node_directed_connections<'a, 'b>(
        &'a self,
        id: &'b Self::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = Edge<'a, Self>> + 'b;

    fn node_directed_connections_mut<'a: 'b, 'b>(
        &'a mut self,
        id: &'b Self::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = EdgeMut<'a, Self>> + 'b;

    fn node_directed_neighbours<'a: 'b, 'b>(
        &'a self,
        id: &'b Self::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = Node<'a, Self>> + 'b {
        self.node_directed_connections(id, direction)
            .filter_map(move |edge| match direction {
                Direction::Outgoing => edge.target(),
                Direction::Incoming => edge.source(),
            })
    }

    fn node_directed_neighbours_mut<'a: 'b, 'b>(
        &'a mut self,
        id: &'b Self::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = NodeMut<'a, Self>> + 'b {
        self.node_directed_connections_mut(id, direction)
            .filter_map(move |mut edge| match direction {
                Direction::Outgoing => edge.target_mut(),
                Direction::Incoming => edge.source_mut(),
            })
    }
}
