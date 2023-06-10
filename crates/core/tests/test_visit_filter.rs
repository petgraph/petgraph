use petgraph::{graph::UnGraph, Graph};
use petgraph_core::{
    edge::Direction,
    visit::{
        EdgeFiltered, EdgeRef, IntoEdges, IntoEdgesDirected, IntoNodeIdentifiers, NodeFiltered,
        Reversed, VisitMap, Visitable,
    },
};

/// Given a graph with 3 nodes and 3 edges, test that the filtered edges iterator returns the
/// correct edges.
///
/// ```text
/// A → B
/// ↓ ↘
/// D   C
/// ```
///
/// We remove the edge `A → B` from the graph and verify that the filtered edges iterator only
/// returns the edges `B → C` and `B → D`.
#[test]
fn edge_filtered_edges_directed() {
    let mut graph = Graph::new();

    let a = graph.add_node("A");
    let b = graph.add_node("B");
    let c = graph.add_node("C");
    let d = graph.add_node("D");

    let ab = graph.add_edge(a, b, "A → B");
    let ac = graph.add_edge(a, c, "A → C");
    let ad = graph.add_edge(a, d, "A → D");

    let filtered = EdgeFiltered::from_fn(&graph, |edge| edge.id() != ab);

    let received = filtered
        .edges_directed(a, Direction::Outgoing)
        .map(|edge| edge.id())
        .collect::<Vec<_>>();
    let expected = vec![ad, ac];

    assert_eq!(received, expected);

    let received = filtered
        .edges_directed(b, Direction::Incoming)
        .map(|edge| edge.id())
        .collect::<Vec<_>>();
    let expected = vec![];

    assert_eq!(received, expected);
}

#[test]
fn edge_filtered_edges_directed_reverse() {
    let mut graph = Graph::new();

    let a = graph.add_node("A");
    let b = graph.add_node("B");

    let ab = graph.add_edge(a, b, "A → B");

    // do not filter anything, we just want to test the reverse direction
    let filtered = EdgeFiltered::from_fn(&graph, |_| true);
    let reversed = Reversed(&filtered);

    assert_eq!(
        graph
            .edges_directed(a, Direction::Outgoing)
            .map(|edge| edge.id())
            .collect::<Vec<_>>(),
        [ab]
    );
    assert_eq!(
        reversed
            .edges_directed(a, Direction::Outgoing)
            .map(|edge| edge.id())
            .collect::<Vec<_>>(),
        []
    );

    assert_eq!(
        graph
            .edges_directed(a, Direction::Incoming)
            .map(|edge| edge.id())
            .collect::<Vec<_>>(),
        []
    );
    assert_eq!(
        reversed
            .edges_directed(a, Direction::Incoming)
            .map(|edge| edge.id())
            .collect::<Vec<_>>(),
        [ab]
    );
}

#[test]
fn edge_filtered_undirected_filter_by_weight() {
    let mut graph = UnGraph::new_undirected();

    let a = graph.add_node("A");
    let b = graph.add_node("B");
    let c = graph.add_node("C");

    graph.extend_with_edges([
        (a, b, 0), //
        (a, c, 1),
        (b, c, -1),
    ]);

    let filtered = EdgeFiltered::from_fn(&graph, |edge| *edge.weight() >= 0);

    assert_eq!(
        filtered
            .edges(a)
            .map(|edge| *edge.weight())
            .collect::<Vec<_>>(),
        [1, 0]
    );
}

/// Same graph as `edge_filtered_edges_directed`, but we filter out the node `B` instead of the edge
/// `A → B`.
#[test]
fn node_filtered_edges_directed() {
    let mut graph = Graph::new();

    let a = graph.add_node("A");
    let b = graph.add_node("B");
    let c = graph.add_node("C");
    let d = graph.add_node("D");

    let ab = graph.add_edge(a, b, "A → B");
    let ac = graph.add_edge(a, c, "A → C");
    let ad = graph.add_edge(a, d, "A → D");

    let filtered = NodeFiltered::from_fn(&graph, |node| node != b);

    let received = filtered
        .edges_directed(a, Direction::Outgoing)
        .map(|edge| edge.id())
        .collect::<Vec<_>>();
    let expected = vec![ad, ac];

    assert_eq!(received, expected);

    let received = filtered
        .edges_directed(b, Direction::Incoming)
        .map(|edge| edge.id())
        .collect::<Vec<_>>();
    let expected = vec![];

    assert_eq!(received, expected);
}

#[test]
fn node_filtered_node_identifiers() {
    let mut graph = Graph::<_, ()>::new();

    let a = graph.add_node("A");
    let b = graph.add_node("B");
    let c = graph.add_node("C");
    let d = graph.add_node("D");

    let filtered = NodeFiltered::from_fn(&graph, |node| node != b);

    let received = filtered.node_identifiers().collect::<Vec<_>>();
    let expected = vec![a, c, d];

    assert_eq!(received, expected);
}

#[test]
fn node_filtered_by_fixed_bit_set() {
    let mut graph = Graph::<_, ()>::new();

    let a = graph.add_node("A");
    let b = graph.add_node("B");
    let c = graph.add_node("C");

    let mut map = graph.visit_map();
    map.visit(a);
    map.visit(c);

    let filtered = NodeFiltered(&graph, map);

    assert_eq!(filtered.node_identifiers().collect::<Vec<_>>(), vec![a, c]);
}
