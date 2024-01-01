use alloc::vec::Vec;
use core::hash::Hash;

use error_stack::{Report, Result};
use fxhash::FxBuildHasher;
use hashbrown::HashMap;
use numi::num::{identity::Zero, ops::AddRef};
use petgraph_core::{node::NodeId, Graph, GraphStorage, Node};

use super::error::BellmanFordError;
use crate::shortest_paths::{
    bellman_ford::{measure::BellmanFordMeasure, CandidateOrder},
    common::{
        connections::Connections,
        cost::GraphCost,
        queue::double_ended::DoubleEndedQueue,
        transit::{reconstruct_paths_between, PredecessorMode},
    },
    Cost, Path, Route,
};

fn small_label_first<'graph, S, E>(
    node: Node<'graph, S>,
    cost: E::Value,
    queue: &mut DoubleEndedQueue<'graph, S, E::Value>,
) -> bool
where
    S: GraphStorage,
    E: GraphCost<S>,
    E::Value: BellmanFordMeasure,
{
    let Some(item) = queue.peek_front() else {
        // queue is empty, therefore we can just simply push the cost
        return queue.push_front(node, cost);
    };

    if &cost < item.priority() {
        // the cost is smaller than the current smallest cost, therefore we push it to the front

        return queue.push_front(node, cost);
    }

    // the cost is larger than the current smallest cost, therefore we push it to the back
    queue.push_back(node, cost)
}

fn large_label_last<'graph, S, E>(
    node: Node<'graph, S>,
    cost: E::Value,
    queue: &mut DoubleEndedQueue<'graph, S, E::Value>,
) -> bool
where
    S: GraphStorage,
    NodeId: Eq + Hash,
    E: GraphCost<S>,
    E::Value: BellmanFordMeasure,
{
    if queue.is_empty() {
        // queue is empty, therefore we can just simply push the cost
        return queue.push_back(node, cost);
    }

    // always push the item to the back of the queue
    let did_push = queue.push_back(node, cost);

    let Some(average) = queue.average_priority() else {
        // this only happens if the length is 0 or we were unable to convert the length (usize)
        // to the value type (E::Value)
        return did_push;
    };

    loop {
        // TODO: should we panic here instead? This should never happen.
        let Some(front) = queue.peek_front() else {
            // this should never happen, but if it does, we can just stop
            // (previous check for empty queue should have caught this)
            return did_push;
        };

        if *front.priority() <= average {
            // the front item is smaller than the average, therefore we can stop
            return did_push;
        }

        let Some(front) = queue.pop_front() else {
            // this should never happen, but if it does, we can just stop
            // (previous check for empty queue should have caught this)
            return did_push;
        };

        let (node, priority) = front.into_parts();
        queue.push_back(node, priority);
    }
}

// TODO: make use of auxiliary graph storage
struct Heuristic {
    enabled: bool,
    recent_update: HashMap<NodeId, (NodeId, NodeId)>,
    predecessor: HashMap<NodeId, NodeId>,
}

impl Heuristic {
    fn new(enabled: bool) -> Self {
        Self {
            enabled,
            recent_update: HashMap::default(),
            predecessor: HashMap::default(),
        }
    }

    fn update(&mut self, source: NodeId, target: NodeId) -> core::result::Result<(), NodeId> {
        if !self.enabled {
            return Ok(());
        }

        // source = u
        // target = v

        // the heuristic is used to find a negative cycle before it is fully constructed.
        // this is done via an implied check over multiple iterations.
        // if it happens that some earlier update added the target node (as signified by the recent
        // update) we know,
        // that we have a negative cycle
        // as the same node would be on the same path twice.
        if let Some((u, v)) = self.recent_update.get(&source) {
            if target == *u || target == *v {
                return Err(target);
            }
        }

        if self.predecessor.get(&target) == Some(&source) {
            if let Some(previous) = self.recent_update.get(&source) {
                self.recent_update.insert(target, *previous);
            }
        } else {
            self.recent_update.insert(target, (source, target));
        }

        self.predecessor.insert(target, source);
        Ok(())
    }
}

pub(super) struct ShortestPathFasterImpl<'graph: 'parent, 'parent, S, E, G>
where
    S: GraphStorage,
    E: GraphCost<S>,
{
    graph: &'graph Graph<S>,
    source: Node<'graph, S>,

    edge_cost: &'parent E,
    connections: G,

    predecessor_mode: PredecessorMode,
    candidate_order: CandidateOrder,
    negative_cycle_heuristics: bool,

    distances: HashMap<NodeId, E::Value, FxBuildHasher>,
    predecessors: HashMap<NodeId, Vec<Node<'graph, S>>, FxBuildHasher>,
}

