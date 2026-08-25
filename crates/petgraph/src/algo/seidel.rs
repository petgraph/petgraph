use alloc::{collections::VecDeque, vec, vec::Vec};
use core::hash::Hash;

use hashbrown::HashMap;

use crate::visit::{EdgeRef, IntoEdgeReferences, NodeCompactIndexable};

/// Distance returned for a pair of nodes that are not connected by any path.
const UNREACHABLE: usize = usize::MAX;

/// [Seidel's algorithm](https://en.wikipedia.org/wiki/Seidel%27s_algorithm) for the all-pairs
/// shortest path problem on an **unweighted, undirected** graph.
///
/// Every edge is treated as undirected and of unit length; edge weights, self-loops and parallel
/// edges are ignored. The algorithm works by repeatedly squaring the adjacency matrix, which lets
/// it compute all shortest path *lengths* in **O(|V|³ log |V|)** time using integer matrix
/// multiplication. For dense graphs this is often noticeably faster than running a breadth-first
/// search from every node, and it avoids the negative-cycle bookkeeping of
/// [`floyd_warshall`](fn@crate::algo::floyd_warshall).
///
/// The graph does not need to be connected: pairs of nodes that lie in different connected
/// components are reported with a distance of [`usize::MAX`].
///
/// # Arguments
/// * `graph`: an unweighted graph, interpreted as undirected.
///
/// # Returns
/// A [`struct@hashbrown::HashMap`] that maps every ordered pair of nodes `(u, v)` to the number of
/// edges on a shortest path between them. The distance from a node to itself is `0`, and the
/// distance between two nodes in different components is [`usize::MAX`].
///
/// # Complexity
/// * Time complexity: **O(|V|³ log |V|)**.
/// * Auxiliary space: **O(|V|²)**.
///
/// where **|V|** is the number of nodes.
///
/// # Examples
/// ```rust
/// use petgraph::{algo::seidel, prelude::*};
///
/// let mut graph: UnGraph<(), ()> = UnGraph::new_undirected();
/// let a = graph.add_node(());
/// let b = graph.add_node(());
/// let c = graph.add_node(());
/// let d = graph.add_node(());
///
/// // a -- b -- c -- d
/// graph.extend_with_edges([(a, b), (b, c), (c, d)]);
///
/// let distances = seidel(&graph);
///
/// assert_eq!(distances[&(a, a)], 0);
/// assert_eq!(distances[&(a, b)], 1);
/// assert_eq!(distances[&(a, d)], 3);
/// // Distances are symmetric.
/// assert_eq!(distances[&(d, a)], 3);
/// ```
pub fn seidel<G>(graph: G) -> HashMap<(G::NodeId, G::NodeId), usize>
where
    G: NodeCompactIndexable + IntoEdgeReferences,
    G::NodeId: Eq + Hash,
{
    let n = graph.node_bound();

    // Undirected adjacency of the whole graph, keyed by the compact node indices. Self-loops are
    // dropped, and parallel edges are harmless: the per-component matrix below is boolean, so a
    // repeated neighbor simply sets the same entry to one again.
    let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); n];
    for edge in graph.edge_references() {
        let u = graph.to_index(edge.source());
        let v = graph.to_index(edge.target());
        if u != v {
            adjacency[u].push(v);
            adjacency[v].push(u);
        }
    }

    let mut distances = HashMap::with_capacity(n * n);
    // Default every pair to unreachable; connected pairs are overwritten below.
    for i in 0..n {
        let from = graph.from_index(i);
        for j in 0..n {
            distances.insert((from, graph.from_index(j)), UNREACHABLE);
        }
    }

    // Seidel's algorithm assumes a connected graph, so run it once per connected component. Each
    // component is relabeled to a contiguous `0..k` range, solved on its own dense matrix, and the
    // resulting distances are mapped back to the original node ids.
    let mut component = vec![usize::MAX; n];
    let mut members = Vec::new();
    let mut queue = VecDeque::new();

    for start in 0..n {
        if component[start] != usize::MAX {
            continue;
        }

        members.clear();
        component[start] = members.len();
        members.push(start);
        queue.push_back(start);
        while let Some(node) = queue.pop_front() {
            for &next in &adjacency[node] {
                if component[next] == usize::MAX {
                    component[next] = members.len();
                    members.push(next);
                    queue.push_back(next);
                }
            }
        }

        let k = members.len();
        if k == 1 {
            let node = graph.from_index(members[0]);
            distances.insert((node, node), 0);
            continue;
        }

        // Local adjacency matrix for this component.
        let mut matrix = vec![vec![0u64; k]; k];
        for (local, &global) in members.iter().enumerate() {
            for &neighbor in &adjacency[global] {
                matrix[local][component[neighbor]] = 1;
            }
        }

        let component_distances = all_pairs_distances(&matrix);
        for (local_i, &global_i) in members.iter().enumerate() {
            let from = graph.from_index(global_i);
            for (local_j, &global_j) in members.iter().enumerate() {
                let to = graph.from_index(global_j);
                distances.insert((from, to), component_distances[local_i][local_j]);
            }
        }
    }

    distances
}

