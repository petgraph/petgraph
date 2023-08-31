use crate::{
    adjacency_matrix::AdjacencyMatrix,
    id::LinearGraphId,
    storage::{DirectedGraphStorage, GraphStorage},
};

// TODO: in graph
pub trait GraphStorageAdjacencyMatrix: GraphStorage
where
    Self::NodeId: LinearGraphId,
{
    fn linearize_node_id(&self, id: &Self::NodeId) -> usize;

    fn undirected_adjacency_matrix(&self) -> AdjacencyMatrix<Self> {
        let mut matrix = AdjacencyMatrix::new_undirected(self);

        for edge in self.edges() {
            matrix.mark(&edge);
        }

        matrix.freeze()
    }
}

pub trait DirectedGraphStorageAdjacencyMatrix:
    DirectedGraphStorage + GraphStorageAdjacencyMatrix
where
    Self::NodeId: LinearGraphId,
{
    fn directed_adjacency_matrix(&self) -> AdjacencyMatrix<Self> {
        let mut matrix = AdjacencyMatrix::new_directed(self);

        for edge in self.edges() {
            matrix.mark(&edge);
        }

        matrix.freeze()
    }
}
