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

        let id = <S::NodeId as GraphId>::convert(&self.storage, id);

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

            Ok(node.into_ref())
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

        let id = <S::EdgeId as GraphId>::convert(&self.storage, id);

        self.storage.insert_edge(id, source, target, weight)
    }
}

impl<S> Graph<S>
where
    S: GraphStorage,
    S::EdgeId: ArbitraryGraphId<Storage = S>,
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

            Ok(edge.into_ref())
        } else {
            self.storage.insert_edge(id, source, target, weight)
        }
    }
}
