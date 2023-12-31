use crate::{GraphStorage, Node};

pub trait ReverseGraphStorage: GraphStorage {
    type NodeKey;
    type EdgeKey;

    fn node_by_weight(&self, weight: &Self::NodeKey) -> Option<Node<Self>>;
    fn contains_node_weight(&self, weight: &Self::NodeKey) -> bool {
        self.node_by_weight(weight).is_some()
    }

    fn edge_by_weight(&self, weight: &Self::EdgeKey) -> Option<Node<Self>>;
    fn contains_edge_weight(&self, weight: &Self::EdgeKey) -> bool {
        self.edge_by_weight(weight).is_some()
    }
}
