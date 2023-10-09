use error_stack::Result;

use crate::{
    attributes::{Attributes, NoValue},
    edge::EdgeMut,
    graph::Graph,
    id::{ArbitraryGraphId, GraphId, ManagedGraphId},
    node::{Node, NodeMut},
    storage::GraphStorage,
};

impl<S> Graph<S>
where
    S: GraphStorage,
{
    pub fn try_insert_node(
        &mut self,
        attributes: impl Into<Attributes<<S::NodeId as GraphId>::AttributeIndex, S::NodeWeight>>,
    ) -> Result<NodeMut<S>, S::Error> {
        let Attributes { id, weight } = attributes.into();

        let id = self.storage.next_node_id(id);
        self.storage.insert_node(id, weight)
    }

    // TODO: I don't like how we return `NodeMut`, we should just return the id here?, but then
    // there would be a problem with lifetimes as we cannot give out a value, as we don't know if
    // the key is clone.
    // I think this is for the better, but may need to be revisited.
    pub fn insert_node(
        &mut self,
        attributes: impl Into<Attributes<<S::NodeId as GraphId>::AttributeIndex, S::NodeWeight>>,
    ) -> NodeMut<S> {
        self.try_insert_node(attributes)
            .expect("unable to insert node")
    }
}

impl<S> Graph<S>
where
    S: GraphStorage,
    S::NodeId: ManagedGraphId,
{
    pub fn insert_node_with(
        &mut self,
        weight: impl FnOnce(&S::NodeId) -> S::NodeWeight,
    ) -> Result<NodeMut<S>, S::Error> {
        let id = self.storage.next_node_id(NoValue::new());
        let weight = weight(&id);

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
    ) -> Result<NodeMut<S>, S::Error> {
        // we cannot use `if let` here due to limitations of the borrow checker
        if self.storage.contains_node(&id) {
            let mut node = self
                .storage
                .node_mut(&id)
                .expect("inconsistent storage, node must exist");

            *node.weight_mut() = weight;

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
    pub fn try_insert_edge(
        &mut self,
        attributes: impl Into<Attributes<<S::EdgeId as GraphId>::AttributeIndex, S::EdgeWeight>>,
        source: &S::NodeId,
        target: &S::NodeId,
    ) -> Result<EdgeMut<S>, S::Error> {
        let Attributes { id, weight } = attributes.into();

        let id = self.storage.next_edge_id(id);
        self.storage.insert_edge(id, weight, source, target)
    }

    pub fn insert_edge(
        &mut self,
        attributes: impl Into<Attributes<<S::EdgeId as GraphId>::AttributeIndex, S::EdgeWeight>>,
        source: &S::NodeId,
        target: &S::NodeId,
    ) -> EdgeMut<S> {
        self.try_insert_edge(attributes, source, target)
            .expect("unable to insert edge")
    }
}

impl<S> Graph<S>
where
    S: GraphStorage,
    S::EdgeId: ManagedGraphId,
{
    pub fn insert_edge_with(
        &mut self,
        weight: impl FnOnce(&S::EdgeId) -> S::EdgeWeight,
        source: &S::NodeId,
        target: &S::NodeId,
    ) -> Result<EdgeMut<S>, S::Error> {
        let id = self.storage.next_edge_id(NoValue::new());
        let weight = weight(&id);

        self.storage.insert_edge(id, weight, source, target)
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
        weight: S::EdgeWeight,

        source: &S::NodeId,
        target: &S::NodeId,
    ) -> Result<EdgeMut<S>, S::Error> {
        if self.storage.contains_edge(&id) {
            let mut edge = self
                .storage
                .edge_mut(&id)
                .expect("inconsistent storage, edge must exist");

            *edge.weight_mut() = weight;

            Ok(edge)
        } else {
            self.storage.insert_edge(id, weight, source, target)
        }
    }
}
