#![no_std]
#![allow(clippy::missing_errors_doc, reason = "bootstrap")]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod edge;
pub mod graph;
pub mod id;
pub mod node;
#[cfg(feature = "test-utils")]
pub mod utils;
