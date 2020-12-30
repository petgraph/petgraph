use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

use crate::{
    graph::NodeIndex,
    stable_graph::StableGraph,
    visit::{EdgeRef, IntoEdgeReferences},
    Direction,
};

/// Finds a set of edges which can be removed to make the specified directe graph acyclic
/// ([feedback arc set]). Uses a [greedy heuristic algorithm] to select a small number of edges in
/// reasonable (linear) time, but does not necessarily find the minimum feedback arc set.
///
/// Does not consider edge weights when selecting edges for the feedback arc set.
///
/// Loops are included in the returned set.
///
/// [feedback arc set]: https://en.wikipedia.org/wiki/Feedback_arc_set
/// [greedy heuristic algorithm]: https://doi.org/10.1016/0020-0190(93)90079-O
// TODO: docs about more efficient alternative for undirected graphs
// And/or add trait requirement `GraphProp<EdgeType=Directed>`
pub fn greedy_feedback_arc_set<G>(g: G) -> impl Iterator<Item = G::EdgeRef>
where
    G: IntoEdgeReferences,
    G::NodeId: Eq + Hash + Into<NodeIndex>,
{
    let stable_clone =
        StableGraph::<(), ()>::from_edges(g.edge_references().map(|e| (e.source(), e.target())));
    let node_seq = good_node_sequence(stable_clone);

    g.edge_references()
        .filter(move |e| node_seq[&e.source().into()] >= node_seq[&e.target().into()])
}

// TODO: handle 64-bit node index size
fn good_node_sequence(mut g: StableGraph<(), ()>) -> HashMap<NodeIndex, u32> {
    let mut s_1 = VecDeque::new();
    let mut s_2 = VecDeque::new();

    while g.node_count() > 0 {
        while let Some(sink_node) = g.node_indices().find(|n| node_is_sink(*n, &g)) {
            g.remove_node(sink_node);
            s_2.push_front(sink_node);
        }

        while let Some(source_node) = g.node_indices().find(|n| node_is_source(*n, &g)) {
            g.remove_node(source_node);
            s_1.push_back(source_node);
        }

        if g.node_count() > 0 {
            let to_remove = g
                .node_indices()
                .max_by_key(|n| delta_degree(*n, &g))
                .unwrap();

            g.remove_node(to_remove);
            s_1.push_back(to_remove);
        }
    }

    s_1.into_iter()
        .chain(s_2)
        .enumerate()
        .map(|(seq_order, node_index)| (node_index, seq_order as u32))
        .collect()
}

fn node_is_sink(n: NodeIndex, g: &StableGraph<(), ()>) -> bool {
    !g.edges_directed(n, Direction::Outgoing).any(|_| true)
}

fn node_is_source(n: NodeIndex, g: &StableGraph<(), ()>) -> bool {
    !g.edges_directed(n, Direction::Incoming).any(|_| true)
}

fn delta_degree(n: NodeIndex, g: &StableGraph<(), ()>) -> isize {
    g.edges_directed(n, Direction::Outgoing).count() as isize
        - g.edges_directed(n, Direction::Incoming).count() as isize
}
