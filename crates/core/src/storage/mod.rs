mod adjacency_matrix;
mod directed;
mod resize;
mod retain;

use error_stack::{Context, Result};

pub use self::{
    adjacency_matrix::{DirectedGraphStorageAdjacencyMatrix, GraphStorageAdjacencyMatrix},
    directed::DirectedGraphStorage,
    resize::ResizableGraphStorage,
    retain::RetainGraphStorage,
};
use crate::{
    edge::{DetachedEdge, Edge, EdgeMut},
    index::GraphIndex,
    node::{DetachedNode, Node, NodeMut},
};

pub trait GraphStorage: Sized {
    type Error: Context;

    type NodeIndex: GraphIndex<Storage = Self>;

    type NodeWeight;

    type EdgeIndex: GraphIndex<Storage = Self>;
    type EdgeWeight;

    fn with_capacity(node_capacity: Option<usize>, edge_capacity: Option<usize>) -> Self
    where
        Self: Sized;

    fn from_parts(
        nodes: impl IntoIterator<Item = DetachedNode<Self::NodeIndex, Self::NodeWeight>>,
        edges: impl IntoIterator<
            Item = DetachedEdge<Self::EdgeIndex, Self::NodeIndex, Self::EdgeWeight>,
        >,
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
            if let Err(error) = graph.insert_edge(edge.id, edge.source, edge.target, edge.weight) {
                match &mut result {
                    Err(errors) => errors.extend_one(error),
                    result => *result = Err(error),
                }
            }
        }

        result.map(|()| graph)
    }

    type IntoPartsNodesIter: Iterator<Item = DetachedNode<Self::NodeIndex, Self::NodeWeight>>;
    type IntoPartsEdgesIter: Iterator<
        Item = DetachedEdge<Self::EdgeIndex, Self::NodeIndex, Self::EdgeWeight>,
    >;

    fn into_parts(self) -> (Self::IntoPartsNodesIter, Self::IntoPartsEdgesIter);

    fn num_nodes(&self) -> usize {
        self.nodes().count()
    }

    fn num_edges(&self) -> usize {
        self.edges().count()
    }

    /// Inserts a new node into the graph.
    ///
    /// # Errors
    ///
    /// Returns an error if the node index is already in use.
    fn insert_node(
        &mut self,
        id: Self::NodeIndex,
        weight: Self::NodeWeight,
    ) -> Result<Node<Self>, Self::Error>;

    /// Inserts a new edge into the graph.
    ///
    /// # Errors
    ///
    /// Returns an error if parallel edges are not allowed and an edge between the given source and
    /// target already exists.
    fn insert_edge(
        &mut self,
        id: Self::EdgeIndex,

        source: Self::NodeIndex,
        target: Self::NodeIndex,

        weight: Self::EdgeWeight,
    ) -> Result<Edge<Self>, Self::Error>;

    fn remove_node(
        &mut self,
        id: &Self::NodeIndex,
    ) -> Option<DetachedNode<Self::NodeIndex, Self::NodeWeight>>;
    fn remove_edge(
        &mut self,
        id: &Self::EdgeIndex,
    ) -> Option<DetachedEdge<Self::EdgeIndex, Self::NodeIndex, Self::EdgeWeight>>;

    fn clear(&mut self) {
        for node in self.nodes() {
            self.remove_node(node.id());
        }

        for edge in self.edges() {
            self.remove_edge(edge.id());
        }
    }

    fn node(&self, id: &Self::NodeIndex) -> Option<Node<Self>>;
    fn node_mut(&mut self, id: &Self::NodeIndex) -> Option<NodeMut<Self>>;

    fn edge(&self, id: &Self::EdgeIndex) -> Option<Edge<Self>>;
    fn edge_mut(&mut self, id: &Self::EdgeIndex) -> Option<EdgeMut<Self>>;

    type FindUndirectedEdgeIter<'a>: Iterator<Item = Edge<'a, Self>> + 'a = impl Iterator<Item = Edge<'a, Self>> + 'a
    where
        Self: 'a;

    fn find_undirected_edges(
        &self,
        source: &Self::NodeIndex,
        target: &Self::NodeIndex,
    ) -> Self::FindUndirectedEdgeIter<'_> {
        // How does this work with a default implementation?
        let from_source = self
            .node_connections(source)
            .filter(|edge| edge.target_id() == target);

        let from_target = self
            .node_connections(target)
            .filter(|edge| edge.source_id() == source);

        from_source.chain(from_target)
    }

    type NodeConnectionIter<'a>: Iterator<Item = Edge<'a, Self>> + 'a
    where
        Self: 'a;

    fn node_connections<'a>(&self, id: &'a Self::NodeIndex) -> Self::NodeConnectionIter<'a>;

    type NodeConnectionMutIter<'a>: Iterator<Item = EdgeMut<'a, Self>> + 'a
    where
        Self: 'a;

    fn node_connections_mut<'a>(
        &mut self,
        id: &'a Self::NodeIndex,
    ) -> Self::NodeConnectionMutIter<'a>;

    type NodeNeighbourIter<'a>: Iterator<Item = Node<'a, Self>> + 'a
    where
        Self: 'a;

    fn node_neighbours<'a>(&'a self, id: &'a Self::NodeIndex) -> Self::NodeNeighbourIter<'a> {
        self.node_connections(id).filter_map(|edge: Edge<Self>| {
            // doing it this way allows us to also get ourselves as a neighbour if we have a
            // self-loop
            if edge.source_id() == id {
                edge.target()
            } else {
                edge.source()
            }
        })
    }

    type NodeNeighbourMutIter<'a>: Iterator<Item = NodeMut<'a, Self>> + 'a
    where
        Self: 'a;

    fn node_neighbours_mut<'a>(
        &mut self,
        id: &'a Self::NodeIndex,
    ) -> Self::NodeNeighbourMutIter<'a> {
        self.node_connections_mut(id)
            .filter_map(|mut edge: EdgeMut<Self>| {
                // doing it this way allows us to also get ourselves as a neighbour if we have a
                // self-loop
                if edge.source_id() == id {
                    edge.target_mut()
                } else {
                    edge.source_mut()
                }
            })
    }

    type ExternalNodeIter<'a>: Iterator<Item = Node<'a, Self>> + 'a
    where
        Self: 'a;

    fn external_nodes(&self) -> Self::ExternalNodeIter<'_> {
        self.nodes()
            .filter(|node| self.node_neighbours(node.id()).next().is_none())
    }

    type ExternalNodeMutIter<'a>: Iterator<Item = NodeMut<'a, Self>> + 'a
    where
        Self: 'a;

    fn external_nodes_mut(&mut self) -> Self::ExternalNodeMutIter<'_> {
        self.nodes_mut()
            .filter(|node| self.node_neighbours(node.id()).next().is_none())
    }

    type NodeIter<'a>: Iterator<Item = Node<'a, Self>> + 'a
    where
        Self: 'a;

    fn nodes(&self) -> Self::NodeIter<'_>;

    type NodeMutIter<'a>: Iterator<Item = NodeMut<'a, Self>> + 'a
    where
        Self: 'a;

    fn nodes_mut(&mut self) -> Self::NodeMutIter<'_>;

    type EdgeIter<'a>: Iterator<Item = Edge<'a, Self>> + 'a
    where
        Self: 'a;

    fn edges(&self) -> Self::EdgeIter<'_>;

    type EdgeMutIter<'a>: Iterator<Item = EdgeMut<'a, Self>> + 'a
    where
        Self: 'a;

    fn edges_mut(&mut self) -> Self::EdgeMutIter<'_>;
}
