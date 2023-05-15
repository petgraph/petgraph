#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

// these modules define trait-implementing macros
#[macro_use]
pub mod visit;
