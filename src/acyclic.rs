//! A wrapper around graph types that enforces an acyclicity invariant.

use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
};

use crate::{
    algo::Cycle,
    data::{Build, Create, DataMap, DataMapMut},
    visit::{
        dfs_visitor, Control, Data, DfsEvent, EdgeCount, EdgeIndexable, GetAdjacencyMatrix,
        GraphBase, GraphProp, IntoEdgeReferences, IntoEdges, IntoEdgesDirected, IntoNeighbors,
        IntoNeighborsDirected, IntoNodeIdentifiers, IntoNodeReferences, NodeCompactIndexable,
        NodeCount, NodeIndexable, Reversed, Time, Visitable,
    },
    Direction,
};

mod order_map;
use order_map::OrderMap;

/// A directed acyclic graph.
///
/// Wrap directed acyclic graphs and expose an API that ensures the invariant
/// is maintained, i.e. no cycles can be created. This uses a topological order
/// that is dynamically updated when edges are added. In the worst case, the
/// runtime may be linear in the number of vertices, but it has been shown to
/// be fast in practice, particularly on sparse graphs (Pierce and Kelly, 2004).
///
/// To be modifiable (and hence to be useful), the graphs of generic type `G`
/// should implement the [`Build`] trait. Good candidates for `G` are thus
/// [`crate::graph::DiGraph`] and [`crate::stable_graph::StableDiGraph`].
///
/// ## Algorithm
/// This implements the PK algorithm for dynamic topological sort described in
/// "A Dynamic Topological Sort Algorithm for Directed Acyclic Graphs" by
/// D. Pierce and P. Kelly, JEA, 2004. It maintains a topological order of the
/// nodes that can be efficiently updated when edges are added. Achieves a good
/// balance between simplicity and performance in practice, see the paper for
/// discussions of the running time.
///
/// ## Graph traits
/// All graph traits are delegated to the inner graph, with the exception of
/// the graph construction trait [`Build`]. The wrapped graph can thus only
/// be modified through the wrapped API that ensures no cycles are created.
///
/// ## Behaviour on cycles
/// By design, edge additions to this datatype may fail. It is recommended to
/// prefer the dedicated [`Acyclic::try_add_edge`] method whenever possible. The
/// [`Build::update_edge`] methods will panic if it is attempted to add an edge
/// that would create a cycle. The [`Build::add_edge`] on the other hand method
/// will return `None` if the edge cannot be added (either it already exists on
/// a graph type that does not support it or would create a cycle).
#[derive(Clone, Debug)]
pub struct Acyclic<G: Visitable> {
    /// The underlying graph, accessible through the `inner` method.
    graph: G,
    /// The current topological order of the nodes.
    order_map: OrderMap<G::NodeId>,
    /// Helper map for DFS tracking discovered nodes.
    discovered: G::Map,
    /// Helper map for DFS tracking finished nodes.
    finished: G::Map,
}

impl<G: Visitable> Acyclic<G> {
    /// Create a new empty acyclic graph.
    pub fn new() -> Self
    where
        G: Default,
    {
        Default::default()
    }

    /// Get the current topological order of the nodes.
    pub fn order(&self) -> &[G::NodeId] {
        self.order_map.as_slice()
    }

    /// Get the underlying graph.
    pub fn inner(&self) -> &G {
        &self.graph
    }

    /// Get the underlying graph mutably.
    pub fn inner_mut(&mut self) -> &mut G {
        &mut self.graph
    }

    /// Consume the `Acyclic` wrapper and return the underlying graph.
    pub fn into_inner(self) -> G {
        self.graph
    }

    fn reset_maps(&mut self) {
        self.graph.reset_map(&mut self.discovered);
        self.graph.reset_map(&mut self.finished);
    }
}

