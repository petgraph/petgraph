use either::Either;
use petgraph_core::{
    edge::{Direction, Edge, EdgeMut},
    node::{Node, NodeMut},
    storage::{DirectedGraphStorage, GraphStorage},
};

use crate::{
    iter::directed::NodeDirectedConnectionsIter, node::NodeClosures, DinoStorage, Directed,
};

impl<N, E> DirectedGraphStorage for DinoStorage<N, E, Directed> {
    fn directed_edges_between<'a: 'b, 'b>(
        &'a self,
        source: &'b Self::NodeId,
        target: &'b Self::NodeId,
    ) -> impl Iterator<Item = Edge<'a, Self>> + 'b {
        self.closures
            .edges()
            .endpoints_to_edges(*source, *target)
            .filter_map(|id| self.edge(&id))
    }

    fn directed_edges_between_mut<'a: 'b, 'b>(
        &'a mut self,
        source: &'b Self::NodeId,
        target: &'b Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'a, Self>> + 'b {
        let Self {
            closures, edges, ..
        } = self;

        let available = closures.edges().endpoints_to_edges(*source, *target);

        edges
            .filter_mut(available)
            .map(|edge| EdgeMut::new(&edge.id, &mut edge.weight, &edge.source, &edge.target))
    }

    fn node_directed_connections<'a: 'b, 'b>(
        &'a self,
        id: &'b Self::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = Edge<'a, Self>> + 'b {
        NodeDirectedConnectionsIter {
            storage: self,
            iter: self.nodes.get(*id).map(|node| match direction {
                Direction::Incoming => node.closures.incoming_edges(),
                Direction::Outgoing => node.closures.outgoing_edges(),
            }),
        }
    }

    fn node_directed_connections_mut<'a: 'b, 'b>(
        &'a mut self,
        id: &'b Self::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = EdgeMut<'a, Self>> + 'b {
        let Self { nodes, edges, .. } = self;

        let allow = nodes
            .get(*id)
            .into_iter()
            .flat_map(move |node| match direction {
                Direction::Incoming => Either::Left(node.closures.incoming_edges()),
                Direction::Outgoing => Either::Right(node.closures.outgoing_edges()),
            });

        edges
            .filter_mut(allow)
            .map(|edge| EdgeMut::new(&edge.id, &mut edge.weight, &edge.source, &edge.target))
    }

    fn node_directed_neighbours<'a: 'b, 'b>(
        &'a self,
        id: &'b Self::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = Node<'a, Self>> + 'b {
        self.nodes
            .get(*id)
            .into_iter()
            .flat_map(move |node| match direction {
                Direction::Incoming => Either::Left(node.closures.incoming_neighbours()),
                Direction::Outgoing => Either::Right(node.closures.outgoing_neighbours()),
            })
            .filter_map(move |id| self.node(&id))
    }

    fn node_directed_neighbours_mut<'a: 'b, 'b>(
        &'a mut self,
        id: &'b Self::NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = NodeMut<'a, Self>> + 'b {
        let Some(node) = self.nodes.get(*id) else {
            return Either::Right(core::iter::empty());
        };

        // SAFETY: we never access the closure argument mutably, only the weight.
        // Therefore it is safe for us to access both at the same time.
        let closure: &NodeClosures = unsafe { &*(&node.closures as *const _) };
        let neighbours = match direction {
            Direction::Incoming => Either::Left(closure.incoming_neighbours()),
            Direction::Outgoing => Either::Right(closure.outgoing_neighbours()),
        };

        Either::Left(
            self.nodes
                .filter_mut(neighbours)
                .map(move |node| NodeMut::new(&node.id, &mut node.weight)),
        )
    }
}
