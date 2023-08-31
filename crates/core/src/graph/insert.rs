use error_stack::Result;

use crate::{
    attributes::Attributes,
    edge::Edge,
    graph::Graph,
    id::{ArbitraryGraphId, GraphId},
    node::Node,
    storage::GraphStorage,
};

impl<S> Graph<S>
where
    S: GraphStorage,
{
    pub fn insert_node(
        &mut self,
        attributes: Attributes<<S::NodeId as GraphId>::AttributeIndex, S::NodeWeight>,
    ) -> Result<Node<S>, S::Error> {
        let Attributes { id, weight } = attributes;

        let id = self.storage.next_node_id(id);
        self.storage.insert_node(id, weight)
    }
}

impl<S> Graph<S>
where
    S: GraphStorage,
    S::NodeId: ArbitraryGraphId,
{
    pub fn upsert_node(
        &mut self,
        id: S::NodeId,
        weight: S::NodeWeight,
    ) -> Result<Node<S>, S::Error> {
        // we cannot use `if let` here due to limitations of the borrow checker
        if self.storage.contains_node(&id) {
            let mut node = self
                .storage
                .node_mut(&id)
                .expect("inconsistent storage, node must exist");

            *node.weight_mut() = weight;

            let node = self
                .storage
                .node(&id)
                .expect("inconsistent storage, node must exist");

            Ok(node)
        } else {
            self.storage.insert_node(id, weight)
        }
    }
}

impl<S> Graph<S>
where
    S: GraphStorage,
{
    pub fn insert_edge(
        &mut self,
        attributes: Attributes<<S::EdgeId as GraphId>::AttributeIndex, S::EdgeWeight>,
        source: S::NodeId,
        target: S::NodeId,
    ) -> Result<Edge<S>, S::Error> {
        let Attributes { id, weight } = attributes;

        let id = self.storage.next_edge_id(id);
        self.storage.insert_edge(id, source, target, weight)
    }
}

impl<S> Graph<S>
where
    S: GraphStorage,
    S::EdgeId: ArbitraryGraphId,
{
    pub fn upsert_edge(
        &mut self,
        id: S::EdgeId,
        source: S::NodeId,
        target: S::NodeId,
        weight: S::EdgeWeight,
    ) -> Result<Edge<S>, S::Error> {
        if self.storage.contains_edge(&id) {
            let mut edge = self
                .storage
                .edge_mut(&id)
                .expect("inconsistent storage, edge must exist");

            *edge.weight_mut() = weight;

            // TODO: do not use expect!
            // TODO: I'd like to use `downgrade()` + `bind()` here, but the lifetime is still
            // tracked for the mutable borrow that happened. Therefore not really possible :/
            let edge = self
                .storage
                .edge(&id)
                .expect("inconsistent storage, edge must exist");

            Ok(edge)
        } else {
            self.storage.insert_edge(id, source, target, weight)
        }
    }
}
