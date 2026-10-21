use petgraph_core::graph::EditableGraph;

/// Adds `count` nodes to `graph` and returns their ids.
///
/// # Panics
/// Panics if any node could not be added.
pub(crate) fn add_nodes<G>(graph: &mut G, count: usize) -> Vec<G::NodeId>
where
    G: EditableGraph<OwnedNodeData = ()>,
{
    (0..count)
        .map(|_| graph.add_node(()).ok().expect("failed to add node"))
        .collect()
}

/// Adds an edge from `source` to `target` with the given `data` to `graph`.
///
/// # Panics
/// Panics if the edge could not be added.
pub(crate) fn add_edge<G>(
    graph: &mut G,
    source: G::NodeId,
    target: G::NodeId,
    data: G::OwnedEdgeData,
) where
    G: EditableGraph,
{
    graph
        .add_edge(source, target, data)
        .ok()
        .expect("failed to add edge");
}
