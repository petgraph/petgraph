use hashbrown::HashMap;
use petgraph::algo::spfa;
use petgraph::visit::NodeIndexable;
use petgraph::{prelude::*, Directed, Graph, Undirected};

#[test]
fn spfa_uniform_weight() {
    let mut graph: Graph<(), (), Directed> = Graph::new();
    let a = graph.add_node(());
    let b = graph.add_node(());
    let c = graph.add_node(());
    let d = graph.add_node(());
    let e = graph.add_node(());
    let f = graph.add_node(());
    let g = graph.add_node(());
    let h = graph.add_node(());

    graph.extend_with_edges([
        (a, b),
        (b, c),
        (c, d),
        (d, a),
        (e, f),
        (b, e),
        (f, g),
        (g, h),
        (h, e),
    ]);
    // a ----> b ----> e ----> f
    // ^       |       ^       |
    // |       v       |       v
    // d <---- c       h <---- g

    let inf = i32::MAX;
    let expected_res: HashMap<(NodeIndex, NodeIndex), i32> = [
        ((a, a), 0),
        ((a, b), 1),
        ((a, c), 2),
        ((a, d), 3),
        ((a, e), 2),
        ((a, f), 3),
        ((a, g), 4),
        ((a, h), 5),
        ((b, a), 3),
        ((b, b), 0),
        ((b, c), 1),
        ((b, d), 2),
        ((b, e), 1),
        ((b, f), 2),
        ((b, g), 3),
        ((b, h), 4),
        ((c, a), 2),
        ((c, b), 3),
        ((c, c), 0),
        ((c, d), 1),
        ((c, e), 4),
        ((c, f), 5),
        ((c, g), 6),
        ((c, h), 7),
        ((d, a), 1),
        ((d, b), 2),
        ((d, c), 3),
        ((d, d), 0),
        ((d, e), 3),
        ((d, f), 4),
        ((d, g), 5),
        ((d, h), 6),
        ((e, a), inf),
        ((e, b), inf),
        ((e, c), inf),
        ((e, d), inf),
        ((e, e), 0),
        ((e, f), 1),
        ((e, g), 2),
        ((e, h), 3),
        ((f, a), inf),
        ((f, b), inf),
        ((f, c), inf),
        ((f, d), inf),
        ((f, e), 3),
        ((f, f), 0),
        ((f, g), 1),
        ((f, h), 2),
        ((g, a), inf),
        ((g, b), inf),
        ((g, c), inf),
        ((g, d), inf),
        ((g, e), 2),
        ((g, f), 3),
        ((g, g), 0),
        ((g, h), 1),
        ((h, a), inf),
        ((h, b), inf),
        ((h, c), inf),
        ((h, d), inf),
        ((h, e), 1),
        ((h, f), 2),
        ((h, g), 3),
        ((h, h), 0),
    ]
    .iter()
    .cloned()
    .collect();

    for source in graph.node_indices() {
        let s = graph.to_index(source);
        let res = spfa(&graph, source, |_| 1_i32).unwrap();

        let nodes = [a, b, c, d, e, f, g, h];

        for node in &nodes {
            let n = graph.to_index(*node);
            let dist = res.distances[n];
            let expected_dist = *expected_res.get(&(source, *node)).unwrap();
            assert_eq!(dist, expected_dist, "The distance between source '{s}' and target '{n}' is {dist}, but {expected_dist} is expected");
        }
    }
}

#[test]
fn spfa_weighted() {
    let mut graph: Graph<(), i32, Directed> = Graph::new();
    let a = graph.add_node(());
    let b = graph.add_node(());
    let c = graph.add_node(());
    let d = graph.add_node(());

    graph.extend_with_edges([
        (a, b, 1),
        (a, c, 4),
        (a, d, 10),
        (b, c, 2),
        (b, d, 2),
        (c, d, 2),
    ]);

    let inf = i32::MAX;
    let expected_res: HashMap<(NodeIndex, NodeIndex), i32> = [
        ((a, a), 0),
        ((a, b), 1),
        ((a, c), 3),
        ((a, d), 3),
        ((b, a), inf),
        ((b, b), 0),
        ((b, c), 2),
        ((b, d), 2),
        ((c, a), inf),
        ((c, b), inf),
        ((c, c), 0),
        ((c, d), 2),
        ((d, a), inf),
        ((d, b), inf),
        ((d, c), inf),
        ((d, d), 0),
    ]
    .iter()
    .cloned()
    .collect();

    for source in graph.node_indices() {
        let s = graph.to_index(source);
        let res = spfa(&graph, source, |edge| *edge.weight()).unwrap();

        let nodes = [a, b, c, d];

        for node in &nodes {
            let n = graph.to_index(*node);
            let dist = res.distances[n];
            let expected_dist = *expected_res.get(&(source, *node)).unwrap();
            assert_eq!(dist, expected_dist, "The distance between source '{s}' and target '{n}' is {dist}, but {expected_dist} is expected");
        }
    }
}

