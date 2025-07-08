use hashbrown::HashSet;
use petgraph::{
    algo::community::{louvain_communities, modularity},
    graph::{DiGraph, NodeIndex, UnGraph},
    Graph,
};

#[test]
fn test_modularity_on_empty_graph() {
    let graph: UnGraph<f64, f64> = UnGraph::new_undirected();
    let communities: HashSet<NodeIndex> = HashSet::new();
    let resolution = 1.0;

    assert_eq!(modularity(&graph, &[communities], resolution), Ok(0.0));
}

#[test]
fn test_modularity_on_edgeless_graph() {
    let mut graph: Graph<(), f64, petgraph::Undirected> = UnGraph::new_undirected();
    let n0 = graph.add_node(());
    let n1 = graph.add_node(());
    let n2 = graph.add_node(());

    let communities: Vec<HashSet<NodeIndex>> = vec![
        HashSet::from([n0]),
        HashSet::from([n1]),
        HashSet::from([n2]),
    ];
    let resolution = 1.0;

    assert_eq!(modularity(&graph, &communities, resolution), Ok(0.0));
}

#[test]
fn test_modularity_on_full_graph() {
    let mut graph: Graph<u32, f64, petgraph::Undirected> = UnGraph::new_undirected();
    let nodes: Vec<NodeIndex> = (0..4).map(|i| graph.add_node(i)).collect();
    graph.add_edge(nodes[0], nodes[1], 1.0);
    graph.add_edge(nodes[0], nodes[2], 1.0);
    graph.add_edge(nodes[0], nodes[3], 1.0);
    graph.add_edge(nodes[1], nodes[2], 1.0);
    graph.add_edge(nodes[1], nodes[3], 1.0);
    graph.add_edge(nodes[2], nodes[3], 1.0);

    let resolution = 1.0;

    // all nodes in one community
    let single_community_partition = vec![nodes.iter().cloned().collect::<HashSet<_>>()];
    let result1 = modularity(&graph, &single_community_partition, resolution).unwrap();
    assert!(
        (result1 - 0.0).abs() < f64::EPSILON,
        "Expected 0.0 for single community, got {result1}",
    );

    // split into communities
    let two_communities_partition = vec![
        HashSet::from([nodes[0], nodes[1]]),
        HashSet::from([nodes[2], nodes[3]]),
    ];

    let expected_q = -1.0 / 6.0;
    let result2 = modularity(&graph, &two_communities_partition, resolution).unwrap();
    assert!(
        (result2 - expected_q).abs() < f64::EPSILON,
        "Expected {expected_q}, got {result2}"
    );

    // negative weights
    let mut graph_negative: Graph<u32, f64, petgraph::Undirected> = UnGraph::new_undirected();
    let nodes_neg: Vec<NodeIndex> = (0..4).map(|i| graph_negative.add_node(i)).collect();

    graph_negative.add_edge(nodes_neg[0], nodes_neg[1], 2.0);
    graph_negative.add_edge(nodes_neg[0], nodes_neg[2], -1.0);
    graph_negative.add_edge(nodes_neg[1], nodes_neg[3], 1.0);
    graph_negative.add_edge(nodes_neg[2], nodes_neg[3], -0.5);

    let partition_negative = vec![
        HashSet::from([nodes_neg[0], nodes_neg[1]]),
        HashSet::from([nodes_neg[2], nodes_neg[3]]),
    ];

    let expected_q_negative = -8.0 / 9.0;
    let result3 = modularity(&graph_negative, &partition_negative, resolution).unwrap();
    assert!(
        (result3 - expected_q_negative).abs() < 0.001,
        "Expected {expected_q_negative} for negative weights, got {result3}"
    );

    // self-loops
    let mut graph_self_loops: Graph<u32, f64, petgraph::Undirected> = UnGraph::new_undirected();
    let nodes_self: Vec<NodeIndex> = (0..3).map(|i| graph_self_loops.add_node(i)).collect();

    graph_self_loops.add_edge(nodes_self[0], nodes_self[1], 1.0);
    graph_self_loops.add_edge(nodes_self[1], nodes_self[2], 1.0);
    graph_self_loops.add_edge(nodes_self[0], nodes_self[0], 0.5);
    graph_self_loops.add_edge(nodes_self[2], nodes_self[2], 1.0);

    let partition_self_loops = vec![
        HashSet::from([nodes_self[0]]),
        HashSet::from([nodes_self[1]]),
        HashSet::from([nodes_self[2]]),
    ];

    let result4 = modularity(&graph_self_loops, &partition_self_loops, resolution).unwrap();

    assert!(
        result4.is_finite(),
        "Modularity with self-loops should be finite, got {result4}",
    );

    // edge with 0 weight
    let mut graph_zero: Graph<u32, f64, petgraph::Undirected> = UnGraph::new_undirected();
    let nodes_zero: Vec<NodeIndex> = (0..2).map(|i| graph_zero.add_node(i)).collect();
    graph_zero.add_edge(nodes_zero[0], nodes_zero[1], 1.0);
    graph_zero.add_edge(nodes_zero[0], nodes_zero[1], -1.0);

    let partition_zero = vec![
        HashSet::from([nodes_zero[0]]),
        HashSet::from([nodes_zero[1]]),
    ];
    let result5 = modularity(&graph_zero, &partition_zero, resolution);
    if let Ok(val) = result5 {
        assert!(
            val.is_finite() || val.is_nan(),
            "Result should be finite or NaN"
        )
    }
}
#[test]
fn test_modularity_on_directed_graph() {
    let mut graph: Graph<u32, f64, petgraph::Directed> = DiGraph::new();
    let nodes: Vec<NodeIndex> = (0..4).map(|i| graph.add_node(i)).collect();

    graph.add_edge(nodes[0], nodes[1], 1.0);
    graph.add_edge(nodes[1], nodes[0], 1.0);
    graph.add_edge(nodes[0], nodes[2], 1.0);
    graph.add_edge(nodes[2], nodes[3], 1.0);
    graph.add_edge(nodes[3], nodes[1], 1.0);
    graph.add_edge(nodes[1], nodes[3], 1.0);

    let resolution = 1.0;
    let single_community_partition = vec![nodes.iter().cloned().collect::<HashSet<_>>()];
    let result1 = modularity(&graph, &single_community_partition, resolution).unwrap();
    assert!(
        (result1 - 0.0).abs() < f64::EPSILON,
        "Expected 0.0 for single community, got {result1}",
    );
    let two_communities_partition = vec![
        HashSet::from([nodes[0], nodes[1]]),
        HashSet::from([nodes[2], nodes[3]]),
    ];

    let _result2 = modularity(&graph, &two_communities_partition, resolution).unwrap();
    let mut graph_asym: Graph<u32, f64, petgraph::Directed> = DiGraph::new();
    let nodes_asym: Vec<NodeIndex> = (0..4).map(|i| graph_asym.add_node(i)).collect();

    graph_asym.add_edge(nodes_asym[0], nodes_asym[1], 2.0);
    graph_asym.add_edge(nodes_asym[1], nodes_asym[2], 1.0);
    graph_asym.add_edge(nodes_asym[2], nodes_asym[0], 1.5);
    graph_asym.add_edge(nodes_asym[3], nodes_asym[0], 1.0);

    let partition_asym = vec![
        HashSet::from([nodes_asym[0], nodes_asym[1]]),
        HashSet::from([nodes_asym[2], nodes_asym[3]]),
    ];

    let result3 = modularity(&graph_asym, &partition_asym, resolution).unwrap();
    assert!(
        result3.is_finite(),
        "Modularity for asymmetric directed graph should be finite, got {result3}",
    );
    let mut graph_self_loops: Graph<u32, f64, petgraph::Directed> = DiGraph::new();
    let nodes_self: Vec<NodeIndex> = (0..3).map(|i| graph_self_loops.add_node(i)).collect();

    graph_self_loops.add_edge(nodes_self[0], nodes_self[1], 1.0);
    graph_self_loops.add_edge(nodes_self[1], nodes_self[2], 1.0);

    graph_self_loops.add_edge(nodes_self[0], nodes_self[0], 0.5);
    graph_self_loops.add_edge(nodes_self[2], nodes_self[2], 1.0);

    let partition_self_loops = vec![
        HashSet::from([nodes_self[0]]),
        HashSet::from([nodes_self[1]]),
        HashSet::from([nodes_self[2]]),
    ];

    let result4 = modularity(&graph_self_loops, &partition_self_loops, resolution).unwrap();
    assert!(
        result4.is_finite(),
        "Modularity with self-loops in directed graph should be finite, got {result4}",
    );

    // strongly connected components
    let mut graph_scc: Graph<u32, f64, petgraph::Directed> = DiGraph::new();
    let nodes_scc: Vec<NodeIndex> = (0..4).map(|i| graph_scc.add_node(i)).collect();

    graph_scc.add_edge(nodes_scc[0], nodes_scc[1], 1.0);
    graph_scc.add_edge(nodes_scc[1], nodes_scc[2], 1.0);
    graph_scc.add_edge(nodes_scc[2], nodes_scc[0], 1.0);
    graph_scc.add_edge(nodes_scc[3], nodes_scc[0], 1.0); // connection to cycle

    let partition_scc = vec![
        HashSet::from([nodes_scc[0], nodes_scc[1], nodes_scc[2]]), // the cycle
        HashSet::from([nodes_scc[3]]),                             // isolated node
    ];

    let result5 = modularity(&graph_scc, &partition_scc, resolution).unwrap();
    assert!(
        result5.is_finite(),
        "Modularity for strongly connected components should be finite, got {result5}",
    );

    // negative weights in directed graph
    let mut graph_negative: Graph<u32, f64, petgraph::Directed> = DiGraph::new();
    let nodes_neg: Vec<NodeIndex> = (0..3).map(|i| graph_negative.add_node(i)).collect();

    // mixed positive and negative weights
    graph_negative.add_edge(nodes_neg[0], nodes_neg[1], 2.0);
    graph_negative.add_edge(nodes_neg[1], nodes_neg[2], -1.0);
    graph_negative.add_edge(nodes_neg[2], nodes_neg[0], 1.5);

    let partition_negative = vec![
        HashSet::from([nodes_neg[0], nodes_neg[1]]),
        HashSet::from([nodes_neg[2]]),
    ];

    let result6 = modularity(&graph_negative, &partition_negative, resolution).unwrap();
    assert!(
        result6.is_finite(),
        "Modularity with negative weights in directed graph should be finite, got {result6}",
    );

    println!("All directed graph modularity tests passed!");
}
#[test]
fn test_louvain_empty_graph() {
    let g = Graph::<i32, f64>::new();
    let louvain = louvain_communities(&g, 0.01, 0.0, None, None).unwrap();
    let expected_clique: Vec<HashSet<NodeIndex>> = Vec::new();
    assert_eq!(expected_clique, louvain.communities)
}

