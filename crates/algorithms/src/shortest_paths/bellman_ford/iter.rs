use alloc::{vec, vec::Vec};
use core::{hash::Hash, ops::Add};

use error_stack::{Report, Result};
use fxhash::FxBuildHasher;
use hashbrown::{HashMap, HashSet};
use num_traits::{Bounded, Zero};
use petgraph_core::{Edge, Graph, GraphStorage, Node};

use super::error::ShortestPathFasterError;
use crate::shortest_paths::{
    bellman_ford::CandidateOrder,
    common::{
        connections::Connections, cost::GraphCost, queue::double_ended::DoubleEndedQueue,
        transit::PredecessorMode,
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
    E::Value: PartialOrd,
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
    E: GraphCost<S>,
    E::Value: PartialOrd,
{
    if queue.is_empty() {
        // queue is empty, therefore we can just simply push the cost
        return queue.push_back(node, cost);
    }

    // always push the item to the back of the queue
    let did_push = queue.push_back(node, cost);

    let Some(average) = queue.average_priority() else {
        // this should never happen, but if it does, we can just stop
        // (previous check for empty queue should have caught this)
        return did_push;
    };

    loop {
        // TODO: should we panic here instead? This should never happen.
        let Some(front) = queue.peek_front() else {
            // this should never happen, but if it does, we can just stop
            // (previous check for empty queue should have caught this)
            return did_push;
        };

        if front.priority() <= &average {
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

struct Heuristic<'graph, S>
where
    S: GraphStorage,
{
    enabled: bool,
    recent_update: HashMap<&'graph S::NodeId, (&'graph S::NodeId, &'graph S::NodeId)>,
    predecessor: HashMap<&'graph S::NodeId, &'graph S::NodeId>,
}

impl<'graph, S> Heuristic<'graph, S>
where
    S: GraphStorage,
{
    fn new(enabled: bool) -> Self {
        Self {
            enabled,
            recent_update: HashMap::default(),
            predecessor: HashMap::default(),
        }
    }

    fn update(&mut self, source: &'graph S::NodeId, target: &'graph S::NodeId) {
        if !self.enabled {
            return;
        }

        // TODO: docs
        if let Some((u, v)) = self.recent_update.get(source) {
            if target == u || target == v {
                // we found a cycle
                todo!()
            }
        }

        if self.predecessor.get(target) == Some(source) {
            self.recent_update
                .insert(target, self.recent_update[source]);
        } else {
            self.recent_update.insert(target, (source, target));
        }

        self.predecessor.insert(target, source);
    }
}

pub(super) struct ShortestPathFasterIter<'graph: 'parent, 'parent, S, E, G>
where
    S: GraphStorage,
    E: GraphCost<S>,
    E::Value: Ord,
{
    graph: &'graph Graph<S>,

    edge_cost: &'parent E,
    connections: G,

    predecessor_mode: PredecessorMode,
    candidate_order: CandidateOrder,
    negative_cycle_heuristics: bool,

    distances: HashMap<&'graph S::NodeId, E::Value, FxBuildHasher>,
    predecessors: HashMap<&'graph S::NodeId, Option<Node<'graph, S>>, FxBuildHasher>,
}

impl<'graph: 'parent, 'parent, S, E, G> ShortestPathFasterIter<'graph, 'parent, S, E, G>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    E: GraphCost<S>,
    E::Value: Zero,
    for<'a> &'a E::Value: Add<Output = E::Value>,
    G: Connections<'graph, S>,
{
    pub(super) fn new(
        graph: &'graph Graph<S>,

        edge_cost: &'parent E,
        connections: G,

        source: &'graph S::NodeId,

        predecessor_mode: PredecessorMode,
        candidate_order: CandidateOrder,
        negative_cycle_heuristics: bool,
    ) -> Result<Self, ShortestPathFasterError> {
        let source_node = graph
            .node(source)
            .ok_or_else(|| Report::new(ShortestPathFasterError::NodeNotFound))?;

        let mut distances = HashMap::with_hasher(FxBuildHasher::default());
        distances.insert(source, E::Value::zero());

        let mut predecessors = HashMap::with_hasher(FxBuildHasher::default());
        predecessors.insert(source, None);

        let mut this = Self {
            graph,
            edge_cost,
            connections,
            predecessor_mode,
            candidate_order,
            negative_cycle_heuristics,
            distances,
            predecessors,
        };

        this.relax(source_node);

        Ok(this)
    }

    /// Inner Relaxation Loop for the Bellman-Ford algorithm, an implementation of SPFA.
    ///
    /// Based on [networkx](https://github.com/networkx/networkx/blob/f93f0e2a066fc456aa447853af9d00eec1058542/networkx/algorithms/shortest_paths/weighted.py#L1363)
    fn relax(&mut self, source: Node<'graph, S>) -> core::result::Result<(), &'graph S::NodeId> {
        let mut queue = DoubleEndedQueue::new();
        let mut heuristic = Heuristic::new(self.negative_cycle_heuristics);
        let mut predecessors = HashMap::new();
        let mut occurrences = HashMap::new();
        let num_nodes = self.graph.num_nodes();

        queue.push_back(source, E::Value::zero());

        while let Some(item) = queue.pop_front() {
            let (source, priority) = item.into_parts();

            // skip relaxations if any of the predecessors of node are in the queue
            let previous = predecessors.get(source.id()).unwrap_or(&Vec::new());
            if previous.iter().any(|p| queue.contains_node(p)) {
                continue;
            }

            let edges = self.connections.connections(&source);

            for edge in edges {
                let (u, v) = edge.endpoints();
                let target = if u.id() == source.id() { v } else { u };

                let alternative = &priority + self.edge_cost.cost(edge).as_ref();

                if let Some(distance) = self.distances.get(target.id()) {
                    if self.predecessor_mode == PredecessorMode::Record && alternative == *distance
                    {
                        predecessors
                            .entry(target.id())
                            .or_insert_with(Vec::new)
                            .push(source);
                        continue;
                    }

                    if alternative >= *distance {
                        continue;
                    }
                }

                heuristic.update(source.id(), target.id());

                let did_push = match self.candidate_order {
                    CandidateOrder::SmallFirst => {
                        small_label_first(target, alternative.clone(), &mut queue)
                    }
                    CandidateOrder::LargeLast => {
                        large_label_last(target, alternative.clone(), &mut queue)
                    }
                };

                if did_push {
                    let count = occurrences.entry(target.id()).or_insert(0usize);
                    *count += 1;

                    if *count == num_nodes {
                        // negative cycle detected
                        return Err(target.id());
                    }
                }

                self.distances.insert(target.id(), alternative);

                if self.predecessor_mode == PredecessorMode::Record {
                    // re-use the same buffer so that we don't need to allocate a new one
                    let previous = predecessors.entry(target.id()).or_insert_with(Vec::new);
                    previous.clear();
                    previous.push(source);
                }
            }
        }

        Ok(())
    }
}

impl<'graph: 'parent, 'parent, S, E, G> Iterator
    for ShortestPathFasterIter<'graph, 'parent, S, E, G>
where
    S: GraphStorage,
    S::NodeId: PartialEq + Eq + Hash,
    E: GraphCost<S>,
    E::Value: PartialOrd + Ord + Zero + Bounded + Clone + 'graph,
    for<'a> &'a E::Value: Add<Output = E::Value>,
    G: Connections<'graph, S>,
{
    type Item = Route<'graph, S, E::Value>;

    // The concrete implementation is the SPFA (Shortest Path Faster Algorithm) algorithm, which is
    // a variant of Bellman-Ford that uses a queue to avoid unnecessary relaxation.
    // https://en.wikipedia.org/wiki/Shortest_path_faster_algorithm
    // We've made use of optimization techniques for candidate order
    // as well as a variation to terminate on negative cycles.
    // https://konaeakira.github.io/posts/using-the-shortest-path-faster-algorithm-to-find-negative-cycles.html
    fn next(&mut self) -> Option<Self::Item> {
        let default_distance = E::Value::max_value();

        let node_id = self.next?;
        let node = self.graph.node(&node_id).expect("node to be present");
        let connections = self.connections.connections(&node);

        for edge in connections {
            let (u, v) = edge.endpoints();
            let target = if v.id() == node_id { u.id() } else { v.id() };

            let next_distance_cost = &self.distances[&node_id] + self.edge_cost.cost(edge).as_ref();

            let distance = self.distances.get(target).unwrap_or(&default_distance);

            if next_distance_cost >= *distance {
                continue;
            }

            self.distances.insert(target, next_distance_cost);
            self.predecessors.insert(
                target,
                Some(self.graph.node(node_id).expect("node to exist")),
            );

            self.iteration += 1;

            if self.iteration == self.num_nodes {
                self.iteration = 0;
                if self.has_cycle() {
                    // A shortest path can at most go to n nodes, therefore we
                    // terminate early if we detected a cycle at the nth iteration
                    return None;
                }
            }

            if !self.in_queue.contains(target) {
                self.queue.push_back(target);
                self.in_queue.insert(target);
            }
        }

        let Some(node) = self.queue.pop_front() else {
            // No more elements in the queue, we're done.
            self.next = None;
            return None;
        };
        self.in_queue.remove(node);

        self.next = Some(node);

        // we're currently visiting the node that has the shortest distance, therefore we know
        // that the distance is the shortest possible
        let distance = self.distances[node_id].clone();
        let intermediates = reconstruct_intermediates(&self.predecessors, node);

        let path = Path {
            source: self.source,
            target: self.graph.node(node).expect("node to exist"),
            intermediates,
        };

        Some(Route {
            path,
            cost: Cost(distance),
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.num_nodes))
    }
}
