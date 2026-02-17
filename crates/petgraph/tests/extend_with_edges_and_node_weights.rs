// To test the new function
// [`extend_with_edges_and_node_weights`](struct.Graph.html#method
// .extend_with_edges_and_node_weights).

use std::collections::HashMap;

use petgraph::{
    Graph,
    graph::{DefaultIx, NodeIndex},
    prelude::*,
};

#[test]
fn extend_with_edges_and_node_weights_empty() {
    let mut g = Graph::<i32, i32>::new();
    g.extend_with_edges_and_node_weights(&[] as &[(u32, u32, i32); 0], |_| None);

    assert_eq!(g.node_count(), 0);
    assert_eq!(g.edge_count(), 0);
}

#[test]
fn extend_with_edges_and_node_weights_none() {
    let mut g = Graph::<(), i32>::new();
    g.extend_with_edges_and_node_weights([(0, 1, 7), (1, 2, 8)], |_| None);

    assert_eq!(g.node_count(), 3);
    assert_eq!(g.edge_count(), 2);

    // Check that the edges were really added, and have the expected weights.
    assert!(g.contains_edge(0.into(), 1.into()));
    assert!(g.contains_edge(1.into(), 2.into()));
    assert_eq!(
        g.edge_weight(g.find_edge(0.into(), 1.into()).unwrap()),
        Some(&7)
    );
    assert_eq!(
        g.edge_weight(g.find_edge(1.into(), 2.into()).unwrap()),
        Some(&8)
    );

    // Check that the node weights are all the default (), since None was provided.
    assert_eq!(g.node_weight(0.into()), Some(&()));
    assert_eq!(g.node_weight(1.into()), Some(&()));
    assert_eq!(g.node_weight(2.into()), Some(&()));
}

#[test]
fn extend_with_edges_and_node_weights_const() {
    let mut g = Graph::<i32, i32>::new();
    g.extend_with_edges_and_node_weights([(0, 1, 7), (1, 2, 8)], |_| Some(42));

    assert_eq!(g.node_count(), 3);
    assert_eq!(g.edge_count(), 2);

    // Check that the edges were really added, and have the expected weights.
    assert!(g.contains_edge(0.into(), 1.into()));
    assert!(g.contains_edge(1.into(), 2.into()));
    assert_eq!(
        g.edge_weight(g.find_edge(0.into(), 1.into()).unwrap()),
        Some(&7)
    );
    assert_eq!(
        g.edge_weight(g.find_edge(1.into(), 2.into()).unwrap()),
        Some(&8)
    );

    // Check that the node weights are all the specified 42.
    assert_eq!(g.node_weight(0.into()), Some(&42));
    assert_eq!(g.node_weight(1.into()), Some(&42));
    assert_eq!(g.node_weight(2.into()), Some(&42));
}

#[test]
fn extend_with_edges_and_node_weights_subset() {
    let mut g = Graph::<i32, i32>::new();

    let node_weights: HashMap<usize, i32> = [(0, 10), (2, 20)].into();

    g.extend_with_edges_and_node_weights([(0, 1, 7), (1, 2, 8)], |idx| {
        node_weights.get(&idx.index()).copied()
    });

    assert_eq!(g.node_count(), 3);
    assert_eq!(g.edge_count(), 2);

    // Check that the edges were really added, and have the expected weights.
    assert!(g.contains_edge(0.into(), 1.into()));
    assert!(g.contains_edge(1.into(), 2.into()));
    assert_eq!(
        g.edge_weight(g.find_edge(0.into(), 1.into()).unwrap()),
        Some(&7)
    );
    assert_eq!(
        g.edge_weight(g.find_edge(1.into(), 2.into()).unwrap()),
        Some(&8)
    );

    // Check that the node weights are as specified by node_weights, and the
    // default for i32, otherwise.
    assert_eq!(g.node_weight(0.into()), Some(&10));
    assert_eq!(g.node_weight(1.into()), Some(&i32::default()));
    assert_eq!(g.node_weight(2.into()), Some(&20));
}

#[test]
fn extend_with_edges_and_node_weights_single_edge() {
    let mut g = Graph::<String, f64>::new();
    g.extend_with_edges_and_node_weights([(0, 1, 3.15)], |idx| {
        Some(format!("Node {}", idx.index()))
    });

    assert_eq!(g.node_count(), 2);
    assert_eq!(g.edge_count(), 1);

    // Check that the edge was really added, and has the expected weight.
    assert!(g.contains_edge(0.into(), 1.into()));
    assert_eq!(
        g.edge_weight(g.find_edge(0.into(), 1.into()).unwrap()),
        Some(&3.15)
    );

    // Check that the node weights are as specified by the closure.
    assert_eq!(g.node_weight(0.into()), Some(&"Node 0".to_string()));
    assert_eq!(g.node_weight(1.into()), Some(&"Node 1".to_string()));
}

