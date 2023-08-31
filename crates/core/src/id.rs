use crate::attributes::Never;

pub trait GraphId: PartialEq {
    type AttributeIndex;
}

pub trait LinearGraphId: GraphId {
    fn as_linear(&self) -> usize;
}

pub trait ManagedGraphId: GraphId<AttributeIndex = Never> {}

pub trait ArbitraryGraphId: GraphId<AttributeIndex = Self> {}
