use crate::attributes::NoValue;

pub trait GraphId: PartialEq {
    type AttributeIndex;
}

pub trait LinearGraphId: GraphId {}

pub trait ManagedGraphId: GraphId<AttributeIndex =NoValue> {}

pub trait ArbitraryGraphId: GraphId<AttributeIndex = Self> {}
