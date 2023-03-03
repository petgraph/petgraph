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

/// Given a collection of [`NodeWeight`s](../visit/trait.Data.html#associatedtype.NodeWeight),
/// an output buffer for storing the new graph's [`NodeId`s](../visit/trait.GraphBase.html#associatedtype.NodeId),
/// and a function for determining [`EdgeWeight`s](../visit/trait.Data.html#associatedtype.EdgeWeight),
/// generate the [complete graph](https://en.wikipedia.org/wiki/Complete_graph).
/// # Example
/// ```rust
/// use petgraph::{generators::complete_graph, graph::UnGraph};
/// type G = UnGraph<(), ()>;
/// let mut nodes = [Default::default(); 4];
/// let complete: G = complete_graph(core::iter::repeat(()), &mut nodes, |_, _| ());
///
/// let expected = G::from_edges(&[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]);
///
/// assert_eq!(format!("{:?}", complete), format!("{:?}", expected));
/// ```
pub fn complete_graph<I, N, E, G>(node_weights: I, mut node_ids: N, mut edge_weights: E) -> G
where
    I: IntoIterator<Item = G::NodeWeight>,
    N: AsMut<[G::NodeId]>,
    E: FnMut(G::NodeId, G::NodeId) -> G::EdgeWeight,
    G: Create + GraphProp,
    G::EdgeType: CompleteEdgeCount,
{
    let (node_weights, node_ids) = (node_weights.into_iter(), node_ids.as_mut());
    let mut graph = G::with_capacity(
        node_ids.len(),
        <G as GraphProp>::EdgeType::complete_edge_count(node_ids.len()),
    );
    for (node_id, node_weight) in node_ids.iter_mut().zip(node_weights) {
        *node_id = graph.add_node(node_weight);
    }
    let is_directed = graph.is_directed();
    for (i, &from) in node_ids.iter().enumerate() {
        for &to in node_ids[..if is_directed { i } else { 0 }]
            .iter()
            .chain(&node_ids[i + 1..])
        {
            graph.add_edge(from, to, edge_weights(from, to));
        }
    }
    graph
}