#[test]
fn test_louvain_self_loop() {
    // a single node that has a self-loop
    let mut g = Graph::<i32, f64, petgraph::Undirected>::new_undirected();
    let node = g.add_node(42);
    g.add_edge(node, node, 2.0); // Self-loop with weight 2.0

    let louvain_result = louvain_communities(&g, 0.01, 0.0, None, None).unwrap();

    let mut expected_communities = Vec::new();
    let mut single_node_community = HashSet::new();
    single_node_community.insert(node);
    expected_communities.push(single_node_community);

    assert_eq!(expected_communities, louvain_result.communities);
    assert!(louvain_result.modularity >= 0.0);
}

#[test]
fn test_louvain_single_node_graph() {
    // create a new graph and add a single node
    let mut g = Graph::<i32, f64>::new();
    let node_data = 1;
    let node_idx = g.add_node(node_data);

    let louvain_result = louvain_communities(&g, 0.01, 0.0, None, None).unwrap();
    let mut expected_communities = Vec::new();
    let mut single_node_community = HashSet::new();
    single_node_community.insert(node_idx);
    expected_communities.push(single_node_community);

    assert_eq!(expected_communities, louvain_result.communities);
}

#[test]
fn test_louvain_two_nodes_one_edge() {
    // create a graph with two nodes connected by a single edge
    let mut g = Graph::<i32, f64, petgraph::Undirected>::new_undirected();
    let n1 = g.add_node(10);
    let n2 = g.add_node(20);
    g.add_edge(n1, n2, 1.0);

    let communities = louvain_communities(&g, 0.01, 0.0, None, None).unwrap();
    let mut expected_communities = Vec::new();
    let mut community = HashSet::new();
    community.insert(n1);
    community.insert(n2);
    expected_communities.push(community);

    assert_eq!(expected_communities, communities.communities);
}

