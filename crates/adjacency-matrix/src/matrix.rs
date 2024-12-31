use core::marker::PhantomData;

use fixedbitset::FixedBitSet;
use petgraph_core::{
    edge::{
        Edge,
        marker::{Directed, GraphDirectionality, Undirected},
    },
    graph::{DirectedGraph, Graph},
    id::{LinearGraphId, LinearGraphIdMapper},
};

// Thanks to: https://stackoverflow.com/a/27088560/9077988
// and: https://math.stackexchange.com/a/2134297
const fn matrix_index_into_linear_index(x: usize, y: usize, n: usize) -> usize {
    let (x, y) = if x > y { (y, x) } else { (x, y) };

    ((n * (n - 1)) / 2) - (((n - x) * (n - x - 1)) / 2) + y
}

const fn length_of_linear_index(n: usize) -> usize {
    // The length of the upper triangle of a matrix (with the diagonal) is:
    // n * (n + 1) / 2
    (n * (n + 1)) / 2
}

// TODO: implement `GraphStorage`?

pub struct Frozen;
pub struct Mutable;

pub struct AdjacencyMatrix<'a, S, D = Directed, T = Frozen>
where
    S: Graph,
    S::NodeId: LinearGraphId<Storage = S>,
    D: GraphDirectionality,
{
    storage: &'a S,
    mapper: <S::NodeId as LinearGraphId>::Mapper<'a>,

    num_nodes: usize,

    matrix: FixedBitSet,

    _marker: PhantomData<(D, T)>,
}

impl<'a, S, D, T> AdjacencyMatrix<'a, S, D, T>
where
    S: Graph,
    S::NodeId: LinearGraphId<Storage = S>,
    D: GraphDirectionality,
{
    const fn index(&self, source: usize, target: usize) -> usize {
        if D::is_directed() {
            source * self.num_nodes + target
        } else {
            matrix_index_into_linear_index(source, target, self.num_nodes)
        }
    }

    fn set(&mut self, source: usize, target: usize, value: bool) {
        self.matrix.set(self.index(source, target), value);
    }

    pub fn mark(&mut self, edge: &Edge<'_, S>) {
        let Some(source) = edge.source() else {
            return;
        };

        let Some(target) = edge.target() else {
            return;
        };

        let source = self.mapper.map(source.id());
        let target = self.mapper.map(target.id());

        self.set(source, target, true);
    }
}

impl<'a, S, D> AdjacencyMatrix<'a, S, D, Mutable>
where
    S: Graph,
    S::NodeId: LinearGraphId<Storage = S>,
    D: GraphDirectionality,
{
    #[must_use]
    pub fn freeze(self) -> AdjacencyMatrix<'a, S, D, Frozen> {
        AdjacencyMatrix {
            storage: self.storage,
            mapper: self.mapper,

            num_nodes: self.num_nodes,

            matrix: self.matrix,

            _marker: PhantomData,
        }
    }
}

impl<'a, S> AdjacencyMatrix<'a, S, Undirected, Mutable>
where
    S: Graph,
    S::NodeId: LinearGraphId<Storage = S>,
{
    pub fn new_undirected(storage: &'a S) -> Self {
        let num_nodes = storage.num_nodes();
        let mapper = S::NodeId::mapper(storage);

        Self {
            storage,
            mapper,

            num_nodes,
            matrix: FixedBitSet::with_capacity(length_of_linear_index(num_nodes)),

            _marker: PhantomData,
        }
    }
}

impl<'a, S> AdjacencyMatrix<'a, S, Directed, Mutable>
where
    S: DirectedGraph,
    S::NodeId: LinearGraphId<Storage = S>,
{
    pub fn new_directed(storage: &'a S) -> Self {
        let num_nodes = storage.num_nodes();
        let mapper = S::NodeId::mapper(storage);

        Self {
            storage,
            mapper,

            num_nodes,

            matrix: FixedBitSet::with_capacity(num_nodes * num_nodes),

            _marker: PhantomData,
        }
    }
}

impl<'a, S, D> AdjacencyMatrix<'a, S, D, Frozen>
where
    S: Graph,
    S::NodeId: LinearGraphId<Storage = S>,
    D: GraphDirectionality,
{
    pub fn is_adjacent(&self, source: &S::NodeId, target: &S::NodeId) -> bool {
        let source = self.mapper.map(source);
        let target = self.mapper.map(target);

        let index = self.index(source, target);
        self.matrix[index]
    }
}

// TODO: test
