use core::fmt;

use petgraph_core::visit::{GraphRef, IntoNeighbors, VisitMap, Visitable};

/// Return `true` if the graph is bipartite. A graph is bipartite if its nodes can be divided into
/// two disjoint and indepedent sets U and V such that every edge connects U to one in V. This
/// algorithm implements 2-coloring algorithm based on the BFS algorithm.
///
/// Always treats the input graph as if undirected.
pub fn is_bipartite_undirected<G, N, VM>(g: G, start: N) -> bool
where
    G: GraphRef + Visitable<NodeId = N, Map = VM> + IntoNeighbors<NodeId = N>,
    N: Copy + PartialEq + fmt::Debug,
    VM: VisitMap<N>,
{
    let mut red = g.visit_map();
    red.visit(start);
    let mut blue = g.visit_map();

    let mut stack = ::std::collections::VecDeque::new();
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