#[test]
fn spfa_weighted_undirected() {
    let mut graph: Graph<(), i32, Undirected> = Graph::new_undirected();
    let a = graph.add_node(());
    let b = graph.add_node(());
    let c = graph.add_node(());
    let d = graph.add_node(());

    graph.extend_with_edges([
        (a, b, 1),
        (a, c, 4),
        (a, d, 10),
        (b, d, 2),
        (c, b, 2),
        (c, d, 2),
    ]);

    let expected_res: HashMap<(NodeIndex, NodeIndex), i32> = [
        ((a, a), 0),
        ((a, b), 1),
        ((a, c), 3),
        ((a, d), 3),
        ((b, a), 1),
        ((b, b), 0),
        ((b, c), 2),
        ((b, d), 2),
        ((c, a), 3),
        ((c, b), 2),
        ((c, c), 0),
        ((c, d), 2),
        ((d, a), 3),
        ((d, b), 2),
        ((d, c), 2),
        ((d, d), 0),
    ]
    .iter()
    .cloned()
    .collect();

    for source in graph.node_indices() {
        let s = graph.to_index(source);
        let res = spfa(&graph, source, |edge| *edge.weight()).unwrap();

        let nodes = [a, b, c, d];

        for node in &nodes {
            let n = graph.to_index(*node);
            let dist = res.distances[n];
            let expected_dist = *expected_res.get(&(source, *node)).unwrap();
            assert_eq!(dist, expected_dist, "The distance between source '{s}' and target '{n}' is {dist}, but {expected_dist} is expected");
        }
    }
}

#[test]
fn spfa_negative_cycle() {
    let mut graph: Graph<(), i32, Directed> = Graph::new();
    let a = graph.add_node(());
    let b = graph.add_node(());
    let c = graph.add_node(());

    graph.extend_with_edges([(a, b, 1), (b, c, -3), (c, a, 1)]);

    for source in graph.node_indices() {
        let res = spfa(&graph, source, |edge| *edge.weight());
        assert!(res.is_err());
    }
}

#[test]
fn spfa_multiple_edges() {
    let mut graph: Graph<(), i32, Directed> = Graph::new();
    let a = graph.add_node(());
    let b = graph.add_node(());
    let c = graph.add_node(());
    let d = graph.add_node(());

    graph.extend_with_edges([
        (a, b, 10),
        (a, b, 1),
        (a, c, 4),
        (a, d, 10),
        (b, c, 2),
        (b, d, 2),
        (c, d, 2),
        (a, d, 100),
        (c, d, 20),
        (a, a, 5),
    ]);

    let inf = i32::MAX;
    let expected_res: HashMap<(NodeIndex, NodeIndex), i32> = [
        ((a, a), 0),
        ((a, b), 1),
        ((a, c), 3),
        ((a, d), 3),
        ((b, a), inf),
        ((b, b), 0),
        ((b, c), 2),
        ((b, d), 2),
        ((c, a), inf),
        ((c, b), inf),
        ((c, c), 0),
        ((c, d), 2),
        ((d, a), inf),
        ((d, b), inf),
        ((d, c), inf),
        ((d, d), 0),
    ]
    .iter()
    .cloned()
    .collect();

    for source in graph.node_indices() {
        let s = graph.to_index(source);
        let res = spfa(&graph, source, |edge| *edge.weight()).unwrap();

        let nodes = [a, b, c, d];
        for node in &nodes {
            let n = graph.to_index(*node);
            let dist = res.distances[n];
            let expected_dist = *expected_res.get(&(source, *node)).unwrap();
            assert_eq!(dist, expected_dist, "The distance between source '{s}' and target '{n}' is {dist}, but {expected_dist} is expected");
        }
    }
}
