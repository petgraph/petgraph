use petgraph::Graph;
use petgraph_core::{
    edge::Direction,
    visit::{EdgeFiltered, EdgeRef, IntoEdgesDirected, NodeFiltered},
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
