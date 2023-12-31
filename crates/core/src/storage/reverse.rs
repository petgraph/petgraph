use crate::{Edge, EdgeMut, GraphStorage, Node, NodeMut};

pub trait ReverseGraphStorage: GraphStorage {
    type NodeKey;
    type EdgeKey;

    fn contains_node_key(&self, key: &Self::NodeKey) -> bool {
        self.node_by_key(key).is_some()
    }
    fn node_by_key(&self, key: &Self::NodeKey) -> Option<Node<Self>>;
    fn node_by_key_mut(&mut self, key: &Self::NodeKey) -> Option<NodeMut<Self>>;

    fn contains_edge_key(&self, key: &Self::EdgeKey) -> bool {
        self.edge_by_key(key).is_some()
    }
    fn edge_by_key(&self, key: &Self::EdgeKey) -> Option<Edge<Self>>;
    fn edge_by_key_mut(&mut self, key: &Self::EdgeKey) -> Option<EdgeMut<Self>>;
}
