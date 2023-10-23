mod matrix;

use core::ops::Add;

use num_traits::Zero;
use petgraph_core::{id::LinearGraphId, Graph, GraphStorage};

use crate::shortest_paths::floyd_warshall::matrix::Matrix;

fn eval<S>(graph: &Graph<S>)
where
    S: GraphStorage,
    S::NodeId: LinearGraphId<S>,
    S::EdgeWeight: Zero,
    for<'a> &'a S::EdgeWeight: Add<Output = S::EdgeWeight>,
{
    let mut distance = Matrix::new_from_option(graph);
    let mut previous = Matrix::new_from_default(graph);

    for edge in graph.edges() {
        let (u, v) = edge.endpoints();
        // in a directed graph we would need to assign to both
        // distance[(u, v)] and distance[(v, u)]
        distance.set(u.id(), v.id(), Some(edge.weight()));
        // distance.set(v.id(), u.id(), Some(edge.weight()))

        previous.set(u.id(), v.id(), Some(u.id()))
        // previous.set(v.id(), u.id(), Some(v.id()))
    }

    for node in graph.nodes() {
        distance.set(node.id(), node.id(), Some(S::EdgeWeight::zero()));
        previous.set(node.id(), node.id(), Some(node.id()));
    }

    for k in graph.nodes() {
        let k = k.id();

        for i in graph.nodes() {
            let i = i.id();

            for j in graph.nodes() {
                let j = j.id();

                // TODO: do we need to check the other direction, no, right?
                // https://cs.stackexchange.com/questions/26344/floyd-warshall-algorithm-on-undirected-graph

                let Some(ik) = distance.get(i, k) else {
                    continue;
                };

                let Some(kj) = distance.get(k, j) else {
                    continue;
                };

                let alternative = *ik + *kj;

                if let Some(Some(current)) = distance.get(i, j) {
                    if alternative >= *current {
                        continue;
                    }
                }

                distance.set(i, j, Some(alternative));
                previous.set(i, j, *previous.get(k, j));
            }
        }
    }
}
