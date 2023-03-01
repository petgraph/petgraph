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

/// Given a collection of [`NodeWeight`s](../visit/trait.Data.html#associatedtype.NodeWeight)
/// and a function for determining [`EdgeWeight`s](../visit/trait.Data.html#associatedtype.EdgeWeight),
/// generate the [complete graph](https://en.wikipedia.org/wiki/Complete_graph).
/// # Example
/// ```rust
/// use petgraph::{generators::complete_graph, graphmap::DiGraphMap};
/// let mut count = 0;
/// let complete = complete_graph::<DiGraphMap<_, _>>(1..=4, |_, _| {
///     count += 1;
///     count
/// });
///
/// let expected = DiGraphMap::<_, _>::from_edges([
///     (1, 2, 1),
///     (1, 3, 2),
///     (1, 4, 3),
///     (2, 1, 4),
///     (2, 3, 5),
///     (2, 4, 6),
///     (3, 1, 7),
///     (3, 2, 8),
///     (3, 4, 9),
///     (4, 1, 10),
///     (4, 2, 11),
///     (4, 3, 12),
/// ]);
///
/// assert_eq!(format!("{:?}", complete), format!("{:?}", expected));
/// ```
pub fn complete_graph<G>(
    nodes: impl IntoIterator<Item = G::NodeWeight>,
    mut edges: impl FnMut(G::NodeId, G::NodeId) -> G::EdgeWeight,
) -> G
where
    G: Create + GraphProp,
    G::EdgeType: CompleteEdgeCount,
{
    let nodes = nodes.into_iter();
    let node_count = nodes.size_hint().1.unwrap_or_default();
    let mut graph = G::with_capacity(node_count, G::EdgeType::complete_edge_count(node_count));
    let nodes = nodes.map(|node| graph.add_node(node)).collect::<Vec<_>>();
    let is_directed = graph.is_directed();
    for (i, &from) in nodes.iter().enumerate() {
        for &to in nodes[..i]
            .iter()
            .take_while(|_| is_directed)
            .chain(&nodes[i + 1..])
        {
            graph.add_edge(from, to, edges(from, to));
        }
    }
    graph
}
