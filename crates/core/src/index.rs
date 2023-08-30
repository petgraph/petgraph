use crate::{attributes::Never, storage::GraphStorage};

pub trait GraphIndex: PartialEq {
    type Storage: GraphStorage;

    type InsertValue;

    fn convert(storage: &Self::Storage, value: Self::InsertValue) -> Self;
}

pub trait LinearGraphIndex: GraphIndex {
    fn as_linear(&self, storage: &Self::Storage) -> usize;
}

pub trait ManagedGraphIndex: GraphIndex<InsertValue = Never> {
    fn next(storage: &Self::Storage) -> Self;
}

pub trait ArbitraryGraphIndex: GraphIndex<InsertValue = Self> {}
