/// Generates a suite of tests for [`StorableGraph`][crate::graph::storable::StorableGraph]
/// implementations, which are also [`DirectedGraph`][crate::graph::DirectedGraph]s.
///
/// The ids of the nodes and edges remaining after removing a node are obtained using
/// [`DirectedGraph::nodes`][crate::graph::DirectedGraph::nodes] and
/// [`DirectedGraph::edges`][crate::graph::DirectedGraph::edges].
///
/// For the expected invariants and the arguments of this macro, see
/// [`__test_storable_graph_inner`][crate::__test_storable_graph_inner].
///
/// For a reference usage, see the tests in [`crate::utils::test_graphs::directed`].
#[macro_export]
macro_rules! test_directed_storable_graph {
    ($graph_constructor:expr,) => {
        $crate::__test_storable_graph_inner!(
            $graph_constructor,
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
/// implementations, which are also [`UndirectedGraph`][crate::graph::UndirectedGraph]s.
///
/// The ids of the nodes and edges remaining after removing a node are obtained using
/// [`UndirectedGraph::nodes`][crate::graph::UndirectedGraph::nodes] and
/// [`UndirectedGraph::edges`][crate::graph::UndirectedGraph::edges].
///
/// For the expected invariants and the arguments of this macro, see
/// [`__test_storable_graph_inner`][crate::__test_storable_graph_inner].
///
/// For a reference usage, see the tests in [`crate::utils::test_graphs::undirected`].
#[macro_export]
macro_rules! test_undirected_storable_graph {
    ($graph_constructor:expr,) => {
        $crate::__test_storable_graph_inner!(
            $graph_constructor,
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
/// implementations.
///
/// One test is generated for each method in the
/// [`StorableGraph`][crate::graph::storable::StorableGraph] trait, which checks every method of the
/// returned [`VisitContainer`][crate::graph::storable::VisitContainer] or
/// [`DataContainer`][crate::graph::storable::DataContainer]. An additional test checks that
/// containers created after nodes and edges were removed and added still work for all nodes and
/// edges in the graph. Another test checks that the containers have sufficient capacity for all
/// nodes and edges in the graph, even if the largest ids are not smaller than the number of nodes
/// and edges, as is the case for graphs with stable ids after removing the oldest node. The
/// following invariants are expected from the graph implementation:
/// - If the most recently added node or edge is removed, its ID will no longer be valid.
/// - Removing the most recently added node or edge does not change the IDs of any other nodes or
///   edges.
///
/// The graph type has to implement [`EditableGraph`] to construct the test graph.
///
/// The arguments to this macro are as follows (`G` is used to denote the graph type being tested):
/// - `$graph_constructor`: `impl Fn() -> G`. An expression that constructs a new instance of the
///   graph type to be tested. The generated graph must be empty (e.g. `G::new()`).
/// - `$node_ids`: `fn(&G) -> Vec<G::NodeId>`. Returns the ids of all nodes in the graph.
/// - `$edge_ids`: `fn(&G) -> Vec<G::EdgeId>`. Returns the ids of all edges in the graph.
///
/// Prefer using [`test_directed_storable_graph`][crate::test_directed_storable_graph] or
/// [`test_undirected_storable_graph`][crate::test_undirected_storable_graph] instead.
#[doc(hidden)]
#[macro_export]
macro_rules! __test_storable_graph_inner {
    ($graph_constructor:expr, $node_ids:expr, $edge_ids:expr,) => {
        mod storable_graph_tests {
            #[allow(unused_imports)]
            use super::*;

            #[test]
            fn test_node_visit_container() {
                $crate::utils::testing::storable::__private_testing::test_node_visit_container_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_node_data_container() {
                $crate::utils::testing::storable::__private_testing::test_node_data_container_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_edge_visit_container() {
                $crate::utils::testing::storable::__private_testing::test_edge_visit_container_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_edge_data_container() {
                $crate::utils::testing::storable::__private_testing::test_edge_data_container_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_containers_after_graph_edits() {
                $crate::utils::testing::storable::__private_testing::test_containers_after_graph_edits_inner(
                    $graph_constructor,
                );
            }

            #[test]
            fn test_container_capacity() {
                $crate::utils::testing::storable::__private_testing::test_container_capacity_inner(
                    $graph_constructor,
                    $node_ids,
                    $edge_ids,
                );
            }
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
        utils::testing::{TEST_GRAPH_EDGE_COUNT, TEST_GRAPH_NODE_COUNT, create_test_graph},
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

    /// Checks all methods of a [`VisitContainer`] for the given ids, which have to contain at
    /// least three ids.
    fn check_visit_container<I: Id, C: VisitContainer<I>>(
        mut container: C,
        ids: &[I],
        method_name: &'static str,
    ) {
        assert!(ids.len() >= 3);
        let last = ids.len() - 1;

        assert_visited(
            &container,
            ids,
            |_| false,
            method_name,
            "for a new container",
        );

        container.mark_visited(ids[0]);
        container.mark_visited(ids[2]);
        assert_visited(
            &container,
            ids,
            |index| index == 0 || index == 2,
            method_name,
            "after marking ids 0 and 2 as visited",
        );

        container.mark_visited(ids[0]);
        assert_visited(
            &container,
            ids,
            |index| index == 0 || index == 2,
            method_name,
            "after marking id 0 as visited twice",
        );

        container.mark_unvisited(ids[0]);
        assert_visited(
            &container,
            ids,
            |index| index == 2,
            method_name,
            "after marking id 0 as unvisited",
        );

        container.mark_unvisited(ids[1]);
        assert_visited(
            &container,
            ids,
            |index| index == 2,
            method_name,
            "after marking the never visited id 1 as unvisited",
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

    /// Checks all methods of a [`DataContainer`] for the given ids, which have to contain at
    /// least four ids. The container has to be created with [`DEFAULT_DATA`] as default value.
    fn check_data_container<I: Id, C: DataContainer<I, usize>>(
        mut container: C,
        ids: &[I],
        method_name: &'static str,
    ) {
        assert!(ids.len() >= 4);

        assert_data(
            &container,
            ids,
            |_| DEFAULT_DATA,
            method_name,
            "for a new container",
        );

        // Writing through `get_mut` must not change the default value of the other ids.
        *container.get_mut(ids[1]) = 7;
        assert_data(
            &container,
            ids,
            |index| if index == 1 { 7 } else { DEFAULT_DATA },
            method_name,
            "after writing to id 1 via DataContainer::get_mut()",
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

        *container.get_mut(ids[0]) += 1;
        assert_data(
            &container,
            ids,
            |index| if index == 0 { 1 } else { index * 10 },
            method_name,
            "after modifying id 0 via DataContainer::get_mut()",
        );

        container.insert(ids[2], 1000);
        assert_data(
            &container,
            ids,
            |index| match index {
                0 => 1,
                2 => 1000,
                _ => index * 10,
            },
            method_name,
            "after replacing the data of id 2",
        );

        container.remove(ids[2]);
        assert_data(
            &container,
            ids,
            |index| match index {
                0 => 1,
                2 => DEFAULT_DATA,
                _ => index * 10,
            },
            method_name,
            "after removing the data of id 2",
        );

        container.remove(ids[2]);
        assert_data(
            &container,
            ids,
            |index| match index {
                0 => 1,
                2 => DEFAULT_DATA,
                _ => index * 10,
            },
            method_name,
            "after removing the data of id 2 twice",
        );

        container.clear();
        assert_data(
            &container,
            ids,
            |_| DEFAULT_DATA,
            method_name,
            "after clearing the container",
        );

        container.insert(ids[3], 3);
        assert_data(
            &container,
            ids,
            |index| if index == 3 { 3 } else { DEFAULT_DATA },
            method_name,
            "after inserting data for id 3 after clearing the container",
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
    ) {
        let (graph, nodes, _) = create_test_graph(graph_constructor);

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
    ) {
        let (graph, nodes, _) = create_test_graph(graph_constructor);

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
    ) {
        let (graph, _, edges) = create_test_graph(graph_constructor);

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
    ) {
        let (graph, _, edges) = create_test_graph(graph_constructor);

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
    ) {
        let (mut graph, nodes, edges) = create_test_graph(graph_constructor);

        // The newest edge and node are removed, as some graph implementations might not have
        // stable ids, but the newest ones added are likely to be removable without another edge or
        // node taking their ids.
        graph.remove_edge(edges[3]).unwrap();
        graph.remove_node(nodes[4]).unwrap();

        // Depending on the implementation, the new node might reuse the id of the removed node or
        // get an id beyond the ids of the original nodes. The containers have to support both.
        let new_node = graph.add_node(()).unwrap();
        let new_incoming_edge = graph.add_edge(nodes[3], new_node, ()).unwrap();
        let new_outgoing_edge = graph.add_edge(new_node, nodes[0], ()).unwrap();

        let current_nodes = [nodes[0], nodes[1], nodes[2], nodes[3], new_node];
        let current_edges = [
            edges[0],
            edges[1],
            edges[2],
            new_incoming_edge,
            new_outgoing_edge,
        ];

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
        node_ids: fn(&G) -> Vec<G::NodeId>,
        edge_ids: fn(&G) -> Vec<G::EdgeId>,
    ) {
        let (mut graph, nodes, _) = create_test_graph(graph_constructor);

        // These edges are not incident to node 0, so that enough edges remain after removing it.
        graph.add_edge(nodes[2], nodes[4], ()).unwrap();
        graph.add_edge(nodes[4], nodes[1], ()).unwrap();

        // Node 0 is the oldest node and incident to the two oldest edges. For graph
        // implementations with stable ids, removing it leaves gaps in the ids, so that the largest
        // remaining ids are larger than the number of nodes and edges.
        graph.remove_node(nodes[0]).unwrap();

        // Other graph implementations might change the ids of the remaining nodes and edges, so
        // the current ids are obtained from the graph.
        let current_nodes = node_ids(&graph);
        let current_edges = edge_ids(&graph);
        assert_eq!(
            current_nodes.len(),
            TEST_GRAPH_NODE_COUNT - 1,
            "unexpected number of node ids returned by the graph after removing node 0"
        );
        assert_eq!(
            current_edges.len(),
            TEST_GRAPH_EDGE_COUNT,
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
