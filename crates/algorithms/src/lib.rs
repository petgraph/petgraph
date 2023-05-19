//! Graph algorithms.
//!
//! It is a goal to gradually migrate the algorithms to be based on graph traits
//! so that they are generally applicable. For now, some of these still require
//! the `Graph` type.
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(all(doc, nightly), feature(doc_auto_cfg))]
#![cfg_attr(nightly, feature(error_in_core))]

extern crate alloc;

pub mod bipartite;
pub mod components;
pub mod connectivity;
pub mod cycles;
pub mod dag;
pub mod dominance;
pub mod error;
pub mod heuristics;
pub mod isomorphism;
pub mod operators;
pub mod shortest_paths;
pub mod simple_paths;
pub mod traversal;
pub mod tree;
mod utilities;
