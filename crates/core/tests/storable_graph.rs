#![cfg(feature = "test-utils")]

use petgraph_core::{
    graph::{
        EditableGraph,
        storable::{DataContainer, StorableGraph, VisitContainer},
    },
    utils::test_graphs::{directed::DirectedTestGraph, undirected::UndirectedTestGraph},
};

fn check_data_containers<Id: Copy, C: DataContainer<Id, Vec<u8>>>(
    mut data: C,
    independent: &C,
    [first, second]: [Id; 2],
) {
    assert_eq!(data.get(first), &[1]);
    assert_eq!(data.get(second), &[1]);

    data.get_mut(first).push(2);
    assert_eq!(data.get(first), &[1, 2]);
    assert_eq!(data.get(second), &[1]);
    assert_eq!(independent.get(first), &[1]);

    data.insert(first, vec![3]);
    data.insert(second, vec![4]);
    assert_eq!(data.get(first), &[3]);
    assert_eq!(data.get(second), &[4]);

    data.remove(first);
    data.remove(first);
    assert_eq!(data.get(first), &[1]);
    assert_eq!(data.get(second), &[4]);
    data.get_mut(first).push(5);
    assert_eq!(data.get(first), &[1, 5]);

    data.clear();
    assert_eq!(data.get(first), &[1]);
    assert_eq!(data.get(second), &[1]);
    data.insert(second, vec![6]);
    assert_eq!(data.get(second), &[6]);
    assert_eq!(data.get(first), &[1]);
}

fn check_visit_containers<Id: Copy, C: VisitContainer<Id>>(
    mut visited: C,
    independent: &C,
    [first, second]: [Id; 2],
) {
    assert!(!visited.is_visited(first));
    assert!(!visited.is_visited(second));

    visited.mark_visited(first);
    visited.mark_visited(first);
    assert!(visited.is_visited(first));
    assert!(!visited.is_visited(second));
    assert!(!independent.is_visited(first));

    visited.mark_visited(second);
    visited.mark_unvisited(first);
    visited.mark_unvisited(first);
    assert!(!visited.is_visited(first));
    assert!(visited.is_visited(second));

    visited.mark_visited(first);
    visited.clear();
    assert!(!visited.is_visited(first));
    assert!(!visited.is_visited(second));
    visited.mark_visited(second);
    assert!(visited.is_visited(second));
    assert!(!visited.is_visited(first));
}

fn check_storage<G>(mut graph: G)
where
    G: StorableGraph + EditableGraph<OwnedNodeData = (), OwnedEdgeData = ()>,
{
    let first = graph.add_node(()).unwrap();
    let second = graph.add_node(()).unwrap();
    let third = graph.add_node(()).unwrap();
    let first_edge = graph.add_edge(first, second, ()).unwrap();
    let second_edge = graph.add_edge(second, third, ()).unwrap();
    let nodes = [first, third];
    let edges = [first_edge, second_edge];
    check_data_containers(
        graph.node_data_container(vec![1]),
        &graph.node_data_container(vec![1]),
        nodes,
    );
    check_data_containers(
        graph.edge_data_container(vec![1]),
        &graph.edge_data_container(vec![1]),
        edges,
    );
    check_visit_containers(
        graph.node_visit_container(),
        &graph.node_visit_container(),
        nodes,
    );
    check_visit_containers(
        graph.edge_visit_container(),
        &graph.edge_visit_container(),
        edges,
    );
}

#[test]
fn directed_graph_storage() {
    check_storage(DirectedTestGraph::<(), ()>::new());
}

#[test]
fn undirected_graph_storage() {
    check_storage(UndirectedTestGraph::<(), ()>::new());
}