impl<G: Visitable + NodeIndexable> Acyclic<G>
where
    for<'a> &'a G: IntoNeighborsDirected
        + IntoNodeIdentifiers
        + Visitable<Map = G::Map>
        + GraphBase<NodeId = G::NodeId>,
{
    /// Wrap a graph into an acyclic graph.
    pub fn try_from_graph(graph: G) -> Result<Self, Cycle<G::NodeId>> {
        let order_map = OrderMap::try_from_graph(&graph)?;
        let discovered = graph.visit_map();
        let finished = graph.visit_map();
        Ok(Self {
            graph,
            order_map,
            discovered,
            finished,
        })
    }

    /// Add an edge to the graph.
    ///
    /// Returns the id of the added edge, or a [`Cycle`] error if the edge would
    /// create a cycle.
    pub fn try_add_edge(
        &mut self,
        a: G::NodeId,
        b: G::NodeId,
        weight: G::EdgeWeight,
    ) -> Result<G::EdgeId, Cycle<G::NodeId>>
    where
        G: Build,
    {
        if a == b {
            // No self-loops allowed
            return Err(Cycle(a));
        }
        self.update_ordering(a, b)?;
        Ok(self.graph.update_edge(a, b, weight))
    }

    /// Update the ordering of the nodes in the order map resulting from adding an
    /// edge a -> b.
    ///
    /// If a cycle is detected, an error is returned and `self` remains unchanged.
    ///
    /// Implements the core update logic of the PK algorithm.
    fn update_ordering(&mut self, a: G::NodeId, b: G::NodeId) -> Result<(), Cycle<G::NodeId>> {
        let min_order = self.get_order(b);
        let max_order = self.get_order(a);
        if min_order >= max_order {
            // Order is already correct
            return Ok(());
        }

        // Reset the maps to clear any previous state (and resize if needed)
        self.reset_maps();

        // Get all nodes reachable from b with min_order <= order < max_order
        let b_fut = dfs(
            &self.graph,
            b,
            &self.order_map,
            |order| {
                debug_assert!(order >= min_order, "invalid topological order");
                match order.cmp(&max_order) {
                    Ordering::Less => Ok(true),       // node within b_fut
                    Ordering::Equal => Err(Cycle(a)), // cycle!
                    Ordering::Greater => Ok(false),   // node beyond b_fut
                }
            },
            &mut self.discovered,
            &mut self.finished,
        )?;
        // Get all remaining nodes that can reach a with min_order < order <= max_order
        let a_past = dfs(
            Reversed(&self.graph),
            a,
            &self.order_map,
            |order| {
                debug_assert!(order <= max_order, "invalid topological order");
                match order.cmp(&min_order) {
                    Ordering::Less => Ok(false),      // node beyond a_past
                    Ordering::Equal => Err(Cycle(b)), // cycle!
                    Ordering::Greater => Ok(true),    // node within a_past
                }
            },
            &mut self.discovered,
            &mut self.finished,
        )?;

        // Now reorder of nodes in a_past and b_fut such that
        //  i) within each vec, the nodes are in topological order,
        // ii) all elements of b_fut come before all elements of a_past in the new order.
        let all_positions: BTreeSet<_> = b_fut.keys().chain(a_past.keys()).copied().collect();
        let all_nodes = a_past.values().chain(b_fut.values()).copied();

        debug_assert_eq!(all_positions.len(), b_fut.len() + a_past.len());

        for (pos, node) in all_positions.into_iter().zip(all_nodes) {
            self.order_map.set_order(node, pos, &self.graph);
        }
        Ok(())
    }
}

impl<G: Visitable> GraphBase for Acyclic<G> {
    type NodeId = G::NodeId;
    type EdgeId = G::EdgeId;
}

impl<G: Default + Visitable> Default for Acyclic<G> {
    fn default() -> Self {
        let graph: G = Default::default();
        let order_map = Default::default();
        let discovered = graph.visit_map();
        let finished = graph.visit_map();
        Self {
            graph,
            order_map,
            discovered,
            finished,
        }
    }
}

