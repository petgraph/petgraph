//! A wrapper around graph types that enforces an acyclicity invariant.

use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
};

use crate::{
    algo::Cycle,
    data::{Build, Create, DataMap, DataMapMut},
    visit::{
        depth_first_search, Control, Data, DfsEvent, EdgeCount, EdgeIndexable, GetAdjacencyMatrix,
        GraphBase, GraphProp, IntoEdgeReferences, IntoEdges, IntoEdgesDirected, IntoNeighbors,
        IntoNeighborsDirected, IntoNodeIdentifiers, IntoNodeReferences, NodeCompactIndexable,
        NodeCount, NodeIndexable, Reversed, Visitable,
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
pub struct Acyclic<G: GraphBase>(G, OrderMap<G::NodeId>);

impl<G: GraphBase> Acyclic<G> {
    /// Create a new empty acyclic graph.
    pub fn new() -> Self
    where
        G: Default,
    {
        Default::default()
    }

    /// Get the current topological order of the nodes.
    pub fn order(&self) -> &[G::NodeId] {
        self.1.as_slice()
    }

    /// Get the underlying graph.
    pub fn inner(&self) -> &G {
        &self.0
    }

    /// Consume the `Acyclic` wrapper and return the underlying graph.
    pub fn into_inner(self) -> G {
        self.0
    }

    /// Get a reference to the underlying graph.
    pub fn as_ref(&self) -> Acyclic<&G> {
        Acyclic(&self.0, self.1.clone())
    }
}

impl<G: GraphBase> Acyclic<G>
where
    for<'a> &'a G: NodeIndexable
        + IntoNeighborsDirected
        + IntoNodeIdentifiers
        + Visitable
        + GraphBase<NodeId = G::NodeId>,
{
    /// Wrap a graph into an acyclic graph.
    pub fn try_from_graph(graph: G) -> Result<Self, Cycle<G::NodeId>> {
        let order_map = OrderMap::try_from_graph(&graph)?;
        Ok(Self(graph, order_map))
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
        update_ordering(&mut self.1, a, b, &self.0)?;
        Ok(self.0.update_edge(a, b, weight))
    }
}

impl<G: GraphBase> GraphBase for Acyclic<G> {
    type NodeId = G::NodeId;
    type EdgeId = G::EdgeId;
}

impl<G: Default + GraphBase> Default for Acyclic<G> {
    fn default() -> Self {
        Self(Default::default(), Default::default())
    }
}

impl<G: Build> Build for Acyclic<G>
where
    for<'a> &'a G: NodeIndexable
        + IntoNeighborsDirected
        + IntoNodeIdentifiers
        + Visitable
        + GraphBase<NodeId = G::NodeId>,
{
    fn add_node(&mut self, weight: Self::NodeWeight) -> Self::NodeId {
        let n = self.0.add_node(weight);
        self.1.add_node(n, &self.0);
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

impl<G: Create + GraphBase> Create for Acyclic<G>
where
    for<'a> &'a G: NodeIndexable
        + IntoNeighborsDirected
        + IntoNodeIdentifiers
        + Visitable
        + GraphBase<NodeId = G::NodeId>,
{
    fn with_capacity(nodes: usize, edges: usize) -> Self {
        let graph = G::with_capacity(nodes, edges);
        let order_map = OrderMap::default();
        Self(graph, order_map)
    }
}

/// Update the ordering of the nodes in the order map resulting from adding an
/// edge a -> b.
///
/// Implements the core update logic of the PK algorithm.
fn update_ordering<G: NodeIndexable + IntoNeighborsDirected + IntoNodeIdentifiers + Visitable>(
    order: &mut OrderMap<G::NodeId>,
    a: G::NodeId,
    b: G::NodeId,
    graph: G,
) -> Result<(), Cycle<G::NodeId>> {
    let min_order = order.get_order(b, graph);
    let max_order = order.get_order(a, graph);
    if min_order >= max_order {
        // Order is already correct
        return Ok(());
    }
    // Get all nodes reachable from b with min_order <= order < max_order
    let b_fut = dfs(graph, b, order, |order| {
        debug_assert!(order >= min_order, "invalid topological order");
        match order.cmp(&max_order) {
            Ordering::Less => Ok(true),       // node within b_fut
            Ordering::Equal => Err(Cycle(a)), // cycle!
            Ordering::Greater => Ok(false),   // node beyond b_fut
        }
    })?;
    // Get all remaining nodes that can reach a with min_order < order <= max_order
    let a_past = dfs(Reversed(graph), a, order, |order| {
        debug_assert!(order <= max_order, "invalid topological order");
        if b_fut.contains_key(&order) {
            return Ok(false);
        }
        match order.cmp(&min_order) {
            Ordering::Less => Ok(false),      // node beyond a_past
            Ordering::Equal => Err(Cycle(b)), // cycle!
            Ordering::Greater => Ok(true),    // node within a_past
        }
    })?;

    // Now reorder of nodes in a_past and b_fut such that
    //  i) within each vec, the nodes are in topological order,
    // ii) all elements of b_fut come before all elements of a_past in the new order.
    let all_positions: BTreeSet<_> = b_fut.keys().chain(a_past.keys()).copied().collect();
    let all_nodes = a_past.values().chain(b_fut.values()).copied();

    debug_assert_eq!(all_positions.len(), b_fut.len() + a_past.len());

    for (pos, node) in all_positions.into_iter().zip(all_nodes) {
        order.set_order(node, pos, graph);
    }
    Ok(())
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
) -> Result<BTreeMap<usize, G::NodeId>, Cycle<G::NodeId>> {
    let mut res = BTreeMap::new();
    depth_first_search(
        graph,
        Some(start),
        |ev| -> Result<Control<()>, Cycle<G::NodeId>> {
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
    )?;
    Ok(res)
}

/////////////////////// Pass-through graph traits ///////////////////////
Data! {delegate_impl [[G], G, Acyclic<G>, access0]}
DataMap! {delegate_impl [[G], G, Acyclic<G>, access0]}
DataMapMut! {delegate_impl [[G], G, Acyclic<G>, access0]}
EdgeCount! {delegate_impl [[G], G, Acyclic<G>, access0]}
EdgeIndexable! {delegate_impl [[G], G, Acyclic<G>, access0]}
GetAdjacencyMatrix! {delegate_impl [[G], G, Acyclic<G>, access0]}
GraphProp! {delegate_impl [[G], G, Acyclic<G>, access0]}
NodeCompactIndexable! {delegate_impl [[G], G, Acyclic<G>, access0]}
NodeCount! {delegate_impl [[G], G, Acyclic<G>, access0]}
NodeIndexable! {delegate_impl [[G], G, Acyclic<G>, access0]}
Visitable! {delegate_impl [[G], G, Acyclic<G>, access0]}

impl<'a, G: GraphBase + Data> IntoEdgeReferences for &'a Acyclic<G>
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

impl<'a, G: GraphBase + Data> IntoEdges for &'a Acyclic<G>
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

impl<'a, G: GraphBase + Data> IntoEdgesDirected for &'a Acyclic<G>
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

impl<'a, G: GraphBase> IntoNeighbors for &'a Acyclic<G>
where
    &'a G: IntoNeighbors<NodeId = G::NodeId>,
{
    type Neighbors = <&'a G as IntoNeighbors>::Neighbors;

    fn neighbors(self, a: Self::NodeId) -> Self::Neighbors {
        self.inner().neighbors(a)
    }
}

impl<'a, G: GraphBase> IntoNeighborsDirected for &'a Acyclic<G>
where
    &'a G: IntoNeighborsDirected<NodeId = G::NodeId>,
{
    type NeighborsDirected = <&'a G as IntoNeighborsDirected>::NeighborsDirected;

    fn neighbors_directed(self, n: Self::NodeId, d: Direction) -> Self::NeighborsDirected {
        self.inner().neighbors_directed(n, d)
    }
}

impl<'a, G: GraphBase> IntoNodeIdentifiers for &'a Acyclic<G>
where
    &'a G: IntoNodeIdentifiers<NodeId = G::NodeId>,
{
    type NodeIdentifiers = <&'a G as IntoNodeIdentifiers>::NodeIdentifiers;

    fn node_identifiers(self) -> Self::NodeIdentifiers {
        self.inner().node_identifiers()
    }
}

impl<'a, G: GraphBase> IntoNodeReferences for &'a Acyclic<G>
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
        G: GraphBase,
        &'a G: NodeIndexable
            + IntoNodeReferences
            + IntoNeighborsDirected
            + GraphBase<NodeId = G::NodeId>,
    {
        let order = acyclic.order();
        for (idx, &node) in order.iter().enumerate() {
            for neighbor in acyclic.neighbors(node) {
                assert!(order.iter().position(|&n| n == neighbor).unwrap() > idx);
            }
        }
    }
}
