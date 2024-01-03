use core::hash::Hash;

use petgraph_core::{
    storage::reverse::ReverseGraphStorage, Edge, EdgeMut, GraphDirectionality, GraphStorage, Node,
    NodeMut,
};

use crate::{hash::ValueHash, EntryStorage};

impl<NK, NV, EK, EV, D> ReverseGraphStorage for EntryStorage<NK, NV, EK, EV, D>
where
    D: GraphDirectionality,
    NK: Hash,
    EK: Hash,
{
    type EdgeKey = EK;
    type NodeKey = NK;

    fn contains_node_key(&self, key: &Self::NodeKey) -> bool {
        let hash = ValueHash::new(&self.hasher, key);
        self.nodes.contains_key(&hash)
    }

    fn node_by_key(&self, key: &Self::NodeKey) -> Option<Node<Self>> {
        let hash = ValueHash::new(&self.hasher, key);

        self.nodes.get(&hash).and_then(|id| self.node(*id))
    }

    fn node_by_key_mut(&mut self, key: &Self::NodeKey) -> Option<NodeMut<Self>> {
        let hash = ValueHash::new(&self.hasher, key);

        let &node = self.nodes.get(&hash)?;

        self.node_mut(node)
    }

    fn contains_edge_key(&self, key: &Self::EdgeKey) -> bool {
        let hash = ValueHash::new(&self.hasher, key);

        self.edges.contains_key(&hash)
    }

    fn edge_by_key(&self, key: &Self::EdgeKey) -> Option<Edge<Self>> {
        let hash = ValueHash::new(&self.hasher, key);

        self.edges.get(&hash).and_then(|id| self.edge(*id))
    }

    fn edge_by_key_mut(&mut self, key: &Self::EdgeKey) -> Option<EdgeMut<Self>> {
        let hash = ValueHash::new(&self.hasher, key);

        let &edge = self.edges.get(&hash)?;

        self.edge_mut(edge)
    }
}
