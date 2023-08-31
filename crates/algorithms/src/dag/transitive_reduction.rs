//! Compute the transitive reduction and closure of a directed acyclic graph
//!
//! ## Transitive reduction and closure
//! The *transitive closure* of a graph **G = (V, E)** is the graph **Gc = (V, Ec)**
//! such that **(i, j)** belongs to **Ec** if and only if there is a path connecting
//! **i** to **j** in **G**. The *transitive reduction* of **G** is the graph **Gr
//! = (V, Er)** such that **Er** is minimal wrt. inclusion in **E** and the transitive
//! closure of **Gr** is the same as that of **G**.
//! The transitive reduction is well-defined for acyclic graphs only.

use alloc::{vec, vec::Vec};

use fixedbitset::FixedBitSet;
use petgraph_adjacency_matrix::{AdjacencyList, NodeIndex, UnweightedAdjacencyList};
use petgraph_core::{
    edge::Direction,
    id::{FromIndexType, IndexType, IntoIndexType, SafeCast},
    visit::{GraphBase, IntoNeighbors, IntoNeighborsDirected, NodeCompactIndexable, NodeCount},
};

// TODO: does this need access to the AdjaencyMatrix? Or could we model this with the builder
//  methods instead? I don't think so.
/// Creates a representation of the same graph respecting topological order for use in
/// `tred::dag_transitive_reduction_closure`.
///
/// `toposort` must be a topological order on the node indices of `g` (for example obtained
/// from [`toposort`]).
///
/// Returns a pair of a graph `res` and the reciprocal of the topological sort `revmap`.
///
/// `res` is the same graph as `g` with the following differences:
/// * Node and edge weights are stripped,
/// * Node indices are replaced by the corresponding rank in `toposort`,
/// * Iterating on the neighbors of a node respects topological order.
///
/// `revmap` is handy to get back to map indices in `g` to indices in `res`.
/// ```
/// use petgraph_algorithms::dag::dag_to_toposorted_adjacency_list;
/// use petgraph_core::{
///     edge::Directed,
///     id::{DefaultIx, SafeCast},
///     visit::IntoNeighbors,
/// };
/// use petgraph_graph::{Graph, NodeIndex};
///
/// let mut g = Graph::<&str, (), Directed, DefaultIx>::new();
/// let second = g.add_node("second child");
/// let top = g.add_node("top");
/// let first = g.add_node("first child");
/// g.extend_with_edges(&[(top, second), (top, first), (first, second)]);
///
/// let toposort = vec![top, first, second];
///
/// let (res, revmap) = dag_to_toposorted_adjacency_list(&g, &toposort);
///
/// // let's compute the children of top in topological order
/// let children: Vec<NodeIndex> = res
///     .neighbors(revmap[top.index()])
///     .map(|ix| toposort[ix.cast()])
///     .collect();
/// assert_eq!(children, vec![first, second])
/// ```
///
/// Runtime: **O(|V| + |E|)**.
///
/// Space complexity: **O(|V| + |E|)**.
pub fn dag_to_toposorted_adjacency_list<G, Ix: IndexType>(
    g: G,
    toposort: &[G::NodeId],
) -> (UnweightedAdjacencyList<Ix>, Vec<NodeIndex<Ix>>)
where
    G: GraphBase + IntoNeighborsDirected + NodeCompactIndexable + NodeCount,
    G::NodeId: IntoIndexType<Index = Ix> + Copy,
{
    let mut res = AdjacencyList::with_capacity(g.node_count());
    // map from old node index to rank in toposort
    let mut revmap = vec![NodeIndex::new(Ix::ZERO); g.node_bound()];
    for (ix, &old_ix) in toposort.iter().enumerate() {
        let ix: NodeIndex<Ix> = NodeIndex::from_usize(ix);
        revmap[old_ix.into_index().cast()] = ix;
        let iter = g.neighbors_directed(old_ix, Direction::Incoming);
        let new_ix = res.add_node_with_capacity(iter.size_hint().0);
        debug_assert_eq!(new_ix, ix);
        for old_pre in iter {
            let pre = revmap[old_pre.into_index().cast()];
            res.add_edge(pre, ix, ());
        }
    }
    (res, revmap)
}

