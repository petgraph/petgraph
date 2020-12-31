use std::collections::{HashMap, VecDeque};

use crate::{
    graph::{GraphIndex, NodeIndex},
    stable_graph::StableDiGraph,
    visit::{EdgeRef, GraphProp, IntoEdgeReferences},
    Directed, Direction,
};

/// Isomorphic to input graph; used in the process of determining a good node sequence from which to
/// extract a feedback arc set.
type SeqSourceGraph = StableDiGraph<(), (), SeqGraphIx>;
type SeqGraphIx = usize;

/// \[Generic\] Finds a [feedback arc set]: a set of edges in the given directed graph, which when
/// removed, make the graph acyclic.
///
/// Uses a [greedy heuristic algorithm] to select a small number of edges in reasonable time, but
/// does not necessarily find the minimum feedback arc set.
///
/// Does not consider edge/node weights when selecting edges for the feedback arc set.
///
/// Loops (edges to and from the same node) are always included in the returned set.
///
/// # Example
///
/// ```
/// use petgraph::{
///     algo::{greedy_feedback_arc_set, is_cyclic_directed},
///     graph::EdgeIndex,
///     stable_graph::StableGraph,
///     visit::EdgeRef,
/// };
///
/// let mut g: StableGraph<(), ()> = StableGraph::from_edges(&[
///     (0, 1),
///     (1, 2),
///     (2, 3),
///     (3, 4),
///     (4, 5),
///     (5, 0),
///     (4, 1),
///     (1, 3),
/// ]);
///
/// assert!(is_cyclic_directed(&g));
///
/// let fas: Vec<EdgeIndex> = greedy_feedback_arc_set(&g).map(|e| e.id()).collect();
///
/// // Remove edges in feedback arc set from original graph
/// for edge_id in fas {
///     g.remove_edge(edge_id);
/// }
///
/// assert!(!is_cyclic_directed(&g));
/// ```
///
/// [feedback arc set]: https://en.wikipedia.org/wiki/Feedback_arc_set
/// [greedy heuristic algorithm]: https://doi.org/10.1016/0020-0190(93)90079-O
pub fn greedy_feedback_arc_set<G>(g: G) -> impl Iterator<Item = G::EdgeRef>
where
    G: IntoEdgeReferences + GraphProp<EdgeType = Directed>,
    G::NodeId: GraphIndex,
{
    let stable_clone = SeqSourceGraph::from_edges(
        g.edge_references()
            .map(|e| (e.source().index(), e.target().index())),
    );
    let node_seq = good_node_sequence(stable_clone);

    g.edge_references()
        .filter(move |e| node_seq[&e.source().index()] >= node_seq[&e.target().index()])
}

fn good_node_sequence(mut g: SeqSourceGraph) -> HashMap<SeqGraphIx, SeqGraphIx> {
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
        .map(|(seq_order, node_index)| (node_index.index(), seq_order))
        .collect()
}

fn node_is_sink(n: NodeIndex<SeqGraphIx>, g: &SeqSourceGraph) -> bool {
    !g.edges_directed(n, Direction::Outgoing).any(|_| true)
}

fn node_is_source(n: NodeIndex<SeqGraphIx>, g: &SeqSourceGraph) -> bool {
    !g.edges_directed(n, Direction::Incoming).any(|_| true)
}

fn delta_degree(n: NodeIndex<SeqGraphIx>, g: &SeqSourceGraph) -> isize {
    g.edges_directed(n, Direction::Outgoing).count() as isize
        - g.edges_directed(n, Direction::Incoming).count() as isize
}
