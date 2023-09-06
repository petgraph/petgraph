mod directed;
mod linear;
mod resize;
mod retain;

use error_stack::{Context, Result};

pub use self::{
    directed::DirectedGraphStorage,
    linear::{LinearGraphStorage, LinearIndexLookup},
    resize::ResizableGraphStorage,
    retain::RetainableGraphStorage,
};
use crate::{
    edge::{DetachedEdge, Edge, EdgeMut},
    id::GraphId,
    node::{DetachedNode, Node, NodeMut},
};

pub trait GraphStorage: Sized {
    type EdgeId: GraphId;
    type EdgeWeight;

    type Error: Context;

    type NodeId: GraphId;

    type NodeWeight;

    fn with_capacity(node_capacity: Option<usize>, edge_capacity: Option<usize>) -> Self;

    fn from_parts(
        nodes: impl IntoIterator<Item = DetachedNode<Self::NodeId, Self::NodeWeight>>,
        edges: impl IntoIterator<Item = DetachedEdge<Self::EdgeId, Self::NodeId, Self::EdgeWeight>>,
    ) -> Result<Self, Self::Error> {
        let nodes = nodes.into_iter();
        let edges = edges.into_iter();

        let (_, nodes_max) = nodes.size_hint();
        let (_, edges_max) = edges.size_hint();

        let mut graph = Self::with_capacity(nodes_max, edges_max);

        // by default we try to fail slow, this way we can get as much data about potential errors
        // as possible.
        let mut result: Result<(), Self::Error> = Ok(());

        for node in nodes {
            if let Err(error) = graph.insert_node(node.id, node.weight) {
                match &mut result {
                    Err(errors) => errors.extend_one(error),
                    result => *result = Err(error),
                }
            }
        }

        result?;

        // we need to ensure that all nodes are inserted before we insert edges, otherwise we might
        // end up with invalid data (or redundant errors).
        let mut result: Result<(), Self::Error> = Ok(());

        for edge in edges {
            if let Err(error) = graph.insert_edge(edge.id, edge.weight, &edge.source, &edge.target)
            {
                match &mut result {
                    Err(errors) => errors.extend_one(error),
                    result => *result = Err(error),
                }
            }
        }

        result.map(|()| graph)
    }

    fn into_parts(
        self,
    ) -> (
        impl Iterator<Item = DetachedNode<Self::NodeId, Self::NodeWeight>>,
        impl Iterator<Item = DetachedEdge<Self::EdgeId, Self::NodeId, Self::EdgeWeight>>,
    );

    fn num_nodes(&self) -> usize {
        self.nodes().count()
    }

    fn num_edges(&self) -> usize {
        self.edges().count()
    }

    fn next_node_id(&self, attribute: <Self::NodeId as GraphId>::AttributeIndex) -> Self::NodeId;

    /// Inserts a new node into the graph.
    ///
    /// # Errors
    ///
    /// Returns an error if the node index is already in use.
    fn insert_node(
        &mut self,
        id: Self::NodeId,
        weight: Self::NodeWeight,
    ) -> Result<NodeMut<Self>, Self::Error>;

    fn next_edge_id(&self, attribute: <Self::EdgeId as GraphId>::AttributeIndex) -> Self::EdgeId;

    /// Inserts a new edge into the graph.
    ///
    /// # Errors
    ///
    /// Returns an error if parallel edges are not allowed and an edge between the given source and
    /// target already exists.
    fn insert_edge(
        &mut self,
        id: Self::EdgeId,
        weight: Self::EdgeWeight,

        source: &Self::NodeId,
        target: &Self::NodeId,
    ) -> Result<EdgeMut<Self>, Self::Error>;

    fn remove_node(
        &mut self,
        id: &Self::NodeId,
    ) -> Option<DetachedNode<Self::NodeId, Self::NodeWeight>>;
    fn remove_edge(
        &mut self,
        id: &Self::EdgeId,
    ) -> Option<DetachedEdge<Self::EdgeId, Self::NodeId, Self::EdgeWeight>>;

    fn clear(&mut self) -> Result<(), Self::Error>;

    fn node(&self, id: &Self::NodeId) -> Option<Node<Self>>;
    fn node_mut(&mut self, id: &Self::NodeId) -> Option<NodeMut<Self>>;

    fn contains_node(&self, id: &Self::NodeId) -> bool {
        self.node(id).is_some()
    }

    fn edge(&self, id: &Self::EdgeId) -> Option<Edge<Self>>;
    fn edge_mut(&mut self, id: &Self::EdgeId) -> Option<EdgeMut<Self>>;

    fn contains_edge(&self, id: &Self::EdgeId) -> bool {
        self.edge(id).is_some()
    }

    fn find_undirected_edges<'a: 'b, 'b>(
        &'a self,
        source: &'b Self::NodeId,
        target: &'b Self::NodeId,
    ) -> impl Iterator<Item = Edge<'a, Self>> + 'b {
        // How does this work with a default implementation?
        let from_source = self
            .node_connections(source)
            .filter(move |edge| edge.target_id() == target);

        let from_target = self
            .node_connections(target)
            .filter(move |edge| edge.source_id() == source);

        from_source.chain(from_target)
    }

    // TODO: do we want to provide a `find_undirected_edges_mut`?

    fn node_connections<'a: 'b, 'b>(
        &'a self,
        id: &'b Self::NodeId,
    ) -> impl Iterator<Item = Edge<'a, Self>> + 'b;

    fn node_connections_mut<'a: 'b, 'b>(
        &'a mut self,
        id: &'b Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'a, Self>> + 'b;

    fn node_neighbours<'a: 'b, 'b>(
        &'a self,
        id: &'b Self::NodeId,
    ) -> impl Iterator<Item = Node<'a, Self>> + 'b {
        self.node_connections(id)
            .filter_map(move |edge: Edge<Self>| {
                // doing it this way allows us to also get ourselves as a neighbour if we have a
                // self-loop
                if edge.source_id() == id {
                    edge.target()
                } else {
                    edge.source()
                }
            })
    }

    // I'd love to provide a default implementation for this, but I just can't get it to work.
    fn node_neighbours_mut<'a: 'b, 'b>(
        &'a mut self,
        id: &'b Self::NodeId,
    ) -> impl Iterator<Item = NodeMut<'a, Self>> + 'b;

    fn external_nodes(&self) -> impl Iterator<Item = Node<Self>> {
        self.nodes()
            .filter(|node| self.node_neighbours(node.id()).next().is_none())
    }

    // I'd love to provide a default implementation for this, but I just can't get it to work.
    fn external_nodes_mut(&mut self) -> impl Iterator<Item = NodeMut<Self>>;

    fn nodes(&self) -> impl Iterator<Item = Node<Self>>;

    fn nodes_mut(&mut self) -> impl Iterator<Item = NodeMut<Self>>;

    fn edges(&self) -> impl Iterator<Item = Edge<Self>>;

    fn edges_mut(&mut self) -> impl Iterator<Item = EdgeMut<Self>>;
}
