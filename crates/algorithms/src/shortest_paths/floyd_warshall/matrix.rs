use alloc::{vec, vec::Vec};
use core::{iter, iter::repeat_with};

use petgraph_core::{
    id::{ContinuousIndexMapper, IndexMapper, LinearGraphId},
    Graph, GraphStorage,
};

pub(super) struct SlotMatrix<'a, S, T>
where
    S: GraphStorage + 'a,
    S::NodeId: LinearGraphId<S>,
{
    mapper: ContinuousIndexMapper<<S::NodeId as LinearGraphId<S>>::Mapper<'a>, S::NodeId>,
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
        let mapper = ContinuousIndexMapper::new(<S::NodeId as LinearGraphId<S>>::index_mapper(
            graph.storage(),
        ));

        let mut matrix = Vec::with_capacity(length * length);
        matrix.extend(repeat_with(|| None).take(length * length));

        Self {
            mapper,
            matrix,
            length,
        }
    }

    pub(crate) fn set(&mut self, source: &S::NodeId, target: &S::NodeId, value: Option<T>) {
        let source = self.mapper.map(source);
        let target = self.mapper.map(target);

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
