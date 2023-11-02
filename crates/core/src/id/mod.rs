//! Module for identifiers.
//!
//! Identifiers are used to _identify_ edges and nodes.
//!
//! Primarily this module defines one type: [`GraphId`], which should not be implemented alone, but
//! instead be implemented alongside [`ManagedGraphId`] and [`ArbitraryGraphId`], which are mutually
//! exclusive and define if a node or edge id will be automatically assigned by the graph (are
//! managed) and a user has no control over their value or are arbitrary, allowing the user to use
//! _any_ value.
mod attributes;
mod flag;
mod linear;

pub use self::{
    attributes::{AttributeGraphId, AttributeStorage},
    flag::{FlagStorage, FlaggableGraphId},
    linear::{IndexMapper, LinearGraphId},
};
use crate::attributes::NoValue;

// The `PartialEq` bound is required for the default implementation, we could in theory remove it,
// but would need to remove _many_ default implementations that rely on it.
// Another possibility would've been to use `Hash` for `GraphId`, but `ArbitaryGraphId` needs to be
// a wrapper type anyway, so it could just require `Hash` for the inner type, and then implement
// `PartialEq` based on that.
/// A unique identifier for a node or edge.
///
/// This trait is implemented for all types that are used as node or edge identifiers in the graph.
/// A type should never only implement this trait, but also [`ManagedGraphId`] or
/// [`ArbitraryGraphId`].
pub trait GraphId: PartialEq {
    /// The type of value used to index attributes.
    ///
    /// Used to differentiate between [`ManagedGraphId`] and [`ArbitraryGraphId`] and to allow for
    /// inference on weights via the [`Attributes`] type.
    ///
    /// There are essentially two valid values for this type: [`NoValue`] and `Self`.
    ///
    /// [`Attributes`]: crate::attributes::Attributes
    type AttributeIndex;
}

/// A unique identifier for a node or edge that is managed by the graph.
///
/// Marker trait to indicate that the graph manages the identifier of the node or edge, and cannot
/// be specified by the user itself.
///
/// This is analogous to an index in a `Vec`.
pub trait ManagedGraphId: GraphId<AttributeIndex = NoValue> {}

/// A unique identifier for a node or edge that is not managed by the graph.
///
/// Marker trait to indicate that the graph does not manage the identifier of the node or edge, and
/// must be specified by the user itself.
///
/// This is analogous to a key in a `HashMap`.
pub trait ArbitraryGraphId: GraphId<AttributeIndex = Self> {}
