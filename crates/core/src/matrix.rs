use core::marker::PhantomData;

use fixedbitset::FixedBitSet;

use crate::edge::Edge;

pub struct AdjacencyMatrix<N> {
    nodes: usize,

    matrix: FixedBitSet,

    _marker: PhantomData<N>,
}

impl<N> AdjacencyMatrix<N> {
    pub fn new(nodes: usize) -> Self {
        Self {
            nodes,
            matrix: FixedBitSet::with_capacity(nodes * nodes),
            _marker: PhantomData,
        }
    }

    fn set(&mut self, source: usize, target: usize, value: bool) {
        let index = source * self.nodes + target;
        self.matrix.set(index, value);
    }

    // TODO: we only need the upper triangle of the matrix, so we can save some space by only saving
    // that.
    // To be able to do that we need to know though with which graph we're working with!
    pub fn mark_undirected_edge<E, W>(&mut self, edge: Edge<'_, E, N, W>) {
        let source = edge.source();
        let target = edge.target();

        // TODO: figure out a way to get the numerical index! (needs the underlying graph most
        // likely)

        // self.matrix.set(source, target, true);
        todo!()
    }

    pub fn mark_directed_edge<E, W>(&mut self, edge: Edge<'_, E, N, W>) {
        let source = edge.source();
        let target = edge.target();

        // TODO: figure out a way to get the numerical index! (needs the underlying graph most
        // likely)
        todo!()
    }
}
