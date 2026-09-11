use hashbrown::HashMap;
use petgraph::{Graph, Undirected, algo::seidel, prelude::*};

/// Reference all-pairs shortest path lengths on an unweighted undirected graph, computed with a
/// breadth-first search from every node. Unreachable pairs are reported as `usize::MAX`.
fn bfs_all_pairs(graph: &Graph<(), (), Undirected>) -> HashMap<(NodeIndex, NodeIndex), usize> {
    let n = graph.node_count();
    let mut distances = HashMap::new();

    for source in graph.node_indices() {
        let mut depth = vec![usize::MAX; n];
        depth[source.index()] = 0;
        let mut queue = vec![source];
        let mut head = 0;
        while head < queue.len() {
            let node = queue[head];
            head += 1;
            let d = depth[node.index()];
            for neighbor in graph.neighbors(node) {
                if depth[neighbor.index()] == usize::MAX {
                    depth[neighbor.index()] = d + 1;
                    queue.push(neighbor);
                }
            }
        }
        for target in graph.node_indices() {
            distances.insert((source, target), depth[target.index()]);
        }
    }

    distances
}

fn assert_matches_bfs(graph: &Graph<(), (), Undirected>) {
    let expected = bfs_all_pairs(graph);
    let actual = seidel(graph);

    assert_eq!(actual.len(), expected.len());
    for (pair, &distance) in &expected {
        assert_eq!(
            actual[pair], distance,
            "distance between {:?} and {:?}",
            pair.0, pair.1
        );
    }
}

#[test]
fn matches_bfs_on_named_graphs() {
    // Empty graph.
    assert_matches_bfs(&Graph::new_undirected());

    // Path of length 6.
    let mut path: Graph<(), (), Undirected> = Graph::new_undirected();
    let path_nodes: Vec<_> = (0..7).map(|_| path.add_node(())).collect();
    for window in path_nodes.windows(2) {
        path.add_edge(window[0], window[1], ());
    }
    assert_matches_bfs(&path);

    // Balanced binary tree with 15 nodes.
    let mut tree: Graph<(), (), Undirected> = Graph::new_undirected();
    let tree_nodes: Vec<_> = (0..15).map(|_| tree.add_node(())).collect();
    for i in 1..tree_nodes.len() {
        tree.add_edge(tree_nodes[(i - 1) / 2], tree_nodes[i], ());
    }
    assert_matches_bfs(&tree);

    // Grid graph (4 x 5).
    let (rows, cols) = (4, 5);
    let mut grid: Graph<(), (), Undirected> = Graph::new_undirected();
    let grid_nodes: Vec<_> = (0..rows * cols).map(|_| grid.add_node(())).collect();
    for r in 0..rows {
        for c in 0..cols {
            let node = grid_nodes[r * cols + c];
            if c + 1 < cols {
                grid.add_edge(node, grid_nodes[r * cols + c + 1], ());
            }
            if r + 1 < rows {
                grid.add_edge(node, grid_nodes[(r + 1) * cols + c], ());
            }
        }
    }
    assert_matches_bfs(&grid);
}

/// Minimal deterministic pseudo-random generator so the fuzz test is reproducible without pulling
/// in a full RNG. This is a standard xorshift and is more than good enough to shuffle edge choices.
struct XorShift(u64);

impl XorShift {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

#[test]
fn matches_bfs_on_random_graphs() {
    let mut rng = XorShift(0x9E3779B97F4A7C15);

    for &n in &[1usize, 2, 3, 5, 8, 13, 20] {
        // A range of densities, including graphs that are almost certainly disconnected.
        for &threshold in &[10u64, 40, 128, 210] {
            for _ in 0..25 {
                let mut graph: Graph<(), (), Undirected> = Graph::new_undirected();
                let nodes: Vec<_> = (0..n).map(|_| graph.add_node(())).collect();
                for i in 0..n {
                    for j in (i + 1)..n {
                        if rng.next() % 256 < threshold {
                            graph.add_edge(nodes[i], nodes[j], ());
                        }
                    }
                }
                assert_matches_bfs(&graph);
            }
        }
    }
}
