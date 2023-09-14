#![feature(return_position_impl_trait_in_trait)]
#![feature(impl_trait_in_assoc_type)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod attributes;
#[deprecated(since = "0.1.0")]
pub mod deprecated;
pub mod edge;
pub mod graph;
pub mod id;
pub mod node;
pub mod storage;
