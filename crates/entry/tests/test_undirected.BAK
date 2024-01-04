use petgraph_core::{edge::Direction, visit::NodeCount};
use petgraph_graphmap::UnGraphMap;

#[test]
fn self_loop() {
    let mut gr = UnGraphMap::default();
    gr.add_node("A");
    gr.add_edge("A", "A", 0);

    assert_eq!(gr.node_count(), 1);
    assert_eq!(gr.edge_count(), 1);
    assert_eq!(gr.edge_weight("A", "A"), Some(&0));
}

#[test]
fn add_edge_previous_weight() {
    let mut gr = UnGraphMap::default();
    gr.add_edge("A", "B", 0);
    assert_eq!(gr.add_edge("A", "B", 1), Some(0));
}

#[test]
fn add_edge_previous_weight_does_not_exist() {
    let mut graph = UnGraphMap::default();
    graph.add_node("A");
    graph.add_node("B");

    assert_eq!(graph.add_edge("A", "B", 0), None);
}

#[test]
fn edge_weight() {
    let mut graph = UnGraphMap::default();
    graph.add_edge("A", "B", 0);

    assert_eq!(graph.edge_weight("A", "B"), Some(&0));
}

#[test]
fn edge_weight_does_not_exist() {
    let mut graph = UnGraphMap::<_, ()>::default();
    graph.add_node("A");
    graph.add_node("B");

    assert_eq!(graph.edge_weight("A", "B"), None);
}

#[test]
fn edge_weight_mut() {
    let mut graph = UnGraphMap::default();
    graph.add_edge("A", "B", 0);

    assert_eq!(graph.edge_weight_mut("A", "B"), Some(&mut 0));

    *graph.edge_weight_mut("A", "B").unwrap() = 1;

    assert_eq!(graph.edge_weight("A", "B"), Some(&1));
}

#[test]
fn edge_weight_mut_does_not_exist() {
    let mut graph = UnGraphMap::<_, ()>::default();
    graph.add_node("A");
    graph.add_node("B");

    assert_eq!(graph.edge_weight_mut("A", "B"), None);
}

#[test]
fn remove_edge() {
    let mut graph = UnGraphMap::default();
    graph.add_edge("A", "B", 0);

    assert_eq!(graph.remove_edge("A", "B"), Some(0));
    assert_eq!(graph.edge_count(), 0);
}

#[test]
fn remove_edge_does_not_exist() {
    let mut graph = UnGraphMap::<_, ()>::default();
    graph.add_node("A");
    graph.add_node("B");

    assert_eq!(graph.remove_edge("A", "B"), None);
}

#[test]
fn remove_node() {
    let mut graph = UnGraphMap::<_, ()>::default();
    graph.add_node("A");

    assert_eq!(graph.node_count(), 1);
    assert!(graph.remove_node("A"));
    assert_eq!(graph.node_count(), 0);
    assert!(!graph.remove_node("A"));
}

#[test]
fn remove_node_remove_edges() {
    let mut graph = UnGraphMap::default();
    graph.add_edge("A", "B", 0);
    graph.add_edge("B", "C", 1);

    assert_eq!(graph.edge_count(), 2);
    assert!(graph.remove_node("B"));
    assert_eq!(graph.edge_count(), 0);
}

#[test]
fn remove_node_does_not_exist() {
    let mut graph = UnGraphMap::<_, ()>::default();
    graph.add_node("A");

    assert!(!graph.remove_node("B"));
}

#[test]
fn neighbors() {
    let mut graph = UnGraphMap::default();
    graph.add_edge("A", "B", 0);
    graph.add_edge("A", "C", 1);

    assert_eq!(graph.neighbors("A").collect::<Vec<_>>(), vec!["B", "C"]);
}

#[test]
fn neighbors_does_not_exist() {
    let mut graph = UnGraphMap::<_, ()>::default();
    graph.add_node("A");

    assert_eq!(graph.neighbors("B").count(), 0);
}

#[test]
fn neighbors_directed() {
    let mut graph = UnGraphMap::default();
    graph.add_edge("A", "B", 0);
    graph.add_edge("A", "C", 1);

    assert_eq!(
        graph
            .neighbors_directed("A", Direction::Outgoing)
            .collect::<Vec<_>>(),
        vec!["B", "C"]
    );
    assert_eq!(
        graph
            .neighbors_directed("A", Direction::Incoming)
            .collect::<Vec<_>>(),
        vec!["B", "C"]
    );
}

#[test]
fn neighbors_directed_does_not_exist() {
    let mut graph = UnGraphMap::<_, ()>::default();
    graph.add_node("A");

    assert_eq!(
        graph.neighbors_directed("B", Direction::Outgoing).count(),
        0
    );
    assert_eq!(
        graph.neighbors_directed("B", Direction::Incoming).count(),
        0
    );
}

#[test]
fn edges() {
    let mut graph = UnGraphMap::default();
    graph.add_edge("A", "B", 0);
    graph.add_edge("A", "C", 1);

    assert_eq!(graph.edges("A").collect::<Vec<_>>(), vec![
        ("A", "B", &0),
        ("A", "C", &1)
    ]);
}

