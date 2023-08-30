use error_stack::Result;

use crate::{
    edge::{Direction, Edge, EdgeMut},
    matrix::AdjacencyMatrix,
    node::{Node, NodeMut},
    storage::GraphStorage,
};

pub trait DirectedGraphStorage: GraphStorage {
    type FindDirectedEdgeIter<'a>: Iterator<Item = Edge<'a, Self>> + 'a
    where
        Self: 'a;

    fn find_directed_edges<'a>(
        &'a self,
        source: &'a Self::NodeIndex,
        target: &'a Self::NodeIndex,
    ) -> Self::FindDirectedEdgeIter<'a> {
        self.node_directed_connections(source, Direction::Outgoing)
            .filter(move |edge| edge.target_id() == target)
    }

    fn directed_adjacency_matrix(&self) -> AdjacencyMatrix<Self::NodeIndex> {
        let mut matrix = AdjacencyMatrix::new_directed(self.num_nodes());

        for edge in self.edges() {
            matrix.mark(edge);
        }

        matrix
    }

    type NodeDirectedConnectionIter<'a>: Iterator<Item = Edge<'a, Self>> + 'a
    where
        Self: 'a;

    fn node_directed_connections<'a>(
        &self,
        id: &'a Self::NodeIndex,
        direction: Direction,
    ) -> Self::NodeDirectedConnectionIter<'a>;

    type NodeDirectedConnectionMutIter<'a>: Iterator<Item = EdgeMut<'a, Self>> + 'a
    where
        Self: 'a;

    fn node_directed_connections_mut<'a>(
        &mut self,
        id: &'a Self::NodeIndex,
        direction: Direction,
    ) -> Self::NodeDirectedConnectionMutIter<'a>;

    type NodeDirectedNeighbourIter<'a>: Iterator<Item = Node<'a, Self>> + 'a
    where
        Self: 'a;

    fn node_directed_neighbours<'a>(
        &self,
        id: &'a Self::NodeIndex,
        direction: Direction,
    ) -> Self::NodeDirectedNeighbourIter<'a> {
        self.node_directed_connections(id, direction)
            .map(|edge| match direction {
                Direction::Outgoing => edge.target(),
                Direction::Incoming => edge.source(),
            })
    }

    type NodeDirectedNeighbourMutIter<'a>: Iterator<Item = NodeMut<'a, Self>> + 'a
    where
        Self: 'a;

    fn node_directed_neighbours_mut<'a>(
        &mut self,
        id: &'a Self::NodeIndex,
        direction: Direction,
    ) -> Self::NodeDirectedNeighbourMutIter<'a> {
        self.node_directed_connections_mut(id, direction)
            .map(|mut edge| match direction {
                Direction::Outgoing => edge.target_mut(),
                Direction::Incoming => edge.source_mut(),
            })
    }
}
