//! # Numi
//!
//! `numi` is a crate with a collection of common utility traits and abstractions for working with
//! numbers and borrowed data.
//!
//! The library has been developed in conjunction with [`petgraph`], but is not specific to it.
// TODO: extensive documentation with examples

#![no_std]

pub mod borrow;
pub mod cast;
mod macros;
pub mod num;
pub mod primitive;
