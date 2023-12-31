use petgraph_core::{
    deprecated::visit::{
        EdgeFiltered, EdgeRef, IntoEdges, IntoEdgesDirected, IntoNodeIdentifiers, NodeFiltered,
        Reversed, VisitMap, Visitable,
    },
    edge::Direction,
};
use petgraph_dino::{DiDinoGraph, UnDinoGraph};

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
    let mut graph = DiDinoGraph::new();

    let a = graph.insert_node("A").id();
    let b = graph.insert_node("B").id();
    let c = graph.insert_node("C").id();
    let d = graph.insert_node("D").id();

    let ab = graph.insert_edge("A → B", a, b).id();
    let ac = graph.insert_edge("A → C", a, c).id();
    let ad = graph.insert_edge("A → D", a, d).id();

    let filtered = EdgeFiltered::from_fn(&graph, |edge| edge.id() != ab);

    let received = filtered
        .edges_directed(a, Direction::Outgoing)
        .map(|edge| edge.id())
        .collect::<Vec<_>>();
    let expected = vec![ac, ad];

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
    let mut graph = DiDinoGraph::new();

    let a = graph.insert_node("A").id();
    let b = graph.insert_node("B").id();

    let ab = graph.insert_edge("A → B", a, b).id();

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
    let mut graph = UnDinoGraph::new();

    let a = graph.insert_node("A").id();
    let b = graph.insert_node("B").id();
    let c = graph.insert_node("C").id();

    for (source, target, weight) in [(a, b, 0), (a, c, 1), (b, c, -1)] {
        graph.insert_edge(weight, source, target);
    }

    let filtered = EdgeFiltered::from_fn(&graph, |edge| *edge.weight() >= 0);

    assert_eq!(
        filtered
            .edges(a)
            .map(|edge| *edge.weight())
            .collect::<Vec<_>>(),
        [0, 1]
    );
}

/// Same graph as `edge_filtered_edges_directed`, but we filter out the node `B` instead of the edge
/// `A → B`.
#[test]
fn node_filtered_edges_directed() {
    let mut graph = DiDinoGraph::new();

    let a = graph.insert_node("A").id();
    let b = graph.insert_node("B").id();
    let c = graph.insert_node("C").id();
    let d = graph.insert_node("D").id();

    let ab = graph.insert_edge("A → B", a, b).id();
    let ac = graph.insert_edge("A → C", a, c).id();
    let ad = graph.insert_edge("A → D", a, d).id();

    let filtered = NodeFiltered::from_fn(&graph, |node| node != b);

    let received = filtered
        .edges_directed(a, Direction::Outgoing)
        .map(|edge| edge.id())
        .collect::<Vec<_>>();
    let expected = vec![ac, ad];

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
    let mut graph = DiDinoGraph::<_, ()>::new();

    let a = graph.insert_node("A").id();
    let b = graph.insert_node("B").id();
    let c = graph.insert_node("C").id();
    let d = graph.insert_node("D").id();

    let filtered = NodeFiltered::from_fn(&graph, |node| node != b);

    let received = filtered.node_identifiers().collect::<Vec<_>>();
    let expected = vec![a, c, d];

    assert_eq!(received, expected);
}

#[test]
fn node_filtered_by_fixed_bit_set() {
    let mut graph = DiDinoGraph::<_, ()>::new();

    let a = graph.insert_node("A").id();
    let b = graph.insert_node("B").id();
    let c = graph.insert_node("C").id();

    let mut map = (&graph).visit_map();
    map.visit(a);
    map.visit(c);

    let filtered = NodeFiltered(&graph, map);

    assert_eq!(filtered.node_identifiers().collect::<Vec<_>>(), vec![a, c]);
}
