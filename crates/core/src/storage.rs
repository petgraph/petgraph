use error_stack::{Context, Result};

use crate::{
    edge::{DetachedEdge, Direction, Edge, EdgeMut},
    index::GraphIndex,
    matrix::AdjacencyMatrix,
    node::{DetachedNode, Node, NodeMut},
};

pub trait GraphStorage {
    type Error: Context;

    type NodeIndex: GraphIndex<Self>;
    type NodeWeight;

    type EdgeIndex: GraphIndex<Self>;
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
        let mut result = Ok(());

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
        let mut result = Ok(());

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

    fn reserve_nodes(&mut self, additional: usize) -> Result<(), Self::Error>;
    fn reserve_edges(&mut self, additional: usize) -> Result<(), Self::Error>;

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
    ) -> Result<(), Self::Error>;

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
    ) -> Result<(), Self::Error>;

    fn remove_node(
        &mut self,
        id: Self::NodeIndex,
    ) -> Option<DetachedNode<Self::NodeIndex, Self::NodeWeight>>;
    fn remove_edge(
        &mut self,
        id: Self::EdgeIndex,
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

    fn node_neighbours<'a>(&self, id: &'a Self::NodeIndex) -> Self::NodeNeighbourIter<'a> {
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

    fn undirected_adjacency_matrix(&self) -> AdjacencyMatrix<Self::NodeIndex> {
        let mut matrix = AdjacencyMatrix::new_undirected(self.num_nodes());

        for edge in self.edges() {
            matrix.mark(edge);
        }

        matrix
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

pub trait DirectedGraphStorage: GraphStorage {
    fn directed_adjacency_matrix(&self) -> AdjacencyMatrix<Self::NodeIndex> {
        let mut matrix = AdjacencyMatrix::new_directed(self.num_nodes());

        for edge in self.edges() {
            matrix.mark(edge);
        }

        matrix
    }

    type NodeDirectedConnectionIter<'a>: Iterator<Item = Edge<'a, Self>> + 'a
    where
        Self: 'a;

    fn node_directed_connections<'a>(
        &self,
        id: &'a Self::NodeIndex,
        direction: Direction,
    ) -> Self::NodeDirectedConnectionIter<'a>;

    type NodeDirectedConnectionMutIter<'a>: Iterator<Item = EdgeMut<'a, Self>> + 'a
    where
        Self: 'a;

    fn node_directed_connections_mut<'a>(
        &mut self,
        id: &'a Self::NodeIndex,
        direction: Direction,
    ) -> Self::NodeDirectedConnectionMutIter<'a>;

    type NodeDirectedNeighbourIter<'a>: Iterator<Item = Node<'a, Self>> + 'a
    where
        Self: 'a;

    fn node_directed_neighbours<'a>(
        &self,
        id: &'a Self::NodeIndex,
        direction: Direction,
    ) -> Self::NodeDirectedNeighbourIter<'a> {
        self.node_directed_connections(id, direction)
            .map(|edge| match direction {
                Direction::Outgoing => edge.target(),
                Direction::Incoming => edge.source(),
            })
    }

    type NodeDirectedNeighbourMutIter<'a>: Iterator<Item = NodeMut<'a, Self>> + 'a
    where
        Self: 'a;

    fn node_directed_neighbours_mut<'a>(
        &mut self,
        id: &'a Self::NodeIndex,
        direction: Direction,
    ) -> Self::NodeDirectedNeighbourMutIter<'a> {
        self.node_directed_connections_mut(id, direction)
            .map(|mut edge| match direction {
                Direction::Outgoing => edge.target_mut(),
                Direction::Incoming => edge.source_mut(),
            })
    }
}
