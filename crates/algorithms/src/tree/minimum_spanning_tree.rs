use alloc::collections::BinaryHeap;

use indexmap::IndexMap;
use petgraph_core::{
    data::Element,
    visit::{Data, EdgeRef, IntoEdgeReferences, IntoNodeReferences, NodeIndexable, NodeRef},
};

use crate::{
    shortest_paths::TotalOrd,
    utilities::{min_scored::MinScored, union_find::UnionFind},
};

/// An iterator producing a minimum spanning forest of a graph.
#[derive(Debug, Clone)]
pub struct MinSpanningTree<G>
where
    G: Data + IntoNodeReferences,
{
    graph: G,
    node_ids: Option<G::NodeReferences>,
    subgraphs: UnionFind<usize>,
    #[allow(clippy::type_complexity)]
    sort_edges: BinaryHeap<MinScored<G::EdgeWeight, (G::NodeId, G::NodeId)>>,
    node_map: IndexMap<usize, usize>,
    node_count: usize,
}

impl<G> Iterator for MinSpanningTree<G>
where
    G: IntoNodeReferences + NodeIndexable,
    G::NodeWeight: Clone,
    G::EdgeWeight: TotalOrd,
{
    type Item = Element<G::NodeWeight, G::EdgeWeight>;

    fn next(&mut self) -> Option<Self::Item> {
        let g = self.graph;
        if let Some(ref mut iter) = self.node_ids {
            if let Some(node) = iter.next() {
                self.node_map.insert(g.to_index(node.id()), self.node_count);
                self.node_count += 1;
                return Some(Element::Node {
                    weight: node.weight().clone(),
                });
            }
        }
        self.node_ids = None;

        // Kruskal's algorithm.
        // Algorithm is this:
        //
        // 1. Create a pre-MST with all the vertices and no edges.
        // 2. Repeat:
        //
        //  a. Remove the shortest edge from the original graph.
        //  b. If the edge connects two disjoint trees in the pre-MST,
        //     add the edge.
        while let Some(MinScored(score, (a, b))) = self.sort_edges.pop() {
            // check if the edge would connect two disjoint parts
            let (a_index, b_index) = (g.to_index(a), g.to_index(b));
            if self.subgraphs.union(a_index, b_index) {
                let (&a_order, &b_order) =
                    match (self.node_map.get(&a_index), self.node_map.get(&b_index)) {
                        (Some(a_id), Some(b_id)) => (a_id, b_id),
                        _ => panic!("Edge references unknown node"),
                    };
                return Some(Element::Edge {
                    source: a_order,
                    target: b_order,
                    weight: score,
                });
            }
        }
        None
    }
}