impl<G: Build + Visitable + NodeIndexable> Build for Acyclic<G>
where
    for<'a> &'a G: IntoNeighborsDirected
        + IntoNodeIdentifiers
        + Visitable<Map = G::Map>
        + GraphBase<NodeId = G::NodeId>,
{
    fn add_node(&mut self, weight: Self::NodeWeight) -> Self::NodeId {
        let n = self.graph.add_node(weight);
        self.order_map.add_node(n, &self.graph);
        n
    }

    fn add_edge(
        &mut self,
        a: Self::NodeId,
        b: Self::NodeId,
        weight: Self::EdgeWeight,
    ) -> Option<Self::EdgeId> {
        self.try_add_edge(a, b, weight).ok()
    }

    fn update_edge(
        &mut self,
        a: Self::NodeId,
        b: Self::NodeId,
        weight: Self::EdgeWeight,
    ) -> Self::EdgeId {
        self.try_add_edge(a, b, weight).ok().unwrap()
    }
}

impl<G: Create + Visitable + NodeIndexable> Create for Acyclic<G>
where
    for<'a> &'a G: IntoNeighborsDirected
        + IntoNodeIdentifiers
        + Visitable<Map = G::Map>
        + GraphBase<NodeId = G::NodeId>,
{
    fn with_capacity(nodes: usize, edges: usize) -> Self {
        let graph = G::with_capacity(nodes, edges);
        let order_map = OrderMap::with_capacity(nodes);
        let discovered = graph.visit_map();
        let finished = graph.visit_map();
        Self {
            graph,
            order_map,
            discovered,
            finished,
        }
    }
}

/// Traverse nodes in `graph` in DFS order, starting from `start`, for as long
/// as the predicate `valid_order` returns `true` on the current node's order.
fn dfs<G: NodeIndexable + IntoNeighborsDirected + IntoNodeIdentifiers + Visitable>(
    graph: G,
    start: G::NodeId,
    order_map: &OrderMap<G::NodeId>,
    // A predicate that returns whether to continue the search from a node,
    // or an error to stop and shortcircuit the search.
    mut valid_order: impl FnMut(usize) -> Result<bool, Cycle<G::NodeId>>,
    discovered: &mut G::Map,
    finished: &mut G::Map,
) -> Result<BTreeMap<usize, G::NodeId>, Cycle<G::NodeId>> {
    let mut res = BTreeMap::new();
    dfs_visitor(
        graph,
        start,
        &mut |ev| -> Result<Control<()>, Cycle<G::NodeId>> {
            let DfsEvent::Discover(u, _) = ev else {
                return Ok(Control::Continue);
            };
            let order = order_map.get_order(u, &graph);
            match valid_order(order)? {
                true => {
                    res.insert(order, u);
                    Ok(Control::Continue)
                }
                false => Ok(Control::Prune),
            }
        },
        discovered,
        finished,
        &mut Time::default(),
    )?;
    Ok(res)
}

/////////////////////// Pass-through graph traits ///////////////////////
// We implement all the following traits by delegating to the inner graph:
// - Data
// - DataMap
// - DataMapMut
// - EdgeCount
// - EdgeIndexable
// - GetAdjacencyMatrix
// - GraphProp
// - NodeCompactIndexable
// - NodeCount
// - NodeIndexable
// - Visitable
//
// Further, we also implement the following traits that act on references to
// graphs `&G`:
// - IntoEdgeReferences
// - IntoEdges
// - IntoEdgesDirected
// - IntoNeighbors
// - IntoNeighborsDirected
// - IntoNodeIdentifiers
// - IntoNodeReferences

impl<G: Visitable + Data> Data for Acyclic<G> {
    type NodeWeight = G::NodeWeight;
    type EdgeWeight = G::EdgeWeight;
}

impl<G: Visitable + DataMap> DataMap for Acyclic<G> {
    fn node_weight(self: &Self, id: Self::NodeId) -> Option<&Self::NodeWeight> {
        self.inner().node_weight(id)
    }

