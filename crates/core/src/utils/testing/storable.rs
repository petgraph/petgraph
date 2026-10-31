/// Generates a suite of tests for [`StorableGraph`][crate::graph::storable::StorableGraph]
/// implementations, which are also [`DirectedGraph`][crate::graph::DirectedGraph]s. It runs on the
/// [`DEFAULT_TEST_GRAPH`][crate::utils::testing::__common::DEFAULT_TEST_GRAPH] and
/// [`ISOLATED_TEST_GRAPH`][crate::utils::testing::__common::ISOLATED_TEST_GRAPH].
///
/// For the expected invariants and the arguments of this macro, see
/// [`__test_storable_graph_inner`][crate::__test_storable_graph_inner]. To run the suite on other
/// test graphs, use
/// [`__test_directed_storable_graph_inner`][crate::__test_directed_storable_graph_inner].
///
/// For a reference usage, see the tests in [`crate::utils::test_graphs::directed`].
#[macro_export]
macro_rules! test_directed_storable_graph {
    ($graph_constructor:expr,) => {
        mod storable_graph_macro {
            mod default {
                #[allow(unused_imports)]
                use super::super::*;

                $crate::__test_directed_storable_graph_inner!(
                    $graph_constructor,
                    $crate::utils::testing::__common::DEFAULT_TEST_GRAPH,
                );
            }

            mod isolated {
                #[allow(unused_imports)]
                use super::super::*;

                $crate::__test_directed_storable_graph_inner!(
                    $graph_constructor,
                    $crate::utils::testing::__common::ISOLATED_TEST_GRAPH,
                );
            }
        }
    };
}

/// Generates a suite of tests for [`StorableGraph`][crate::graph::storable::StorableGraph]
/// implementations, which are also [`DirectedGraph`][crate::graph::DirectedGraph]s and support
/// self-loops. It runs on the
/// [`LOOP_TEST_GRAPH`][crate::utils::testing::__common::LOOP_TEST_GRAPH].
///
/// For the expected invariants and the arguments of this macro, see
/// [`__test_storable_graph_inner`][crate::__test_storable_graph_inner]. To run the suite on other
/// test graphs, use
/// [`__test_directed_storable_graph_inner`][crate::__test_directed_storable_graph_inner].
///
/// For a reference usage, see the tests in [`crate::utils::test_graphs::directed`].
#[macro_export]
macro_rules! test_directed_storable_graph_self_loops {
    ($graph_constructor:expr,) => {
        mod storable_graph_macro_self_loops {
            #[allow(unused_imports)]
            use super::*;

            $crate::__test_directed_storable_graph_inner!(
                $graph_constructor,
                $crate::utils::testing::__common::LOOP_TEST_GRAPH,
            );
        }
    };
}

/// Generates a suite of tests for [`StorableGraph`][crate::graph::storable::StorableGraph]
/// implementations, which are also [`DirectedGraph`][crate::graph::DirectedGraph]s and support
/// self-loops and parallel edges. It runs on the
/// [`MULTI_TEST_GRAPH`][crate::utils::testing::__common::MULTI_TEST_GRAPH].
///
/// For the expected invariants and the arguments of this macro, see
/// [`__test_storable_graph_inner`][crate::__test_storable_graph_inner]. To run the suite on other
/// test graphs, use
/// [`__test_directed_storable_graph_inner`][crate::__test_directed_storable_graph_inner].
///
/// For a reference usage, see the tests in [`crate::utils::test_graphs::directed`].
#[macro_export]
macro_rules! test_directed_storable_graph_parallel_edges {
    ($graph_constructor:expr,) => {
        mod storable_graph_macro_parallel_edges {
            #[allow(unused_imports)]
            use super::*;

            $crate::__test_directed_storable_graph_inner!(
                $graph_constructor,
                $crate::utils::testing::__common::MULTI_TEST_GRAPH,
            );
        }
    };
}

