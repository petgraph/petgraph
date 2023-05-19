use alloc::vec::Vec;

use petgraph_core::visit::{
    Dfs, DfsPostOrder, IntoNeighborsDirected, IntoNodeIdentifiers, Reversed, VisitMap, Visitable,
};

/// \[Generic\] Compute the *strongly connected components* using [Kosaraju's algorithm][1].
///
/// [1]: https://en.wikipedia.org/wiki/Kosaraju%27s_algorithm
///
/// Return a vector where each element is a strongly connected component (scc).
/// The order of node ids within each scc is arbitrary, but the order of
/// the sccs is their postorder (reverse topological sort).
///
/// For an undirected graph, the sccs are simply the connected components.
///
/// This implementation is iterative and does two passes over the nodes.
pub fn kosaraju_scc<G>(g: G) -> Vec<Vec<G::NodeId>>
where
    G: IntoNeighborsDirected + Visitable + IntoNodeIdentifiers,
{
    let mut dfs = DfsPostOrder::empty(g);

    // First phase, reverse dfs pass, compute finishing times.
    // http://stackoverflow.com/a/26780899/161659
    let mut finish_order = Vec::with_capacity(0);
    for i in g.node_identifiers() {
        if dfs.discovered.is_visited(&i) {
            continue;
        }

        dfs.move_to(i);
        while let Some(nx) = dfs.next(Reversed(g)) {
            finish_order.push(nx);
        }
    }

    let mut dfs = Dfs::from_parts(dfs.stack, dfs.discovered);
    dfs.reset(g);
    let mut sccs = Vec::new();

    // Second phase
    // Process in decreasing finishing time order
    for i in finish_order.into_iter().rev() {
        if dfs.discovered.is_visited(&i) {
            continue;
        }
        // Move to the leader node `i`.
        dfs.move_to(i);
        let mut scc = Vec::new();
        while let Some(nx) = dfs.next(g) {
            scc.push(nx);
        }
        sccs.push(scc);
    }
    sccs
}
