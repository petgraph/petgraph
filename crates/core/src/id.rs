use crate::{attributes::Never, storage::GraphStorage};

pub trait GraphId: PartialEq {
    type AttributeIndex;
}

pub trait LinearGraphIdMapper: Sized {
    type Id: LinearGraphId;

    fn map(&self, id: &Self::Id) -> usize;
}

pub trait LinearGraphId: GraphId {
    type Storage: GraphStorage;
    type Mapper<'a>: LinearGraphIdMapper<Id = Self> + 'a
    where
        Self: 'a;

    fn mapper(storage: &Self::Storage) -> Self::Mapper<'_>;
}

pub trait ManagedGraphId: GraphId<AttributeIndex = Never> {}

pub trait ArbitraryGraphId: GraphId<AttributeIndex = Self> {}
