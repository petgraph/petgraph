#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

// these modules define trait-implementing macros
#[macro_use]
#[deprecated(since = "0.1.0")]
pub mod visit;
pub mod edge;
pub mod index;
#[macro_use]
#[deprecated(since = "0.1.0")]
#[cfg(feature = "alloc")]
pub mod data;
#[deprecated(since = "0.1.0")]
pub mod deprecated;
mod graph;
#[doc(hidden)]
#[deprecated(since = "0.1.0")]
mod macros;
mod matrix;
mod node;
mod resize;
mod storage;
mod utils;
