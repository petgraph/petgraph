extern crate quickcheck;

use self::quickcheck::{Gen, Arbitrary};

use {
    Graph,
    EdgeType,
};
use graph::{
    IndexType,
};

use graphmap::{
    GraphMap,
    NodeTrait,
};

/// `Arbitrary` for `Graph` creates a graph by selecting a node count
/// and a probability for each possible edge to exist.
///
/// The result will be simple graph or digraph, with possible
/// self loops, no parallel edges.
impl<N, E, Ty, Ix> Arbitrary for Graph<N, E, Ty, Ix>
    where N: Arbitrary,
          E: Arbitrary,
          Ty: EdgeType + Send + 'static,
          Ix: IndexType + Send,
{
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let nodes = usize::arbitrary(g);
        if nodes == 0 {
            return Graph::with_capacity(0, 0);
        }
        // use X² for edge probability (bias towards lower)
        let edge_prob = g.gen_range(0., 1.) * g.gen_range(0., 1.);
        let edges = ((nodes as f64).powi(2) * edge_prob) as usize;
        let mut gr = Graph::with_capacity(nodes, edges);
        for _ in 0..nodes {
            gr.add_node(N::arbitrary(g));
        }
        for i in gr.node_indices() {
            for j in gr.node_indices() {
                if !gr.is_directed() && i > j {
                    continue;
                }
                let p: f64 = g.gen();
                if p <= edge_prob {
                    gr.add_edge(i, j, E::arbitrary(g));
                }
            }
        }
        gr
    }
}

/// `Arbitrary` for `GraphMap` creates a graph by selecting a node count
/// and a probability for each possible edge to exist.
///
/// The result will be simple graph, selfloops possible.
impl<N, E> Arbitrary for GraphMap<N, E>
    where N: NodeTrait + Arbitrary,
          E: Arbitrary,
{
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let nodes = usize::arbitrary(g);
        if nodes == 0 {
            return GraphMap::with_capacity(0, 0);
        }
        let mut nodes = (0..nodes).map(|_| N::arbitrary(g)).collect::<Vec<_>>();
        nodes.sort();
        nodes.dedup();

        // use X² for edge probability (bias towards lower)
        let edge_prob = g.gen_range(0., 1.) * g.gen_range(0., 1.);
        let edges = ((nodes.len() as f64).powi(2) * edge_prob) as usize;
        let mut gr = GraphMap::with_capacity(nodes.len(), edges);
        for &node in &nodes {
            gr.add_node(node);
        }
        for (index, &i) in nodes.iter().enumerate() {
            for &j in &nodes[index..] {
                let p: f64 = g.gen();
                if p <= edge_prob {
                    gr.add_edge(i, j, E::arbitrary(g));
                }
            }
        }
        gr
    }
}
