#![feature(return_position_impl_trait_in_trait)]
//! Graph algorithms.
//!
//! It is a goal to gradually migrate the algorithms to be based on graph traits
//! so that they are generally applicable. For now, some of these still require
//! the `Graph` type.
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(all(doc, nightly), feature(doc_auto_cfg))]
#![cfg_attr(nightly, feature(error_in_core))]

// TODO: instead of returning `IndexMap` we should return a container that is specific to the
// algorithm with common methods (and traits?).
// That way we can ensure immutability and are independent of the underlying container.

extern crate alloc;

pub mod bipartite;
mod common;
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

#[cfg(test)]
pub(crate) mod tests {
    use petgraph_core::{edge::Directed, id::IndexType};
    use petgraph_graph::{Graph, NodeIndex};

    // A graph is topologically sorted if for every edge `(u, v)`, `u` comes before `v` in the
    // ordering.
    fn assert_topologically_sorted_edges<N, E, Ix>(
        graph: &Graph<N, E, Directed, Ix>,
        order: &[NodeIndex<Ix>],
    ) where
        Ix: IndexType,
    {
        // check all the edges of the graph
        for edge in graph.raw_edges() {
            let source = edge.source();
            let target = edge.target();

            if source == target {
                continue;
            }

            let source_index = order
                .iter()
                .position(|x| *x == source)
                .expect("Source node not found");

            let target_index = order
                .iter()
                .position(|x| *x == target)
                .expect("Target node not found");

            assert!(
                source_index < target_index,
                "Graph is not topologically sorted ({target} comes before {source})",
            );
        }
    }

    // A graph is topologically sorted if for every edge `(u, v)`, `u` comes before `v` in the
    // ordering.
    pub fn assert_topologically_sorted<N, E, Ix>(
        graph: &Graph<N, E, Directed, Ix>,
        order: &[NodeIndex<Ix>],
    ) where
        Ix: IndexType,
    {
        assert_eq!(graph.node_count(), order.len());

        assert_topologically_sorted_edges(graph, order);
    }

    pub fn assert_subset_topologically_sorted<N, Ix>(
        graph: &Graph<N, (), Directed, Ix>,
        order: &[NodeIndex<Ix>],
    ) where
        Ix: IndexType,
    {
        // To be a subset it must smaller or equal to the graph
        assert!(graph.node_count() >= order.len());

        assert_topologically_sorted_edges(graph, order);
    }
}
