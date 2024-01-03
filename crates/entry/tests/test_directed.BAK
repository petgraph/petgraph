use petgraph_core::{
    edge::{Directed, Direction},
    id::DefaultIx,
    visit::{EdgeRef, IntoEdgeReferences, IntoNodeReferences},
};
#[cfg(feature = "convert")]
use petgraph_graph::{Graph, NodeIndex};
use petgraph_graphmap::{DiGraphMap, EntryStorage};

#[test]
fn self_loop() {
    let mut gr = DiGraphMap::new();
    gr.add_node("A");
    gr.add_edge("A", "A", 0);

    assert_eq!(gr.node_count(), 1);
    assert_eq!(gr.edge_count(), 1);
    assert_eq!(gr.edge_weight("A", "A"), Some(&0));
}

#[test]
fn add_reverse_edge() {
    let mut gr = DiGraphMap::new();
    gr.add_edge("A", "B", 0);
    gr.add_edge("B", "A", 1);

    assert_eq!(gr.node_count(), 2);
    assert_eq!(gr.edge_count(), 2);
    assert_eq!(gr.edge_weight("A", "B"), Some(&0));
    assert_eq!(gr.edge_weight("B", "A"), Some(&1));
}

#[test]
fn add_duplicate_edge() {
    let mut gr = DiGraphMap::new();

    assert_eq!(gr.add_edge("A", "B", 0), None);
    assert_eq!(gr.add_edge("A", "B", 1), Some(0));

    assert_eq!(gr.node_count(), 2);
    assert_eq!(gr.edge_count(), 1);
    assert_eq!(gr.edge_weight("A", "B"), Some(&1));
}

#[test]
fn add_duplicate_self_loop() {
    let mut gr = DiGraphMap::new();

    assert_eq!(gr.add_edge("A", "A", 0), None);
    assert_eq!(gr.add_edge("A", "A", 1), Some(0));

    assert_eq!(gr.node_count(), 1);
    assert_eq!(gr.edge_count(), 1);
    assert_eq!(gr.edge_weight("A", "A"), Some(&1));
}

// Test case for regression discovered in [Issue #431](https://github.com/petgraph/petgraph/issues/431).
#[test]
fn regression_remove_node_431() {
    let mut graph = DiGraphMap::<u32, ()>::new();
    graph.add_edge(1, 2, ());
    graph.remove_node(2);

    let neighbors: Vec<u32> = graph.neighbors(1).collect();
    assert_eq!(neighbors, []);

    let edges: Vec<(u32, u32, _)> = graph.all_edges().collect();
    assert_eq!(edges, []);
}

#[test]
#[cfg(feature = "convert")]
fn from_graph() {
    // Graph: a → b → c → a   d
    let mut graph = Graph::new();

    let a = graph.add_node(0);
    let b = graph.add_node(1);
    let c = graph.add_node(2);
    graph.add_node(3);

    graph.add_edge(a, b, -1);
    graph.add_edge(b, c, -2);
    graph.add_edge(c, a, -3);

    let graphmap = DiGraphMap::from_graph(graph.clone());

    assert_eq!(graphmap.node_count(), 4);
    assert_eq!(graphmap.edge_count(), 3);

    assert_eq!(graphmap.neighbors(0).collect::<Vec<_>>(), vec![1]);
    assert_eq!(graphmap.neighbors(1).collect::<Vec<_>>(), vec![2]);
    assert_eq!(graphmap.neighbors(2).collect::<Vec<_>>(), vec![0]);
    assert_eq!(graphmap.neighbors(3).collect::<Vec<_>>(), vec![]);

    assert_eq!(graphmap.edges(0).collect::<Vec<_>>(), vec![(0, 1, &-1)]);
    assert_eq!(graphmap.edges(1).collect::<Vec<_>>(), vec![(1, 2, &-2)]);
    assert_eq!(graphmap.edges(2).collect::<Vec<_>>(), vec![(2, 0, &-3)]);

    assert_eq!(
        graphmap.all_edges().collect::<Vec<_>>(),
        vec![(0, 1, &-1), (1, 2, &-2), (2, 0, &-3),]
    );
}

#[test]
#[cfg(feature = "convert")]
fn into_graph() {
    // Graph: a → b → c → a   d
    let mut graphmap = DiGraphMap::new();

    graphmap.add_edge(0, 1, -1);
    graphmap.add_edge(1, 2, -2);
    graphmap.add_edge(2, 0, -3);

    graphmap.add_node(3);

    let graph = graphmap.into_graph::<DefaultIx>();

    assert_eq!(graph.node_count(), 4);
    assert_eq!(graph.edge_count(), 3);

    assert_eq!(
        graph.node_references().collect::<Vec<_>>(),
        vec![
            (NodeIndex::new(0), &0),
            (NodeIndex::new(1), &1),
            (NodeIndex::new(2), &2),
            (NodeIndex::new(3), &3),
        ]
    );

    assert_eq!(
        graph
            .edge_references()
            .map(|edge| (edge.source(), edge.target(), edge.weight()))
            .collect::<Vec<_>>(),
        vec![
            (NodeIndex::new(0), NodeIndex::new(1), &-1),
            (NodeIndex::new(1), NodeIndex::new(2), &-2),
            (NodeIndex::new(2), NodeIndex::new(0), &-3),
        ]
    );
}

#[test]
fn neighbours_includes_self_loop() {
    let mut graph = DiGraphMap::new();
    graph.add_edge("A", "A", ());

    assert_eq!(
        graph
            .neighbors_directed("A", Direction::Incoming)
            .collect::<Vec<_>>(),
        vec!["A"]
    );
    assert_eq!(
        graph
            .neighbors_directed("A", Direction::Outgoing)
            .collect::<Vec<_>>(),
        vec!["A"]
    );
}

#[test]
fn remove_edge() {
    let mut graph = DiGraphMap::new();

    graph.add_edge("A", "B", 0);
    graph.add_edge("B", "C", 1);

    assert_eq!(graph.edge_count(), 2);

    assert_eq!(graph.remove_edge("A", "B"), Some(0));

    assert_eq!(graph.edge_count(), 1);

    assert_eq!(graph.remove_edge("A", "B"), None);

    assert_eq!(graph.edge_count(), 1);

    assert_eq!(graph.edge_weight("A", "B"), None);
}

#[test]
fn edges_directed() {
    let mut graph = DiGraphMap::new();

    graph.add_edge("A", "B", 0);
    graph.add_edge("B", "C", 1);
    graph.add_edge("C", "A", 2);

    assert_eq!(
        graph
            .edges_directed("A", Direction::Incoming)
            .collect::<Vec<_>>(),
        vec![("C", "A", &2)]
    );
    assert_eq!(
        graph
            .edges_directed("A", Direction::Outgoing)
            .collect::<Vec<_>>(),
        vec![("A", "B", &0)]
    );

    assert_eq!(
        graph
            .edges_directed("B", Direction::Incoming)
            .collect::<Vec<_>>(),
        vec![("A", "B", &0)]
    );
    assert_eq!(
        graph
            .edges_directed("B", Direction::Outgoing)
            .collect::<Vec<_>>(),
        vec![("B", "C", &1)]
    );

    assert_eq!(
        graph
            .edges_directed("C", Direction::Incoming)
            .collect::<Vec<_>>(),
        vec![("B", "C", &1)]
    );
    assert_eq!(
        graph
            .edges_directed("C", Direction::Outgoing)
            .collect::<Vec<_>>(),
        vec![("C", "A", &2)]
    );
}
