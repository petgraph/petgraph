use std::{
    cmp::min,
    collections::VecDeque,
    ops::{Add, Index, Sub},
};

use graph_impl::Edge;

use crate::{
    graph::{EdgeWeightsMut, IndexType},
    graph_impl,
    visit::{IntoEdges, NodeCount, NodeIndexable, NodeRef, Visitable},
};

use super::{EdgeRef, FloatMeasure};

fn has_augmented_path<N>(
    network: N,
    source: N::NodeId,
    destination: N::NodeId,
    edge_to: &mut [Option<N::NodeId>],
    capacities: &Vec<N::EdgeWeight>,
) -> bool
where
    N: NodeCount + IntoEdges + NodeIndexable, // + Visitable,
    N::EdgeWeight: Sub<Output = N::EdgeWeight> + FloatMeasure,
    // <N::EdgeWeight as Sub>::Output: PartialOrd<N::EdgeWeight>,
{
    let mut marked = vec![false; network.node_count()];
    let mut queue = VecDeque::new();
    // let nodeix = |i| network.from_index(i);

    marked[network.to_index(source)] = true;
    queue.push_back(source);

    while let Some(vertex) = queue.pop_front() {
        let mut edges = network.edges(vertex);
        while let Some(edge) = edges.next() {
            let next = edge.target();
            let index_next = network.to_index(next);
            let residual_capacity = capacities[index_next] - *edge.weight();
            if !marked[index_next] && (residual_capacity > N::EdgeWeight::zero()) {
                marked[index_next] = true;
                edge_to[index_next] = Some(vertex);
                if next == destination {
                    return true;
                }
                queue.push_back(next);
            }
        }
    }
    false
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Graph;

    #[test]
    fn test_has_augmented_path() {
        // Example from CLRS book
        let mut graph = Graph::<usize, f32>::new();
        let a = graph.add_node(0);
        let b = graph.add_node(1);
        let c = graph.add_node(2);
        let d = graph.add_node(3);
        let e = graph.add_node(4);
        let f = graph.add_node(5);
        graph.extend_with_edges(&[
            (0, 1),
            (0, 2),
            (1, 3),
            (2, 1),
            (2, 4),
            (3, 2),
            (3, 5),
            (4, 3),
            (4, 5),
        ]);
        let capacities: Vec<f32> = vec![16., 13., 12., 4., 14., 9., 20., 7., 4.];

        let mut edge_to = vec![None; 13];
        let flag = has_augmented_path(&graph, a, f, &mut edge_to, &capacities);
        assert!(flag);
        // println!("{:?}", flag);
    }
}
