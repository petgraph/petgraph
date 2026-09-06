use alloc::vec::Vec;
use core::{fmt::Debug, hash::Hash};

use hashbrown::{HashMap, HashSet};

use crate::{
    Undirected,
    algo::{
        BoundedMeasure, Measure, dijkstra, floyd_warshall::floyd_warshall_path, min_spanning_tree,
    },
    data::FromElements,
    graph::{IndexType, NodeIndex, UnGraph},
    visit::{
        Data, EdgeRef, GraphBase, GraphProp, IntoEdgeReferences, IntoEdges, IntoNeighbors,
        IntoNodeIdentifiers, IntoNodeReferences, NodeCompactIndexable, NodeIndexable, Visitable,
    },
};
#[cfg(feature = "stable_graph")]
use crate::{graph::EdgeIndex, stable_graph::StableGraph, unionfind::UnionFind};

type Edge<G> = (<G as GraphBase>::NodeId, <G as GraphBase>::NodeId);
type Subgraph<G> = HashSet<<G as GraphBase>::NodeId>;

fn compute_shortest_path_length<G>(graph: G, source: G::NodeId, target: G::NodeId) -> G::EdgeWeight
where
    G: Visitable + IntoEdges,
    G::NodeId: Eq + Hash,
    G::EdgeWeight: Measure + Copy,
{
    let output = dijkstra(graph, source, Some(target), |e| *e.weight());
    output[&target]
}

fn compute_metric_closure<G>(
    graph: G,
    terminals: &[G::NodeId],
) -> HashMap<(usize, usize), G::EdgeWeight>
where
    G: Data + IntoNodeReferences + NodeIndexable + Visitable + IntoEdges,
    G::EdgeWeight: Copy + Measure,
    G::NodeId: PartialOrd + Eq + Hash,
{
    let mut closure = HashMap::new();
    for (i, node_id_1) in terminals.iter().enumerate() {
        for node_id_2 in terminals.iter().skip(i + 1) {
            closure.insert(
                (graph.to_index(*node_id_1), graph.to_index(*node_id_2)),
                compute_shortest_path_length(graph, *node_id_1, *node_id_2),
            );
        }
    }
    closure
}

fn subgraph_edges_from_metric_closure<G>(
    graph: G,
    minimum_spanning_closure: G,
) -> (Vec<Edge<G>>, Subgraph<G>)
where
    G: GraphBase
        + NodeCompactIndexable
        + IntoEdgeReferences
        + IntoNodeIdentifiers
        + GraphProp
        + IntoNodeReferences,
    G::EdgeWeight: BoundedMeasure + Copy,
    G::NodeId: Eq + Hash + Ord + Debug,
{
    let mut retained_nodes = HashSet::new();
    let mut retained_edges = Vec::new();
    let (_, prev) = floyd_warshall_path(graph, |e| *e.weight()).unwrap();

    for edge in minimum_spanning_closure.edge_references() {
        let target = graph.to_index(edge.target());
        let source = graph.to_index(edge.source());

        let mut current = target;
        while current != source {
            if let Some(prev_node) = prev[source][current] {
                retained_nodes.insert(graph.from_index(prev_node));
                retained_nodes.insert(graph.from_index(current));
                retained_edges.push((graph.from_index(prev_node), graph.from_index(current)));
                current = prev_node;
            }
        }
    }

    (retained_edges, retained_nodes)
}

fn non_terminal_leaves<G>(graph: G, terminals: &[G::NodeId]) -> HashSet<G::NodeId>
where
    G: GraphBase + IntoNodeReferences + IntoNodeIdentifiers + IntoNeighbors,
    G::NodeId: Hash + Eq + Debug,
    G::NodeRef: Eq + Hash,
{
    let mut removed_leaves = HashSet::new();

    let mut remaining_leaves = graph
        .node_identifiers()
        .filter(|node_id| {
            graph.neighbors(*node_id).collect::<HashSet<_>>().len() == 1
                && !terminals.contains(node_id)
        })
        .collect::<HashSet<_>>();

    while !remaining_leaves.is_empty() {
        remaining_leaves = graph
            .node_identifiers()
            .filter(|node_id| {
                !terminals.contains(node_id)
                    && !removed_leaves.contains(node_id)
                    && (graph
                        .neighbors(*node_id)
                        .collect::<HashSet<_>>()
                        .difference(&removed_leaves))
                    .collect::<Vec<_>>()
                    .len()
                        == 1
            })
            .collect::<HashSet<_>>();

        removed_leaves = removed_leaves
            .union(&remaining_leaves)
            .cloned()
            .collect::<HashSet<_>>();
    }

    removed_leaves
}

