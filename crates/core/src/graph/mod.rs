mod directed;

use crate::{
    edge::{Edge, EdgeMut},
    id::Id,
    node::{Node, NodeMut},
};

pub trait Graph {
    type NodeId: Id;
    type Node<'graph>: Node<'graph, Id = Self::NodeId>
    where
        Self: 'graph;
    type NodeRef<'graph>: Node<'graph, Id = Self::NodeId>
    where
        Self: 'graph;
    type NodeMut<'graph>: NodeMut<'graph, Id = Self::NodeId>
    where
        Self: 'graph;

    type EdgeId: Id;
    type Edge<'graph>: Edge<'graph, Id = Self::EdgeId, Node = Self::NodeId>
    where
        Self: 'graph;
    type EdgeRef<'graph>: Edge<'graph, Id = Self::EdgeId, Node = Self::NodeId>
    where
        Self: 'graph;
    type EdgeMut<'graph>: EdgeMut<'graph, Id = Self::EdgeId, Node = Self::NodeId>
    where
        Self: 'graph;
}
