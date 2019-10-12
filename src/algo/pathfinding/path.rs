use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};

use std::hash::{Hash, BuildHasher};

use std::ops::Index;

use std::marker::PhantomData;

use crate::algo::{Measure, FloatMeasure};
use crate::visit::{GraphBase, NodeIndexable};

/// Path returned by a pathfinding operation, such as
/// [`dijkstra`](fn.dijkstra.html).
///
/// This struct only exists as a convenience for the
/// [`into_nodes`](struct.Path.html#method.into_nodes) method, you can always
/// [`unpack`](struct.Path.html#method.unpack) its members.
pub struct Path<G, K, C, P>
    where G: GraphBase,
          K: Measure + Copy,
          C: CostMap<G, Cost = K>,
          P: PredecessorMap<G>,
{
    goal: Option<G::NodeId>,
    predecessors: P,
    costs: C,
    k: PhantomData<K>,
}

impl<G, K, C, P> Path<G, K, C, P>
    where G: GraphBase,
          K: Measure + Copy,
          C: CostMap<G, Cost = K>,
          P: PredecessorMap<G>,
{
    /// Construct a path from its members.
    pub fn new(predecessors: P, costs: C, goal: Option<G::NodeId>) -> Self {
        Path {
            goal: goal,
            predecessors: predecessors,
            costs: costs,
            k: PhantomData,
        }
    }

    /// Get the predecessor map.
    pub fn predecessors(&self) -> &P {
        &self.predecessors
    }

    /// Get the cost map.
    pub fn costs(&self) -> &C {
        &self.costs
    }

    /// Extract the path's predecessor map.
    pub fn into_predecessors(self) -> P {
        self.predecessors
    }

    /// Extract the path's cost map.
    pub fn into_costs(self) -> C {
        self.costs
    }

    /// Unpack the path's members as a tuple.
    pub fn unpack(self) -> (C, P, Option<G::NodeId>) {
        (self.costs, self.predecessors, self.goal)
    }
}

impl<G, K, C, P> Path<G, K, C, P>
    where G: GraphBase,
          K: Measure + Copy,
          C: CostMap<G, Cost = K>,
          P: PredecessorMap<G> + PredecessorMapConfigured<G>,
{
    /// Compute the list of nodes from the start node to the goal node (both included), along with
    /// the total cost of this path.
    ///
    /// Note that this method is only available if the predecessor map was explicitly given when
    /// configuring the pathfinding algorithm.
    pub fn into_nodes(self) -> Option<(K, Vec<G::NodeId>)> {
        self.goal
            .map(|node| {
                let total_cost = self.costs.get(&node).unwrap().clone();
                let mut path = vec![node];

                let mut current = node;
                while let Some(previous) = self.predecessors.get(current) {
                    path.push(previous);
                    current = previous;
                }

                path.reverse();

                (total_cost, path)
            })
    }
}

pub trait PredecessorMap<G: GraphBase> {
    /// Perform any initialization code for the given graph, necessarily called at the beginning of
    /// each pathfinding algorithm.
    fn initialize(&mut self, graph: G);

    /// Look up the predecessor for a node.
    fn get(&self, node: G::NodeId) -> Option<G::NodeId>;

    /// Set the predecessor for a node.
    fn set(&mut self, node: G::NodeId, pred: G::NodeId);
}

/// Marker trait used to differentiate explicitly configured predecessor maps from the default.
/// This is needed to cause a compile-time error when the user tries to rebuild the path without
/// configuring a predecessor map, which would likely cause a panic.
pub trait PredecessorMapConfigured<G: GraphBase> : PredecessorMap<G> {
}

pub trait CostMap<G: GraphBase> {
    /// The cost stored in this map.
    type Cost: Measure;

    /// Perform any initialization code for the given graph, necessarily called at the beginning of
    /// each pathfinding algorithm.
    fn initialize(&mut self, graph: G, node: G::NodeId);

    /// Look up the cost for a node.
    fn get(&self, node: &G::NodeId) -> Option<&Self::Cost>;

    /// Look up the cost for a node, returning
    /// [`infinite()`](../../trait.FloatMeasure.html#tymethod.infinite) by default.
    ///
    /// If your map uses a sentinal value to represent absence of a cost, or if the default value
    /// of the cost in your map is infinity, you can override this method for a small speed bump in
    /// [`bellman_ford`](../fn.bellman_ford.html).
    fn get_or_infinite(&self, node: &G::NodeId) -> Self::Cost
        where Self::Cost: FloatMeasure,
    {
        self.get(node).cloned().unwrap_or(Self::Cost::infinite())
    }

    /// Consider the provided cost for insertion. If the cost is lower than the current cost in the
    /// map, or if there's no cost for this node, insert it.
    ///
    /// Returns if the given cost was indeed inserted.
    ///
    /// Note: the method is in this form for optimal use with any entry API.
    fn consider(&mut self, node: G::NodeId, cost: Self::Cost) -> bool;
}

impl<G, S> PredecessorMap<G> for HashMap<G::NodeId, G::NodeId, S>
    where G: GraphBase,
          G::NodeId: Eq + Hash,
          S: BuildHasher,
{
    fn initialize(&mut self, _graph: G) {
        self.clear();
    }

    fn get(&self, node: G::NodeId) -> Option<G::NodeId> {
        self.get(&node).map(|n| n.clone())
    }

    fn set(&mut self, node: G::NodeId, pred: G::NodeId) {
        self.insert(node, pred);
    }
}

impl<G, S> PredecessorMapConfigured<G> for HashMap<G::NodeId, G::NodeId, S>
    where G: GraphBase,
          G::NodeId: Eq + Hash,
          S: BuildHasher,
{
}

