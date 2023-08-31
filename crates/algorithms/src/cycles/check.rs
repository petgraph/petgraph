use petgraph_core::deprecated::visit::{
    depth_first_search, DfsEvent, EdgeRef, IntoEdgeReferences, IntoNeighbors, IntoNodeIdentifiers,
    NodeIndexable, Visitable,
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

#[cfg(test)]
mod tests {
    use super::{is_cyclic_directed, is_cyclic_undirected};

    #[test]
    fn self_loop_cyclic_directed() {
        let mut graph = petgraph_graph::Graph::new();

        let a = graph.add_node("A");
        graph.add_edge(a, a, "A → A");

        assert!(is_cyclic_directed(&graph));
    }

    #[test]
    fn self_loop_cyclic_undirected() {
        let mut graph = petgraph_graph::Graph::new_undirected();

        let a = graph.add_node("A");
        graph.add_edge(a, a, "A - A");

        assert!(is_cyclic_undirected(&graph));
    }

    #[test]
    fn minimal_loop_cyclic_directed() {
        let mut graph = petgraph_graph::Graph::new();

        let a = graph.add_node("A");
        let b = graph.add_node("B");
        graph.add_edge(a, b, "A → B");
        graph.add_edge(b, a, "B → A");

        assert!(is_cyclic_directed(&graph));
    }

    #[test]
    fn minimal_loop_cyclic_undirected() {
        let mut graph = petgraph_graph::Graph::new_undirected();

        let a = graph.add_node("A");
        let b = graph.add_node("B");

        graph.add_edge(a, b, "A - B");
        graph.add_edge(b, a, "B - A");

        assert!(is_cyclic_undirected(&graph));
    }

    #[test]
    fn simple_cyclic_directed() {
        let mut graph = petgraph_graph::Graph::new();

        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");

        graph.add_edge(a, b, "A → B");
        graph.add_edge(b, c, "B → C");
        graph.add_edge(c, a, "C → A");

        assert!(is_cyclic_directed(&graph));
    }

    #[test]
    fn simple_cyclic_undirected() {
        let mut graph = petgraph_graph::Graph::new_undirected();

        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");

        graph.add_edge(a, b, "A - B");
        graph.add_edge(b, c, "B - C");
        graph.add_edge(c, a, "C - A");

        assert!(is_cyclic_undirected(&graph));
    }

    #[test]
    fn simple_acyclic_directed() {
        let mut graph = petgraph_graph::Graph::new();

        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");

        graph.add_edge(a, b, "A → B");
        graph.add_edge(b, c, "B → C");

        assert!(!is_cyclic_directed(&graph));
    }

    #[test]
    fn simple_acyclic_directed_blocked_by_direction() {
        let mut graph = petgraph_graph::Graph::new();

        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");

        graph.add_edge(a, b, "A → B");
        graph.add_edge(b, c, "B → C");
        graph.add_edge(a, c, "A → C");

        assert!(!is_cyclic_directed(&graph));
    }

    #[test]
    fn simple_acyclic_undirected() {
        let mut graph = petgraph_graph::Graph::new_undirected();

        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");

        graph.add_edge(a, b, "A - B");
        graph.add_edge(b, c, "B - C");

        assert!(!is_cyclic_undirected(&graph));
    }
}
