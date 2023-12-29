use alloc::vec::Vec;

use numi::borrow::Moo;
use petgraph_core::{
    id::{IndexMapper, LinearGraphId},
    Graph, GraphStorage,
};

enum MatrixIndexMapper<I> {
    Store(I),
    Discard,
}

impl<I, T> IndexMapper<T> for MatrixIndexMapper<I>
where
    I: IndexMapper<T>,
    T: PartialEq,
{
    fn max(&self) -> usize {
        match self {
            Self::Store(mapper) => mapper.max(),
            Self::Discard => 0,
        }
    }

    fn get(&self, from: &T) -> Option<usize> {
        match self {
            Self::Store(mapper) => mapper.get(from),
            Self::Discard => None,
        }
    }

    fn reverse(&self, to: usize) -> Option<Moo<T>> {
        match self {
            Self::Store(mapper) => mapper.reverse(to),
            Self::Discard => None,
        }
    }
}

pub(super) struct SlotMatrix<'graph, S, T>
where
    S: GraphStorage + 'graph,
    S::NodeId: LinearGraphId<S>,
{
    mapper: MatrixIndexMapper<<S::NodeId as LinearGraphId<S>>::Mapper<'graph>>,
    matrix: Vec<Option<T>>,
    length: usize,
}

impl<'graph, S, T> SlotMatrix<'graph, S, T>
where
    S: GraphStorage,
    S::NodeId: LinearGraphId<S>,
{
    pub(crate) fn new(graph: &'graph Graph<S>) -> Self {
        let length = graph.num_nodes();
        let mapper = MatrixIndexMapper::Store(<S::NodeId as LinearGraphId<S>>::index_mapper(
            graph.storage(),
        ));

        let mut matrix = Vec::with_capacity(length * length);
        matrix.resize_with(length * length, Default::default);

        Self {
            mapper,
            matrix,
            length,
        }
    }

    pub(crate) fn empty() -> Self {
        let mapper = MatrixIndexMapper::Discard;
        let matrix = Vec::new();
        let length = 0;

        Self {
            mapper,
            matrix,
            length,
        }
    }

    pub(crate) fn set(&mut self, source: &S::NodeId, target: &S::NodeId, value: Option<T>) {
        if matches!(self.mapper, MatrixIndexMapper::Discard) {
            // this should never happen, even if it does, we don't want to panic here (map call)
            // so we simply return.
            return;
        }

        let Some(source) = self.mapper.get(source) else {
            return;
        };

        let Some(target) = self.mapper.get(target) else {
            return;
        };

        self.matrix[source * self.length + target] = value;
    }

    /// Get the value at the given index.
    ///
    /// Returns `None` if the node cannot be looked up, this only happens if you try to query for a
    /// value on an index that has not yet been set via `set`.
    ///
    /// See the contract described on the [`IndexMapper`] for more information about the
    /// `map/lookup` contract.
    pub(crate) fn get(&self, source: &S::NodeId, target: &S::NodeId) -> Option<&T> {
        let source = self.mapper.get(source)?;
        let target = self.mapper.get(target)?;

        self.matrix[source * self.length + target].as_ref()
    }

    pub(crate) fn resolve(&self, index: usize) -> Option<Moo<S::NodeId>> {
        self.mapper.reverse(index)
    }
}

// https://stackoverflow.com/a/50548538/9077988
pub(crate) trait Captures<'a> {}
impl<'a, T: ?Sized> Captures<'a> for T {}

impl<'a, S, T> SlotMatrix<'a, S, T>
where
    S: GraphStorage,
    S::NodeId: LinearGraphId<S> + Clone,
{
    pub(crate) fn diagonal(&self) -> impl Iterator<Item = Option<&T>> + Captures<'a> + '_ {
        let len = self.length;

        (0..len).map(move |i| self.matrix[i * len + i].as_ref())
    }
}
