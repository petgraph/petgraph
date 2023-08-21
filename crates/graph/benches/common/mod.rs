use petgraph_graph::{stable::StableGraph, DiGraph, EdgeIndex, Graph, NodeIndex};
pub use petgraph_test_utils::*;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

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
