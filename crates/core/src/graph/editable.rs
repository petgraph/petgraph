use crate::graph::Graph;

pub trait EditableGraph: Graph {
    fn add_node(&mut self, node: Self::NodeData<'_>) -> Self::NodeId;
    fn add_edge(
        &mut self,
        source: Self::NodeId,
        target: Self::NodeId,
        edge: Self::EdgeData<'_>,
    ) -> Option<Self::EdgeId>;
    fn remove_node(&mut self, node_id: Self::NodeId) -> Option<Self::NodeData<'_>>;
    fn remove_edge(&mut self, edge_id: Self::EdgeId) -> Option<Self::EdgeData<'_>>;
}
