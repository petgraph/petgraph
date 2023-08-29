use error_stack::{Context, Result};

use crate::{
    edge::{DetachedEdge, Direction, Edge, EdgeMut},
    matrix::AdjacencyMatrix,
    node::{DetachedNode, Node, NodeMut},
};

pub trait GraphStorage {
    type Error: Context;

    type NodeIndex;
    type NodeWeight;

    type EdgeIndex;
    type EdgeWeight;

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

        let mut graph = Self::with_capacity(nodes_max, edges_max)?;

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

    fn with_capacity(
        node_capacity: Option<usize>,
        edge_capacity: Option<usize>,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized;

    fn reserve_nodes(&mut self, additional: usize) -> Result<(), Self::Error>;
    fn reserve_edges(&mut self, additional: usize) -> Result<(), Self::Error>;

    fn num_nodes(&self) -> usize {
        self.nodes().count()
    }

    fn num_edges(&self) -> usize {
        self.edges().count()
    }

    fn insert_node(
        &mut self,
        id: Self::NodeIndex,
        weight: Self::NodeWeight,
    ) -> Result<(), Self::Error>;

    fn insert_edge(
        &mut self,
        id: Self::EdgeIndex,
        source: Self::NodeIndex,
        target: Self::NodeIndex,
        weight: Self::EdgeWeight,
    ) -> Result<(), Self::Error>;

    fn remove_node(&mut self, id: Self::NodeIndex)
    -> Result<Option<Self::NodeWeight>, Self::Error>;
    fn remove_edge(&mut self, id: Self::EdgeIndex)
    -> Result<Option<Self::EdgeWeight>, Self::Error>;

    fn node(&self, id: &Self::NodeIndex) -> Option<Node<Self::NodeIndex, Self::NodeWeight>>;
    fn node_mut(
        &mut self,
        id: &Self::NodeIndex,
    ) -> Option<NodeMut<Self::NodeIndex, Self::NodeWeight>>;

    fn edge(
        &self,
        id: &Self::EdgeIndex,
    ) -> Option<Edge<Self::NodeIndex, Self::EdgeIndex, Self::EdgeWeight>>;
    fn edge_mut(
        &mut self,
        id: &Self::EdgeIndex,
    ) -> Option<EdgeMut<Self::NodeIndex, Self::EdgeIndex, Self::EdgeWeight>>;

    type NodeConnectionIter<'a>: Iterator<Item = Edge<'a, Self::NodeIndex, Self::EdgeIndex, Self::EdgeWeight>>
        + 'a
    where
        Self: 'a;

    fn node_connections<'a>(&self, id: &'a Self::NodeIndex) -> Self::NodeConnectionIter<'a>;

    type NodeNeighbourIter<'a>: Iterator<Item = Node<'a, Self::NodeIndex, Self::NodeWeight>> + 'a
    where
        Self: 'a;

    fn node_neighbours<'a>(&self, id: &'a Self::NodeIndex) -> Self::NodeNeighbourIter<'a> {
        self.node_connections(id)
            .filter_map(|edge| {
                let source = edge.source();
                let target = edge.target();

                // doing it this way allows us to also get ourselves as a neighbour if we have a
                // self-loop
                if source == id {
                    Some(target)
                } else {
                    Some(source)
                }
            })
            .filter_map(|id| self.node(id))
    }

    fn undirected_adjacency_matrix(&self) -> AdjacencyMatrix<Self::NodeIndex> {
        let mut matrix = AdjacencyMatrix::new(self.num_nodes());

        for edge in self.edges() {
            matrix.mark_undirected_edge(edge);
        }

        matrix
    }

    type NodeIter<'a>: Iterator<Item = Node<'a, Self::NodeIndex, Self::NodeWeight>> + 'a
    where
        Self: 'a;

    fn nodes(&self) -> Self::NodeIter<'_>;

    type EdgeIter<'a>: Iterator<Item = Edge<'a, Self::NodeIndex, Self::EdgeIndex, Self::EdgeWeight>>
        + 'a
    where
        Self: 'a;

    fn edges(&self) -> Self::EdgeIter<'_>;
}

pub trait DirectedGraphStorage: GraphStorage {
    fn directed_adjacency_matrix(&self) -> AdjacencyMatrix<Self::NodeIndex> {
        let mut matrix = AdjacencyMatrix::new(self.num_nodes());

        for edge in self.edges() {
            matrix.mark_directed_edge(edge);
        }

        matrix
    }

    type NodeDirectedConnectionIter<'a>: Iterator<Item = Edge<'a, Self::NodeIndex, Self::EdgeIndex, Self::EdgeWeight>>
        + 'a
    where
        Self: 'a;

    fn node_directed_connections<'a>(
        &self,
        id: &'a Self::NodeIndex,
        direction: Direction,
    ) -> Self::NodeDirectedConnectionIter<'a>;

    type NodeDirectedNeighbourIter<'a>: Iterator<Item = Node<'a, Self::NodeIndex, Self::NodeWeight>>
        + 'a
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
            .filter_map(|id| self.node(id))
    }
}