/// Recursive core of Seidel's algorithm on the adjacency matrix of a single **connected**
/// component.
///
/// `adjacency` must be a symmetric `k x k` matrix of zeros and ones with a zero diagonal. The
/// returned matrix holds the shortest path length (number of edges) between every pair of nodes;
/// because the component is connected every entry is finite.
fn all_pairs_distances(adjacency: &[Vec<u64>]) -> Vec<Vec<usize>> {
    let k = adjacency.len();

    // Z = A * A. `z[i][j]` counts the walks of length two from `i` to `j`; combined with the direct
    // edges in `A` it tells us which pairs are within distance two of each other.
    let z = multiply(adjacency, adjacency);

    // B connects any two distinct nodes that are at distance one or two in the current graph.
    let mut b = vec![vec![0u64; k]; k];
    let mut base_case = true;
    for i in 0..k {
        for j in 0..k {
            if i != j && (adjacency[i][j] == 1 || z[i][j] > 0) {
                b[i][j] = 1;
            } else if i != j {
                base_case = false;
            }
        }
    }

    // Base case: every pair of distinct nodes is already within distance two, so the shortest path
    // is `1` for neighbors and `2` for everyone else.
    if base_case {
        let mut distances = vec![vec![0usize; k]; k];
        for i in 0..k {
            for j in 0..k {
                if i != j {
                    distances[i][j] = if adjacency[i][j] == 1 { 1 } else { 2 };
                }
            }
        }
        return distances;
    }

    // Recurse on the "squared" graph B, whose diameter is half of the current one, then lift the
    // distances back using the classic Seidel correction.
    let half = all_pairs_distances(&b);

    // `degree[j]` is the number of neighbors of `j` in the original component.
    let mut degree = vec![0u64; k];
    for (j, deg) in degree.iter_mut().enumerate() {
        *deg = adjacency.iter().map(|row| row[j]).sum();
    }

    // X = D(B) * A, where D(B) is the recursively computed distance matrix of B.
    let half_u64: Vec<Vec<u64>> = half
        .iter()
        .map(|row| row.iter().map(|&d| d as u64).collect())
        .collect();
    let x = multiply(&half_u64, adjacency);

    let mut distances = vec![vec![0usize; k]; k];
    for i in 0..k {
        for j in 0..k {
            if i == j {
                continue;
            }
            // A shortest path in G has length 2*d(B) or 2*d(B) - 1; the sum over neighbors decides
            // which, exactly as in Seidel's original derivation.
            distances[i][j] = if x[i][j] >= half[i][j] as u64 * degree[j] {
                2 * half[i][j]
            } else {
                2 * half[i][j] - 1
            };
        }
    }

    distances
}

