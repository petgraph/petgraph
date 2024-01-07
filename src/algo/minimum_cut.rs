use std::collections::{BinaryHeap, HashMap};
use std::hash::Hash;
use crate::algo::Measure;
use crate::{Graph, Undirected};
use crate::graph::{IndexType, NodeIndex};
use crate::scored::MaxScored;
use crate::visit::{EdgeRef, IntoNodeIdentifiers, IntoEdgeReferences, Visitable, VisitMap};

/// \[Generic\] Stoer–Wagner algorithm to solve the minimum cut problem on undirected weighted graphs.
/// https://en.wikipedia.org/wiki/Stoer%E2%80%93Wagner_algorithm
///
/// A (global) minimum cut of a graph is a set of edges of minimum weight whose deletion disconnects the graph.
/// https://en.wikipedia.org/wiki/Minimum_cut
/// 
/// The graph must be undirected. It must implement `IntoNodeIdentifiers` and `IntoEdgeReferences`.
/// The function `edge_cost` should return the cost for a particular edge. Edge costs must be
/// non-negative.
/// Returns a tuple composed of a vector of `EdgeId` that represents the cut and the cut weight.
/// 
/// Computes in **O(|V| |E| + |V|²*log(|V|)))** time
///
/// # Example
/// ```rust
/// use petgraph::{Graph, Undirected};
/// use petgraph::algo::minimum_cut;
/// 
/// let mut graph: Graph<(), u32, Undirected> = Graph::new_undirected();
/// let a = graph.add_node(());
/// let b = graph.add_node(());
/// let c = graph.add_node(());
/// let d = graph.add_node(());
/// let e = graph.add_node(());
/// let f = graph.add_node(());
/// let g = graph.add_node(());
/// let h = graph.add_node(());
///
/// graph.extend_with_edges(&[
///     (a, b, 7),
///     (b, c, 2),
///     (c, d, 8),
///     (d, a, 3),
///     (e, f, 6),
///     (b, e, 1),
///     (f, g, 1),
///     (g, h, 5),
///     (h, e, 4),
///     (c, h, 3),
/// ]);
///
/// // a --7-- b --1-- e --6-- f
/// // |       |       |       |
/// // 3       2       4       1
/// // |       |       |       |
/// // d --8-- c---3-- h --5-- g
///
/// let (cut, weight) = minimum_cut(&graph, |e| *e.weight());
/// assert_eq!(cut.len(), 2);
/// assert_eq!(weight, 4);
/// ```

pub fn minimum_cut<G, F, K>(
    graph: G,
    edge_cost: F,
    ) -> (Vec<G::EdgeId>, K)
where
    G: IntoNodeIdentifiers + IntoEdgeReferences,
    F: Fn(G::EdgeRef) -> K,
    G::NodeId: Eq + Hash,
    K: Measure + Copy,
{
    let mut node_to_node = HashMap::new();
    let mut graph2 = Graph::new_undirected();
    for node in graph.node_identifiers() {
        let node2 = graph2.add_node(node);
        node_to_node.insert(node, node2);
    }

    for edge in graph.edge_references() {
        graph2.add_edge(
            node_to_node[&edge.source()],
            node_to_node[&edge.target()],
            (edge.id(), edge_cost(edge))
        );
    }

    minimum_cut_aux(&mut graph2)
}

// same as .sum() but don't require the Sum trait
#[inline]
fn sum<I,K>(iter: I) -> K
    where
        I: Iterator<Item = K>,
        K: Measure,
{
    iter.fold(K::default(), |a, b| a + b)
}

fn merge_vertices<N, E, Ix>(
    graph: &mut Graph<N, E, Undirected, Ix>,
    node1: NodeIndex<Ix>,
    node2: NodeIndex<Ix>,
)
where
    Ix: IndexType,
    E: Copy,
{
    let edges: Vec<_> = graph.edges(node2).map(|e| (node1, e.target(), *e.weight())).collect();
    for (s, t, w) in edges {
        if t != node1 && t != node2 {
            graph.add_edge(s, t, w);
        }
    }
    graph.remove_node(node2);
}

fn minimum_cut_phase<N, EdgeId, K, Ix>(
    graph: &Graph<N, (EdgeId, K), Undirected, Ix>,
    start: NodeIndex<Ix>,
) -> (Vec<EdgeId>, K, NodeIndex<Ix>, NodeIndex<Ix>) 
where
    Ix: IndexType,
    K: Measure + Copy,
    EdgeId: Copy,
{
    let mut queue = BinaryHeap::new();
    let mut seen = graph.visit_map();
    let mut seen_list = Vec::new();
    let mut max_adj_map: HashMap<NodeIndex<Ix>, K> = HashMap::new();
    queue.push(MaxScored(Default::default(), start));

    while let Some(MaxScored(_, node)) = queue.pop() {
        if !seen.is_visited(&node) {
            seen.visit(node);
            seen_list.push(node);
            for edge in graph.edges(node) {
                let target = edge.target();
                let max_adj = max_adj_map
                                .get(&edge.target())
                                .map_or_else(|| edge.weight().1
                                            , |&w| w + edge.weight().1);
                max_adj_map.insert(target, max_adj);
                queue.push(MaxScored(max_adj, target));
            }
        }
    }

    let len = seen_list.len();
    let last = seen_list[len-1];
    let before_last = seen_list[len-2];
    let cut: Vec<_> = graph.edges(last).map(|e| e.weight().0).collect();
    let cut_weight = sum(graph.edges(last).map(|e| e.weight().1));
    
    (cut, cut_weight, last, before_last)
}

fn minimum_cut_aux<N, EdgeId, K, Ix>(
    graph: &mut Graph<N, (EdgeId, K), Undirected, Ix>
) -> (Vec<EdgeId>, K)
where
    Ix: IndexType,
    K: Measure + Copy,
    EdgeId: Copy,
{
    if graph.node_count() == 0 {
        return (vec![], K::default());
    }
    
    let a = graph.node_indices().next().unwrap();
    let mut best_cut = graph.edges(a).map(|e| e.weight().0).collect();
    let mut best_weight = sum(graph.edges(a).map(|e| e.weight().1));

    while graph.node_count() > 1 {
        let a = graph.node_indices().next().unwrap();
        if graph.edges(a).next().is_none() {
            return (vec! [], Default::default());
        }
        let (cut, weight, node1, node2) = minimum_cut_phase(&graph, a);
        if weight < best_weight {
            best_cut = cut;
            best_weight = weight;
        }
        merge_vertices(graph, node1, node2);
    }
    (best_cut, best_weight)
}