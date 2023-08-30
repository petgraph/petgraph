use crate::{attributes::Never, storage::GraphStorage};

pub trait GraphIndex: PartialEq {
    type Storage: GraphStorage;

    type InsertValue;

    fn convert(storage: &Self::Storage, value: Self::InsertValue) -> Self;
}

pub trait ManagedGraphIndex: GraphIndex<InsertValue = Never> {
    fn next(storage: &Self::Storage) -> Self;
}

pub trait ArbitraryGraphIndex: GraphIndex<InsertValue = Self> {}
