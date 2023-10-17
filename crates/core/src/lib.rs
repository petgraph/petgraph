//! # `petgraph-core`: Core Capabilities for Graph Manipulation
//!
//! Welcome to `petgraph-core`, a Rust library that serves as the bedrock for the petgraph
//! ecosystem.
//! The library is by default `no-std` and `no-alloc` and is focused on providing core
//! functionalities for graph operations, while abstracting over any specific graph implementation.
//! This means to fully utilize this library you need to either provide your own graph
//! implementation or use one of the many existing graph implementations.
//!
//! Within `petgraph-core` you'll discover the fundamental types crucial to working with graphs,
//! such as [`Node`] and [`Edge`].
//! The central [`Graph`] type which is the main entry point for working with graphs.
//! The [`GraphStorage`] and [`DirectedGraphStorage`] traits which define the storage requirements
//! for a graph.
//!
//! ## Key Features
//!
//! - **Graph Storage Agnostic**: `petgraph-core` is designed to be agnostic to the underlying
//!   storage of the graph. This means that you can use any graph implementation that implements the
//!   [`GraphStorage`] and [`DirectedGraphStorage`] traits.
//! - **Graph Manipulation**: `petgraph-core` provides the core types and traits for working with
//!   graphs. This includes the [`Graph`] type which is the main entry point for working with
//!   graphs.
//! - **Node and Edge Types**: `petgraph-core` provides the [`Node`] and [`Edge`] types which are
//!   used to access specific nodes and edges in a graph, as well as provide a way to explore the
//!   graph in a more ergonomic way.
//! - **Graph Ids**: `petgraph-core` provides the [`GraphId`] trait which is used to identify a
//!  graph.
//! - **Graph Directionality**: `petgraph-core` provides the [`GraphDirectionality`] trait which can
//!   be used by graph implementations to indicate whether the graph is directed or undirected.
//!
//! The focus of `petgraph-core` is to provide the core types and traits for working with graphs.
//! While preserving flexibility and increasing usability.
//!
//! ## Usage
//!
//! To use `petgraph-core` you need to add it as a dependency in your `Cargo.toml` file:
//!
//! ```toml
//! [dependencies]
//! petgraph-core = "0.1.0"
//! ```
//!
//! ## Deprecated
//!
//! `petgraph` 0.7.0 signaled a major change in the library, with the introduction of a new paradigm
//! and all new traits. To make conversion of code easier the old traits are deprecated but
//! preserved in the `deprecated` module.
//!
//! This module will be removed in `petgraph` 0.8.0.
//!
//! ## Examples
//!
//! The following example demonstrates how to create a graph and add nodes and edges to it.
//! The chosen implementation in this example (and all other examples of the crate) use
//! `petgraph-dino`, a graph implementation that is based on a generational arena and fast lookup
//! through compressed bitsets.
//!
//! ```rust
//! use petgraph_core::{edge::marker::Directed, Graph};
//! use petgraph_dino::DinoStorage;
//!
//! let mut graph = Graph::<DinoStorage<_, _, Directed>>::new();
//! // ^ a more convenient alias `DiDinoGraph` exists.
//!
//! // Add some nodes to the graph.
//! // `insert_node` returns the `NodeMut` of the inserted node.
//! let a = *graph.insert_node("A").id();
//! let b = *graph.insert_node("B").id();
//! let c = *graph.insert_node("C").id();
//!
//! // Add some edges to the graph.
//! // `insert_edge` returns the `EdgeMut` of the inserted edge.
//! let ab = *graph.insert_edge("A → A", &a, &b).id();
//! let bc = *graph.insert_edge("B → C", &b, &c).id();
//! let ca = *graph.insert_edge("C → A", &c, &a).id();
//!
//! // We can now access the nodes and edges in the graph.
//! assert_eq!(graph.node(&a).map(|node| node.weight()), Some(&"A"));
//! assert_eq!(graph.node(&b).map(|node| node.weight()), Some(&"B"));
//! assert_eq!(graph.node(&c).map(|node| node.weight()), Some(&"C"));
//!
//! assert_eq!(graph.edge(&ab).map(|edge| edge.weight()), Some(&"A → A"));
//! assert_eq!(graph.edge(&bc).map(|edge| edge.weight()), Some(&"B → C"));
//! assert_eq!(graph.edge(&ca).map(|edge| edge.weight()), Some(&"C → A"));
//!
//! // We can even mutate the nodes and edges in the graph.
//! if let Some(mut node) = graph.node_mut(&a) {
//!     *node.weight_mut() = "A'";
//! }
//!
//! assert_eq!(graph.node(&a).map(|node| node.weight()), Some(&"A'"));
//! ```
//!
//! ## Feature Flags
//!
//! | Feature       | Description                                                                                       | Default  |
//! |---------------|---------------------------------------------------------------------------------------------------|----------|
//! | `alloc`       | Enables [`DataMap`] on [`NodeFiltered`], enables [`ContinuousIndexMapper`]                        | enabled  |
//! | `std`         | Enables [`VisitMap`] and [`FilterNode`] and on [`HashSet`]                                        | enabled  |
//! | `fixedbitset` | Enables [`GetAdjacencyMatrix`] for [`Graph`], [`VisitMap`] and [`FilterNode`] for [`FixedBitSet`] | disabled |
//! | `indexmap`    | Enables [`VisitMap`] for [`IndexMap`]                                                             | disabled |
//!
//! `std`, `fixedbitset` and `indexmap` only exist for the `deprecated` module and will be removed
//! in `petgraph` 0.8.0.
//!
//! [`GetAdjacencyMatrix`]: deprecated::visit::GetAdjacencyMatrix
//! [`VisitMap`]: deprecated::visit::VisitMap
//! [`FilterNode`]: deprecated::visit::FilterNode
//! [`DataMap`]: deprecated::data::DataMap
//! [`NodeFiltered`]: deprecated::visit::NodeFiltered
//! [`ContinuousIndexMapper`]: id::ContinuousIndexMapper
//! [`FixedBitSet`]: fixedbitset::FixedBitSet
//! [`IndexMap`]: indexmap::IndexMap
//! [`HashSet`]: std::collections::HashSet
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
