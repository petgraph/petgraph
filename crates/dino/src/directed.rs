use core::iter::empty;

use either::Either;
use petgraph_core::{
    edge::{Direction, Edge, EdgeMut},
    node::{Node, NodeMut},
    storage::{DirectedGraphStorage, GraphStorage},
};

use crate::{DinosaurStorage, Directed};

impl<N, E> DirectedGraphStorage for DinosaurStorage<N, E, Directed> {
    fn find_directed_edges<'a: 'b, 'b>(
        &'a self,
        source: &'b Self::NodeId,
        target: &'b Self::NodeId,
    ) -> impl Iterator<Item = Edge<'a, Self>> + 'b {
        self.closures
            .edges
            .endpoints_to_edges()
            .get(&(*source, *target))
            .into_iter()
            .flatten()
            .filter_map(|id| self.edge(id))
    }

    fn node_directed_connections<'a: 'b, 'b>(
        &'a self,
        id: &'b Self::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = Edge<'a, Self>> + 'b {
        self.closures
            .nodes
            .get(*id)
            .into_iter()
            .flat_map(move |closure| match direction {
                Direction::Incoming => closure.incoming_edges(),
                Direction::Outgoing => closure.outgoing_edges(),
            })
            .filter_map(move |id| self.edge(id))
    }

    fn node_directed_connections_mut<'a: 'b, 'b>(
        &'a mut self,
        id: &'b Self::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = EdgeMut<'a, Self>> + 'b {
        let Self {
            closures, edges, ..
        } = self;

        let closure = closures.nodes.get(*id);

        let available = match (closure, direction) {
            (Some(closure), Direction::Incoming) => closure.incoming_edges(),
            (Some(closure), Direction::Outgoing) => closure.outgoing_edges(),
            (None, _) => return Either::Left(empty()),
        };

        if available.is_empty() {
            return Either::Left(empty());
        }

        Either::Right(
            edges
                .iter_mut()
                .filter(move |edge| available.contains(&edge.id))
                .map(|edge| EdgeMut::new(&edge.id, &edge.source, &edge.target, &mut edge.weight)),
        )
    }

    fn node_directed_neighbours<'a: 'b, 'b>(
        &'a self,
        id: &'b Self::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = Node<'a, Self>> + 'b {
        self.closures
            .nodes
            .get(*id)
            .into_iter()
            .flat_map(move |closure| match direction {
                Direction::Incoming => closure.incoming_neighbours(),
                Direction::Outgoing => closure.outgoing_neighbours(),
            })
            .filter_map(move |id| self.node(id))
    }

    fn node_directed_neighbours_mut<'a: 'b, 'b>(
        &'a mut self,
        id: &'b Self::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = NodeMut<'a, Self>> + 'b {
        let Self {
            closures, nodes, ..
        } = self;

        let closure = closures.nodes.get(*id);

        let available = match (closure, direction) {
            (Some(closure), Direction::Incoming) => closure.incoming_neighbours(),
            (Some(closure), Direction::Outgoing) => closure.outgoing_neighbours(),
            (None, _) => return Either::Left(empty()),
        };

        if available.is_empty() {
            return Either::Left(empty());
        }

        Either::Right(
            nodes
                .iter_mut()
                .filter(move |node| available.contains(&node.id))
                .map(|node| NodeMut::new(&node.id, &mut node.weight)),
        )
    }
}