#[test]
fn test_louvain_two_clear_communities() {
    // This test creates a graph with two distinct cliques and a single bridge between them

    let mut g = Graph::<i32, f64, petgraph::Undirected>::new_undirected();
    let nodes: Vec<NodeIndex> = (0..8).map(|i| g.add_node(i)).collect();
    for i in 0..4 {
        for j in (i + 1)..4 {
            g.add_edge(nodes[i], nodes[j], 1.0);
        }
    }
    for i in 4..8 {
        for j in (i + 1)..8 {
            g.add_edge(nodes[i], nodes[j], 1.0);
        }
    }
    g.add_edge(nodes[0], nodes[4], 0.1);

    let communities = louvain_communities(&g, 1.0, 0.0001, None, None).unwrap();

    let mut community_a = HashSet::new();
    for node in nodes.iter().take(4) {
        community_a.insert(*node);
    }
    let mut community_b = HashSet::new();
    for node in nodes.iter().take(8).skip(4) {
        community_b.insert(*node);
    }
    let expected_communities = vec![community_a, community_b];

    // sort the communities to make sure they're the same
    let mut sorted_communities: Vec<Vec<NodeIndex>> = communities
        .communities
        .into_iter()
        .map(|set| {
            let mut v: Vec<_> = set.into_iter().collect();
            v.sort();
            v
        })
        .collect();
    sorted_communities.sort();
    let mut sorted_expected: Vec<Vec<NodeIndex>> = expected_communities
        .into_iter()
        .map(|set| {
            let mut v: Vec<_> = set.into_iter().collect();
            v.sort();
            v
        })
        .collect();
    sorted_expected.sort();

    assert_eq!(sorted_expected, sorted_communities);
}