/// [Steiner Tree][1] algorithm.
///
/// Computes the Steiner tree of an undirected connected graph given a set of terminal nodes via
/// [Kou's algorithm][2]. Implementation details are the same as in the [NetworkX
/// implementation][3].
///
/// ## Arguments
/// * `graph`: The undirected graph in which to find the Steiner tree.
/// * `terminals`: A slice of node indices representing the terminals for which the Steiner tree is
///   computed.
///
/// ## Returns
/// A `StableGraph` containing the nodes and edges of the Steiner tree.
///
/// ## Complexity
/// Time complexity: **O(|S| |V|²)**.
/// where **|V|** the number of vertices (i.e nodes) and **|S|** the number of provided terminals.
///
/// [1]: https://en.wikipedia.org/wiki/Steiner_tree_problem
/// [2]: https://doi.org/10.1007/BF00288961
/// [3]: https://networkx.org/documentation/stable/_modules/networkx/algorithms/approximation/steinertree.html#steiner_tree
///
/// # Example
///
/// ```
/// use petgraph::{Graph, algo::steiner_tree::steiner_tree, graph::UnGraph};
/// let mut graph = UnGraph::<(), i32>::default();
/// let a = graph.add_node(());
/// let b = graph.add_node(());
/// let c = graph.add_node(());
/// let d = graph.add_node(());
/// let e = graph.add_node(());
/// let f = graph.add_node(());
/// graph.extend_with_edges([
///     (a, b, 7),
///     (a, f, 6),
///     (b, c, 1),
///     (b, f, 5),
///     (c, d, 1),
///     (c, e, 3),
///     (d, e, 1),
///     (d, f, 4),
///     (e, f, 10),
/// ]);
/// let terminals = vec![a, c, e, f];
/// let tree = steiner_tree(&graph, &terminals);
/// assert_eq!(tree.edge_weights().sum::<i32>(), 12);
/// ```
#[cfg(feature = "stable_graph")]
pub fn steiner_tree<N, E, Ix>(
    graph: &UnGraph<N, E, Ix>,
    terminals: &[NodeIndex<Ix>],
) -> StableGraph<N, E, Undirected, Ix>
where
    N: Default + Clone + Eq + Hash + Debug,
    E: Copy + Eq + Ord + Measure + BoundedMeasure,
    Ix: IndexType,
{
    let metric_closure = compute_metric_closure(&graph, terminals);
    let metric_closure_graph: UnGraph<N, E, _> = UnGraph::from_edges(
        metric_closure
            .iter()
            .map(|((node1, node2), &weight)| (*node1, *node2, weight)),
    );

    let minimum_spanning = UnGraph::from_elements(min_spanning_tree(&metric_closure_graph));

    let (subgraph_edges, subgraph_nodes) =
        subgraph_edges_from_metric_closure(graph, &minimum_spanning);

    let mut graph = StableGraph::from(graph.clone());
    graph.retain_edges(|graph, e| {
        let edge = graph.edge_endpoints(e).unwrap();
        subgraph_edges.contains(&(edge.0, edge.1)) || subgraph_edges.contains(&(edge.1, edge.0))
    });
    graph.retain_nodes(|_, n| subgraph_nodes.contains(&n));

    let spanning_edges = spanning_tree_edges(&graph);
    graph.retain_edges(|_, e| spanning_edges.contains(&e));

    let non_terminal_nodes = non_terminal_leaves(&graph, terminals);
    graph.retain_nodes(|_, n| !non_terminal_nodes.contains(&n));

    graph
}