/// Generates a suite of tests for [`StorableGraph`][crate::graph::storable::StorableGraph]
/// implementations, which are also [`DirectedGraph`][crate::graph::DirectedGraph]s. It runs on the
/// given test graph.
///
/// For the expected invariants and the arguments `$graph_constructor` and `$test_graph`, see
/// [`__test_storable_graph_inner`][crate::__test_storable_graph_inner].
///
/// Prefer using [`test_directed_storable_graph`][crate::test_directed_storable_graph] to run
/// the suite on the default test graphs.
#[doc(hidden)]
#[macro_export]
macro_rules! __test_directed_storable_graph_inner {
    ($graph_constructor:expr, $test_graph:expr,) => {
        $crate::__test_storable_graph_inner!(
            $graph_constructor,
            $test_graph,
            |graph| $crate::graph::DirectedGraph::nodes(graph)
                .map(|node| node.id)
                .collect(),
            |graph| $crate::graph::DirectedGraph::edges(graph)
                .map(|edge| edge.id)
                .collect(),
        );
    };
}

/// Generates a suite of tests for [`StorableGraph`][crate::graph::storable::StorableGraph]
/// implementations, which are also [`UndirectedGraph`][crate::graph::UndirectedGraph]s. It runs on
/// the [`DEFAULT_TEST_GRAPH`][crate::utils::testing::__common::DEFAULT_TEST_GRAPH] and
/// [`ISOLATED_TEST_GRAPH`][crate::utils::testing::__common::ISOLATED_TEST_GRAPH].
///
/// For the expected invariants and the arguments of this macro, see
/// [`__test_storable_graph_inner`][crate::__test_storable_graph_inner]. To run the suite on other
/// test graphs, use
/// [`__test_undirected_storable_graph_inner`][crate::__test_undirected_storable_graph_inner].
///
/// For a reference usage, see the tests in [`crate::utils::test_graphs::undirected`].
#[macro_export]
macro_rules! test_undirected_storable_graph {
    ($graph_constructor:expr,) => {
        mod storable_graph_macro {
            mod default {
                #[allow(unused_imports)]
                use super::super::*;

                $crate::__test_undirected_storable_graph_inner!(
                    $graph_constructor,
                    $crate::utils::testing::__common::DEFAULT_TEST_GRAPH,
                );
            }

            mod isolated {
                #[allow(unused_imports)]
                use super::super::*;

                $crate::__test_undirected_storable_graph_inner!(
                    $graph_constructor,
                    $crate::utils::testing::__common::ISOLATED_TEST_GRAPH,
                );
            }
        }
    };
}

/// Generates a suite of tests for [`StorableGraph`][crate::graph::storable::StorableGraph]
/// implementations, which are also [`UndirectedGraph`][crate::graph::UndirectedGraph]s and support
/// self-loops. It runs on the
/// [`LOOP_TEST_GRAPH`][crate::utils::testing::__common::LOOP_TEST_GRAPH].
///
/// For the expected invariants and the arguments of this macro, see
/// [`__test_storable_graph_inner`][crate::__test_storable_graph_inner]. To run the suite on other
/// test graphs, use
/// [`__test_undirected_storable_graph_inner`][crate::__test_undirected_storable_graph_inner].
///
/// For a reference usage, see the tests in [`crate::utils::test_graphs::undirected`].
#[macro_export]
macro_rules! test_undirected_storable_graph_self_loops {
    ($graph_constructor:expr,) => {
        mod storable_graph_macro_self_loops {
            #[allow(unused_imports)]
            use super::*;

            $crate::__test_undirected_storable_graph_inner!(
                $graph_constructor,
                $crate::utils::testing::__common::LOOP_TEST_GRAPH,
            );
        }
    };
}

/// Generates a suite of tests for [`StorableGraph`][crate::graph::storable::StorableGraph]
/// implementations, which are also [`UndirectedGraph`][crate::graph::UndirectedGraph]s and support
/// self-loops and parallel edges. It runs on the
/// [`MULTI_TEST_GRAPH`][crate::utils::testing::__common::MULTI_TEST_GRAPH].
///
/// For the expected invariants and the arguments of this macro, see
/// [`__test_storable_graph_inner`][crate::__test_storable_graph_inner]. To run the suite on other
/// test graphs, use
/// [`__test_undirected_storable_graph_inner`][crate::__test_undirected_storable_graph_inner].
///
/// For a reference usage, see the tests in [`crate::utils::test_graphs::undirected`].
#[macro_export]
macro_rules! test_undirected_storable_graph_parallel_edges {
    ($graph_constructor:expr,) => {
        mod storable_graph_macro_parallel_edges {
            #[allow(unused_imports)]
            use super::*;

            $crate::__test_undirected_storable_graph_inner!(
                $graph_constructor,
                $crate::utils::testing::__common::MULTI_TEST_GRAPH,
            );
        }
    };
}

