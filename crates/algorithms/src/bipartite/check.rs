use alloc::collections::VecDeque;
use core::fmt;

use petgraph_core::deprecated::visit::{GraphRef, IntoNeighbors, VisitMap, Visitable};

/// Return `true` if the graph is bipartite. A graph is bipartite if its nodes can be divided into
/// two disjoint and indepedent sets U and V such that every edge connects U to one in V. This
/// algorithm implements 2-coloring algorithm based on the BFS algorithm.
///
/// Always treats the input graph as if undirected.
// TODO: this algorithm is incomplete, if there are multiple connected components, it will only
//  check the one of the start node.
// TODO: The documentation is also ambitious, it doesn't treat the graph as undirected, it assumes
//  that the input graph is undirected.
pub fn is_bipartite_undirected<G, N, VM>(g: G, start: N) -> bool
where
    G: GraphRef + Visitable<NodeId = N, Map = VM> + IntoNeighbors<NodeId = N>,
    N: Copy + PartialEq + fmt::Debug,
    VM: VisitMap<N>,
{
    let mut red = g.visit_map();
    red.visit(start);
    let mut blue = g.visit_map();

    let mut stack = VecDeque::new();
    stack.push_front(start);

    while let Some(node) = stack.pop_front() {
        let is_red = red.is_visited(&node);
        let is_blue = blue.is_visited(&node);

        assert!(is_red ^ is_blue);

        for neighbour in g.neighbors(node) {
            let is_neigbour_red = red.is_visited(&neighbour);
            let is_neigbour_blue = blue.is_visited(&neighbour);

            if (is_red && is_neigbour_red) || (is_blue && is_neigbour_blue) {
                return false;
            }

            if !is_neigbour_red && !is_neigbour_blue {
                //hasn't been visited yet

                match (is_red, is_blue) {
                    (true, false) => {
                        blue.visit(neighbour);
                    }
                    (false, true) => {
                        red.visit(neighbour);
                    }
                    (..) => {
                        panic!("Invariant doesn't hold");
                    }
                }

                stack.push_back(neighbour);
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use petgraph_core::edge::Undirected;

    use super::is_bipartite_undirected;

    /// Graph:
    ///
    /// ```text
    /// A → B   C
    /// ```
    ///
    /// is bipartite with `U = {A, C}` and `V = {B}`. `C` can be in either set.
    #[test]
    fn disconnected_graph() {
        let mut graph = petgraph_graph::Graph::new_undirected();

        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");

        graph.add_edge(a, b, "A → B");

        assert!(is_bipartite_undirected(&graph, a));
        assert!(is_bipartite_undirected(&graph, b));
        assert!(is_bipartite_undirected(&graph, c));

        // directionality doesn't matter
        let graph = graph.into_edge_type::<Undirected>();

        assert!(is_bipartite_undirected(&graph, a));
        assert!(is_bipartite_undirected(&graph, b));
        assert!(is_bipartite_undirected(&graph, c));
    }

    /// Self-loops are inherently not bipartite.
    #[test]
    fn self_loop_cyclic_undirected() {
        let mut graph = petgraph_graph::Graph::new();

        let a = graph.add_node("A");
        graph.add_edge(a, a, "A → A");

        assert!(!is_bipartite_undirected(&graph, a));

        // directionality doesn't matter
        let graph = graph.into_edge_type::<Undirected>();

        assert!(!is_bipartite_undirected(&graph, a));
    }

    /// Graph:
    ///
    /// ```text
    /// A → B
    /// ```
    ///
    /// is bipartite with `U = {A}` and `V = {B}`.
    #[test]
    fn minimal() {
        let mut graph = petgraph_graph::Graph::new();

        let a = graph.add_node("A");
        let b = graph.add_node("B");
        graph.add_edge(a, b, "A → B");

        assert!(is_bipartite_undirected(&graph, a));
        assert!(is_bipartite_undirected(&graph, b));

        // directionality doesn't matter
        let graph = graph.into_edge_type::<Undirected>();

        assert!(is_bipartite_undirected(&graph, a));
        assert!(is_bipartite_undirected(&graph, b));
    }

    /// Graph:
    ///
    /// ```text
    /// A → B → C
    /// ```
    ///
    /// is bipartite with `U = {A, C}` and `V = {B}`.
    #[test]
    fn small() {
        let mut graph = petgraph_graph::Graph::new();

        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");

        graph.add_edge(a, b, "A → B");
        graph.add_edge(b, c, "B → C");

        assert!(is_bipartite_undirected(&graph, a));
        assert!(is_bipartite_undirected(&graph, b));
        assert!(is_bipartite_undirected(&graph, c));

        // directionality doesn't matter
        let graph = graph.into_edge_type::<Undirected>();

        assert!(is_bipartite_undirected(&graph, a));
        assert!(is_bipartite_undirected(&graph, b));
        assert!(is_bipartite_undirected(&graph, c));
    }

    /// Graph:
    ///
    /// ```text
    /// A → C
    /// ↓ ↗
    /// B
    /// ```
    ///
    /// is not bipartite. (The connection `A → B` or `B → C` will be in either `U` or `V`, which is
    /// invalid.)
    #[test]
    fn small_not_bipartite() {
        let mut graph = petgraph_graph::Graph::new();

        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");

        graph.add_edge(a, b, "A → B");
        graph.add_edge(a, c, "A → C");
        graph.add_edge(b, c, "B → C");

        // TODO: The wording in the documentation is a bit confusing. It says: "Always treats the
        //  input graph as if undirected." This is not correct, it makes the assumption that the
        //  graph is undirected, but will still work with directed graphs. The function signature
        //  should be changed to disallow directed graphs.
        // assert!(!is_bipartite_undirected(&graph, a));
        // assert!(!is_bipartite_undirected(&graph, b));
        // assert!(!is_bipartite_undirected(&graph, c));

        // directionality doesn't matter
        let graph = graph.into_edge_type::<Undirected>();

        assert!(!is_bipartite_undirected(&graph, a));
        assert!(!is_bipartite_undirected(&graph, b));
        assert!(!is_bipartite_undirected(&graph, c));
    }
}
