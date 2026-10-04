use crate::graph::Graph;

pub trait EditableGraph: Graph {
    type OwnedNodeData;
    type OwnedEdgeData;
    type AddNodeError;
    type AddEdgeError;

    /// Add a node to the graph and return its `NodeId`.
    ///
    /// # Errors
    /// May return an error, whose causes depend on the implementation.
    fn add_node(&mut self, data: Self::OwnedNodeData) -> Result<Self::NodeId, Self::AddNodeError>;

    /// Add an edge to the graph and return its `EdgeId`.
    ///
    /// For `UndirectedGraph`s the order of `source` and `target` is irrelevant.
    ///
    /// # Errors
    /// May return an error, whose causes depend on the implementation.
    fn add_edge(
        &mut self,
        source: Self::NodeId,
        target: Self::NodeId,
        data: Self::OwnedEdgeData,
    ) -> Result<Self::EdgeId, Self::AddEdgeError>;

    /// Removes the node and all edges incident to it, returning the node's data.
    ///
    /// Returns `None` if the node was not in the graph.
    fn remove_node(&mut self, id: Self::NodeId) -> Option<Self::OwnedNodeData>;

    /// Removes the edge, returning its data.
    ///
    /// Returns `None` if the edge was not in the graph.
    fn remove_edge(&mut self, id: Self::EdgeId) -> Option<Self::OwnedEdgeData>;
}