impl<'graph: 'parent, 'parent, S, E, G> ShortestPathFasterImpl<'graph, 'parent, S, E, G>
where
    S: GraphStorage,
    E: GraphCost<S>,
    E::Value: BellmanFordMeasure,
    G: Connections<'graph, S>,
{
    pub(super) fn new(
        graph: &'graph Graph<S>,

        edge_cost: &'parent E,
        connections: G,

        source: NodeId,

        predecessor_mode: PredecessorMode,
        candidate_order: CandidateOrder,
        negative_cycle_heuristics: bool,
    ) -> Result<Self, BellmanFordError> {
        let source_node = graph
            .node(source)
            .ok_or_else(|| Report::new(BellmanFordError::NodeNotFound))?;

        let mut distances = HashMap::with_hasher(FxBuildHasher::default());
        distances.insert(source, E::Value::zero());

        let mut predecessors = HashMap::with_hasher(FxBuildHasher::default());
        predecessors.insert(source, Vec::new());

        let mut this = Self {
            graph,
            source: source_node,
            edge_cost,
            connections,
            predecessor_mode,
            candidate_order,
            negative_cycle_heuristics,
            distances,
            predecessors,
        };

        // TODO: reconstruct negative cycle (needs `NodeId` to have additional trait bounds for
        //  error-stack)
        if this.relax().is_err() {
            return Err(Report::new(BellmanFordError::NegativeCycle));
        }

        Ok(this)
    }

    /// Inner Relaxation Loop for the Bellman-Ford algorithm, an implementation of SPFA.
    ///
    /// Based on [networkx](https://github.com/networkx/networkx/blob/f93f0e2a066fc456aa447853af9d00eec1058542/networkx/algorithms/shortest_paths/weighted.py#L1363)
    fn relax(&mut self) -> core::result::Result<(), NodeId> {
        // we always need to record predecessors to be able to skip relaxations
        let mut queue = DoubleEndedQueue::new();
        let mut heuristic = Heuristic::new(self.negative_cycle_heuristics);
        let mut occurrences = HashMap::new();
        let num_nodes = self.graph.num_nodes();

        queue.push_back(self.source, E::Value::zero());

        while let Some(item) = queue.pop_front() {
            let (source, priority) = item.into_parts();

            // skip relaxations if any of the predecessors of node are in the queue
            if let Some(predecessors) = self.predecessors.get(&source.id()) {
                if predecessors
                    .iter()
                    .any(|node| queue.contains_node(&node.id()))
                {
                    continue;
                }
            }

            let edges = self.connections.connections(source.id());

            for edge in edges {
                let (u, v) = edge.endpoints();
                let target = if u.id() == source.id() { v } else { u };

                let alternative = priority.add_ref(self.edge_cost.cost(edge).as_ref());

                if let Some(distance) = self.distances.get(&target.id()) {
                    if alternative == *distance {
                        self.predecessors
                            .entry(target.id())
                            .or_insert_with(Vec::new)
                            .push(source);
                        continue;
                    }

                    if alternative >= *distance {
                        continue;
                    }
                }

                if let Err(node) = heuristic.update(source.id(), target.id()) {
                    self.predecessors
                        .entry(target.id())
                        .or_insert_with(Vec::new)
                        .push(source);
                    return Err(node);
                };

                // we have a concrete problem here: we do not update the priority in the queue
                // if it is larger.
                // Could we in theory use the HashBrown API here instead, referencing the item in
                // question?
                let did_push = match self.candidate_order {
                    CandidateOrder::SmallFirst => {
                        small_label_first::<S, E>(target, alternative.clone(), &mut queue)
                    }
                    CandidateOrder::LargeLast => {
                        large_label_last::<S, E>(target, alternative.clone(), &mut queue)
                    }
                    CandidateOrder::Naive => queue.push_back(target, alternative.clone()),
                };

                if did_push {
                    let count = occurrences.entry(target.id()).or_insert(0usize);
                    *count += 1;

                    // If the heuristic failed (or is disabled) this is the fail-safe mechanism
                    // to detect any negative cycles.
                    // We know that a shortest path can at most go through n nodes, therefore we
                    // can detect a negative cycle,
                    // if we have visited the same node n times.
                    //
                    // As we can only detect a negative cycle quite late this is the worst case and
                    // the heuristic should be used instead.
                    if *count == num_nodes {
                        // negative cycle detected
                        return Err(target.id());
                    }
                }

                self.distances.insert(target.id(), alternative);

                // re-use the same buffer so that we don't need to allocate a new one
                let predecessors = self
                    .predecessors
                    .entry(target.id())
                    .or_insert_with(Vec::new);
                predecessors.clear();
                predecessors.push(source);
            }
        }

        Ok(())
    }

    pub(crate) fn between(mut self, target: NodeId) -> Option<Route<'graph, S, E::Value>> {
        let cost = self.distances.remove(&target)?;
        let target = self.graph.node(target)?;

        let transit = if self.predecessor_mode == PredecessorMode::Record {
            reconstruct_paths_between(&self.predecessors, self.source.id(), target)
                .next()
                .unwrap_or_else(Vec::new)
        } else {
            Vec::new()
        };

        Some(Route::new(
            Path::new(self.source, transit, target),
            Cost::new(cost),
        ))
    }

    pub(crate) fn all(self) -> impl Iterator<Item = Route<'graph, S, E::Value>> {
        let Self {
            graph,
            source,
            predecessor_mode,
            distances,
            predecessors,
            ..
        } = self;

        distances
            .into_iter()
            .filter_map(|(target, cost)| graph.node(target).map(|target| (target, cost)))
            .map(move |(target, cost)| {
                let transit = if predecessor_mode == PredecessorMode::Record {
                    reconstruct_paths_between(&predecessors, source.id(), target)
                        .next()
                        .unwrap_or_else(Vec::new)
                } else {
                    Vec::new()
                };

                Route::new(Path::new(source, transit, target), Cost::new(cost))
            })
    }
}
