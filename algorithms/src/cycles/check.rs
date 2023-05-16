use petgraph_core::visit::{
    depth_first_search, IntoEdgeReferences, IntoNeighbors, IntoNodeIdentifiers, NodeIndexable,
    Visitable,
};

use crate::utilities::union_find::UnionFind;

/// \[Generic\] Return `true` if the input graph contains a cycle.
///
/// Always treats the input graph as if undirected.
pub fn is_cyclic_undirected<G>(g: G) -> bool
where
    G: NodeIndexable + IntoEdgeReferences,
{
    let mut edge_sets = UnionFind::new(g.node_bound());
    for edge in g.edge_references() {
        let (a, b) = (edge.source(), edge.target());

        // union the two vertices of the edge
        //  -- if they were already the same, then we have a cycle
        if !edge_sets.union(g.to_index(a), g.to_index(b)) {
            return true;
        }
    }
    false
}

/// \[Generic\] Return `true` if the input directed graph contains a cycle.
///
/// This implementation is recursive; use `toposort` if an alternative is
/// needed.
pub fn is_cyclic_directed<G>(g: G) -> bool
where
    G: IntoNodeIdentifiers + IntoNeighbors + Visitable,
{
    depth_first_search(g, g.node_identifiers(), |event| match event {
        DfsEvent::BackEdge(..) => Err(()),
        _ => Ok(()),
    })
    .is_err()
}
