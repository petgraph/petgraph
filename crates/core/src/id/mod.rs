mod linear;

pub use self::linear::{ContinuousIndexMapper, IndexMapper, LinearGraphId};
use crate::attributes::NoValue;

// TODO: can we remove the `PartialEq` bound?
pub trait GraphId: PartialEq {
    type AttributeIndex;
}

pub trait ManagedGraphId: GraphId<AttributeIndex = NoValue> {}

pub trait ArbitraryGraphId: GraphId<AttributeIndex = Self> {}