#[test]
fn extend_with_edges_and_node_weights_gaps_in_indices() {
    let mut g = Graph::<u32, char>::new();
    g.extend_with_edges_and_node_weights([(0, 3, 'a'), (5, 6, 'b')], |idx| {
        Some(idx.index() as u32 * 10)
    });

    assert_eq!(g.node_count(), 7);
    assert_eq!(g.edge_count(), 2);

    // Check that the edges were really added, and have the expected weights.
    assert!(g.contains_edge(0.into(), 3.into()));
    assert!(g.contains_edge(5.into(), 6.into()));
    assert_eq!(
        g.edge_weight(g.find_edge(0.into(), 3.into()).unwrap()),
        Some(&'a')
    );
    assert_eq!(
        g.edge_weight(g.find_edge(5.into(), 6.into()).unwrap()),
        Some(&'b')
    );

    // Check that all nodes, including gaps, have weights from the closure.
    for i in 0..7 {
        let expected = (i as u32) * 10;
        assert_eq!(g.node_weight(NodeIndex::new(i)), Some(&expected));
    }
}

#[test]
fn extend_with_edges_and_node_weights_existing_nodes_no_overwrite() {
    let mut g = Graph::<i32, i32>::new();
    let n0 = g.add_node(100); // Node 0
    let n1 = g.add_node(200); // Node 1
    let n2 = g.add_node(400); // Node 2

    g.extend_with_edges_and_node_weights([(0, 1, 7), (1, 2, 8), (3, 4, 9)], |_| Some(42));

    assert_eq!(g.node_count(), 5);
    assert_eq!(g.edge_count(), 3);

    // Check that the edges were really added, and have the expected weights.
    assert!(g.contains_edge(0.into(), 1.into()));
    assert!(g.contains_edge(1.into(), 2.into()));
    assert!(g.contains_edge(3.into(), 4.into()));
    assert_eq!(
        g.edge_weight(g.find_edge(0.into(), 1.into()).unwrap()),
        Some(&7)
    );
    assert_eq!(
        g.edge_weight(g.find_edge(1.into(), 2.into()).unwrap()),
        Some(&8)
    );
    assert_eq!(
        g.edge_weight(g.find_edge(3.into(), 4.into()).unwrap()),
        Some(&9)
    );

    // Check that existing node weights are unchanged, new ones are 42.
    assert_eq!(g.node_weight(n0), Some(&100)); // Existing
    assert_eq!(g.node_weight(n1), Some(&200)); // Existing
    assert_eq!(g.node_weight(n2), Some(&400)); // Existing
    assert_eq!(g.node_weight(3.into()), Some(&42)); // New
    assert_eq!(g.node_weight(4.into()), Some(&42)); // New
}

#[test]
fn extend_with_edges_and_node_weights_self_loop() {
    let mut g = Graph::<bool, String>::new();
    g.extend_with_edges_and_node_weights(&[(2, 2, "loop".to_string())], |idx| {
        Some(idx.index() % 2 == 0)
    });

    assert_eq!(g.node_count(), 3);
    assert_eq!(g.edge_count(), 1);

    // Check that the self-loop was added.
    assert!(g.contains_edge(2.into(), 2.into()));
    assert_eq!(
        g.edge_weight(g.find_edge(2.into(), 2.into()).unwrap()),
        Some(&"loop".to_string())
    );

    // Check node weights, including gaps.
    assert_eq!(g.node_weight(0.into()), Some(&true)); // 0 % 2 == 0
    assert_eq!(g.node_weight(1.into()), Some(&false));
    assert_eq!(g.node_weight(2.into()), Some(&true));
}