/// Computes the transitive reduction and closure of a DAG.
///
/// The algorithm implemented here comes from [On the calculation of
/// transitive reduction-closure of
/// orders](https://www.sciencedirect.com/science/article/pii/0012365X9390164O) by Habib, Morvan
/// and Rampon.
///
/// The input graph must be in a very specific format: an adjacency
/// list such that:
/// * Node indices are a toposort, and
/// * The neighbors of all nodes are stored in topological order.
/// To get such a representation, use the function [`dag_to_toposorted_adjacency_list`].
///
/// [`dag_to_toposorted_adjacency_list`]: ./fn.dag_to_toposorted_adjacency_list.html
///
/// The output is the pair of the transitive reduction and the transitive closure.
///
/// Runtime complexity: **O(|V| + \sum_{(x, y) \in Er} d(y))** where **d(y)**
/// denotes the outgoing degree of **y** in the transitive closure of **G**.
/// This is still **O(|V|³)** in the worst case like the naive algorithm but
/// should perform better for some classes of graphs.
///
/// Space complexity: **O(|E|)**.
pub fn dag_transitive_reduction_closure<E, Ix: IndexType>(
    g: &AdjacencyList<E, Ix>,
) -> (UnweightedAdjacencyList<Ix>, UnweightedAdjacencyList<Ix>) {
    let mut tred = AdjacencyList::with_capacity(g.node_count());
    let mut tclos = AdjacencyList::with_capacity(g.node_count());
    let mut mark = FixedBitSet::with_capacity(g.node_count());

    for i in g.node_indices() {
        tred.add_node();
        tclos.add_node_with_capacity(g.neighbors(i).len());
    }
    // the algorithm relies on this iterator being toposorted
    for i in g.node_indices().rev() {
        // the algorighm relies on this iterator being toposorted
        for x in g.neighbors(i) {
            if !mark[x.cast()] {
                tred.add_edge(i, x, ());
                tclos.add_edge(i, x, ());
                for e in tclos.edge_indices_from(x) {
                    let y = tclos.edge_endpoints(e).unwrap().1;
                    if !mark[y.cast()] {
                        mark.insert(y.cast());
                        tclos.add_edge(i, y, ());
                    }
                }
            }
        }
        for y in tclos.neighbors(i) {
            mark.set(y.cast(), false);
        }
    }
    (tred, tclos)
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use petgraph_adjacency_matrix::{AdjacencyList, NodeIndex};
    use petgraph_core::{
        edge::Directed,
        id::{IndexType, SafeCast},
        visit::{
            Dfs, EdgeFiltered, EdgeRef, IntoEdgeReferences, IntoNeighbors, IntoNodeIdentifiers,
            NodeCount, Visitable,
        },
    };
    use petgraph_graph::Graph;
    use petgraph_proptest::dag::graph_dag_strategy;
    use proptest::prelude::*;

    use crate::dag::{dag_transitive_reduction_closure, toposort};

    #[test]
    fn simple() {
        let mut input = AdjacencyList::new();

        let a: NodeIndex<u8> = input.add_node();
        let b = input.add_node();
        let c = input.add_node();

        input.add_edge(a, b, ());
        input.add_edge(a, c, ());
        input.add_edge(b, c, ());

        let (reduction, closure) = dag_transitive_reduction_closure(&input);

        assert_eq!(reduction.node_count(), 3);
        assert_eq!(closure.node_count(), 3);

        assert!(reduction.find_edge(a, b).is_some());
        assert!(reduction.find_edge(b, c).is_some());
        assert!(reduction.find_edge(a, c).is_none());

        assert!(closure.find_edge(a, b).is_some());
        assert!(closure.find_edge(b, c).is_some());
        assert!(closure.find_edge(a, c).is_some());
    }

    fn naive_closure_foreach<G, F>(graph: G, mut closure: F)
    where
        G: Visitable + IntoNodeIdentifiers + IntoNeighbors,
        F: FnMut(G::NodeId, G::NodeId),
    {
        let mut dfs = Dfs::empty(&graph);

        for index in graph.node_identifiers() {
            dfs.reset(&graph);
            dfs.move_to(index);

            while let Some(nx) = dfs.next(&graph) {
                if index != nx {
                    closure(index, nx);
                }
            }
        }
    }

    fn naive_closure<G>(graph: G) -> Vec<(G::NodeId, G::NodeId)>
    where
        G: Visitable + IntoNodeIdentifiers + IntoNeighbors,
    {
        let mut result = Vec::new();

        naive_closure_foreach(graph, |a, b| result.push((a, b)));

        result
    }

    fn naive_closure_edgecount<G>(graph: G) -> usize
    where
        G: Visitable + IntoNodeIdentifiers + IntoNeighbors,
    {
        let mut count = 0;

        naive_closure_foreach(graph, |_, _| count += 1);

        count
    }

    #[cfg(not(miri))]
    proptest! {
        // a bit convoluted, as all functions build on top of each other
        #[test]
        fn integration(graph in graph_dag_strategy::<Graph<(), (), Directed, u8>>(None, None, None)) {
            let toposort = toposort(&graph, None).unwrap();

            let (toposorted, reverse_toposort) = super::dag_to_toposorted_adjacency_list(&graph, &toposort);

            for (index, node) in toposort.iter().enumerate() {
                assert_eq!(index, reverse_toposort[node.index()].cast());
            }

            let (reduction, closure) = dag_transitive_reduction_closure(&toposorted);

            assert_eq!(reduction.node_count(), graph.node_count());
            assert_eq!(closure.node_count(), graph.node_count());

            // Check the closure
            let mut closure_edges: Vec<(_, _)> = closure.edge_references().map(|e| (e.source(), e.target())).collect();
            closure_edges.sort();

            let mut reduction_closure = naive_closure(&reduction);
            reduction_closure.sort();

            assert_eq!(closure_edges, reduction_closure);

            // Check that the transitive reduction is a reduction
            for index in reduction.edge_references() {
                let filtered = EdgeFiltered::from_fn(&reduction, |edge| {
                    edge.source() != index.source() || edge.target() != index.target()
                });

                let received = naive_closure_edgecount(&filtered);
                assert!(received < closure_edges.len());
            }

            // check that the transitive reduction is included in the original graph
            for index in reduction.edge_references() {
                let source = toposort[index.source().cast()];
                let target = toposort[index.target().cast()];

                assert!(graph.find_edge(source, target).is_some());
            }
        }
    }
}
