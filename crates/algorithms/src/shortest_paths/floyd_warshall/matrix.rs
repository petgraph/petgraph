use alloc::{vec, vec::Vec};
use core::{iter, iter::repeat_with};

use petgraph_core::{
    id::{ContinuousIndexMapper, IndexMapper, LinearGraphId},
    Graph, GraphStorage,
};

pub(super) struct Matrix<'a, S, T>
where
    S: GraphStorage,
    S::NodeId: LinearGraphId<S>,
{
    mapper: ContinuousIndexMapper<<S::NodeId as LinearGraphId<S>>::Mapper<'a>, S::NodeId>,
    matrix: Vec<T>,
    length: usize,
}

impl<'a, S, T> Matrix<'a, S, T>
where
    S: GraphStorage,
    S::NodeId: LinearGraphId<S>,
{
    pub(crate) fn new(graph: &'a Graph<S>, value: T) -> Self
    where
        T: Clone,
    {
        let length = graph.num_nodes();
        let mapper = ContinuousIndexMapper::new(<S::NodeId as LinearGraphId<S>>::index_mapper(
            graph.storage(),
        ));

        let matrix = vec![value; length * length];

        Self {
            mapper,
            matrix,
            length,
        }
    }

    pub(crate) fn new_from_default(graph: &'a Graph<S>) -> Self
    where
        T: Default,
    {
        let length = graph.num_nodes();
        let mapper = <S::NodeId as LinearGraphId<S>>::index_mapper(graph.storage());

        // TODO: potentially can reuse?
        let mut matrix = Vec::with_capacity(length * length);
        matrix.extend(repeat_with(T::default).take(length * length));

        Self {
            mapper,
            matrix,
            length,
        }
    }
}

impl<'a, S, T> Matrix<'a, S, Option<T>>
where
    S: GraphStorage,
    S::NodeId: LinearGraphId<S>,
{
    pub(crate) fn new_from_option(graph: &'a Graph<S>) -> Self {
        let length = graph.num_nodes();
        let mapper = <S::NodeId as LinearGraphId<S>>::index_mapper(graph.storage());

        let matrix = vec![None; length * length];

        Self {
            mapper,
            matrix,
            length,
        }
    }
}

impl<'a, S, T> Matrix<'a, S, T>
where
    S: GraphStorage,
    S::NodeId: LinearGraphId<S>,
{
    pub(crate) fn set(&mut self, source: &S::NodeId, target: &S::NodeId, value: T) {
        let source = self.mapper.map(source);
        let target = self.mapper.map(target);

        self.matrix[source * self.length + target] = value;
    }

    pub(crate) fn get(&mut self, source: &S::NodeId, target: &S::NodeId) -> &T {
        let source = self.mapper.map(source);
        let target = self.mapper.map(target);

        &self.matrix[source * self.length + target]
    }

    pub(crate) fn diagonal(&self) -> impl Iterator<Item = &T> {
        let len = self.length;

        (0..len).map(move |i| &self.matrix[i * len + i])
    }
}
