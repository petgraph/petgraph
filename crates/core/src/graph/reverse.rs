use crate::{storage::reverse::ReverseGraphStorage, Edge, EdgeMut, Graph, Node, NodeMut};

impl<S> Graph<S>
where
    S: ReverseGraphStorage,
{
    pub fn contains_node_key(&self, key: &S::NodeKey) -> bool {
        self.storage.contains_node_key(key)
    }

    pub fn node_by_key(&self, key: &S::NodeKey) -> Option<Node<'_, S>> {
        self.storage.node_by_key(key)
    }

    pub fn node_by_key_mut(&mut self, key: &S::NodeKey) -> Option<NodeMut<'_, S>> {
        self.storage.node_by_key_mut(key)
    }

    pub fn contains_edge_key(&self, key: &S::EdgeKey) -> bool {
        self.storage.contains_edge_key(key)
    }

    pub fn edge_by_key(&self, key: &S::EdgeKey) -> Option<Edge<'_, S>> {
        self.storage.edge_by_key(key)
    }

    pub fn edge_by_key_mut(&mut self, key: &S::EdgeKey) -> Option<EdgeMut<'_, S>> {
        self.storage.edge_by_key_mut(key)
    }
}
