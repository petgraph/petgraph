use either::Either;
use petgraph_core::{
    edge::{Direction, Edge, EdgeId, EdgeMut},
    node::{Node, NodeId, NodeMut},
    storage::{DirectedGraphStorage, GraphStorage},
};

use crate::{
    iter::directed::NodeDirectedConnectionsIter,
    node::{NodeClosures, NodeSlab},
    DinoStorage, Directed,
};

fn directed_edges_between<N>(
    nodes: &NodeSlab<N>,
    source: NodeId,
    target: NodeId,
) -> impl Iterator<Item = EdgeId> + '_ {
    let source = nodes.get(source);
    let target = nodes.get(target);

    source
        .and_then(|source| target.map(|target| (source, target)))
        .into_iter()
        .flat_map(|(source, target)| source.closures.edges_between_directed(&target.closures))
}

impl<N, E> DirectedGraphStorage for DinoStorage<N, E, Directed> {
    fn directed_edges_between(
        &self,
        source: NodeId,
        target: NodeId,
    ) -> impl Iterator<Item = Edge<Self>> {
        directed_edges_between(&self.nodes, source, target).filter_map(move |id| self.edge(id))
    }

    fn directed_edges_between_mut(
        &mut self,
        source: NodeId,
        target: NodeId,
    ) -> impl Iterator<Item = EdgeMut<Self>> {
        let Self { edges, nodes, .. } = self;

        let available = directed_edges_between(nodes, source, target);

        edges
            .filter_mut(available)
            .map(|edge| EdgeMut::new(edge.id, &mut edge.weight, edge.source, edge.target))
    }

    fn node_directed_connections(
        &self,
        id: NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = Edge<Self>> {
        NodeDirectedConnectionsIter {
            storage: self,
            iter: self.nodes.get(id).map(|node| match direction {
                Direction::Incoming => node.closures.incoming_edges(),
                Direction::Outgoing => node.closures.outgoing_edges(),
            }),
        }
    }

    fn node_directed_connections_mut(
        &mut self,
        id: NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = EdgeMut<Self>> {
        let Self { nodes, edges, .. } = self;

        let allow = nodes
            .get(id)
            .into_iter()
            .flat_map(move |node| match direction {
                Direction::Incoming => node.closures.incoming_edges(),
                Direction::Outgoing => node.closures.outgoing_edges(),
            });

        edges
            .filter_mut(allow)
            .map(|edge| EdgeMut::new(edge.id, &mut edge.weight, edge.source, edge.target))
    }

    fn node_directed_neighbours(
        &self,
        id: NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = Node<Self>> {
        self.nodes
            .get(id)
            .into_iter()
            .flat_map(move |node| match direction {
                Direction::Incoming => node.closures.incoming_nodes(),
                Direction::Outgoing => node.closures.outgoing_nodes(),
            })
            .filter_map(move |id| self.node(id))
    }

    fn node_directed_neighbours_mut(
        &mut self,
        id: NodeId,
        direction: Direction,
    ) -> impl Iterator<Item = NodeMut<Self>> {
        let Some(node) = self.nodes.get(id) else {
            return Either::Right(core::iter::empty());
        };

        // SAFETY: we never access the closure argument mutably, only the weight.
        // Therefore it is safe for us to access both at the same time.
        let closure: &NodeClosures = unsafe { &*core::ptr::addr_of!(node.closures) };
        let neighbours = match direction {
            Direction::Incoming => closure.incoming_nodes(),
            Direction::Outgoing => closure.outgoing_nodes(),
        };

        Either::Left(
            self.nodes
                .filter_mut(neighbours)
                .map(move |node| NodeMut::new(node.id, &mut node.weight)),
        )
    }
}
