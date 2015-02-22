use std::collections::BitVec;

use super::{
    EdgeType,
};

use super::graph::{
    Graph,
    IndexType,
    NodeIndex,
};

use super::visit::GetAdjacencyMatrix;

impl<N, E, Ty, Ix> GetAdjacencyMatrix for Graph<N, E, Ty, Ix> where
    Ty: EdgeType,
    Ix: IndexType,
{
    type AdjMatrix = BitVec;

    fn adjacency_matrix(&self) -> BitVec
    {
        let n = self.node_count();
        let mut matrix = BitVec::with_capacity(n * n);
        for row in 0..n {
            for col in 0..n {
                let flag = self.find_edge(NodeIndex::new(row),
                                          NodeIndex::new(col)).is_some();
                matrix.push(flag);
            }
        }
        matrix
    }

    fn is_adjacent(&self, matrix: &BitVec, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> bool
    {
        let n = self.node_count();
        let index = n * a.index() + b.index();
        matrix.get(index).unwrap_or(false)
    }
}


