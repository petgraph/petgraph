use petgraph_graph::{stable::StableGraph, DiGraph, EdgeIndex, Graph, NodeIndex};
use rand::{
    distributions::{Distribution, Standard},
    rngs::StdRng,
    Rng, SeedableRng,
};

// TODO: potentially move this to a separate crate (test-utils?)
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

pub fn cycle_graph<N, E>(
    n: usize,
    seed: Option<u64>,
) -> (Graph<N, E>, Vec<NodeIndex>, Vec<EdgeIndex>)
where
    Standard: Distribution<N> + Distribution<E>,
{
    let mut rng = rng(seed);
    let mut graph = DiGraph::new();

    let nodes: Vec<_> = (0..n).map(|_| graph.add_node(rng.gen())).collect();

    let edges: Vec<_> = (0..n)
        .map(|index| {
            let start = nodes[index];
            let target = nodes[(index + 1) % n];

            graph.add_edge(start, target, rng.gen())
        })
        .collect();

    // create all required
    (graph, nodes, edges)
}

pub fn cycle_stable_graph<N, E>(
    n: usize,
    seed: Option<u64>,
) -> (StableGraph<N, E>, Vec<NodeIndex>, Vec<EdgeIndex>)
where
    Standard: Distribution<N> + Distribution<E>,
{
    let (graph, nodes, edges) = cycle_graph(n, seed);

    (StableGraph::from(graph), nodes, edges)
}
