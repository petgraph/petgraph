use fb::FixedBitSet;

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
    type AdjMatrix = FixedBitSet;

    fn adjacency_matrix(&self) -> FixedBitSet
    {
        let n = self.node_count();
        let mut matrix = FixedBitSet::with_capacity(n * n);
        let mut i = 0;
        for row in 0..n {
            for col in 0..n {
                let flag = self.find_edge(NodeIndex::new(row),
                                          NodeIndex::new(col)).is_some();
                matrix.set(i, flag);
                i += 1;
            }
        }
        matrix
    }

    fn is_adjacent(&self, matrix: &FixedBitSet, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> bool
    {
        let n = self.node_count();
        let index = n * a.index() + b.index();
        matrix.contains(index)
    }
}


