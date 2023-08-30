use core::marker::PhantomData;

use fixedbitset::FixedBitSet;

use crate::edge::Edge;

// Thanks to: https://stackoverflow.com/a/27088560/9077988
// and: https://math.stackexchange.com/a/2134297
fn matrix_index_into_linear_index(x: usize, y: usize, n: usize) -> usize {
    let (x, y) = if x > y { (y, x) } else { (x, y) };

    let index = ((n * (n - 1)) / 2) - (((n - x) * (n - x - 1)) / 2) + y;

    index
}

fn length_of_linear_index(n: usize) -> usize {
    // The length of the upper triangle of a matrix (with the diagonal) is:
    // n * (n + 1) / 2
    (n * (n + 1)) / 2
}

pub struct AdjacencyMatrix<N> {
    nodes: usize,
    directed: bool,

    matrix: FixedBitSet,

    _marker: PhantomData<N>,
}

impl<S> AdjacencyMatrix<S> {
    pub fn new_directed(nodes: usize) -> Self {
        Self {
            nodes,
            directed: true,
            matrix: FixedBitSet::with_capacity(nodes * nodes),
            _marker: PhantomData,
        }
    }

    pub fn new_undirected(nodes: usize) -> Self {
        Self {
            nodes,
            directed: false,
            matrix: FixedBitSet::with_capacity(length_of_linear_index(nodes)),
            _marker: PhantomData,
        }
    }

    fn index(&self, source: usize, target: usize) -> usize {
        if self.directed {
            source * self.nodes + target
        } else {
            matrix_index_into_linear_index(source, target, self.nodes)
        }
    }

    fn set(&mut self, source: usize, target: usize, value: bool) {
        self.matrix.set(self.index(source, target), value);
    }

    // TODO: we only need the upper triangle of the matrix, so we can save some space by only saving
    // that.
    // To be able to do that we need to know though with which graph we're working with!
    pub fn mark(&mut self, edge: Edge<'_, S>) {
        let source = edge.source();
        let target = edge.target();

        // TODO: figure out a way to get the numerical index! (needs the underlying graph most
        // likely)

        // self.matrix.set(source, target, true);
        todo!()
    }

    // TODO: use NodeIndex instead of usize
    pub fn is_adjacent(&self, source: usize, target: usize) -> bool {
        let index = self.index(source, target);
        self.matrix[index]
    }
}

// TODO: test
