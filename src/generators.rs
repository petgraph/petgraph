//! Functions for generating graphs of various kinds.
use crate::{data::Create, visit::GraphProp, Directed, Undirected};

/// A trait for determining the number of edges in a
/// [complete graph](https://en.wikipedia.org/wiki/Complete_graph).
pub trait CompleteEdgeCount {
    /// Return the number of edges contained in a complete graph with `node_count` nodes.
    fn complete_edge_count(node_count: usize) -> usize;
}

// Saturating multiplication is acceptable here because the edge count is used as a hint
// for sizing the graph. If the edge count saturates a usize type, it's likely that
// adding so many edges to the graph will run out memory anyway.
impl CompleteEdgeCount for Directed {
    // A complete directed graph with n nodes has n * (n - 1) edges
    fn complete_edge_count(node_count: usize) -> usize {
        node_count.saturating_mul(node_count.saturating_sub(1))
    }
}

impl CompleteEdgeCount for Undirected {
    // A complete undirected graph with n nodes has n * (n - 1) / 2 edges
    // This function is crafted to avoid overflow during the multiplication
    // by performing the division first.
    fn complete_edge_count(node_count: usize) -> usize {
        // Either node_count or (node_count - 1) will be even.
        // Divide the even number by 2.
        if node_count % 2 == 0 {
            (node_count / 2).saturating_mul(node_count.saturating_sub(1))
        } else {
            node_count.saturating_mul(node_count.saturating_sub(1) / 2)
        }
    }
}

/// Given an output buffer for storing the new graph's [`NodeId`s](../visit/trait.GraphBase.html#associatedtype.NodeId),
/// a collection of [`NodeWeight`s](../visit/trait.Data.html#associatedtype.NodeWeight),
/// and a function for determining [`EdgeWeight`s](../visit/trait.Data.html#associatedtype.EdgeWeight),
/// generate the [complete graph](https://en.wikipedia.org/wiki/Complete_graph).
/// # Example
/// ```rust
/// use petgraph::{generators::complete_graph, graph::UnGraph};
/// type G = UnGraph<(), ()>;
/// let complete: G = complete_graph(&mut [Default::default(); 4], core::iter::repeat(()), |_, _| ());
///
/// let expected = G::from_edges(&[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]);
///
/// assert_eq!(format!("{:?}", complete), format!("{:?}", expected));
/// ```
pub fn complete_graph<N, I, E, G>(node_ids: &mut N, node_weights: I, mut edge_weights: E) -> G
where
    for<'a> &'a mut N: IntoIterator<Item = &'a mut G::NodeId>,
    for<'a> &'a N: IntoIterator<Item = &'a G::NodeId>,
    I: IntoIterator<Item = G::NodeWeight>,
    E: FnMut(G::NodeId, G::NodeId) -> G::EdgeWeight,
    G: Create + GraphProp,
    G::EdgeType: CompleteEdgeCount,
{
    let nodes = node_ids.into_iter().zip(node_weights.into_iter());
    let node_count = nodes.size_hint().1.unwrap_or(core::usize::MAX);
    let mut graph = G::with_capacity(node_count, G::EdgeType::complete_edge_count(node_count));
    nodes.for_each(|(node_id, node_weight)| *node_id = graph.add_node(node_weight));
    let (node_ids, node_count, is_directed): (&N, _, _) =
        (node_ids, graph.node_count(), graph.is_directed());
    let node_ids = || node_ids.into_iter().take(node_count);
    for (i, &from) in node_ids().enumerate() {
        for &to in node_ids()
            .take(if is_directed { i } else { 0 })
            .chain(node_ids().skip(i + 1))
        {
            graph.add_edge(from, to, edge_weights(from, to));
        }
    }
    graph
}
