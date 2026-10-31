/// This module contains several macros for testing graph implementations.
///
/// The macros generate a suite of tests for several of the different traits that `petgraph`
/// provides. There are different macros based on whether the graph is a directed or undirected
/// graph, and whether it supports self-loops or parallel edges.
///
/// The macros run the tests on specified [TestGraph] instances, but these can also be provided by
/// the user. However, this API may break at any point.
pub mod directed;
pub mod editable;
pub mod storable;
pub mod undirected;

pub(crate) use __common::*;

// WARNING: This module contains only internal testing helper functions and should not be considered
// part of the public API. It is subject to change at any time.
#[doc(hidden)]
pub mod __common {
    use alloc::vec::Vec;

    use crate::graph::EditableGraph;

    /// The test graph used by most test macros.
    ///
    /// The graph looks as follows:
    /// ```text
    /// 0 ---> 1
    /// |      |
    /// v      v
    /// 2 <--- 3     4
    /// ```
    pub const DEFAULT_TEST_GRAPH: TestGraph = TestGraph {
        node_count: 5,
        edge_endpoints: &[(0, 1), (0, 2), (1, 3), (3, 2)],
    };

    /// A test graph with only two isolated vertices.
    ///
    /// The graph looks as follows:
    /// ```text
    /// 0       1
    /// ```
    pub const ISOLATED_TEST_GRAPH: TestGraph = TestGraph {
        node_count: 2,
        edge_endpoints: &[],
    };

    /// A test graph containing self-loops.
    ///
    /// The graph looks as follows:
    /// ```text
    ///  ---        ---
    /// |   |      |   |
    /// |   |      |   |
    ///  -->0 ---> 1<--
    ///     ^      |
    ///     |      v
    ///      ----- 2---
    ///            ^   |
    ///            |   |
    ///             ---
    /// ```
    pub const LOOP_TEST_GRAPH: TestGraph = TestGraph {
        node_count: 3,
        edge_endpoints: &[(0, 0), (0, 1), (1, 1), (1, 2), (2, 2), (2, 0)],
    };

    /// A test graph containing self-loops, parallel edges, antiparallel edges and an isolated node.
    ///
    /// The graph looks as follows:
    /// ```text
    ///      --->--      --
    ///     |      |    | /
    ///     |      v    v/
    ///  -->0 ---> 1 -> 2<-      3
    /// |  /^      |    |  |
    /// | / |      |    |  |
    ///  -   --<---      --
    /// ```
    pub const MULTI_TEST_GRAPH: TestGraph = TestGraph {
        node_count: 4,
        edge_endpoints: &[(0, 0), (0, 1), (0, 1), (1, 0), (1, 2), (2, 2), (2, 2)],
    };

    /// Description of a graph that the test macros build and run their tests on.
    ///
    /// Nodes are referred to by their index in `0..node_count`, which is also the order in which
    /// they are added to the graph. Edges are added in the order given in `edge_endpoints`, with
    /// the shown directionality.
    ///
    /// All expected results of the tests must be derived from this description, so the test
    /// macros work with any test graph.
    #[derive(Debug, Copy, Clone)]
    pub struct TestGraph {
        /// The number of nodes in the graph.
        pub node_count: usize,
        /// The `(source, target)` pairs of the edges in the graph, given as node indices.
        pub edge_endpoints: &'static [(usize, usize)],
    }

    impl TestGraph {
        /// Returns the number of edges in the graph.
        #[must_use]
        pub const fn edge_count(&self) -> usize {
            self.edge_endpoints.len()
        }

        /// Builds the test graph by adding its nodes and edges to the empty graph returned by
        /// `graph_constructor`.
        ///
        /// The graphs directionality is determined by the graph type `G`. The edges are added with
        /// the directionality given in [`Self::edge_endpoints`].
        ///
        /// # Panics
        /// Panics if adding a node or an edge to the graph returns an error, or if an edge endpoint
        /// is not a valid node index.
        pub fn build<G: EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>>(
            &self,
            graph_constructor: impl Fn() -> G,
        ) -> (G, Vec<G::NodeId>, Vec<G::EdgeId>) {
            let mut graph = graph_constructor();

            let nodes: Vec<G::NodeId> = (0..self.node_count)
                .map(|_| graph.add_node(()).unwrap())
                .collect();

            let edges = self
                .edge_endpoints
                .iter()
                .map(|&(source, target)| graph.add_edge(nodes[source], nodes[target], ()).unwrap())
                .collect();

            (graph, nodes, edges)
        }

        /// Returns whether the graph contains an edge whose `(source, target)` node indices satisfy
        /// `predicate`.
        pub fn has_edge(&self, predicate: impl Fn(usize, usize) -> bool) -> bool {
            self.edge_endpoints
                .iter()
                .any(|&(source, target)| predicate(source, target))
        }