/// Kruskal over `graph`, returning the indices of the edges that make up a minimum spanning
/// forest. Unlike [`min_spanning_tree`] this keeps the identity of the input edges, so the caller
/// can drop the remaining ones in place instead of rebuilding the graph and losing its indices.
#[cfg(feature = "stable_graph")]
fn spanning_tree_edges<N, E, Ix>(
    graph: &StableGraph<N, E, Undirected, Ix>,
) -> HashSet<EdgeIndex<Ix>>
where
    E: Copy + Ord,
    Ix: IndexType,
{
    let mut edges = graph
        .edge_references()
        .map(|edge| (*edge.weight(), edge.id(), edge.source(), edge.target()))
        .collect::<Vec<_>>();
    edges.sort_unstable_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    let mut components = UnionFind::new(graph.node_bound());
    let mut spanning_edges = HashSet::new();
    for (_, edge_id, source, target) in edges {
        if components.union(source.index(), target.index()) {
            spanning_edges.insert(edge_id);
        }
    }

    spanning_edges
}

#[cfg(test)]
mod test {
    use alloc::{vec, vec::Vec};

    use hashbrown::{HashMap, HashSet};

    #[cfg(feature = "stable_graph")]
    use super::steiner_tree;
    use super::{compute_metric_closure, non_terminal_leaves, subgraph_edges_from_metric_closure};
    use crate::{
        Graph, Undirected,
        algo::{EdgeRef, UnGraph, min_spanning_tree},
        data::FromElements,
        graph::NodeIndex,
    };
    #[cfg(feature = "stable_graph")]
    use crate::{algo::is_cyclic_undirected, visit::IntoEdgeReferences};

    #[test]
    fn test_compute_metric_closure() {
        let mut graph = Graph::<(), i32, Undirected>::new_undirected();

        let a = graph.add_node(());
        let b = graph.add_node(());
        let c = graph.add_node(());
        let d = graph.add_node(());
        let e = graph.add_node(());
        let f = graph.add_node(());
        graph.extend_with_edges([
            (a, b, 7),
            (a, f, 6),
            (b, c, 1),
            (b, f, 5),
            (c, d, 1),
            (c, e, 3),
            (d, e, 1),
            (d, f, 4),
            (e, f, 10),
        ]);

        let terminals = vec![a, c, e, f];
        let metric_closure = compute_metric_closure(&graph, &terminals);

        let metric_closure_graph: UnGraph<&str, _, _> = UnGraph::from_edges(
            metric_closure
                .iter()
                .map(|((node1, node2), &weight)| (*node1, *node2, weight)),
        );

        let ref_weights = HashMap::<_, _>::from([
            ((0, 2), 8),
            ((0, 4), 10),
            ((0, 5), 6),
            ((2, 4), 2),
            ((2, 5), 5),
            ((4, 5), 5),
        ]);
        for ((node1, node2), ref_weight) in ref_weights {
            assert_eq!(metric_closure[&(node1, node2)], ref_weight);
            assert_eq!(
                *metric_closure_graph
                    .edge_weight(
                        metric_closure_graph
                            .find_edge(NodeIndex::new(node1), NodeIndex::new(node2))
                            .unwrap()
                    )
                    .unwrap(),
                ref_weight
            );
        }
    }

