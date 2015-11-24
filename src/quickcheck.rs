extern crate quickcheck;

use self::quickcheck::{Gen, Arbitrary};

use {
    Graph,
    EdgeType,
};
use graph::{
    IndexType,
};

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

