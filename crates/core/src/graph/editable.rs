use core::error::Error;

use crate::graph::Graph;

/// A graph that can be modified by adding and removing nodes and edges.
///
/// Data is moved in and out of the graph using owned types, which may be different from the types
/// used to view the data in the graph. This is mostly an implementation detail and used to detach
/// the lifetime of the returned data from the lifetime of the graph itself in generic code.
pub trait EditableGraph: Graph {
    type OwnedNodeData;
    type OwnedEdgeData;
    type AddNodeError: Error;
    type AddEdgeError: Error;

    /// Add a node to the graph and return its `NodeId`.
    ///
    /// # Errors
    /// May return an error, whose causes depend on the implementation.
    fn add_node(&mut self, data: Self::OwnedNodeData) -> Result<Self::NodeId, Self::AddNodeError>;

    /// Add an edge to the graph and return its `EdgeId`.
    ///
    /// For [`UndirectedGraph`](crate::graph::UndirectedGraph)s the order of `source` and `target`
    /// is irrelevant.
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

impl<G> EditableGraph for &mut G
where
    G: EditableGraph,
{
    type AddEdgeError = G::AddEdgeError;
    type AddNodeError = G::AddNodeError;
    type OwnedEdgeData = G::OwnedEdgeData;
    type OwnedNodeData = G::OwnedNodeData;

    fn add_node(&mut self, data: Self::OwnedNodeData) -> Result<Self::NodeId, Self::AddNodeError> {
        (**self).add_node(data)
    }

    fn add_edge(
        &mut self,
        source: Self::NodeId,
        target: Self::NodeId,
        data: Self::OwnedEdgeData,
    ) -> Result<Self::EdgeId, Self::AddEdgeError> {
        (**self).add_edge(source, target, data)
    }

    fn remove_node(&mut self, id: Self::NodeId) -> Option<Self::OwnedNodeData> {
        (**self).remove_node(id)
    }

    fn remove_edge(&mut self, id: Self::EdgeId) -> Option<Self::OwnedEdgeData> {
        (**self).remove_edge(id)
    }
}

#[cfg(feature = "alloc")]
impl<G> EditableGraph for alloc::boxed::Box<G>
where
    G: EditableGraph,
{
    type AddEdgeError = G::AddEdgeError;
    type AddNodeError = G::AddNodeError;
    type OwnedEdgeData = G::OwnedEdgeData;
    type OwnedNodeData = G::OwnedNodeData;

    fn add_node(&mut self, data: Self::OwnedNodeData) -> Result<Self::NodeId, Self::AddNodeError> {
        (**self).add_node(data)
    }

    fn add_edge(
        &mut self,
        source: Self::NodeId,
        target: Self::NodeId,
        data: Self::OwnedEdgeData,
    ) -> Result<Self::EdgeId, Self::AddEdgeError> {
        (**self).add_edge(source, target, data)
    }

    fn remove_node(&mut self, id: Self::NodeId) -> Option<Self::OwnedNodeData> {
        (**self).remove_node(id)
    }

    fn remove_edge(&mut self, id: Self::EdgeId) -> Option<Self::OwnedEdgeData> {
        (**self).remove_edge(id)
    }
}
