use core::marker::PhantomData;

use fixedbitset::FixedBitSet;

use crate::{edge::Edge, id::LinearGraphId, storage::GraphStorageAdjacencyMatrix};

// TODO: variable storage backend

// Thanks to: https://stackoverflow.com/a/27088560/9077988
// and: https://math.stackexchange.com/a/2134297
fn matrix_index_into_linear_index(x: usize, y: usize, n: usize) -> usize {
    let (x, y) = if x > y { (y, x) } else { (x, y) };

    

    ((n * (n - 1)) / 2) - (((n - x) * (n - x - 1)) / 2) + y
}

fn length_of_linear_index(n: usize) -> usize {
    // The length of the upper triangle of a matrix (with the diagonal) is:
    // n * (n + 1) / 2
    (n * (n + 1)) / 2
}

pub struct Frozen;
pub struct Mutable;

pub struct AdjacencyMatrix<'a, S, T = Frozen> {
    storage: &'a S,
    directed: bool,

    num_nodes: usize,

    matrix: FixedBitSet,

    _marker: PhantomData<T>,
}

impl<'a, S, T> AdjacencyMatrix<'a, S, T> {
    fn index(&self, source: usize, target: usize) -> usize {
        if self.directed {
            source * self.num_nodes + target
        } else {
            matrix_index_into_linear_index(source, target, self.num_nodes)
        }
    }
}

impl<'a, S> AdjacencyMatrix<'a, S, Mutable>
where
    S: GraphStorageAdjacencyMatrix,
    S::NodeId: LinearGraphId,
{
    pub fn new_directed(storage: &'a S) -> Self {
        let num_nodes = storage.num_nodes();

        Self {
            storage,
            directed: true,

            num_nodes,

            matrix: FixedBitSet::with_capacity(num_nodes * num_nodes),

            _marker: PhantomData,
        }
    }

    pub fn new_undirected(storage: &'a S) -> Self {
        let num_nodes = storage.num_nodes();

        Self {
            storage,
            directed: false,

            num_nodes,
            matrix: FixedBitSet::with_capacity(length_of_linear_index(num_nodes)),

            _marker: PhantomData,
        }
    }

    fn set(&mut self, source: usize, target: usize, value: bool) {
        self.matrix.set(self.index(source, target), value);
    }

    // TODO: we only need the upper triangle of the matrix, so we can save some space by only saving
    // that.
    // To be able to do that we need to know though with which graph we're working with!
    pub fn mark(&mut self, edge: Edge<'_, S>) {
        let Some(source) = edge.source() else {
            return;
        };

        let Some(target) = edge.target() else {
            return;
        };

        let source = source.id().as_linear(self.storage);
        let target = target.id().as_linear(self.storage);

        self.set(source, target, true);
    }

    #[must_use] pub fn freeze(self) -> AdjacencyMatrix<'a, S, Frozen> {
        AdjacencyMatrix {
            storage: self.storage,
            directed: self.directed,

            num_nodes: self.num_nodes,

            matrix: self.matrix,

            _marker: PhantomData,
        }
    }
}

impl<'a, S> AdjacencyMatrix<'a, S, Frozen>
where
    S: GraphStorageAdjacencyMatrix,
    S::NodeId: LinearGraphId,
{
    pub fn is_adjacent(&self, source: S::NodeId, target: S::NodeId) -> bool {
        let source = source.as_linear(self.storage);
        let target = target.as_linear(self.storage);

        let index = self.index(source, target);
        self.matrix[index]
    }
}

// TODO: test