#[test]
fn test_louvain_negative_weights() {
    let mut graph = Graph::<i32, f64, petgraph::Undirected>::new_undirected();
    let n1 = graph.add_node(1);
    let n2 = graph.add_node(2);
    let n3 = graph.add_node(3);
    graph.add_edge(n1, n2, 5.0);
    graph.add_edge(n1, n3, -0.5);
    graph.add_edge(n2, n3, -0.5);

    let louvain_result = louvain_communities(&graph, 0.01, 0.0, None, None);

    // it should error for now as this louvain implementation does not support negative weights
    assert!(louvain_result.is_err());
}

#[test]
fn test_louvain_two_communities() {
    // create graph with nodes and edges
    let mut graph: Graph<u32, f64, petgraph::Undirected> = UnGraph::new_undirected();
    let nodes: Vec<_> = (0..4).map(|i| graph.add_node(i)).collect();

    // Add edges with weights
    graph.add_edge(nodes[0], nodes[1], 1.0);
    graph.add_edge(nodes[0], nodes[2], 1.0);
    graph.add_edge(nodes[2], nodes[3], 1.0);

    // The graph looks like this:
    // 0 - 1
    // |
    // 2 - 3

    let communities = louvain_communities(&graph, 1.0, 0.001, None, None).unwrap();

    // two communities: ((0,1), (2,3))
    assert!(communities.communities.len() == 2);
    assert!(communities
        .communities
        .contains(&HashSet::from([nodes[0], nodes[1]])));
    assert!(communities
        .communities
        .contains(&HashSet::from([nodes[2], nodes[3]])));
}
