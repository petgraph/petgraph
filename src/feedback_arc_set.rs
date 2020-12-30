use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

use crate::visit::{EdgeRef, GraphRef, IntoEdgeReferences};

/// Finds a set of edges which can be removed to make the specified directe graph acyclic
/// ([feedback arc set]). Uses a [greedy heuristic algorithm] to select a small number of edges in
/// reasonable (linear) time, but does not necessarily find the minimum feedback arc set.
///
/// [feedback arc set]: https://en.wikipedia.org/wiki/Feedback_arc_set
/// [greedy heuristic algorithm]: https://doi.org/10.1016/0020-0190(93)90079-O
pub fn greedy_feedback_arc_set<G, R>(g: G) -> impl Iterator<Item = G::EdgeRef>
where
    G: IntoEdgeReferences + GraphRef + Clone,
    G::NodeId: Eq + Hash,
{
    let node_seq = good_node_sequence(g);

    g.edge_references()
        .filter(move |e| node_seq[&e.source()] < node_seq[&e.target()])
}

// TODO: make map value type the `Ix: IndexType` in `Graph<_, _, _, Ix>` if possible
fn good_node_sequence<G>(g: G) -> HashMap<G::NodeId, usize>
where
    G: GraphRef + Clone,
    G::NodeId: Eq + Hash,
{
    let mut s_1 = VecDeque::new();
    let mut s_2 = VecDeque::new();
}

// TODO: docs about more efficient alternative for undirected graphs
// And/or add trait requirement `GraphProp<EdgeType=Directed>`
