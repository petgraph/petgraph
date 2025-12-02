use core::fmt::Debug;
use core::hash::Hash;
use hashbrown::HashMap;
use petgraph::algo::johnson;
use petgraph::visit::GraphBase;
use petgraph::{prelude::*, Directed, Graph, Undirected};

#[cfg(feature = "rayon")]
use petgraph::algo::parallel_johnson;

#[test]
fn johnson_uniform_weight() {
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
        ((e, e), 0),
        ((e, f), 1),
        ((e, g), 2),
        ((e, h), 3),
        ((f, e), 3),
        ((f, f), 0),
        ((f, g), 1),
        ((f, h), 2),
        ((g, e), 2),
        ((g, f), 3),
        ((g, g), 0),
        ((g, h), 1),
        ((h, e), 1),
        ((h, f), 2),
        ((h, g), 3),
        ((h, h), 0),
    ]
    .iter()
    .cloned()
    .collect();

    let res = johnson(&graph, |_| 1_i32).unwrap();
    let nodes = [a, b, c, d, e, f, g, h];

    match_results::<Graph<(), i32, Directed>, i32>(res, &expected_res, &nodes);

    #[cfg(feature = "rayon")]
    {
        let res = parallel_johnson(&graph, |_| 1_i32).unwrap();
        match_results::<Graph<(), i32, Directed>, i32>(res, &expected_res, &nodes);
    }
}

#[test]
fn johnson_weighted() {
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

    let expected_res: HashMap<(NodeIndex, NodeIndex), i32> = [
        ((a, a), 0),
        ((a, b), 1),
        ((a, c), 3),
        ((a, d), 3),
        ((b, b), 0),
        ((b, c), 2),
        ((b, d), 2),
        ((c, c), 0),
        ((c, d), 2),
        ((d, d), 0),
    ]
    .iter()
    .cloned()
    .collect();

    let res = johnson(&graph, |edge| *edge.weight()).unwrap();
    let nodes = [a, b, c, d];

    match_results::<Graph<(), i32, Directed>, i32>(res, &expected_res, &nodes);

    #[cfg(feature = "rayon")]
    {
        let res = parallel_johnson(&graph, |edge| *edge.weight()).unwrap();
        match_results::<Graph<(), i32, Directed>, i32>(res, &expected_res, &nodes);
    }
}

#[test]
fn johnson_weighted_undirected() {
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

    let res = johnson(&graph, |edge| *edge.weight()).unwrap();
    let nodes = [a, b, c, d];

    match_results::<Graph<(), i32, Directed>, i32>(res, &expected_res, &nodes);

    #[cfg(feature = "rayon")]
    {
        let res = parallel_johnson(&graph, |edge| *edge.weight()).unwrap();
        match_results::<Graph<(), i32, Directed>, i32>(res, &expected_res, &nodes);
    }
}

#[test]
fn johnson_negative_cycle() {
    let mut graph: Graph<(), f32, Directed> = Graph::new();
    let a = graph.add_node(());
    let b = graph.add_node(());
    let c = graph.add_node(());

    graph.extend_with_edges([(a, b, 1.0), (b, c, -3.0), (c, a, 1.0)]);

    let res = johnson(&graph, |edge| *edge.weight());
    assert!(res.is_err());

    #[cfg(feature = "rayon")]
    {
        let res = parallel_johnson(&graph, |edge| *edge.weight());
        assert!(res.is_err());
    }
}

#[test]
fn johnson_multiple_edges() {
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

    let expected_res: HashMap<(NodeIndex, NodeIndex), i32> = [
        ((a, a), 0),
        ((a, b), 1),
        ((a, c), 3),
        ((a, d), 3),
        ((b, b), 0),
        ((b, c), 2),
        ((b, d), 2),
        ((c, c), 0),
        ((c, d), 2),
        ((d, d), 0),
    ]
    .iter()
    .cloned()
    .collect();

    let res = johnson(&graph, |edge| *edge.weight()).unwrap();
    let nodes = [a, b, c, d];

    match_results::<Graph<(), i32, Directed>, i32>(res, &expected_res, &nodes);

    #[cfg(feature = "rayon")]
    {
        let res = parallel_johnson(&graph, |edge| *edge.weight()).unwrap();
        match_results::<Graph<(), i32, Directed>, i32>(res, &expected_res, &nodes);
    }
}

fn match_results<G, K>(
    res: HashMap<(G::NodeId, G::NodeId), K>,
    expected_res: &HashMap<(G::NodeId, G::NodeId), K>,
    nodes: &[G::NodeId],
) where
    G: GraphBase,
    G::NodeId: Eq + Hash,
    K: Eq + Debug,
{
    for node1 in nodes {
        for node2 in nodes {
            assert_eq!(
                res.get(&(*node1, *node2)),
                expected_res.get(&(*node1, *node2))
            );
        }
    }
}