    fn edge_weight(self: &Self, id: Self::EdgeId) -> Option<&Self::EdgeWeight> {
        self.inner().edge_weight(id)
    }
}

impl<G: Visitable + DataMapMut> DataMapMut for Acyclic<G> {
    fn node_weight_mut(self: &mut Self, id: Self::NodeId) -> Option<&mut Self::NodeWeight> {
        self.inner_mut().node_weight_mut(id)
    }

    fn edge_weight_mut(self: &mut Self, id: Self::EdgeId) -> Option<&mut Self::EdgeWeight> {
        self.inner_mut().edge_weight_mut(id)
    }
}

impl<G: Visitable + EdgeCount> EdgeCount for Acyclic<G> {
    fn edge_count(self: &Self) -> usize {
        self.inner().edge_count()
    }
}

impl<G: Visitable + EdgeIndexable> EdgeIndexable for Acyclic<G> {
    fn edge_bound(self: &Self) -> usize {
        self.inner().edge_bound()
    }

    fn to_index(self: &Self, a: Self::EdgeId) -> usize {
        self.inner().to_index(a)
    }

    fn from_index(self: &Self, i: usize) -> Self::EdgeId {
        self.inner().from_index(i)
    }
}

impl<G: Visitable + GetAdjacencyMatrix> GetAdjacencyMatrix for Acyclic<G> {
    type AdjMatrix = G::AdjMatrix;

    fn adjacency_matrix(self: &Self) -> Self::AdjMatrix {
        self.inner().adjacency_matrix()
    }

    fn is_adjacent(
        self: &Self,
        matrix: &Self::AdjMatrix,
        a: Self::NodeId,
        b: Self::NodeId,
    ) -> bool {
        self.inner().is_adjacent(matrix, a, b)
    }
}

impl<G: Visitable + GraphProp> GraphProp for Acyclic<G> {
    type EdgeType = G::EdgeType;
}

impl<G: Visitable + NodeCompactIndexable> NodeCompactIndexable for Acyclic<G> {}

impl<G: Visitable + NodeCount> NodeCount for Acyclic<G> {
    fn node_count(self: &Self) -> usize {
        self.inner().node_count()
    }
}

impl<G: Visitable + NodeIndexable> NodeIndexable for Acyclic<G> {
    fn node_bound(self: &Self) -> usize {
        self.inner().node_bound()
    }

    fn to_index(self: &Self, a: Self::NodeId) -> usize {
        self.inner().to_index(a)
    }

    fn from_index(self: &Self, i: usize) -> Self::NodeId {
        self.inner().from_index(i)
    }
}

impl<G: Visitable> Visitable for Acyclic<G> {
    type Map = G::Map;

    fn visit_map(self: &Self) -> Self::Map {
        self.inner().visit_map()
    }

    fn reset_map(self: &Self, map: &mut Self::Map) {
        self.inner().reset_map(map)
    }
}

impl<'a, G: Visitable + Data> IntoEdgeReferences for &'a Acyclic<G>
where
    &'a G: IntoEdgeReferences<
        NodeId = G::NodeId,
        EdgeId = G::EdgeId,
        NodeWeight = G::NodeWeight,
        EdgeWeight = G::EdgeWeight,
    >,
{
    type EdgeRef = <&'a G as IntoEdgeReferences>::EdgeRef;
    type EdgeReferences = <&'a G as IntoEdgeReferences>::EdgeReferences;

    fn edge_references(self) -> Self::EdgeReferences {
        self.inner().edge_references()
    }
}

impl<'a, G: Visitable + Data> IntoEdges for &'a Acyclic<G>
where
    &'a G: IntoEdges<
        NodeId = G::NodeId,
        EdgeId = G::EdgeId,
        NodeWeight = G::NodeWeight,
        EdgeWeight = G::EdgeWeight,
    >,
{
    type Edges = <&'a G as IntoEdges>::Edges;

    fn edges(self, a: Self::NodeId) -> Self::Edges {
        self.inner().edges(a)
    }
}