#[test]
fn edges_does_not_exist() {
    let mut graph = UnGraphMap::<_, ()>::default();
    graph.add_node("A");

    assert_eq!(graph.edges("B").count(), 0);
}

#[test]
fn node_count() {
    let mut graph = UnGraphMap::<_, ()>::default();

    assert_eq!(graph.node_count(), 0);

    graph.add_node("A");

    assert_eq!(graph.node_count(), 1);

    graph.add_node("B");

    assert_eq!(graph.node_count(), 2);
}

#[test]
fn node_count_after_remove() {
    let mut graph = UnGraphMap::<_, ()>::default();
    graph.add_node("A");
    graph.add_node("B");

    assert_eq!(graph.node_count(), 2);

    graph.remove_node("A");

    assert_eq!(graph.node_count(), 1);
}

#[test]
fn node_count_after_clear() {
    let mut graph = UnGraphMap::<_, ()>::default();
    graph.add_node("A");
    graph.add_node("B");

    assert_eq!(graph.node_count(), 2);

    graph.clear();

    assert_eq!(graph.node_count(), 0);
}

#[test]
fn node_count_duplicate() {
    let mut graph = UnGraphMap::<_, ()>::default();
    graph.add_node("A");
    graph.add_node("A");

    assert_eq!(graph.node_count(), 1);
}

#[test]
fn edge_count() {
    let mut graph = UnGraphMap::<_, ()>::default();

    assert_eq!(graph.edge_count(), 0);

    graph.add_edge("A", "B", ());

    assert_eq!(graph.edge_count(), 1);

    graph.add_edge("B", "C", ());

    assert_eq!(graph.edge_count(), 2);
}

#[test]
fn edge_count_after_remove() {
    let mut graph = UnGraphMap::<_, ()>::default();
    graph.add_edge("A", "B", ());
    graph.add_edge("B", "C", ());

    assert_eq!(graph.edge_count(), 2);

    graph.remove_edge("A", "B");

    assert_eq!(graph.edge_count(), 1);
}

#[test]
fn edge_count_after_clear() {
    let mut graph = UnGraphMap::<_, ()>::default();
    graph.add_edge("A", "B", ());
    graph.add_edge("B", "C", ());

    assert_eq!(graph.edge_count(), 2);

    graph.clear();

    assert_eq!(graph.edge_count(), 0);
}

#[test]
fn edge_iter() {
    let mut graph = UnGraphMap::default();
    graph.add_edge("A", "B", 0);
    graph.add_edge("A", "C", 1);

    assert_eq!(graph.edge_count(), 2);

    assert_eq!(graph.all_edges().collect::<Vec<_>>(), vec![
        ("A", "B", &0),
        ("A", "C", &1)
    ]);
}

#[test]
fn from_edges() {
    let edges = vec![
        ("A", "B", 0), //
        ("A", "C", 1),
        ("B", "C", 2),
        ("C", "D", 3),
    ];

    let graph = UnGraphMap::from_edges(edges.clone());

    assert_eq!(graph.node_count(), 4);
    assert_eq!(graph.edge_count(), 4);

    assert_eq!(
        graph
            .all_edges()
            .map(|(source, target, &weight)| (source, target, weight))
            .collect::<Vec<_>>(),
        edges
    );

    assert_eq!(graph.neighbors("A").collect::<Vec<_>>(), vec!["B", "C"]);
    assert_eq!(graph.neighbors("B").collect::<Vec<_>>(), vec!["A", "C"]);
    assert_eq!(graph.neighbors("C").collect::<Vec<_>>(), vec![
        "A", "B", "D"
    ]);
    assert_eq!(graph.neighbors("D").collect::<Vec<_>>(), vec!["C"]);
}

#[test]
fn all_edges() {
    let mut graph = UnGraphMap::default();
    graph.add_edge("A", "B", 0);
    graph.add_edge("A", "C", 1);

    assert_eq!(graph.all_edges().count(), 2);
    assert_eq!(graph.all_edges().collect::<Vec<_>>(), vec![
        ("A", "B", &0),
        ("A", "C", &1)
    ]);
}

#[test]
fn all_edges_mut() {
    let mut graph = UnGraphMap::default();
    graph.add_edge("A", "B", 0);
    graph.add_edge("A", "C", 1);

    for (_, _, weight) in graph.all_edges_mut() {
        *weight += 1;
    }

    assert_eq!(graph.all_edges().collect::<Vec<_>>(), vec![
        ("A", "B", &1),
        ("A", "C", &2)
    ]);
}

#[test]
fn neighbours_includes_self_loop() {
    let mut graph = UnGraphMap::default();
    graph.add_edge("A", "A", 0);

    assert_eq!(graph.neighbors("A").collect::<Vec<_>>(), vec!["A"]);
}

#[test]
fn remove_self_loop() {
    let mut graph = UnGraphMap::default();
    graph.add_edge("A", "A", 0);

    assert_eq!(graph.edge_count(), 1);

    graph.remove_edge("A", "A");

    assert_eq!(graph.edge_count(), 0);
    assert_eq!(graph.neighbors("A").count(), 0);
    assert_eq!(graph.node_count(), 1);
}
