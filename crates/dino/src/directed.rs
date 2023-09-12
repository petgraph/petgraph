use core::iter::empty;

use either::Either;
use hashbrown::HashSet;
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
            .edges()
            .endpoints_to_edges(*source, *target)
            .filter_map(|id| self.edge(&id))
    }

    fn node_directed_connections<'a: 'b, 'b>(
        &'a self,
        id: &'b Self::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = Edge<'a, Self>> + 'b {
        let closure = self.closures.nodes();

        match direction {
            Direction::Incoming => Either::Left(closure.incoming_edges(*id)),
            Direction::Outgoing => Either::Right(closure.outgoing_edges(*id)),
        }
        .filter_map(move |id| self.edge(&id))
    }

    fn node_directed_connections_mut<'a: 'b, 'b>(
        &'a mut self,
        id: &'b Self::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = EdgeMut<'a, Self>> + 'b {
        let Self {
            closures, edges, ..
        } = self;

        let closure = closures.nodes();

        let available = match direction {
            Direction::Incoming => Either::Left(closure.incoming_edges(*id)),
            Direction::Outgoing => Either::Right(closure.outgoing_edges(*id)),
        };

        // TODO: we can do this without allocation, as both edges and available are sorted! (except
        // for generation, mhh)
        let available: HashSet<_> = available.collect();

        if available.is_empty() {
            return Either::Left(empty());
        }

        Either::Right(
            edges
                .iter_mut()
                .filter(move |edge| available.contains(&edge.id))
                .map(|edge| EdgeMut::new(&edge.id, &mut edge.weight, &edge.source, &edge.target)),
        )
    }

    fn node_directed_neighbours<'a: 'b, 'b>(
        &'a self,
        id: &'b Self::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = Node<'a, Self>> + 'b {
        let closure = self.closures.nodes();

        match direction {
            Direction::Incoming => Either::Left(closure.incoming_neighbours(*id)),
            Direction::Outgoing => Either::Right(closure.outgoing_neighbours(*id)),
        }
        .filter_map(move |id| self.node(&id))
    }

    fn node_directed_neighbours_mut<'a: 'b, 'b>(
        &'a mut self,
        id: &'b Self::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = NodeMut<'a, Self>> + 'b {
        let Self {
            closures, nodes, ..
        } = self;

        let closure = closures.nodes();

        let available = match direction {
            Direction::Incoming => Either::Left(closure.incoming_neighbours(*id)),
            Direction::Outgoing => Either::Right(closure.outgoing_neighbours(*id)),
        };

        // TODO: we can do this without allocation, as both nodes and available are sorted!
        // (except for generation)
        let available: HashSet<_> = available.collect();

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
