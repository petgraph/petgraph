use petgraph_core::deprecated::visit::{IntoNeighbors, Visitable, Walker};

use crate::traversal::{with_dfs, DfsSpace};

/// \[Generic\] Check if there exists a path starting at `from` and reaching `to`.
///
/// If `from` and `to` are equal, this function returns true.
///
/// If `space` is not `None`, it is used instead of creating a new workspace for
/// graph traversal.
pub fn has_path_connecting<G>(
    g: G,
    from: G::NodeId,
    to: G::NodeId,
    space: Option<&mut DfsSpace<G::NodeId, G::Map>>,
) -> bool
where
    G: IntoNeighbors + Visitable,
{
    with_dfs(g, space, |dfs| {
        dfs.reset(g);
        dfs.move_to(from);
        dfs.iter(g).any(|x| x == to)
    })
}

#[cfg(test)]
mod tests {
    use petgraph_graph::Graph;

    use crate::traversal::DfsSpace;

    #[test]
    fn single_node() {
        let mut graph = Graph::<_, ()>::new();

        let a = graph.add_node("A");

        assert!(super::has_path_connecting(&graph, a, a, None));
    }

    #[test]
    fn link() {
        let mut graph = Graph::new();

        let a = graph.add_node("A");
        let b = graph.add_node("B");

        assert!(!super::has_path_connecting(&graph, a, b, None));
        assert!(!super::has_path_connecting(&graph, b, a, None));

        graph.add_edge(a, b, "A → B");

        assert!(super::has_path_connecting(&graph, a, b, None));
        assert!(!super::has_path_connecting(&graph, b, a, None));

        graph.add_edge(b, a, "B → A");

        assert!(super::has_path_connecting(&graph, a, b, None));
        assert!(super::has_path_connecting(&graph, b, a, None));
    }

    #[test]
    fn multi_link() {
        let mut graph = Graph::new();

        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");

        assert!(!super::has_path_connecting(&graph, a, b, None));
        assert!(!super::has_path_connecting(&graph, b, c, None));
        assert!(!super::has_path_connecting(&graph, c, a, None));
        assert!(!super::has_path_connecting(&graph, a, c, None));

        graph.add_edge(a, b, "A → B");
        graph.add_edge(b, c, "B → C");
        graph.add_edge(c, a, "C → A");

        assert!(super::has_path_connecting(&graph, a, b, None));
        assert!(super::has_path_connecting(&graph, b, c, None));
        assert!(super::has_path_connecting(&graph, c, a, None));
        assert!(super::has_path_connecting(&graph, a, c, None));
    }

    #[test]
    fn space() {
        let mut graph = Graph::new();

        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");

        graph.add_edge(a, b, "A → B");
        graph.add_edge(b, c, "B → C");

        let mut space = DfsSpace::default();

        assert!(super::has_path_connecting(&graph, a, b, Some(&mut space)));
        assert!(super::has_path_connecting(&graph, b, c, Some(&mut space)));
        assert!(super::has_path_connecting(&graph, a, c, Some(&mut space)));
        assert!(!super::has_path_connecting(&graph, c, a, Some(&mut space)));
    }
}
