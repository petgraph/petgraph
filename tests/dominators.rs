use petgraph::algo::dominators::simple_fast;
use petgraph::prelude::*;
use petgraph::visit::Dfs;

// Removing a dominator makes its dominated nodes unreachable from the root.
// Check every directed graph on four nodes (excluding self edges) against
// this independent definition, rather than another dominator algorithm.
#[test]
fn dominators_match_reachability_oracle() {
    let edges: Vec<_> = (0..4)
        .flat_map(|from| (0..4).map(move |to| (from, to)))
        .filter(|(from, to)| from != to)
        .collect();
    for mask in 0..(1 << edges.len()) {
        let mut graph = DiGraph::<(), ()>::new();
        let nodes: Vec<_> = (0..4).map(|_| graph.add_node(())).collect();
        for (bit, &(from, to)) in edges.iter().enumerate() {
            if mask & (1 << bit) != 0 {
                graph.add_edge(nodes[from], nodes[to], ());
            }
        }
        let mut dfs = Dfs::new(&graph, nodes[0]);
        let mut reachable = [false; 4];
        while let Some(node) = dfs.next(&graph) {
            reachable[node.index()] = true;
        }
        let dominators = simple_fast(&graph, nodes[0]);
        for removed in 0..4 {
            let mut seen = [false; 4];
            let mut pending = vec![nodes[0]];
            while let Some(node) = pending.pop() {
                if node.index() == removed || seen[node.index()] {
                    continue;
                }
                seen[node.index()] = true;
                pending.extend(graph.neighbors(node));
            }
            for target in 0..4 {
                let actual = dominators.dominators(nodes[target]);
                if !reachable[target] {
                    assert!(actual.is_none());
                } else {
                    assert_eq!(
                        actual.unwrap().any(|node| node == nodes[removed]),
                        !seen[target],
                        "graph {mask}, removed {removed}, target {target}"
                    );
                }
            }
        }
    }
}

#[test]
fn dominators_support_sparse_indices_and_parallel_edges() {
    let mut graph = StableDiGraph::<(), ()>::new();
    let root = graph.add_node(());
    let hole = graph.add_node(());
    let join = graph.add_node(());
    let leaf = graph.add_node(());
    let unreachable = graph.add_node(());
    graph.remove_node(hole);
    graph.add_edge(root, join, ());
    graph.add_edge(root, join, ());
    graph.add_edge(join, join, ());
    graph.add_edge(join, leaf, ());
    graph.add_edge(leaf, join, ());
    let dominators = simple_fast(&graph, root);
    assert_eq!(dominators.immediate_dominator(join), Some(root));
    assert_eq!(dominators.immediate_dominator(leaf), Some(join));
    assert!(dominators.dominators(unreachable).is_none());
}

#[test]
fn dominators_support_noncompact_graphmap_ids_and_root_only() {
    let mut graph = DiGraphMap::<u32, ()>::new();
    graph.add_node(100);
    assert_eq!(
        simple_fast(&graph, 100)
            .dominators(100)
            .unwrap()
            .collect::<Vec<_>>(),
        vec![100]
    );
    graph.add_edge(100, 400, ());
    graph.add_edge(100, 900, ());
    graph.add_edge(400, 1500, ());
    graph.add_edge(900, 1500, ());
    assert_eq!(
        simple_fast(&graph, 100).immediate_dominator(1500),
        Some(100)
    );
}
