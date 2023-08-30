use crate::storage::GraphStorage;

pub trait GraphIndex<S>
where
    S: GraphStorage,
{
}

// TODO: how tf does that work across crate boundaries with different storages?
pub trait ArbitraryGraphIndex<S>: GraphIndex<S>
where
    S: GraphStorage,
{
}

pub trait ManagedGraphIndex<S>: GraphIndex<S>
where
    S: GraphStorage,
{
    fn next(storage: &S) -> Self;
}
