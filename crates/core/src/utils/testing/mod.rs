pub mod directed;
pub mod undirected;

/// Helper function to check if the edges returned by an iterator match the expected edges.
///
/// The additional arguments are just for better error messages.
#[doc(hidden)]
pub fn check_if_edges_match<T>(
    expected_edges: impl IntoIterator<Item = T>,
    actual_edges: impl Iterator<Item = T>,
    method_name: &'static str,
    graph_type: &'static str,
    context: impl core::fmt::Display,
) where
    T: core::hash::Hash + Eq + core::fmt::Debug,
{
    let mut expected_edges: hashbrown::HashSet<T, foldhash::fast::RandomState> =
        expected_edges.into_iter().collect();

    for edge in actual_edges {
        assert!(
            expected_edges.remove(&edge),
            "{}::{}() contained unexpected edge id: {:?} for {}",
            graph_type,
            method_name,
            edge,
            context
        );
    }

    assert!(
        expected_edges.is_empty(),
        "{}::{}() did not return all expected edges for {}: {:?}",
        graph_type,
        method_name,
        context,
        expected_edges
    );
}

/// Helper function to check if the nodes returned by an iterator match the expected nodes.
///
/// The additional arguments are just for better error messages.
#[doc(hidden)]
pub fn check_if_nodes_match<T>(
    expected_nodes: impl IntoIterator<Item = T>,
    actual_nodes: impl Iterator<Item = T>,
    method_name: &'static str,
    graph_type: &'static str,
    context: impl core::fmt::Debug,
) where
    T: core::hash::Hash + Eq + core::fmt::Debug,
{
    let mut expected_nodes: hashbrown::HashSet<T, foldhash::fast::RandomState> =
        expected_nodes.into_iter().collect();

    for node in actual_nodes {
        assert!(
            expected_nodes.remove(&node),
            "{}::{}() contained unexpected node id: {:?} for {:?}",
            graph_type,
            method_name,
            node,
            context
        );
    }

    assert!(
        expected_nodes.is_empty(),
        "{}::{}() did not return all expected nodes for {:?}: {:?}",
        graph_type,
        method_name,
        context,
        expected_nodes
    );
}
