use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};

use std::hash::Hash;

use std::ops::Index;

use std::marker::PhantomData;

use crate::algo::{Measure, FloatMeasure};
use crate::visit::{GraphBase, NodeIndexable};

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
    pub fn new(predecessors: P, costs: C, goal: Option<G::NodeId>) -> Self {
        Path {
            goal: goal,
            predecessors: predecessors,
            costs: costs,
            k: PhantomData,
        }
    }

    pub fn unpack(self) -> (C, P, Option<G::NodeId>) {
        (self.costs, self.predecessors, self.goal)
    }

    pub fn into_predecessors(self) -> P {
        self.predecessors
    }

    pub fn into_costs(self) -> C {
        self.costs
    }
}

impl<G, K, C, P> Path<G, K, C, P>
    where G: GraphBase,
          K: Measure + Copy,
          C: CostMap<G, Cost = K>,
          P: PredecessorMap<G> + PredecessorMapConfigured<G>,
{
    pub fn into_nodes(self) -> Option<(K, Vec<G::NodeId>)> {
        self.goal
            .map(|node| {
                let total_cost = self.costs.get(node).unwrap();
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
    fn initialize(&mut self, graph: G);

    fn get(&self, node: G::NodeId) -> Option<G::NodeId>;

    fn set(&mut self, node: G::NodeId, pred: G::NodeId);
}

/// Marker trait used to differentiate explicitly configured predecessor maps from the default.
/// This is needed to cause a compile-time error when the user tries to rebuild the path without
/// configuring a predecessor map, which would likely cause a panic.
pub trait PredecessorMapConfigured<G: GraphBase> : PredecessorMap<G> {
}

pub trait CostMap<G: GraphBase> :
    for<'a> Index<&'a G::NodeId, Output = <Self as CostMap<G>>::Cost>
{
    type Cost: Measure;

    fn initialize(&mut self, graph: G, node: G::NodeId);

    fn get(&self, node: G::NodeId) -> Option<Self::Cost>;

    fn consider(&mut self, node: G::NodeId, cost: Self::Cost) -> bool;
}

impl<G> PredecessorMap<G> for HashMap<G::NodeId, G::NodeId>
    where G: GraphBase,
          G::NodeId: Eq + Hash
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

impl<G> PredecessorMapConfigured<G> for HashMap<G::NodeId, G::NodeId>
    where G: GraphBase,
          G::NodeId: Eq + Hash
{
}

impl<G, K> CostMap<G> for HashMap<G::NodeId, K>
    where G: GraphBase,
          G::NodeId: Eq + Hash,
          K: Measure
{
    type Cost = K;

    fn initialize(&mut self, _graph: G, node: G::NodeId) {
        self.clear();
        self.insert(node, <_>::default());
    }

    fn get(&self, node: G::NodeId) -> Option<Self::Cost> {
        self.get(&node).cloned()
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

pub struct IndexableNodeMap<G, T>
    where G: GraphBase + NodeIndexable
{
    graph: Option<G>, // a bit hacky, but works nonetheless
    node_map: Vec<T>,
}

impl<G, T> IndexableNodeMap<G, T>
    where G: GraphBase + NodeIndexable
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
    where G: GraphBase + NodeIndexable
{
    fn initialize(&mut self, graph: G, value: T) {
        self.graph = Some(graph);
        self.node_map = vec![value; self.graph.as_ref().unwrap().node_bound()];
    }
}

impl<'a, G, T> Index<&'a G::NodeId> for IndexableNodeMap<G, T>
    where G: GraphBase + NodeIndexable
{
    type Output = T;

    fn index(&self, node: &G::NodeId) -> &Self::Output {
        self.node_map.index(self.ix(*node))
    }
}

impl<G> PredecessorMap<G> for IndexableNodeMap<G, Option<G::NodeId>>
    where G: GraphBase + NodeIndexable
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
    where G: GraphBase + NodeIndexable
{
}

impl<G, K> CostMap<G> for IndexableNodeMap<G, K>
    where G: GraphBase + NodeIndexable,
          K: FloatMeasure
{
    type Cost = K;

    fn initialize(&mut self, graph: G, node: G::NodeId) {
        self.initialize(graph, K::infinite());
        let ix = self.ix(node);
        self.node_map[ix] = K::zero();
    }

    fn get(&self, node: G::NodeId) -> Option<Self::Cost> {
        let ix = self.ix(node);
        let cost = self.node_map[ix];
        if cost != K::infinite() {
            Some(cost)
        } else {
            None
        }
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
