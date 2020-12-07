use super::algo::{BoundedMeasure, FloatMeasure};
use super::visit::{EdgeRef, IntoEdges, IntoNodeIdentifiers, NodeCount, NodeIndexable};
use crate::algo::NegativeCycle;
use std::collections::HashMap;
use std::hash::Hash;

/// Calculates the distance between any two nodes in the graph using the
/// \[Generic\] The Floyd-Warshall shortest paths algorithm. Calculates the
/// shortest path between each node in the graph in `O(V^3)` time, where `V` is
/// the number of nodes in the graph. Floyd-Warshall takes up `O(V^2)` space.
///
/// Details can be found on
/// [Wikipedia](https://en.wikipedia.org/wiki/Floyd–Warshall_algorithm).
///
/// Returns a PathCostMatrix that maps `(NodeId, NodeId)` to the cost of
/// traveling from the first node to the second.
///
/// # Example
/// ```rust
/// use petgraph::{Graph, algo::floyd_warshall};
///
/// let mut graph = Graph::new();
/// let a = graph.add_node(()); // node with no weight
/// let b = graph.add_node(());
/// let c = graph.add_node(());
/// let d = graph.add_node(());
/// let e = graph.add_node(());
/// let f = graph.add_node(());
/// let g = graph.add_node(());
/// let h = graph.add_node(());
/// // z will be in another connected component
/// let z = graph.add_node(());
///
/// graph.extend_with_edges(&[
///     (a, b, 3),
///     (b, c, 5),
///     (c, d, 2),
///     (d, a, 9),
///     (e, f, 1),
///     (b, e, -2),
///     (f, g, 8),
///     (g, h, -1),
///     (h, e, 0),
/// ]);
/// // a -----> b ----> e ------> f
/// // ^       |       ^       |
/// // |       v       |       v
/// // d <----- c       h <---- g
///
/// let costs = floyd_warshall(&graph).unwrap(); // handle negative cycle
/// assert_eq!(costs[(c,h)], 20);
/// assert_eq!(costs[(b,b)], 0);
/// assert_eq!(costs[(e,h)], 8);
///
/// // There is no valid path from e to b.
/// assert_eq!(costs[(e,b)], i32::MAX);
/// ```

pub fn floyd_warshall<G>(graph: G) -> Result<PathCostMatrix<G>, NegativeCycle>
where
    G: NodeCount + IntoNodeIdentifiers + IntoEdges + NodeIndexable,
    G::EdgeWeight: BoundedMeasure + std::fmt::Debug,
{
    let mut dist: PathCostMatrix<G> = PathCostMatrix::new(graph);
    for edge in graph.node_identifiers().flat_map(|i| graph.edges(i)) {
        dist[(edge.source(), edge.target())] = *edge.weight();
    }
    for vertex in graph.node_identifiers() {
        dist[(vertex, vertex)] = G::EdgeWeight::zero();
    }
    for k in graph.node_identifiers() {
        for i in graph.node_identifiers() {
            for j in graph.node_identifiers() {
                // Overflow guard.
                if dist[(i, k)] != G::EdgeWeight::infinite()
                    && dist[(k, j)] != G::EdgeWeight::infinite()
                {
                    if dist[(i, j)] > dist[(i, k)] + dist[(k, j)] {
                        dist[(i, j)] = dist[(i, k)] + dist[(k, j)]
                    }
                }
            }
        }
    }
    // A negative cycle was found.
    for k in graph.node_identifiers() {
        if dist[(k, k)] < G::EdgeWeight::zero() {
            return Err(NegativeCycle(()));
        }
    }

    Ok(dist)
}

/// Made to be used with with `floyd_warshall` algorithm. the cost of going from
/// each node to each other node.
///
/// ```rust
/// # use petgraph::{algo::floyd_warshall, Graph};
/// # use std::collections::HashMap;
/// # let mut get_graph = Graph::new();
/// # let node1 = get_graph.add_node(1);
/// # let node2 = get_graph.add_node(2);
/// # get_graph.extend_with_edges(&[(node1, node2, 1)]);
/// let g = get_graph;
/// let cost_matrix = floyd_warshall(&g).unwrap(); // Found negative cycle
///
/// assert_eq!(cost_matrix[(node1, node2)], 1);
///
/// // Or equivalently
///
/// let map: HashMap<_,_> = cost_matrix.into_hashmap();
/// assert_eq!(map.get(&(node1, node2)), Some(&1));
/// ```
pub struct PathCostMatrix<G>
where
    G: NodeIndexable + NodeCount + IntoNodeIdentifiers + IntoEdges,
    G::EdgeWeight: Clone,
{
    graph: G,
    weights: Vec<G::EdgeWeight>, // Single vector improves cache locality and prevents unnecessary allocations.
}

impl<G> PathCostMatrix<G>
where
    G: NodeIndexable + NodeCount + IntoNodeIdentifiers + IntoEdges,
    G::EdgeWeight: Clone + BoundedMeasure,
{
    /// Generates `PathWeightMatrix` from a graph. The matrix is initialized,
    /// setting all weights to infinity.
    pub(crate) fn new(graph: G) -> Self {
        let weights = vec![G::EdgeWeight::infinite(); graph.node_bound().pow(2)];
        PathCostMatrix { graph, weights }
    }
}
impl<G> PathCostMatrix<G>
where
    G: NodeIndexable + NodeCount + IntoNodeIdentifiers + IntoEdges,
    G::EdgeWeight: Clone + BoundedMeasure,
    G::NodeId: Eq + Hash,
{
    /// Converts a `PathWeightMatrix` into a hashmap where the keys are index
    /// pairs and values are thier associated trevel costs.
    pub fn into_hashmap(self) -> HashMap<(G::NodeId, G::NodeId), G::EdgeWeight> {
        self.graph
            .node_identifiers()
            .flat_map(|i1| self.graph.node_identifiers().map(move |i2| (i1, i2)))
            .map(|pair| (pair, self[pair]))
            .collect()
    }
}

impl<G> std::ops::Index<(G::NodeId, G::NodeId)> for PathCostMatrix<G>
where
    G: NodeIndexable + NodeCount + IntoNodeIdentifiers + IntoEdges,
    G::EdgeWeight: Clone,
{
    type Output = G::EdgeWeight;
    fn index(&self, index: (G::NodeId, G::NodeId)) -> &Self::Output {
        let index_0 = self.graph.to_index(index.0);
        let index_1 = self.graph.to_index(index.1);
        self.weights
            .index(index_0 * self.graph.node_bound() + index_1)
    }
}

impl<G> std::ops::IndexMut<(G::NodeId, G::NodeId)> for PathCostMatrix<G>
where
    G: NodeIndexable + NodeCount + IntoNodeIdentifiers + IntoEdges,
    G::EdgeWeight: Clone,
{
    fn index_mut(&mut self, index: (G::NodeId, G::NodeId)) -> &mut Self::Output {
        let index_0 = self.graph.to_index(index.0);
        let index_1 = self.graph.to_index(index.1);
        self.weights
            .index_mut(index_0 * self.graph.node_bound() + index_1)
    }
}

impl<G> Clone for PathCostMatrix<G>
where
    G::EdgeWeight: Clone,
    G: Clone + IntoNodeIdentifiers + NodeIndexable + NodeCount + IntoEdges,
{
    fn clone(&self) -> Self {
        Self {
            graph: self.graph.clone(),
            weights: self.weights.clone(),
        }
    }
}