impl<G, K, S> CostMap<G> for HashMap<G::NodeId, K, S>
    where G: GraphBase,
          G::NodeId: Eq + Hash,
          K: Measure,
          S: BuildHasher,
{
    type Cost = K;

    fn initialize(&mut self, _graph: G, node: G::NodeId) {
        self.clear();
        self.insert(node, <_>::default());
    }

    fn get(&self, node: &G::NodeId) -> Option<&Self::Cost> {
        self.get(node)
    }

    fn consider(&mut self, node: G::NodeId, cost: Self::Cost) -> bool {
        match self.entry(node) {
            Occupied(ent) => {
                {
                    let ref old = *ent.get();
                    if !(cost < *old) {
                        return false;
                    }
                }
                *ent.into_mut() = cost;
            }
            Vacant(ent) => {
                ent.insert(cost);
            }
        }

        true
    }
}

/// A mapping for all nodes in the graph, useful for predecessors/costs.
///
/// The only requirement for using this is that the graph should be
/// [`NodeIndexable`](../../visit/trait.NodeIndexable.html), which is typically the case for
/// [`Graph`](../../graph/struct.Graph.html).
///
/// This is a drop-in replacement for `HashMap` when configuring pathfinding algorithms,
/// sacrificing memory for a significant speedup for pathfinding on medium to large graphs.
///
/// There are two implementations for [`CostMap`](traits/trait.CostMap.html), one being optimized
/// for [`FloatMeasure`](../trait.FloatMeasure.html) costs.
pub struct IndexableNodeMap<G, T>
    where G: GraphBase + NodeIndexable,
{
    graph: Option<G>, // a bit hacky, but works nonetheless
    node_map: Vec<T>,
}

impl<G, T> IndexableNodeMap<G, T>
    where G: GraphBase + NodeIndexable,
{
    pub fn new() -> Self {
        IndexableNodeMap {
            graph: None,
            node_map: vec![],
        }
    }

    fn ix(&self, node: G::NodeId) -> usize {
        self.graph.as_ref().unwrap().to_index(node)
    }

    pub fn into_node_map(self) -> Vec<T> {
        self.node_map
    }
}

impl<G, T: Clone> IndexableNodeMap<G, T>
    where G: GraphBase + NodeIndexable,
{
    fn initialize(&mut self, graph: G, value: T) {
        self.graph = Some(graph);
        self.node_map = vec![value; self.graph.as_ref().unwrap().node_bound()];
    }
}

impl<'a, G, T> Index<&'a G::NodeId> for IndexableNodeMap<G, T>
    where G: GraphBase + NodeIndexable,
{
    type Output = T;

    fn index(&self, node: &G::NodeId) -> &Self::Output {
        self.node_map.index(self.ix(*node))
    }
}

impl<G> PredecessorMap<G> for IndexableNodeMap<G, Option<G::NodeId>>
    where G: GraphBase + NodeIndexable,
{
    fn initialize(&mut self, graph: G) {
        self.initialize(graph, None);
    }

    fn get(&self, node: G::NodeId) -> Option<G::NodeId> {
        self.node_map[self.ix(node)]
    }

    fn set(&mut self, node: G::NodeId, pred: G::NodeId) {
        let ix = self.ix(node);
        self.node_map[ix] = Some(pred);
    }
}

impl<G> PredecessorMapConfigured<G> for IndexableNodeMap<G, Option<G::NodeId>>
    where G: GraphBase + NodeIndexable,
{
}

impl<G, K> CostMap<G> for IndexableNodeMap<G, K>
    where G: GraphBase + NodeIndexable,
          K: FloatMeasure,
{
    type Cost = K;

    fn initialize(&mut self, graph: G, node: G::NodeId) {
        self.initialize(graph, K::infinite());
        let ix = self.ix(node);
        self.node_map[ix] = K::zero();
    }

    fn get(&self, node: &G::NodeId) -> Option<&Self::Cost> {
        let ix = self.ix(*node);
        let cost = &self.node_map[ix];
        if *cost != K::infinite() {
            Some(cost)
        } else {
            None
        }
    }

    fn get_or_infinite(&self, node: &G::NodeId) -> Self::Cost
        where Self::Cost : FloatMeasure,
    {
        self.node_map[self.ix(*node)]
    }

    fn consider(&mut self, node: G::NodeId, cost: Self::Cost) -> bool {
        let ix = self.ix(node);
        let better = cost < self.node_map[ix];
        if better {
            self.node_map[ix] = cost;
        }
        better
    }
}

impl<G, K> CostMap<G> for IndexableNodeMap<G, Option<K>>
    where G: GraphBase + NodeIndexable,
          K: Measure,
{
    type Cost = K;

    fn initialize(&mut self, graph: G, node: G::NodeId) {
        self.initialize(graph, None);
        let ix = self.ix(node);
        self.node_map[ix] = Some(K::default());
    }

    fn get(&self, node: &G::NodeId) -> Option<&Self::Cost> {
        let ix = self.ix(*node);
        self.node_map[ix].as_ref()
    }

    fn consider(&mut self, node: G::NodeId, cost: Self::Cost) -> bool {
        let ix = self.ix(node);
        let better = self.node_map[ix].as_ref()
            .map(|old_cost| *old_cost < cost)
            .unwrap_or(true);
        if better {
            self.node_map[ix] = Some(cost);
        }
        better
    }
}

pub struct NoPredecessorMap;

impl<G> PredecessorMap<G> for NoPredecessorMap
    where G: GraphBase,
{
    fn initialize(&mut self, _graph: G) {
        // noop
    }

    fn get(&self, _node: G::NodeId) -> Option<G::NodeId> {
        None
    }

    fn set(&mut self, _node: G::NodeId, _pred: G::NodeId) {
        // noop
    }
}
