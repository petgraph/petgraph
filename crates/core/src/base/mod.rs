//! Base types for the core crate.
//!
//! These are mostly analogue types that are needed to represent specific constructs that are not
//! possible in `core` Rust.
//!
//! You should not rely on these types outside of petgraph specific API code, as they are very much
//! temporary and a stand-in for a more general solution. This may include moving some of the types
//! into their own crate.
//!
//! This currently includes:
//! * [`MaybeOwned`]: A type that can either be owned or borrowed, an analogue to [`Cow`].
//!
//! [`Cow`]: std::borrow::Cow
pub(crate) mod owned;

pub use self::owned::MaybeOwned;
// ^ TODO: into own crate
