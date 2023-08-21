use petgraph_core::{edge::Undirected, visit::EdgeRef};
use petgraph_graph::{DiGraph, Graph, NodeIndex};
use rand::{
    distributions::{Distribution, Standard},
    prelude::SliceRandom,
    rngs::StdRng,
    Rng, SeedableRng,
};

const NODE_COUNTS: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384];

pub fn nodes(max: Option<usize>) -> &'static [usize] {
    let Some(max) = max else {
        return NODE_COUNTS;
    };

    let mut index = 0;

    for &count in NODE_COUNTS {
        if count > max {
            break;
        }

        index += 1;
    }

    &NODE_COUNTS[..index]
}

fn rng(seed: Option<u64>) -> StdRng {
    seed.map_or_else(StdRng::from_entropy, StdRng::seed_from_u64)
}

/// Returns a complete graph with $n$ nodes.
///
/// # Arguments
///
/// * `n` - The number of nodes
/// * `seed` - The seed for the random number generator
pub fn complete_graph<N, E>(n: usize, seed: Option<u64>) -> Graph<N, E, Undirected>
where
    Standard: Distribution<N> + Distribution<E>,
{
    complete_graph_rng(n, &mut rng(seed))
}

fn complete_graph_rng<N, E>(n: usize, rng: &mut impl Rng) -> Graph<N, E, Undirected>
where
    Standard: Distribution<N> + Distribution<E>,
{
    let mut graph = Graph::new_undirected();
    let nodes: Vec<NodeIndex<_>> = (0..n).map(|_| graph.add_node(rng.gen())).collect();

    for i in 0..n {
        for j in i + 1..n {
            graph.add_edge(nodes[i], nodes[j], rng.gen());
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
pub fn newman_watts_strogatz_graph<N, E>(
    n: usize,
    k: usize,
    p: f64,
    seed: Option<u64>,
) -> Graph<N, E, Undirected>
where
    Standard: Distribution<N> + Distribution<E>,
{
    assert!(k <= n, "Neighbours must be less than nodes");

    let mut rng = rng(seed);

    if k == n {
        return complete_graph_rng(n, &mut rng);
    }

    let mut graph = Graph::new_undirected();

    let nodes: Vec<NodeIndex<_>> = (0..n).map(|_| graph.add_node(rng.gen())).collect();

    let from_nodes = &nodes;

    // connect the k/2 neighbours
    for j in 1..=(k / 2) {
        // rotate the from_nodes by j
        let mut to_nodes = from_nodes.clone();
        to_nodes.rotate_left(j);

        for (from, to) in from_nodes.iter().zip(to_nodes.iter()) {
            graph.add_edge(*from, *to, rng.gen());
        }
    }

    // for each edge (u, v), with probability p, randomly select a node w and add the edge (u, w)
    let edges: Vec<_> = graph
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
            graph.add_edge(u, w, rng.gen());
        }
    }

    graph
}

pub fn tournament<N, E>(n: usize, seed: Option<u64>) -> DiGraph<N, E>
where
    Standard: Distribution<N> + Distribution<E>,
{
    let mut rng = rng(seed);

    let mut edge_forward = true;
    let mut graph = DiGraph::new();

    let nodes: Vec<_> = (0..n).map(|_| graph.add_node(rng.gen())).collect();

    for (index, start) in graph.node_indices().enumerate() {
        // This is the same as like `start < end`, because `NodeIndex` is monotonically increasing.
        for target in &nodes[..index] {
            let (source, target) = if edge_forward {
                (start, *target)
            } else {
                (*target, start)
            };

            graph.add_edge(source, target, rng.gen());
            edge_forward = !edge_forward;
        }
    }

    graph
}

/// An `F_(1, n)` graph (where `|E| == 2(|N|) - 1`) with pseudo-random edge directions.
pub fn directed_fan<N, E>(n: usize, seed: Option<u64>) -> DiGraph<N, E>
where
    Standard: Distribution<N> + Distribution<E>,
{
    let mut rng = rng(seed);

    let mut graph = DiGraph::new();

    let nodes: Vec<_> = (0..n).map(|_| graph.add_node(rng.gen())).collect();

    let first = nodes[0];
    let mut previous = None;

    for index in nodes {
        let edge_forward = rng.gen_bool(0.5);

        let (source, target) = if edge_forward {
            (first, index)
        } else {
            (index, first)
        };

        graph.add_edge(source, target, rng.gen());

        if let Some(previous) = previous {
            let (source, target) = if edge_forward {
                (previous, index)
            } else {
                (index, previous)
            };

            graph.add_edge(source, target, rng.gen());
        }

        previous = Some(index);
    }

    graph
}
