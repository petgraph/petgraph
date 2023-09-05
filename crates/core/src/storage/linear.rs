use core::marker::PhantomData;

use crate::{id::LinearGraphId, storage::GraphStorage};

pub trait LinearIndexLookup {
    type GraphId: LinearGraphId;

    fn get(&self, id: &Self::GraphId) -> Option<usize>;
}

struct NoopLinearIndexLookup<I>(PhantomData<fn() -> *const I>);

impl<I> NoopLinearIndexLookup<I> {
    fn new() -> Self {
        Self(PhantomData)
    }
}

impl<I> LinearIndexLookup for NoopLinearIndexLookup<I>
where
    I: LinearGraphId,
{
    type GraphId = I;

    fn get(&self, _id: &Self::GraphId) -> Option<usize> {
        None
    }
}

pub trait LinearGraphStorage: GraphStorage {
    fn linear_edge_index_lookup(&self) -> impl LinearIndexLookup<GraphId = Self::EdgeId> + '_
    where
        Self::EdgeId: LinearGraphId,
    {
        NoopLinearIndexLookup::new()
    }

    fn linear_node_index_lookup(&self) -> impl LinearIndexLookup<GraphId = Self::NodeId> + '_
    where
        Self::NodeId: LinearGraphId,
    {
        NoopLinearIndexLookup::new()
    }
}
