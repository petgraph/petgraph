use crate::visit::{EdgeRef, IntoEdges, IntoNodeReferences, NodeRef, Visitable};
use std::collections::HashSet;
use std::hash::Hash;

/// \[Generic\] Bron-Kerbosch algorithm for finding maximal cliques in an undirected graph.
///
/// Algorithm source: [ACM](https://dl.acm.org/doi/pdf/10.1145/362342.362367)
/// Complexity: [O(3^(n/3))](https://en.wikipedia.org/wiki/Bron%E2%80%93Kerbosch_algorithm#Worst-case_analysis)
///
/// Arguments:
/// * `graph` - Any graph type that implements `IntoEdges` , `Visitable` and `IntoNodeReferences`
///
/// Returns: A vector of vectors, where each inner vector contains node IDs forming a maximal clique
///
/// # Example
/// ```rust
/// use petgraph::graph::UnGraph;
/// use petgraph::algo::bron_kerbosch;
///
/// let mut graph = UnGraph::<(), ()>::new_undirected();
///
/// let a = graph.add_node(());
/// let b = graph.add_node(());
/// let c = graph.add_node(());
/// let d = graph.add_node(());
///
/// graph.extend_with_edges(&[
///    (a, b),
///    (b, c),
///    (c, a),
///    (c, d),
/// ]);
///
/// // a ---- c ---- d
/// // \     /
/// //  \   /
/// //    b
///
/// let cliques = bron_kerbosch(&graph);
///
/// assert_eq!(cliques.len(), 2);
/// ```
pub fn bron_kerbosch<G>(graph: G) -> Vec<Vec<G::NodeId>>
where
    G: IntoEdges + Visitable + IntoNodeReferences,
    G::NodeId: Eq + Hash,
{
    let mut maximal_cliques = Vec::new();
    let mut r = HashSet::new();
    let p: HashSet<G::NodeId> = graph.node_references().map(|n| n.id()).collect();
    let mut x = HashSet::new();

    bron_kerbosch_recursive(&graph, &mut r, &p, &mut x, &mut maximal_cliques);
    maximal_cliques
}

fn bron_kerbosch_recursive<G>(
    graph: &G,
    r: &mut HashSet<G::NodeId>,
    p: &HashSet<G::NodeId>,
    x: &mut HashSet<G::NodeId>,
    maximal_cliques: &mut Vec<Vec<G::NodeId>>,
) where
    G: IntoEdges + Visitable,
    G::NodeId: Eq + Hash,
{
    if p.is_empty() && x.is_empty() {
        maximal_cliques.push(r.iter().cloned().collect());
        return;
    }

    let pivot = p
        .iter()
        .chain(x.iter())
        .max_by_key(|&&u| {
            p.iter()
                .filter(|&&v| graph.edges(u).any(|e| e.target() == v))
                .count()
        })
        .cloned();

    let vertices_to_process = if let Some(pivot) = pivot {
        p.iter()
            .cloned()
            .filter(|&v| !graph.edges(pivot).any(|e| e.target() == v))
            .collect::<Vec<_>>()
    } else {
        p.iter().cloned().collect()
    };

    for v in vertices_to_process {
        let neighbors: HashSet<G::NodeId> = graph.edges(v).map(|e| e.target()).collect();

        let new_p: HashSet<_> = p.intersection(&neighbors).cloned().collect();
        let mut new_x: HashSet<_> = x.intersection(&neighbors).cloned().collect();

        r.insert(v);
        bron_kerbosch_recursive(graph, r, &new_p, &mut new_x, maximal_cliques);
        r.remove(&v);

        x.insert(v);
    }
}