    #[test]
    fn test_subgraph_from_metric_closure() {
        let mut graph = Graph::<(), i32, _>::new_undirected();

        let a = graph.add_node(());
        let b = graph.add_node(());
        let c = graph.add_node(());
        let d = graph.add_node(());
        let e = graph.add_node(());
        let f = graph.add_node(());
        graph.extend_with_edges([
            (a, b, 7),
            (a, f, 6),
            (b, c, 1),
            (b, f, 5),
            (c, d, 1),
            (c, e, 3),
            (d, e, 1),
            (d, f, 4),
            (e, f, 10),
        ]);

        let terminals = vec![a, c, e, f];
        let metric_closure = compute_metric_closure(&graph, &terminals);

        let metric_closure_graph: UnGraph<(), _, _> = UnGraph::from_edges(
            metric_closure
                .iter()
                .map(|((node1, node2), &weight)| (*node1 as u32, *node2 as u32, weight)),
        );

        let minimum_spanning = UnGraph::from_elements(min_spanning_tree(&metric_closure_graph));

        let (subgraph_edges, _subgraph_nodes) =
            subgraph_edges_from_metric_closure(&graph, &minimum_spanning);

        graph.retain_edges(|graph, e| {
            let edge = graph.edge_endpoints(e).unwrap();
            subgraph_edges.contains(&(edge.0, edge.1))
        });

        let mut ref_graph = UnGraph::<(), _>::new_undirected();
        let ref_a = ref_graph.add_node(());
        let _ = ref_graph.add_node(());
        let ref_c = ref_graph.add_node(());
        let ref_d = ref_graph.add_node(());
        let ref_e = ref_graph.add_node(());
        let ref_f = ref_graph.add_node(());

        ref_graph.extend_with_edges([
            (ref_c, ref_d, 1),
            (ref_d, ref_e, 1),
            (ref_d, ref_f, 4),
            (ref_a, ref_f, 6),
        ]);

        for ref_edge in ref_graph.edge_references() {
            let (edge_index, _) = graph
                .find_edge_undirected(ref_edge.source(), ref_edge.target())
                .unwrap();
            let edge_endpoints = graph.edge_endpoints(edge_index).unwrap();
            assert_eq!(graph.edge_weight(edge_index).unwrap(), ref_edge.weight());
            assert_eq!(edge_endpoints.0, ref_edge.source());
            assert_eq!(edge_endpoints.1, ref_edge.target());
        }
    }

    /// Regression test for petgraph#922. The metric closure of this graph has four minimum
    /// spanning trees of equal weight; two of them pair up terminals so that the expanded
    /// shortest paths cover all three edges of the triangle `a`, `b`, `d`, and the returned
    /// graph then has as many edges as nodes. Which spanning tree comes out follows the
    /// iteration order of the metric closure `HashMap`, which is randomly seeded, so the
    /// check is repeated.
    #[cfg(feature = "stable_graph")]
    #[test]
    fn test_steiner_tree_is_a_tree() {
        let mut graph = Graph::<(), i32, _>::new_undirected();

        let a = graph.add_node(());
        let b = graph.add_node(());
        let c = graph.add_node(());
        let d = graph.add_node(());
        let e = graph.add_node(());
        let f = graph.add_node(());
        let g = graph.add_node(());
        graph.extend_with_edges([
            (a, b, 1),
            (a, d, 1),
            (b, d, 1),
            (c, d, 1),
            (c, e, 1),
            (d, g, 1),
            (f, g, 1),
        ]);

        let terminals = vec![a, b, e, f];
        for _ in 0..64 {
            let tree = steiner_tree(&graph, &terminals);
            assert_eq!(
                tree.node_count(),
                tree.edge_count() + 1,
                "not a tree: {:?}",
                tree.edge_references()
                    .map(|edge| (edge.source(), edge.target()))
                    .collect::<Vec<_>>()
            );
            assert!(!is_cyclic_undirected(&tree));
        }
    }

    #[test]
    fn test_remove_non_terminal_nodes() {
        let mut graph = Graph::<(), i32, _>::new_undirected();

        let a = graph.add_node(());
        let b = graph.add_node(());
        let c = graph.add_node(());
        let d = graph.add_node(());
        let e = graph.add_node(());
        let f = graph.add_node(());
        graph.extend_with_edges([(a, b, 7), (b, c, 6), (c, d, 1), (d, e, 5), (e, f, 1)]);

        let terminals = vec![a, c];
        let non_terminal_nodes = non_terminal_leaves(&graph, &terminals);
        let non_terminal_refs = HashSet::from([d, e, f]);
        assert_eq!(non_terminal_refs, non_terminal_nodes);
    }
}
