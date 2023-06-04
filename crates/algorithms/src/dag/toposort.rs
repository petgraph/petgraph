use alloc::vec::Vec;

use petgraph_core::visit::{
    IntoNeighborsDirected, IntoNodeIdentifiers, Reversed, VisitMap, Visitable,
};

use crate::{
    error::CycleError,
    traversal::{with_dfs, DfsSpace},
};

/// \[Generic\] Perform a topological sort of a directed graph.
///
/// If the graph was acyclic, return a vector of nodes in topological order:
/// each node is ordered before its successors.
/// Otherwise, it will return a `Cycle` error. Self loops are also cycles.
///
/// To handle graphs with cycles, use the scc algorithms or `DfsPostOrder`
/// instead of this function.
///
/// If `space` is not `None`, it is used instead of creating a new workspace for
/// graph traversal. The implementation is iterative.
pub fn toposort<G>(
    g: G,
    space: Option<&mut DfsSpace<G::NodeId, G::Map>>,
) -> Result<Vec<G::NodeId>, CycleError<G::NodeId>>
where
    G: IntoNeighborsDirected + IntoNodeIdentifiers + Visitable,
{
    // based on kosaraju scc
    with_dfs(g, space, |dfs| {
        dfs.reset(g);
        let mut finished = g.visit_map();

        let mut finish_stack = Vec::new();
        for i in g.node_identifiers() {
            if dfs.discovered.is_visited(&i) {
                continue;
            }
            dfs.stack.push(i);
            while let Some(&nx) = dfs.stack.last() {
                if dfs.discovered.visit(nx) {
                    // First time visiting `nx`: Push neighbors, don't pop `nx`
                    for succ in g.neighbors(nx) {
                        if succ == nx {
                            // self cycle
                            return Err(CycleError { node: nx });
                        }
                        if !dfs.discovered.is_visited(&succ) {
                            dfs.stack.push(succ);
                        }
                    }
                } else {
                    dfs.stack.pop();
                    if finished.visit(nx) {
                        // Second time: All reachable nodes must have been finished
                        finish_stack.push(nx);
                    }
                }
            }
        }
        finish_stack.reverse();

        dfs.reset(g);
        for &i in &finish_stack {
            dfs.move_to(i);
            let mut cycle = false;
            while let Some(j) = dfs.next(Reversed(g)) {
                if cycle {
                    return Err(CycleError { node: j });
                }
                cycle = true;
            }
        }

        Ok(finish_stack)
    })
}

#[cfg(test)]
mod tests {
    use petgraph_core::edge::Directed;
    use petgraph_graph::{Graph, NodeIndex};

    // A graph is topologically sorted if for every edge `(u, v)`, `u` comes before `v` in the
    // ordering.
    fn assert_topologically_sorted<N, E>(gr: &Graph<N, E, Directed>, order: &[NodeIndex]) {
        assert_eq!(gr.node_count(), order.len());
        // check all the edges of the graph
        for edge in gr.raw_edges() {
            let source = edge.source();
            let target = edge.target();

            let source_index = order
                .iter()
                .position(|x| *x == source)
                .expect("Source node not found");

            let target_index = order
                .iter()
                .position(|x| *x == target)
                .expect("Target node not found");

            assert!(
                source_index < target_index,
                "Graph is not topologically sorted ({target} comes before {source})",
            );
        }
    }

    /// This uses the example from the Wikipedia page on topological sorting:
    /// <https://en.wikipedia.org/wiki/Topological_sorting#Examples>
    ///
    /// Node to name mapping:
    /// * 2: "A"
    /// * 3: "B"
    /// * 5: "C"
    /// * 7: "D"
    /// * 8: "E"
    /// * 9: "F"
    /// * 10: "G"
    /// * 11: "H"
    fn setup() -> Graph<&'static str, &'static str> {
        let mut graph = Graph::new();

        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");
        let d = graph.add_node("D");
        let e = graph.add_node("E");
        let f = graph.add_node("F");
        let g = graph.add_node("G");
        let h = graph.add_node("H");

        graph.extend_with_edges([
            (b, e, "B → E"), //
            (b, g, "B → G"),
            (c, h, "C → H"),
            (d, e, "D → E"),
            (d, h, "D → H"),
            (e, f, "E → F"),
            (h, a, "H → A"),
            (h, f, "H → F"),
            (h, g, "H → G"),
        ]);

        graph
    }

    #[test]
    fn example() {
        let graph = setup();

        let order = super::toposort(&graph, None).expect("graph should be acyclic");
        assert_topologically_sorted(&graph, &order);
    }

    #[test]
    fn disjoint() {
        let mut graph = Graph::new();

        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");
        let d = graph.add_node("D");

        graph.add_edge(a, b, "A → B");
        graph.add_edge(c, d, "C → D");

        let order = super::toposort(&graph, None).expect("graph should be acyclic");

        assert_topologically_sorted(&graph, &order);
    }

    #[test]
    fn path() {
        let mut graph = Graph::new();

        let a = graph.add_node("A");
        let b = graph.add_node("B");

        graph.add_edge(a, b, "A → B");

        let order = super::toposort(&graph, None).expect("graph should be acyclic");

        assert_eq!(order, vec![a, b]);
    }

    #[test]
    fn error_on_cycle() {
        let mut graph = Graph::new();

        let a = graph.add_node("A");
        let b = graph.add_node("B");

        graph.add_edge(a, b, "A → B");
        graph.add_edge(b, a, "B → A");

        let order = super::toposort(&graph, None);
        assert!(order.is_err());
    }
}
