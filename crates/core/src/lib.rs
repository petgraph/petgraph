#![feature(return_position_impl_trait_in_trait)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod adjacency_matrix;
mod attributes;
#[deprecated(since = "0.1.0")]
pub mod deprecated;
mod edge;
mod graph;
mod index;
mod node;
mod storage;