#[test]
fn extend_with_edges_and_node_weights_parallel_edges() {
    let mut g = Graph::<(), u8>::new();
    g.extend_with_edges_and_node_weights([(0, 1, 1), (0, 1, 2), (0, 1, 3)], |_| None);

    assert_eq!(g.node_count(), 2);
    assert_eq!(g.edge_count(), 3);

    // Check that all parallel edges were added with their weights.
    let mut edges_found = 0;
    for e in g.edge_indices() {
        if g.edge_endpoints(e) == Some((0.into(), 1.into())) {
            let weight = g.edge_weight(e).unwrap();
            assert!(matches!(weight, &1 | &2 | &3));
            edges_found += 1;
        }
    }
    assert_eq!(edges_found, 3);

    // Node weights default.
    assert_eq!(g.node_weight(0.into()), Some(&()));
    assert_eq!(g.node_weight(1.into()), Some(&()));
}

#[test]
fn extend_with_edges_and_node_weights_undirected() {
    let mut g: Graph<i32, i32, Undirected> = Graph::new_undirected();

    // (1,0) is same as (0,1) in undirected, but different weight means parallel?
    g.extend_with_edges_and_node_weights([(0, 1, 7), (1, 0, 8)], |_| Some(42));

    assert_eq!(g.node_count(), 2);
    assert_eq!(g.edge_count(), 2); // Allows parallel in undirected if weights differ.

    // Check edges (undirected, order doesn't matter).
    assert!(g.contains_edge(0.into(), 1.into()));
    assert!(g.contains_edge(1.into(), 0.into()));
    let mut weights = vec![];
    for e in g.edge_indices() {
        weights.push(*g.edge_weight(e).unwrap());
    }
    weights.sort();
    assert_eq!(weights, vec![7, 8]);

    // Node weights.
    assert_eq!(g.node_weight(0.into()), Some(&42));
    assert_eq!(g.node_weight(1.into()), Some(&42));
}

#[test]
fn extend_with_edges_and_node_weights_mixed_some_none() {
    let mut g = Graph::<Option<u32>, i64>::new();
    let node_weights: HashMap<usize, Option<u32>> =
        [(0, Some(100)), (2, None), (3, Some(300))].into();

    g.extend_with_edges_and_node_weights([(0, 2, -1), (3, 4, -2)], |idx| {
        Some(node_weights.get(&idx.index()).copied().flatten())
    });

    assert_eq!(g.node_count(), 5);
    assert_eq!(g.edge_count(), 2);

    // Check edges.
    assert!(g.contains_edge(0.into(), 2.into()));
    assert!(g.contains_edge(3.into(), 4.into()));
    assert_eq!(
        g.edge_weight(g.find_edge(0.into(), 2.into()).unwrap()),
        Some(&-1)
    );
    assert_eq!(
        g.edge_weight(g.find_edge(3.into(), 4.into()).unwrap()),
        Some(&-2)
    );

    // Check node weights: from map if Some, default (None) if closure returns None or not in map.
    assert_eq!(g.node_weight(0.into()), Some(&Some(100)));
    assert_eq!(g.node_weight(1.into()), Some(&None)); // Not in map => closure None => default None
    assert_eq!(g.node_weight(2.into()), Some(&None)); // In map as None => flatten None => default
    assert_eq!(g.node_weight(3.into()), Some(&Some(300)));
    assert_eq!(g.node_weight(4.into()), Some(&None));
}

#[test]
fn extend_with_edges_and_node_weights_large_indices() {
    let mut g = Graph::<u64, i8>::new();
    let edges: Vec<(DefaultIx, DefaultIx, i8)> = (0..100)
        .map(|i| {
            (
                (i * 10) as DefaultIx,
                (i * 10 + 1) as DefaultIx,
                (i % 127) as i8,
            )
        })
        .collect();

    g.extend_with_edges_and_node_weights(&edges, |idx| Some((idx.index() as u64).pow(2)));

    let expected_node_count = ((99 * 10 + 1) + 1) as usize; // Max index 991 +1 =992, but with gaps.
    assert_eq!(g.node_count(), expected_node_count);
    assert_eq!(g.edge_count(), 100);

    // Spot check some edges and nodes.
    for i in 0..100 {
        let a = NodeIndex::new(i * 10);
        let b = NodeIndex::new(i * 10 + 1);
        assert!(g.contains_edge(a, b));
        assert_eq!(
            g.edge_weight(g.find_edge(a, b).unwrap()),
            Some(&((i % 127) as i8))
        );
        assert_eq!(g.node_weight(a), Some(&((i * 10) as u64).pow(2)));
        assert_eq!(g.node_weight(b), Some(&((i * 10 + 1) as u64).pow(2)));
    }
    // Check a gap node.
    assert_eq!(g.node_weight(NodeIndex::new(2)), Some(&(2u64.pow(2))));
}
