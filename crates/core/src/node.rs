use crate::graph::Graph;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Node<I, D> {
    pub id: I,
    pub data: D,
}

pub type NodeRef<'graph, G> = Node<<G as Graph>::NodeId, <G as Graph>::NodeDataRef<'graph>>;
pub type NodeMut<'graph, G> = Node<<G as Graph>::NodeId, <G as Graph>::NodeDataMut<'graph>>;