/// Standard cubic multiplication of two `k x k` matrices over `u64`.
fn multiply(a: &[Vec<u64>], b: &[Vec<u64>]) -> Vec<Vec<u64>> {
    let k = a.len();
    let mut product = vec![vec![0u64; k]; k];
    for i in 0..k {
        for l in 0..k {
            let a_il = a[i][l];
            if a_il == 0 {
                continue;
            }
            let row = &b[l];
            let out = &mut product[i];
            for j in 0..k {
                out[j] += a_il * row[j];
            }
        }
    }
    product
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::{UNREACHABLE, seidel};
    use crate::graph::{NodeIndex, UnGraph};

    fn distance(
        distances: &hashbrown::HashMap<(NodeIndex, NodeIndex), usize>,
        u: NodeIndex,
        v: NodeIndex,
    ) -> usize {
        distances[&(u, v)]
    }

    #[test]
    fn single_node() {
        let mut graph: UnGraph<(), ()> = UnGraph::new_undirected();
        let a = graph.add_node(());
        let distances = seidel(&graph);
        assert_eq!(distance(&distances, a, a), 0);
    }

    #[test]
    fn path_graph() {
        let mut graph: UnGraph<(), ()> = UnGraph::new_undirected();
        let nodes: Vec<_> = (0..5).map(|_| graph.add_node(())).collect();
        for window in nodes.windows(2) {
            graph.add_edge(window[0], window[1], ());
        }

        let distances = seidel(&graph);
        for (i, &u) in nodes.iter().enumerate() {
            for (j, &v) in nodes.iter().enumerate() {
                let expected = i.abs_diff(j);
                assert_eq!(distance(&distances, u, v), expected, "d({i}, {j})");
            }
        }
    }

    #[test]
    fn cycle_graph() {
        let mut graph: UnGraph<(), ()> = UnGraph::new_undirected();
        let nodes: Vec<_> = (0..6).map(|_| graph.add_node(())).collect();
        let len = nodes.len();
        for i in 0..len {
            graph.add_edge(nodes[i], nodes[(i + 1) % len], ());
        }

        let distances = seidel(&graph);
        for (i, &u) in nodes.iter().enumerate() {
            for (j, &v) in nodes.iter().enumerate() {
                let diff = i.abs_diff(j);
                let expected = diff.min(len - diff);
                assert_eq!(distance(&distances, u, v), expected, "d({i}, {j})");
            }
        }
    }

    #[test]
    fn complete_graph() {
        let mut graph: UnGraph<(), ()> = UnGraph::new_undirected();
        let nodes: Vec<_> = (0..5).map(|_| graph.add_node(())).collect();
        for (i, &u) in nodes.iter().enumerate() {
            for &v in &nodes[i + 1..] {
                graph.add_edge(u, v, ());
            }
        }

        let distances = seidel(&graph);
        for &u in &nodes {
            for &v in &nodes {
                let expected = if u == v { 0 } else { 1 };
                assert_eq!(distance(&distances, u, v), expected);
            }
        }
    }

    #[test]
    fn star_graph() {
        let mut graph: UnGraph<(), ()> = UnGraph::new_undirected();
        let center = graph.add_node(());
        let leaves: Vec<_> = (0..4).map(|_| graph.add_node(())).collect();
        for &leaf in &leaves {
            graph.add_edge(center, leaf, ());
        }

        let distances = seidel(&graph);
        for &leaf in &leaves {
            assert_eq!(distance(&distances, center, leaf), 1);
        }
        for &a in &leaves {
            for &b in &leaves {
                let expected = if a == b { 0 } else { 2 };
                assert_eq!(distance(&distances, a, b), expected);
            }
        }
    }

    #[test]
    fn disconnected_components() {
        let mut graph: UnGraph<(), ()> = UnGraph::new_undirected();
        let a = graph.add_node(());
        let b = graph.add_node(());
        let c = graph.add_node(());
        let d = graph.add_node(());
        // Two separate edges: {a, b} and {c, d}.
        graph.add_edge(a, b, ());
        graph.add_edge(c, d, ());

        let distances = seidel(&graph);
        assert_eq!(distance(&distances, a, b), 1);
        assert_eq!(distance(&distances, c, d), 1);
        assert_eq!(distance(&distances, a, c), UNREACHABLE);
        assert_eq!(distance(&distances, b, d), UNREACHABLE);
        assert_eq!(distance(&distances, d, a), UNREACHABLE);
    }

    #[test]
    fn ignores_self_loops_and_parallel_edges() {
        let mut graph: UnGraph<(), ()> = UnGraph::new_undirected();
        let a = graph.add_node(());
        let b = graph.add_node(());
        let c = graph.add_node(());
        graph.add_edge(a, a, ());
        graph.add_edge(a, b, ());
        graph.add_edge(a, b, ());
        graph.add_edge(b, c, ());

        let distances = seidel(&graph);
        assert_eq!(distance(&distances, a, a), 0);
        assert_eq!(distance(&distances, a, b), 1);
        assert_eq!(distance(&distances, a, c), 2);
    }
}
