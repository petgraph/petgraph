//! Functions for generating graphs of various kinds.
use crate::{
    data::Create,
    visit::{GraphProp, NodeIndexable},
    Directed, Undirected,
};

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
/// and a function for determining [`EdgeWeight`s](../visit/trait.Data.html#associatedtype.EdgeWeight),
/// generate the [complete graph](https://en.wikipedia.org/wiki/Complete_graph).
/// # Example
/// ```rust
/// use petgraph::{generators::CompleteGraph, graph::UnGraph};
/// type G = UnGraph<(), ()>;
/// let complete = G::complete_graph(core::iter::repeat(()).take(4), |_, _| ());
///
/// let expected = G::from_edges(&[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]);
///
/// assert_eq!(format!("{:?}", complete), format!("{:?}", expected));
/// ```
pub trait CompleteGraph: Create + GraphProp {
    fn complete_graph<I, F>(node_weights: I, mut edge_weights: F) -> Self
    where
        Self::EdgeType: CompleteEdgeCount,
        I: IntoIterator<Item = Self::NodeWeight>,
        F: FnMut(Self::NodeId, Self::NodeId) -> Self::EdgeWeight,
    {
        let node_weights = node_weights.into_iter();
        let node_count = node_weights.size_hint().1.unwrap_or(core::usize::MAX);
        let mut graph =
            Self::with_capacity(node_count, Self::EdgeType::complete_edge_count(node_count));
        let node_ids = node_weights
            .map(|node_weight| graph.add_node(node_weight))
            .collect::<Vec<_>>();
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
}

pub(crate) fn complete_graph_indexable<G, I, F>(node_weights: I, mut edge_weights: F) -> G
where
    G: Create + GraphProp + NodeIndexable,
    G::EdgeType: CompleteEdgeCount,
    I: IntoIterator<Item = G::NodeWeight>,
    F: FnMut(G::NodeId, G::NodeId) -> G::EdgeWeight,
{
    let node_weights = node_weights.into_iter();
    let node_count = node_weights.size_hint().1.unwrap_or(core::usize::MAX);
    let mut graph = G::with_capacity(node_count, G::EdgeType::complete_edge_count(node_count));
    for node_weight in node_weights {
        graph.add_node(node_weight);
    }
    for from in 0..graph.node_count() {
        for to in
            (0..if graph.is_directed() { from } else { 0 }).chain(from + 1..graph.node_count())
        {
            let (from, to) = (graph.from_index(from), graph.from_index(to));
            graph.add_edge(from, to, edge_weights(from, to));
        }
    }
    graph
}
