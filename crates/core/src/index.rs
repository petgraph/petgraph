use crate::{attributes::Never, storage::GraphStorage};

pub trait GraphId: PartialEq {
    type Storage: GraphStorage;

    type AttributeIndex;

    fn convert(storage: &Self::Storage, value: Self::AttributeIndex) -> Self;
}

pub trait LinearGraphId: GraphId {
    fn as_linear(&self, storage: &Self::Storage) -> usize;
}

pub trait ManagedGraphId: GraphId<AttributeIndex = Never> {
    fn next(storage: &Self::Storage) -> Self;
}

pub trait ArbitraryGraphId: GraphId<AttributeIndex = Self> {}