        /// Returns the ids of the nodes whose index satisfies `predicate`.
        ///
        /// `nodes` has to be the node ids returned by [`Self::build`].
        pub fn expected_nodes<NodeId: Copy>(
            &self,
            nodes: &[NodeId],
            predicate: impl Fn(usize) -> bool,
        ) -> impl Iterator<Item = NodeId> {
            (0..self.node_count)
                .zip(nodes)
                .filter(move |&(index, _)| predicate(index))
                .map(|(_, &node)| node)
        }

        /// Returns the ids of the nodes that `select` picks from the `(source, target)` node
        /// indices of each edge. A node is returned once for every edge it is picked from.
        ///
        /// `nodes` has to be the node ids returned by [`Self::build`].
        pub fn expected_neighbors<NodeId: Copy>(
            &self,
            nodes: &[NodeId],
            select: impl Fn(usize, usize) -> Option<usize>,
        ) -> impl Iterator<Item = NodeId> {
            self.edge_endpoints
                .iter()
                .filter_map(move |&(source, target)| select(source, target))
                .map(|index| nodes[index])
        }

        /// Returns the ids of the edges whose `(source, target)` node indices satisfy `predicate`.
        ///
        /// `edges` has to be the edge ids returned by [`Self::build`].
        pub fn expected_edges<EdgeId: Copy>(
            &self,
            edges: &[EdgeId],
            predicate: impl Fn(usize, usize) -> bool,
        ) -> impl Iterator<Item = EdgeId> {
            self.edge_endpoints
                .iter()
                .zip(edges)
                .filter(move |&(&(source, target), _)| predicate(source, target))
                .map(|(_, &edge)| edge)
        }

        /// Returns the `(id, source, target)` triples of the edges in the graph.
        ///
        /// `nodes` and `edges` have to be the ids returned by [`Self::build`].
        pub fn expected_edge_endpoints<NodeId: Copy, EdgeId: Copy>(
            &self,
            nodes: &[NodeId],
            edges: &[EdgeId],
        ) -> Vec<(EdgeId, NodeId, NodeId)> {
            self.edge_endpoints
                .iter()
                .zip(edges)
                .map(|(&(source, target), &edge)| (edge, nodes[source], nodes[target]))
                .collect()
        }
    }

    /// Helper function to check if the edges returned by an iterator match the expected edges,
    /// including how often each edge is returned.
    ///
    /// The additional arguments are just for better error messages.
    pub fn check_if_edges_match<T>(
        expected_edges: impl IntoIterator<Item = T>,
        actual_edges: impl Iterator<Item = T>,
        method_name: &'static str,
        graph_type: &'static str,
        context: impl core::fmt::Display,
    ) where
        T: core::hash::Hash + Eq + core::fmt::Debug,
    {
        check_if_items_match(
            "edge",
            expected_edges,
            actual_edges,
            method_name,
            graph_type,
            context,
        );
    }

    /// Helper function to check if the nodes returned by an iterator match the expected nodes,
    /// including how often each node is returned.
    ///
    /// The additional arguments are just for better error messages.
    pub fn check_if_nodes_match<T>(
        expected_nodes: impl IntoIterator<Item = T>,
        actual_nodes: impl Iterator<Item = T>,
        method_name: &'static str,
        graph_type: &'static str,
        context: impl core::fmt::Display,
    ) where
        T: core::hash::Hash + Eq + core::fmt::Debug,
    {
        check_if_items_match(
            "node",
            expected_nodes,
            actual_nodes,
            method_name,
            graph_type,
            context,
        );
    }

    /// Checks that `actual_items` contains every expected item exactly as often as
    /// `expected_items` does. Items can be expected multiple times, e.g. adjacent nodes in a
    /// graph with parallel edges.
    fn check_if_items_match<T>(
        item_kind: &'static str,
        expected_items: impl IntoIterator<Item = T>,
        actual_items: impl Iterator<Item = T>,
        method_name: &'static str,
        graph_type: &'static str,
        context: impl core::fmt::Display,
    ) where
        T: core::hash::Hash + Eq + core::fmt::Debug,
    {
        let mut remaining_items: hashbrown::HashMap<T, usize, foldhash::fast::RandomState> =
            hashbrown::HashMap::default();
        for item in expected_items {
            *remaining_items.entry(item).or_default() += 1;
        }

        for item in actual_items {
            let Some(count) = remaining_items.get_mut(&item) else {
                panic!(
                    "{graph_type}::{method_name}() contained unexpected {item_kind} id (or \
                     contained it more often than expected): {item:?} for {context}",
                );
            };

            *count -= 1;
            if *count == 0 {
                remaining_items.remove(&item);
            }
        }

        assert!(
            remaining_items.is_empty(),
            "{graph_type}::{method_name}() did not return all expected {item_kind}s for {context} \
             (missing ids with their missing count): {remaining_items:?}",
        );
    }
}
