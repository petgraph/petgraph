//! Generic proptest strategies and support code for petgraph graphs
//!
//! Currently only supports the `default` strategy, which is used by the default implementation of
//! all `Arbitrary` impls.
//!
//! In the future more strategies will be added.
// #![no_std]

extern crate alloc;

pub mod binary;
pub mod dag;
pub mod default;
pub mod tournament;
pub mod vtable;
