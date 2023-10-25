use alloc::{vec, vec::Vec};
use core::{iter, iter::repeat_with};

use petgraph_core::{
    base::MaybeOwned,
    id::{IndexMapper, LinearGraphId},
    Graph, GraphStorage,
};

/// `IndexMapper` that is non-functional and always returns `None` for `lookup` and `reverse`.
///
/// # Panics
///
/// Panics if `map` is called.
/// The caller must ensure, that `map` is never called.
struct DiscardingIndexMapper;

impl<T, U> IndexMapper<T, U> for DiscardingIndexMapper {
    type Continuity = Continuous;

    fn get(&mut self, _: &T) -> U {
        panic!("DiscardingIndexMapper cannot map")
    }

    fn lookup(&self, _: &T) -> Option<U> {
        None
    }

    fn reverse(&mut self, _: &U) -> Option<MaybeOwned<T>> {
        None
    }
}

enum MatrixIndexMapper<I, T> {
    Store(ContinuousIndexMapper<I, T>),
    Discard(DiscardingIndexMapper),
}

impl<I, T> IndexMapper<T, usize> for MatrixIndexMapper<I, T>
where
    I: IndexMapper<T, usize>,
    T: PartialEq + Clone,
{
    type Continuity = Continuous;

    fn get(&mut self, from: &T) -> usize {
        match self {
            Self::Store(mapper) => mapper.map(from),
            Self::Discard(mapper) => mapper.get(from),
        }
    }

    fn lookup(&self, from: &T) -> Option<usize> {
        match self {
            Self::Store(mapper) => mapper.lookup(from),
            Self::Discard(mapper) => mapper.lookup(from),
        }
    }

    fn reverse(&mut self, to: &usize) -> Option<MaybeOwned<T>> {
        match self {
            Self::Store(mapper) => mapper.reverse(to),
            Self::Discard(mapper) => mapper.reverse(to),
        }
    }
}

pub(super) struct SlotMatrix<'a, S, T>
where
    S: GraphStorage + 'a,
    S::NodeId: LinearGraphId<S>,
{
    pub(crate) mapper: MatrixIndexMapper<<S::NodeId as LinearGraphId<S>>::Mapper<'a>, S::NodeId>,
    matrix: Vec<Option<T>>,
    length: usize,
}

impl<'a, S, T> SlotMatrix<'a, S, T>
where
    S: GraphStorage,
    S::NodeId: LinearGraphId<S> + Clone,
{
    pub(crate) fn new(graph: &'a Graph<S>) -> Self {
        let length = graph.num_nodes();
        let mapper =
            MatrixIndexMapper::Store(ContinuousIndexMapper::new(<S::NodeId as LinearGraphId<
                S,
            >>::index_mapper(
                graph.storage()
            )));

        let mut matrix = Vec::with_capacity(length * length);
        matrix.extend(repeat_with(|| None).take(length * length));

        Self {
            mapper,
            matrix,
            length,
        }
    }

    pub(crate) fn empty() -> Self {
        let mapper = MatrixIndexMapper::Discard(DiscardingIndexMapper);
        let matrix = Vec::new();
        let length = 0;

        Self {
            mapper,
            matrix,
            length,
        }
    }

    pub(crate) fn set(&mut self, source: &S::NodeId, target: &S::NodeId, value: Option<T>) {
        if matches!(self.mapper, MatrixIndexMapper::Discard(_)) {
            // this should never happen, even if it does, we don't want to panic here (map call)
            // so we simply return.
            return;
        }

        let source = self.mapper.get(source);
        let target = self.mapper.get(target);

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
        let source = self.mapper.lookup(source)?;
        let target = self.mapper.lookup(target)?;

        self.matrix[source * self.length + target].as_ref()
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
