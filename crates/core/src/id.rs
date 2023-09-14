use crate::attributes::NoValue;

// TODO: can we remove the `PartialEq` bound?
pub trait GraphId: PartialEq {
    type AttributeIndex;
}

pub trait LinearGraphId: GraphId {}

pub trait ManagedGraphId: GraphId<AttributeIndex = NoValue> {}

pub trait ArbitraryGraphId: GraphId<AttributeIndex = Self> {}
