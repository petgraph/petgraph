mod linear;

#[cfg(feature = "alloc")]
pub use self::linear::ContinuousIndexMapper;
pub use self::linear::{Continuity, Continuous, Discrete, IndexMapper, LinearGraphId};
use crate::attributes::NoValue;

// The `PartialEq` bound is required for the default implementation, we could in theory remove it,
// but would need to remove _many_ default implementations that rely on it.
// Another possibility would've been to use `Hash` for `GraphId`, but `ArbitaryGraphId` needs to be
// a wrapper type anyway, so it could just require `Hash` for the inner type, and then implement
// `PartialEq` based on that.
pub trait GraphId: PartialEq {
    type AttributeIndex;
}

pub trait ManagedGraphId: GraphId<AttributeIndex = NoValue> {}

pub trait ArbitraryGraphId: GraphId<AttributeIndex = Self> {}
