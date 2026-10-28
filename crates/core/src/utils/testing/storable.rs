use crate::graph::EditableGraph;

pub const STORABLE_TEST_GRAPH_NODE_COUNT: usize = 5;
pub const STORABLE_TEST_GRAPH_EDGE_COUNT: usize = 4;

/// A helper function to create a simple graph for testing purposes.
///
/// The graph looks as follows:
/// ```text
/// 0 --> 1
/// |      |
/// v      v
/// 2 <----3     4
/// ```
/// For undirected graphs, the arrows only denote the order of the nodes when adding the edges to
/// the graph.
///
/// The function returns a tuple containing the constructed graph,
/// a slice of the ids of added nodes, and a slice of the ids of added edges.
///
/// For ordering of added nodes and edges, see the implementation of the function.
fn create_storable_test_graph<G: EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>>(
    graph_constructor: impl Fn() -> G,
) -> (
    G,
    [G::NodeId; STORABLE_TEST_GRAPH_NODE_COUNT],
    [G::EdgeId; STORABLE_TEST_GRAPH_EDGE_COUNT],
) {
    let mut graph = graph_constructor();

    let node_zero = graph.add_node(()).unwrap();
    let node_one = graph.add_node(()).unwrap();
    let node_two = graph.add_node(()).unwrap();
    let node_three = graph.add_node(()).unwrap();
    let node_four = graph.add_node(()).unwrap();

    let nodes = [node_zero, node_one, node_two, node_three, node_four];

    let edge_zero = graph.add_edge(nodes[0], nodes[1], ()).unwrap();
    let edge_one = graph.add_edge(nodes[0], nodes[2], ()).unwrap();
    let edge_two = graph.add_edge(nodes[1], nodes[3], ()).unwrap();
    let edge_three = graph.add_edge(nodes[3], nodes[2], ()).unwrap();

    let edges = [edge_zero, edge_one, edge_two, edge_three];

    (graph, nodes, edges)
}

/// Generates a suite of tests for [`StorableGraph`][crate::graph::storable::StorableGraph]
/// implementations.
///
/// One test is generated for each method in the
/// [`StorableGraph`][crate::graph::storable::StorableGraph] trait, which checks every method of the
/// returned [`VisitContainer`][crate::graph::storable::VisitContainer] or
/// [`DataContainer`][crate::graph::storable::DataContainer]. An additional test checks that
/// containers created after nodes and edges were removed and added still work for all nodes and
/// edges in the graph. The following invariants are expected from the graph implementation:
/// - If the most recently added node or edge is removed, its ID will no longer be valid.
/// - Removing the most recently added node or edge does not change the IDs of any other nodes or
///   edges.
/// - Writing to the reference returned by
///   [`DataContainer::get_mut`][crate::graph::storable::DataContainer::get_mut] stores the value
///   for that ID only, even if no data was associated with the ID before.
///
/// The graph type has to implement [`EditableGraph`] to construct the test graph.
///
/// The arguments to this macro are as follows (`G` is used to denote the graph type being tested):
/// - `$graph_constructor`: `impl Fn() -> G`. An expression that constructs a new instance of the
///   graph type to be tested. The generated graph must be empty (e.g. `G::new()`).
///
/// For a reference usage, see the tests in [`crate::utils::test_graphs::directed`].
#[macro_export]
macro_rules! test_storable_graph {
    ($graph_constructor:expr,) => {
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
    };
}

// WARNING: This module contains only internal testing helper functions and should not be considered
// part of the public API. It is subject to change at any time.
#[doc(hidden)]
pub mod __private_testing {
    use crate::{
        graph::{
            EditableGraph,
            storable::{DataContainer, StorableGraph, VisitContainer},
        },
        id::Id,
        utils::testing::storable::create_storable_test_graph,
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

    /// Checks that two [`DataContainer`]s created from the same graph do not share state. The
    /// first container has to be created with [`DEFAULT_DATA`] and the second one with
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
        let (graph, nodes, _) = create_storable_test_graph(graph_constructor);

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
        let (graph, nodes, _) = create_storable_test_graph(graph_constructor);

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
        let (graph, _, edges) = create_storable_test_graph(graph_constructor);

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
        let (graph, _, edges) = create_storable_test_graph(graph_constructor);

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
        let (mut graph, nodes, edges) = create_storable_test_graph(graph_constructor);

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
}
