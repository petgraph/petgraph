#![feature(return_position_impl_trait_in_trait)]
#![feature(impl_trait_in_assoc_type)]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod attributes;
pub mod base;
#[deprecated(since = "0.1.0")]
pub mod deprecated;
pub mod edge;
pub(crate) mod graph;
pub mod id;
pub mod node;
pub mod storage;

pub use crate::{
    edge::{DetachedEdge, Edge, EdgeMut, GraphDirectionality},
    graph::Graph,
    id::{ArbitraryGraphId, GraphId, ManagedGraphId},
    node::{DetachedNode, Node, NodeMut},
    storage::{DirectedGraphStorage, GraphStorage},
};
