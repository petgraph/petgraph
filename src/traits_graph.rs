use std::collections::Bitv;

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
    type AdjMatrix = Bitv;

    fn adjacency_matrix(&self) -> Bitv
    {
        let n = self.node_count();
        let mut matrix = Bitv::with_capacity(n * n);
        for row in 0..n {
            for col in 0..n {
                let flag = self.find_edge(NodeIndex::new(row),
                                          NodeIndex::new(col)).is_some();
                matrix.push(flag);
            }
        }
        matrix
    }

    fn is_adjacent(&self, matrix: &Bitv, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> bool
    {
        let n = self.node_count();
        let index = n * a.index() + b.index();
        matrix.get(index).unwrap_or(false)
    }
}