/// \[Generic\] Compute a *minimum spanning tree* of a graph.
///
/// The input graph is treated as if undirected.
///
/// Using Kruskal's algorithm with runtime **O(|E| log |E|)**. We actually
/// return a minimum spanning forest, i.e. a minimum spanning tree for each connected
/// component of the graph.
///
/// The resulting graph has all the vertices of the input graph (with identical node indices),
/// and **|V| - c** edges, where **c** is the number of connected components in `g`.
///
/// Use `from_elements` to create a graph from the resulting iterator.
pub fn minimum_spanning_tree<G>(g: G) -> MinSpanningTree<G>
where
    G::NodeWeight: Clone,
    G::EdgeWeight: Clone + TotalOrd,
    G: IntoNodeReferences + IntoEdgeReferences + NodeIndexable,
{
    // Initially each vertex is its own disjoint subgraph, track the connectedness
    // of the pre-MST with a union & find datastructure.
    let subgraphs = UnionFind::new(g.node_bound());

    let edges = g.edge_references();
    let mut sort_edges = BinaryHeap::with_capacity(edges.size_hint().0);
    for edge in edges {
        sort_edges.push(MinScored(
            edge.weight().clone(),
            (edge.source(), edge.target()),
        ));
    }

    MinSpanningTree {
        graph: g,
        node_ids: Some(g.node_references()),
        subgraphs,
        sort_edges,
        node_map: IndexMap::new(),
        node_count: 0,
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use petgraph_core::data::FromElements;
    use petgraph_graph::{DiGraph, Graph, UnGraph};
    use proptest::prelude::*;

    use super::*;
    use crate::cycles::is_cyclic_undirected;

    /// Setup the graph used in several tests.
    ///
    /// The graph is taken from the Wikipedia article on Kruskal's algorithm.
    /// <https://en.wikipedia.org/wiki/Kruskal%27s_algorithm>
    fn setup_wikipedia() -> Graph<&'static str, u32> {
        let mut graph = Graph::new();

        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");
        let d = graph.add_node("D");
        let e = graph.add_node("E");
        let f = graph.add_node("F");
        let g = graph.add_node("G");

        graph.extend_with_edges([
            (a, b, 7),
            (a, d, 5),
            (b, c, 8),
            (b, d, 9),
            (b, e, 7),
            (c, e, 5),
            (d, e, 15),
            (d, f, 6),
            (e, f, 8),
            (e, g, 9),
            (f, g, 11),
        ]);

        graph
    }

    /// Test that the minimum spanning tree of a graph is correct.
    #[test]
    fn example() {
        let graph = setup_wikipedia();

        let mst = UnGraph::<_, _>::from_elements(minimum_spanning_tree(&graph));

        // convert between node indices and node weights
        let node = |index| *mst.node_weight(index).unwrap();

        let mut edges = mst
            .edge_references()
            .map(|e| (node(e.source()), node(e.target()), e.weight()))
            .collect::<Vec<_>>();

        edges.sort_by_key(|e| (e.0, e.1));

        assert_eq!(edges, [
            ("A", "B", &7),
            ("A", "D", &5),
            ("B", "E", &7),
            ("C", "E", &5),
            ("D", "F", &6),
            ("E", "G", &9),
        ]);
    }

    /// Test that the minimum spanning tree of a disjoint graph is correct.
    ///
    /// ```text
    /// A → B
    ///   ↘ ↓
    ///     C
    ///
    /// D → E
    /// ```
    ///
    /// Where the edges are weighted as follows:
    /// * A → B: 1
    /// * A → C: 2
    /// * B → C: 3
    /// * D → E: 4
    #[test]
    fn disjoint() {
        let mut graph = Graph::new();

        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");
        let d = graph.add_node("D");
        let e = graph.add_node("E");

        graph.extend_with_edges([
            (a, b, 1), //
            (a, c, 2),
            (b, c, 3),
            (d, e, 4),
        ]);

        let mst = UnGraph::<_, _>::from_elements(minimum_spanning_tree(&graph));

        // convert between node indices and node weights
        let node = |index| *mst.node_weight(index).unwrap();

        let mut edges = mst
            .edge_references()
            .map(|e| (node(e.source()), node(e.target()), e.weight()))
            .collect::<Vec<_>>();

        edges.sort_by_key(|e| (e.0, e.1));

        assert_eq!(edges, [
            ("A", "B", &1), //
            ("A", "C", &2),
            ("D", "E", &4),
        ]);
    }

    proptest! {
        /// Verify the assumption that every minimum spanning tree must not be cyclic.
        #[test]
        fn no_cycles_directed(graph in any::<DiGraph<(), u8, u8>>()) {
            let mst = UnGraph::<_, _>::from_elements(minimum_spanning_tree(&graph));

            prop_assert!(!is_cyclic_undirected(&mst));
        }

        /// Verify the assumption that the nodes of a minimum spanning tree always include all nodes.
        #[test]
        fn consistent_node_count_directed(graph in any::<DiGraph<(), u8, u8>>()) {
            let nodes = graph.node_count();

            let mst = UnGraph::<_, _>::from_elements(minimum_spanning_tree(&graph));

            prop_assert_eq!(mst.node_count(), nodes);
        }

        /// Verify the assumption that every minimum spanning tree must not be cyclic.
        #[test]
        fn no_cycles_undirected(graph in any::<UnGraph<(), u8, u8>>()) {
            let mst = UnGraph::<_, _>::from_elements(minimum_spanning_tree(&graph));

            prop_assert!(!is_cyclic_undirected(&mst));
        }

        /// Verify the assumption that the nodes of a minimum spanning tree always include all nodes.
        #[test]
        fn consistent_node_count_undirected(graph in any::<UnGraph<(), u8, u8>>()) {
            let nodes = graph.node_count();

            let mst = UnGraph::<_, _>::from_elements(minimum_spanning_tree(&graph));

            prop_assert_eq!(mst.node_count(), nodes);
        }
    }
}