/// Generates a suite of tests for [`StorableGraph`][crate::graph::storable::StorableGraph]
/// implementations, which are also [`UndirectedGraph`][crate::graph::UndirectedGraph]s. It runs on
/// the given test graph.
///
/// For the expected invariants and the arguments `$graph_constructor` and `$test_graph`, see
/// [`__test_storable_graph_inner`][crate::__test_storable_graph_inner].
///
/// Prefer using [`test_undirected_storable_graph`][crate::test_undirected_storable_graph] to run
/// the suite on the default test graphs.
#[doc(hidden)]
#[macro_export]
macro_rules! __test_undirected_storable_graph_inner {
    ($graph_constructor:expr, $test_graph:expr,) => {
        $crate::__test_storable_graph_inner!(
            $graph_constructor,
            $test_graph,
            |graph| $crate::graph::UndirectedGraph::nodes(graph)
                .map(|node| node.id)
                .collect(),
            |graph| $crate::graph::UndirectedGraph::edges(graph)
                .map(|edge| edge.id)
                .collect(),
        );
    };
}

/// Generates a suite of tests for [`StorableGraph`][crate::graph::storable::StorableGraph]
/// implementations, which runs on the given test graph.
///
/// At least one test is generated for each method in the
/// [`StorableGraph`][crate::graph::storable::StorableGraph] trait, which checks every method of the
/// returned [`VisitContainer`][crate::graph::storable::VisitContainer] or
/// [`DataContainer`][crate::graph::storable::DataContainer]. The containers are checked with the
/// ids of the test graph, so the suite can be run on any test graph that the graph type is able to
/// represent.
///
/// The following invariants are expected from the graph implementation:
/// - If the most recently added node or edge is removed, its ID will no longer be valid.
/// - Removing the most recently added node or edge does not change the IDs of any other nodes or
///   edges.
///
/// The graph type has to implement [`EditableGraph`] to construct the test graph.
///
/// The arguments to this macro are as follows (`G` is used to denote the graph type being tested):
/// - `$graph_constructor`: `impl Fn() -> G`. An expression that constructs a new instance of the
///   graph type to be tested. The generated graph must be empty (e.g. `G::new()`).
/// - `$test_graph`: [`TestGraph`][crate::utils::testing::__common::TestGraph]. The test graph,
///   which is built by adding its nodes and edges to the graphs returned by `$graph_constructor`.
/// - `$node_ids`: `fn(&G) -> Vec<G::NodeId>`. Returns the ids of all nodes in the graph.
/// - `$edge_ids`: `fn(&G) -> Vec<G::EdgeId>`. Returns the ids of all edges in the graph.
///
/// Prefer using [`test_directed_storable_graph`][crate::test_directed_storable_graph] or
/// [`test_undirected_storable_graph`][crate::test_undirected_storable_graph] directly instead.
#[doc(hidden)]
#[macro_export]
macro_rules! __test_storable_graph_inner {
    ($graph_constructor:expr, $test_graph:expr, $node_ids:expr, $edge_ids:expr,) => {
        #[test]
        fn test_node_visit_container() {
            $crate::utils::testing::storable::__private_testing::test_node_visit_container_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_node_data_container() {
            $crate::utils::testing::storable::__private_testing::test_node_data_container_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_edge_visit_container() {
            $crate::utils::testing::storable::__private_testing::test_edge_visit_container_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_edge_data_container() {
            $crate::utils::testing::storable::__private_testing::test_edge_data_container_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_containers_after_graph_edits() {
            $crate::utils::testing::storable::__private_testing::test_containers_after_graph_edits_inner(
                $graph_constructor,
                $test_graph,
            );
        }

        #[test]
        fn test_container_capacity() {
            $crate::utils::testing::storable::__private_testing::test_container_capacity_inner(
                $graph_constructor,
                $test_graph,
                $node_ids,
                $edge_ids,
            );
        }
    };
}

// WARNING: This module contains only internal testing helper functions and should not be considered
// part of the public API. It is subject to change at any time.
#[doc(hidden)]
pub mod __private_testing {
    use alloc::vec::Vec;

    use crate::{
        graph::{
            EditableGraph,
            storable::{DataContainer, StorableGraph, VisitContainer},
        },
        id::Id,
        utils::testing::TestGraph,
    };

    /// The default value passed when creating data containers.
    const DEFAULT_DATA: usize = usize::MAX;
    /// A different default value, used to check that containers do not share their defaults.
    const OTHER_DEFAULT_DATA: usize = usize::MAX - 1;

    /// Asserts that exactly the ids at the indices for which `is_expected_visited` returns `true`
    /// are marked as visited.
    fn assert_visited<I: Id, C: VisitContainer<I>>(
        container: &C,
        ids: &[I],
        is_expected_visited: impl Fn(usize) -> bool,
        method_name: &'static str,
        context: &'static str,
    ) {
        for (index, &id) in ids.iter().enumerate() {
            let expected = is_expected_visited(index);
            assert_eq!(
                container.is_visited(id),
                expected,
                "VisitContainer::is_visited() returned {} for id {id:?} (index {index}) \
                 {context}, using the container returned by StorableGraph::{method_name}()",
                !expected
            );
        }
    }

    /// Asserts that the data stored for each id matches `expected_data` of its index.
    fn assert_data<I: Id, C: DataContainer<I, usize>>(
        container: &C,
        ids: &[I],
        expected_data: impl Fn(usize) -> usize,
        method_name: &'static str,
        context: &'static str,
    ) {
        for (index, &id) in ids.iter().enumerate() {
            assert_eq!(
                *container.get(id),
                expected_data(index),
                "DataContainer::get() returned unexpected data for id {id:?} (index {index}) \
                 {context}, using the container returned by StorableGraph::{method_name}()"
            );
        }
    }

    /// Checks all methods of a [`VisitContainer`] for the given ids.
    fn check_visit_container<I: Id, C: VisitContainer<I>>(
        mut container: C,
        ids: &[I],
        method_name: &'static str,
    ) {
        assert_visited(
            &container,
            ids,
            |_| false,
            method_name,
            "for a new container",
        );

        let Some(&first_id) = ids.first() else {
            return;
        };
        let last = ids.len() - 1;

        for &id in ids.iter().step_by(2) {
            container.mark_visited(id);
        }
        assert_visited(
            &container,
            ids,
            |index| index % 2 == 0,
            method_name,
            "after marking every second id as visited",
        );

        container.mark_visited(first_id);
        assert_visited(
            &container,
            ids,
            |index| index % 2 == 0,
            method_name,
            "after marking id 0 as visited twice",
        );

        container.mark_unvisited(first_id);
        assert_visited(
            &container,
            ids,
            |index| index != 0 && index % 2 == 0,
            method_name,
            "after marking id 0 as unvisited",
        );

        for &id in ids.iter().skip(1).step_by(2) {
            container.mark_unvisited(id);
        }
        assert_visited(
            &container,
            ids,
            |index| index != 0 && index % 2 == 0,
            method_name,
            "after marking the never visited ids as unvisited",
        );

        for &id in ids {
            container.mark_visited(id);
        }
        assert_visited(
            &container,
            ids,
            |_| true,
            method_name,
            "after marking all ids as visited",
        );

        container.clear();
        assert_visited(
            &container,
            ids,
            |_| false,
            method_name,
            "after clearing the container",
        );

        container.mark_visited(ids[last]);
        assert_visited(
            &container,
            ids,
            |index| index == last,
            method_name,
            "after marking the last id as visited after clearing the container",
        );
    }

    /// Checks that two [`VisitContainer`]s created from the same graph do not share state.
    fn check_visit_containers_are_independent<I: Id, C: VisitContainer<I>>(
        mut first_container: C,
        second_container: &C,
        ids: &[I],
        method_name: &'static str,
    ) {
        for &id in ids {
            first_container.mark_visited(id);
        }
        assert_visited(
            second_container,
            ids,
            |_| false,
            method_name,
            "after marking all ids as visited in another container",
        );
    }

    /// Checks all methods of a [`DataContainer`] for the given ids. The container has to be created
    /// with [`DEFAULT_DATA`] as default value.
    fn check_data_container<I: Id, C: DataContainer<I, usize>>(
        mut container: C,
        ids: &[I],
        method_name: &'static str,
    ) {
        assert_data(
            &container,
            ids,
            |_| DEFAULT_DATA,
            method_name,
            "for a new container",
        );

        let Some(&first_id) = ids.first() else {
            return;
        };
        let last = ids.len() - 1;

        // Writing through `get_mut` must not change the default value of the other ids.
        *container.get_mut(ids[last]) = 7;
        assert_data(
            &container,
            ids,
            |index| if index == last { 7 } else { DEFAULT_DATA },
            method_name,
            "after writing to the last id via DataContainer::get_mut()",
        );

        for (index, &id) in ids.iter().enumerate() {
            container.insert(id, index * 10);
        }
        assert_data(
            &container,
            ids,
            |index| index * 10,
            method_name,
            "after inserting data for all ids",
        );

        *container.get_mut(first_id) += 1;
        assert_data(
            &container,
            ids,
            |index| if index == 0 { 1 } else { index * 10 },
            method_name,
            "after modifying id 0 via DataContainer::get_mut()",
        );

        container.insert(ids[last], 1000);
        assert_data(
            &container,
            ids,
            |index| {
                if index == last {
                    1000
                } else if index == 0 {
                    1
                } else {
                    index * 10
                }
            },
            method_name,
            "after replacing the data of the last id",
        );

        let expected_after_removal = |index| {
            if index == last {
                DEFAULT_DATA
            } else if index == 0 {
                1
            } else {
                index * 10
            }
        };

        container.remove(ids[last]);
        assert_data(
            &container,
            ids,
            expected_after_removal,
            method_name,
            "after removing the data of the last id",
        );

        container.remove(ids[last]);
        assert_data(
            &container,
            ids,
            expected_after_removal,
            method_name,
            "after removing the data of the last id twice",
        );

        container.clear();
        assert_data(
            &container,
            ids,
            |_| DEFAULT_DATA,
            method_name,
            "after clearing the container",
        );

        container.insert(first_id, 3);
        assert_data(
            &container,
            ids,
            |index| if index == 0 { 3 } else { DEFAULT_DATA },
            method_name,
            "after inserting data for id 0 after clearing the container",
        );
    }

    /// Calls `check` once for every rotation of `ids`, so that each id is used at every position.
    ///
    /// This way, every container method that `check` calls for a fixed position is called with
    /// every id.
    fn check_all_rotations<I: Id>(ids: &[I], mut check: impl FnMut(&[I])) {
        let mut rotated_ids = ids.to_vec();
        for _ in 0..ids.len() {
            check(&rotated_ids);
            rotated_ids.rotate_left(1);
        }
    }

    /// Checks that two [`DataContainer`]s created from the same graph do not share state.
    ///
    /// The first container has to be created with [`DEFAULT_DATA`] and the second one with
    /// [`OTHER_DEFAULT_DATA`] as default value.
    fn check_data_containers_are_independent<I: Id, C: DataContainer<I, usize>>(
        mut first_container: C,
        second_container: &C,
        ids: &[I],
        method_name: &'static str,
    ) {
        assert_data(
            second_container,
            ids,
            |_| OTHER_DEFAULT_DATA,
            method_name,
            "for a new container with a different default value",
        );

        for (index, &id) in ids.iter().enumerate() {
            first_container.insert(id, index);
        }
        assert_data(
            second_container,
            ids,
            |_| OTHER_DEFAULT_DATA,
            method_name,
            "after inserting data for all ids in another container",
        );
    }

    pub fn test_node_visit_container_inner<
        G: StorableGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) {
        let (graph, nodes, _) = test_graph.build(graph_constructor);

        check_visit_container(
            StorableGraph::node_visit_container(&graph),
            &nodes,
            "node_visit_container",
        );
        check_visit_containers_are_independent(
            StorableGraph::node_visit_container(&graph),
            &StorableGraph::node_visit_container(&graph),
            &nodes,
            "node_visit_container",
        );
    }

    pub fn test_node_data_container_inner<
        G: StorableGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) {
        let (graph, nodes, _) = test_graph.build(graph_constructor);

        check_data_container(
            StorableGraph::node_data_container(&graph, DEFAULT_DATA),
            &nodes,
            "node_data_container",
        );
        check_data_containers_are_independent(
            StorableGraph::node_data_container(&graph, DEFAULT_DATA),
            &StorableGraph::node_data_container(&graph, OTHER_DEFAULT_DATA),
            &nodes,
            "node_data_container",
        );
    }

    pub fn test_edge_visit_container_inner<
        G: StorableGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) {
        let (graph, _, edges) = test_graph.build(graph_constructor);

        check_visit_container(
            StorableGraph::edge_visit_container(&graph),
            &edges,
            "edge_visit_container",
        );
        check_visit_containers_are_independent(
            StorableGraph::edge_visit_container(&graph),
            &StorableGraph::edge_visit_container(&graph),
            &edges,
            "edge_visit_container",
        );
    }

    pub fn test_edge_data_container_inner<
        G: StorableGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) {
        let (graph, _, edges) = test_graph.build(graph_constructor);

        check_data_container(
            StorableGraph::edge_data_container(&graph, DEFAULT_DATA),
            &edges,
            "edge_data_container",
        );
        check_data_containers_are_independent(
            StorableGraph::edge_data_container(&graph, DEFAULT_DATA),
            &StorableGraph::edge_data_container(&graph, OTHER_DEFAULT_DATA),
            &edges,
            "edge_data_container",
        );
    }

    pub fn test_containers_after_graph_edits_inner<
        G: StorableGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
    ) {
        let (mut graph, nodes, edges) = test_graph.build(graph_constructor);
        let mut current_edges = edges;

        // The newest edge and node are removed, as some graph implementations might not have
        // stable ids, but the newest ones added are likely to be removable without another edge or
        // node taking their ids.
        if let Some(last_edge) = current_edges.pop() {
            graph.remove_edge(last_edge).unwrap();
        }
        let removed_node = graph.add_node(()).unwrap();
        graph.remove_node(removed_node).unwrap();

        // Depending on the implementation, the new node might reuse the id of the removed node or
        // get an id beyond the ids of the original nodes. The containers have to support both.
        let new_node = graph.add_node(()).unwrap();
        if let [first_node, .., last_node] = nodes[..] {
            current_edges.push(graph.add_edge(last_node, new_node, ()).unwrap());
            current_edges.push(graph.add_edge(new_node, first_node, ()).unwrap());
        }

        let mut current_nodes = nodes;
        current_nodes.push(new_node);

        check_visit_container(
            StorableGraph::node_visit_container(&graph),
            &current_nodes,
            "node_visit_container",
        );
        check_data_container(
            StorableGraph::node_data_container(&graph, DEFAULT_DATA),
            &current_nodes,
            "node_data_container",
        );
        check_visit_container(
            StorableGraph::edge_visit_container(&graph),
            &current_edges,
            "edge_visit_container",
        );
        check_data_container(
            StorableGraph::edge_data_container(&graph, DEFAULT_DATA),
            &current_edges,
            "edge_data_container",
        );
    }

    pub fn test_container_capacity_inner<
        G: StorableGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
    >(
        graph_constructor: fn() -> G,
        test_graph: TestGraph,
        node_ids: fn(&G) -> Vec<G::NodeId>,
        edge_ids: fn(&G) -> Vec<G::EdgeId>,
    ) {
        let (mut graph, nodes, edges) = test_graph.build(graph_constructor);
        let Some(&first_node) = nodes.first() else {
            return;
        };

        // Node 0 is the oldest node. For graph implementations with stable ids, removing it leaves
        // gaps in the ids, so that the largest remaining ids are larger than the number of nodes
        // and edges.
        graph.remove_node(first_node).unwrap();

        // Other graph implementations might change the ids of the remaining nodes and edges, so
        // the current ids are obtained from the graph.
        let current_nodes = node_ids(&graph);
        let current_edges = edge_ids(&graph);
        assert_eq!(
            current_nodes.len(),
            test_graph.node_count - 1,
            "unexpected number of node ids returned by the graph after removing node 0"
        );
        assert_eq!(
            current_edges.len(),
            test_graph
                .expected_edges(&edges, |source, target| source != 0 && target != 0)
                .count(),
            "unexpected number of edge ids returned by the graph after removing node 0"
        );

        check_all_rotations(&current_nodes, |ids| {
            check_visit_container(
                StorableGraph::node_visit_container(&graph),
                ids,
                "node_visit_container",
            );
            check_data_container(
                StorableGraph::node_data_container(&graph, DEFAULT_DATA),
                ids,
                "node_data_container",
            );
        });
        check_all_rotations(&current_edges, |ids| {
            check_visit_container(
                StorableGraph::edge_visit_container(&graph),
                ids,
                "edge_visit_container",
            );
            check_data_container(
                StorableGraph::edge_data_container(&graph, DEFAULT_DATA),
                ids,
                "edge_data_container",
            );
        });
    }
}