impl<'a, G: Visitable + Data> IntoEdgesDirected for &'a Acyclic<G>
where
    &'a G: IntoEdgesDirected<
        NodeId = G::NodeId,
        EdgeId = G::EdgeId,
        NodeWeight = G::NodeWeight,
        EdgeWeight = G::EdgeWeight,
    >,
{
    type EdgesDirected = <&'a G as IntoEdgesDirected>::EdgesDirected;

    fn edges_directed(self, a: Self::NodeId, dir: Direction) -> Self::EdgesDirected {
        self.inner().edges_directed(a, dir)
    }
}

impl<'a, G: Visitable> IntoNeighbors for &'a Acyclic<G>
where
    &'a G: IntoNeighbors<NodeId = G::NodeId>,
{
    type Neighbors = <&'a G as IntoNeighbors>::Neighbors;

    fn neighbors(self, a: Self::NodeId) -> Self::Neighbors {
        self.inner().neighbors(a)
    }
}

impl<'a, G: Visitable> IntoNeighborsDirected for &'a Acyclic<G>
where
    &'a G: IntoNeighborsDirected<NodeId = G::NodeId>,
{
    type NeighborsDirected = <&'a G as IntoNeighborsDirected>::NeighborsDirected;

    fn neighbors_directed(self, n: Self::NodeId, d: Direction) -> Self::NeighborsDirected {
        self.inner().neighbors_directed(n, d)
    }
}

impl<'a, G: Visitable> IntoNodeIdentifiers for &'a Acyclic<G>
where
    &'a G: IntoNodeIdentifiers<NodeId = G::NodeId>,
{
    type NodeIdentifiers = <&'a G as IntoNodeIdentifiers>::NodeIdentifiers;

    fn node_identifiers(self) -> Self::NodeIdentifiers {
        self.inner().node_identifiers()
    }
}

impl<'a, G: Visitable> IntoNodeReferences for &'a Acyclic<G>
where
    &'a G: IntoNodeReferences,
{
    type NodeRef = <&'a G as IntoNodeReferences>::NodeRef;

    type NodeReferences = <&'a G as IntoNodeReferences>::NodeReferences;

    fn node_references(self) -> Self::NodeReferences {
        self.inner().node_references()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::DiGraph;
    use crate::visit::IntoNodeReferences;

    #[test]
    fn test_acyclic_graph() {
        // Create an acyclic DiGraph
        let mut graph = DiGraph::<(), ()>::new();
        let a = graph.add_node(());
        let c = graph.add_node(());
        let b = graph.add_node(());
        graph.add_edge(a, b, ());
        graph.add_edge(b, c, ());

        // Create an Acyclic object
        let mut acyclic = Acyclic::try_from_graph(graph).unwrap();

        // Test initial topological order
        assert_valid_topological_order(&acyclic);

        // Add a valid edge
        assert!(acyclic.try_add_edge(a, c, ()).is_ok());
        assert_valid_topological_order(&acyclic);

        // Try to add an edge that would create a cycle
        assert!(acyclic.try_add_edge(c, a, ()).is_err());

        // Add another valid edge
        let d = acyclic.add_node(());
        assert!(acyclic.try_add_edge(c, d, ()).is_ok());
        assert_valid_topological_order(&acyclic);

        // Try to add an edge that would create a cycle (using the Build trait)
        assert!(acyclic.add_edge(d, a, ()).is_none());
    }

    fn assert_valid_topological_order<'a, G>(acyclic: &'a Acyclic<G>)
    where
        G: Visitable,
        &'a G: NodeIndexable
            + IntoNodeReferences
            + IntoNeighborsDirected
            + GraphBase<NodeId = G::NodeId>,
    {
        let order = acyclic.order();
        for (idx, &node) in order.iter().enumerate() {
            for neighbor in acyclic.inner().neighbors(node) {
                assert!(order.iter().position(|&n| n == neighbor).unwrap() > idx);
            }
        }
    }
}
