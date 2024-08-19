use crate::{graph::IndexType, Graph, Undirected};

/// Generates a random undirected graph with given `order` and with `p` probability
/// of each possible edge existing.
pub fn random_undirected_graph<Ix: IndexType>(
    order: usize,
    p: f64,
) -> Graph<(), (), Undirected, Ix> {
    let mut graph = Graph::with_capacity(order, 0);

    let mut nodes = vec![];
    for _ in 0..order {
        let node = graph.add_node(());
        nodes.push(node);
    }

    for u in 0..order {
        for v in (u + 1)..order {
            if rand::random::<f64>() < p {
                graph.add_edge(nodes[u], nodes[v], ());
            }
        }
    }

    graph
}
