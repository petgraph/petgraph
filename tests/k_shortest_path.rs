use hashbrown::HashMap;
use petgraph::algo::k_shortest_path;
use petgraph::prelude::*;
use petgraph::Graph;

#[test]
fn second_shortest_path() {
    let mut graph: Graph<(), (), Directed> = Graph::new();
    let a = graph.add_node(());
    let b = graph.add_node(());
    let c = graph.add_node(());
    let d = graph.add_node(());
    let e = graph.add_node(());
    let f = graph.add_node(());
    let g = graph.add_node(());
    let h = graph.add_node(());
    let i = graph.add_node(());
    let j = graph.add_node(());
    let k = graph.add_node(());
    let l = graph.add_node(());
    let m = graph.add_node(());

    graph.extend_with_edges([
        (a, b),
        (b, c),
        (c, d),
        (b, f),
        (f, g),
        (c, g),
        (g, h),
        (d, e),
        (e, h),
        (h, i),
        (h, j),
        (h, k),
        (h, l),
        (i, m),
        (l, k),
        (j, k),
        (j, m),
        (k, m),
        (l, m),
        (m, e),
    ]);

    let res = k_shortest_path(&graph, a, None, 2, |_| 1);

    let expected_res: HashMap<NodeIndex, usize> = [
        (e, 7),
        (g, 3),
        (h, 4),
        (i, 5),
        (j, 5),
        (k, 5),
        (l, 5),
        (m, 6),
    ]
    .iter()
    .cloned()
    .collect();

    assert_eq!(res, expected_res);
}

#[cfg(feature = "stable_graph")]
#[test]
fn shortest_paths_with_removed_nodes() {
    for removed_index in [0, 1, 3] {
        let mut graph = StableDiGraph::<(), u32>::new();
        let nodes: Vec<_> = (0..4).map(|_| graph.add_node(())).collect();
        graph.remove_node(nodes[removed_index]);
        let remaining: Vec<_> = graph.node_indices().collect();
        let [a, b, c] = [remaining[0], remaining[1], remaining[2]];
        graph.extend_with_edges([(a, b, 1), (b, c, 1), (a, c, 10)]);

        for k in 1..=3 {
            let expected: HashMap<_, u32> = match k {
                1 => [(a, 0), (b, 1), (c, 2)].into_iter().collect(),
                2 => [(c, 10)].into_iter().collect(),
                _ => HashMap::new(),
            };
            for goal in [None, Some(c)] {
                let result = k_shortest_path(&graph, a, goal, k, |edge| *edge.weight());
                assert_eq!(
                    result, expected,
                    "removed={removed_index}, k={k}, goal={goal:?}"
                );
            }
        }
    }
}

#[cfg(feature = "stable_graph")]
#[test]
fn shortest_paths_from_sparse_isolated_node() {
    let mut graph = StableDiGraph::<(), u32>::new();
    let removed = graph.add_node(());
    let start = graph.add_node(());
    graph.remove_node(removed);

    for goal in [None, Some(start)] {
        let result = k_shortest_path(&graph, start, goal, 1, |edge| *edge.weight());
        assert_eq!(result, [(start, 0)].into_iter().collect());
        assert!(k_shortest_path(&graph, start, goal, 2, |edge| *edge.weight()).is_empty());
    }
}
