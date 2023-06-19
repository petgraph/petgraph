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

#[cfg(test)]
mod tests {
    use alloc::{format, vec, vec::Vec};

    use petgraph_core::{edge::Directed, index::IndexType, visit::Reversed};
    use petgraph_graph::{Graph, NodeIndex};
    use proptest::prelude::*;

    use super::kosaraju_scc;
    use crate::tests::assert_subset_topologically_sorted;

    /// Test that the algorithm works on a graph with a single component.
    ///
    /// ```text
    /// 0 → 1
    ///   ↖ ↓
    ///     2
    /// ```
    #[test]
    fn single_component() {
        let graph: Graph<(), (), Directed, _> = Graph::from_edges([
            (0u32, 1), //
            (1, 2),
            (2, 0),
        ]);

        let scc = kosaraju_scc(&graph);

        assert_eq!(scc.len(), 1);

        assert_eq!(scc, [vec![
            NodeIndex::new(0),
            NodeIndex::new(1),
            NodeIndex::new(2)
        ]]);
    }

    /// Test that the algorithm works on a graph with multiple components.
    ///
    /// ```text
    /// 0 → 1   3
    ///   ↖ ↓ ↗ ↓ ↖
    ///     2   4 → 5
    /// ```
    #[test]
    fn multiple_components() {
        let graph: Graph<(), (), Directed, _> = Graph::from_edges([
            (0u32, 1), //
            (1, 2),
            (2, 0),
            (3, 4),
            (4, 5),
            (5, 3),
        ]);

        let scc = kosaraju_scc(&graph);

        assert_eq!(scc.len(), 2);

        assert_eq!(scc, [
            vec![NodeIndex::new(3), NodeIndex::new(4), NodeIndex::new(5)],
            vec![NodeIndex::new(0), NodeIndex::new(1), NodeIndex::new(2)],
        ]);
    }

    /// Test that even if we reverse the graph, the algorithm still works.
    ///
    ///
    /// Uses the same graph as `single_components`.
    #[test]
    fn reversed_single_components() {
        let graph: Graph<(), (), Directed, _> = Graph::from_edges([
            (0u32, 1), //
            (1, 2),
            (2, 0),
        ]);

        let graph = Reversed(&graph);

        let scc = kosaraju_scc(graph);

        assert_eq!(scc.len(), 1);

        assert_eq!(scc, [vec![
            NodeIndex::new(0),
            NodeIndex::new(2),
            NodeIndex::new(1),
        ]]);
    }

    /// Test that even if we have a disconnected graph, the algorithm still works.
    ///
    /// ```text
    /// 0 → 1   3
    ///   ↖ ↓   ↓ ↖
    ///     2   4 → 5
    /// ```
    #[test]
    fn disconnected() {
        let graph: Graph<(), (), Directed, _> = Graph::from_edges([
            (0u32, 1), //
            (1, 2),
            (2, 0),
            (3, 4),
            (4, 5),
            (5, 3),
        ]);

        let scc = kosaraju_scc(&graph);

        assert_eq!(scc.len(), 2);

        assert_eq!(scc, [
            vec![NodeIndex::new(3), NodeIndex::new(4), NodeIndex::new(5)],
            vec![NodeIndex::new(0), NodeIndex::new(1), NodeIndex::new(2)],
        ]);
    }

    /// Test against the regression discovered in [issue #14].
    ///
    /// [issue #14]: https://github.com/petgraph/petgraph/issues/14
    #[test]
    fn regression_issue_14() {
        let mut graph: Graph<_, ()> = Graph::new();
        let a = graph.add_node(0);
        let b = graph.add_node(1);
        let c = graph.add_node(2);
        let d = graph.add_node(3);

        graph.extend_with_edges([(d, c), (d, b), (c, a), (b, a)]);

        let scc = kosaraju_scc(&graph);

        assert_eq!(scc, [vec![a], vec![b], vec![c], vec![d]]);
    }

    /// Test against the regression discovered in [issue #60].
    ///
    /// [issue #60]: https://github.com/petgraph/petgraph/issues/60
    #[test]
    fn regression_issue_60() {
        let mut graph = Graph::<(), ()>::new();
        graph.extend_with_edges([(0, 0), (1, 0), (2, 0), (2, 1), (2, 2)]);
        graph.add_node(());

        let scc = kosaraju_scc(&graph);

        // even if we extend with edges and add a node, the algorithm should still work.
        assert_eq!(scc, [
            vec![NodeIndex::new(3)],
            vec![NodeIndex::new(0)],
            vec![NodeIndex::new(1)],
            vec![NodeIndex::new(2)],
        ]);
    }

    proptest! {
        #[test]
        fn topologically_sorted(graph in any::<Graph<(), (), Directed, u8>>()) {
            let order = kosaraju_scc(&graph);
            let firsts = order.iter().rev().map(|v| v[0]).collect::<Vec<_>>();

            assert_subset_topologically_sorted(&graph, &firsts);
        }
    }
}
