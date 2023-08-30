#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[deprecated(since = "0.1.0")]
pub mod deprecated;
pub mod edge;
pub mod graph;
pub mod index;
pub mod matrix;
pub mod node;
pub mod storage;
mod attributes;
