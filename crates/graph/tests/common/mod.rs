use petgraph_core::{edge::EdgeType, index::IndexType};
use petgraph_graph::Graph;

/// # Panics
///
/// Panics if the graph is not consistent.
pub fn assert_graph_consistency<N, E, Ty, Ix>(graph: &Graph<N, E, Ty, Ix>)
where
    Ty: EdgeType,
    Ix: IndexType,
{
    assert_eq!(graph.node_count(), graph.node_indices().count());
    assert_eq!(graph.edge_count(), graph.edge_indices().count());

    for edge in graph.raw_edges() {
        assert!(
            graph.find_edge(edge.source(), edge.target()).is_some(),
            "Edge not in graph! {:?} to {:?}",
            edge.source(),
            edge.target()
        );
    }
}
