use petgraph_core::{edge::Undirected, visit::EdgeRef};
use petgraph_graph::{Graph, NodeIndex};
use rand::{prelude::SliceRandom, rngs::StdRng, Rng, SeedableRng};

pub const NODE_COUNTS: &[usize] = &[8, 16, 32, 64, 128, 256, 1024, 2048, 4096, 8192, 16384];

/// Returns a complete graph with $n$ nodes.
///
/// # Arguments
///
/// * `n` - The number of nodes
pub fn complete_graph(n: usize) -> Graph<(), (), Undirected> {
    let mut graph = Graph::new_undirected();
    let nodes: Vec<NodeIndex<_>> = (0..n).into_iter().map(|_| graph.add_node(())).collect();

    for i in 0..n {
        for j in i + 1..n {
            graph.add_edge(nodes[i], nodes[j], ());
        }
    }

    graph
}

/// Returns a Newman-Watts-Strogatz small-world graph.
///
/// First create a ring over $n$ nodes, then each node in the ring is joined to its $k$ nearest
/// neighbors (or $k - 1$ neighbors if $k$ is odd). Then shortcuts are created by adding new edges
/// as follows: for each $(u, v)$ in the underlying "$n$-ring with $k$ nearest neighbors" with
/// probability $p$ add a new edge $(u, w)$ with $w$ chosen uniformly at random from the $n$ nodes.
///
/// # Arguments
///
/// * `n` - The number of nodes
/// * `k` - Each node is joined with its `k` nearest neighbors in a ring topology.
/// * `p` - The probability of adding a new edge for each edge.
/// * `seed` - Seed to use for the random generator, helpful to make output reproducible
///
/// # Panics
///
/// Panics if `k > n`.
pub fn newman_watts_strogatz_graph(
    n: usize,
    k: usize,
    p: f64,
    seed: Option<u64>,
) -> Graph<(), (), Undirected> {
    assert!(k <= n, "Neighbours must be less than nodes");

    let mut rng = match seed {
        Some(seed) => StdRng::seed_from_u64(seed),
        None => StdRng::from_entropy(),
    };

    if k == n {
        return complete_graph(n);
    }

    let mut graph = Graph::new_undirected();
    let nodes: Vec<NodeIndex<_>> = (0..n).into_iter().map(|_| graph.add_node(())).collect();

    let from_nodes = &nodes;

    // connect the k/2 neighbours
    for j in 1..(k / 2 + 1) {
        // rotate the from_nodes by j
        let mut to_nodes = from_nodes.clone();
        to_nodes.rotate_left(j);

        for (from, to) in from_nodes.iter().zip(to_nodes.iter()) {
            graph.add_edge(*from, *to, ());
        }
    }

    // for each edge (u, v), with probability p, randomly select a node w and add the edge (u, w)
    let mut edges: Vec<_> = graph
        .edge_references()
        .map(|e| (e.source(), e.target()))
        .collect();

    for (u, _) in edges {
        if !(rng.gen_bool(p)) {
            continue;
        }

        let mut w = *nodes.choose(&mut rng).expect("No nodes");
        let mut create_edge = true;

        // no self-loops or multi-edges
        while w == u || graph.contains_edge(u, w) {
            w = *nodes.choose(&mut rng).expect("No nodes");

            if graph.edges(u).count() >= n - 1 {
                create_edge = false;
                break;
            }
        }

        if create_edge {
            graph.add_edge(u, w, ());
        }
    }

    graph
}
