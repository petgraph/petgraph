use core::{hash::Hash, ops::Add};

use error_stack::{Report, Result};
use fxhash::FxBuildHasher;
use hashbrown::{HashMap, HashSet};
use num_traits::{Bounded, Zero};
use petgraph_core::{Graph, GraphStorage, Node};

use super::error::ShortestPathFasterError;
use crate::shortest_paths::{
    common::{connections::Connections, cost::GraphCost, queue::double_ended::DoubleEndedQueue},
    Cost, Path, Route,
};

fn small_label_first<'graph, S, E>(
    node: Node<'graph, S>,
    cost: E::Value,
    queue: &mut DoubleEndedQueue<'graph, S, E::Value>,
) where
    S: GraphStorage,
    E: GraphCost<S>,
    E::Value: PartialOrd,
{
    let Some(item) = queue.peek_front() else {
        // queue is empty, therefore we can just simply push the cost
        queue.push_front(node, cost);
        return;
    };

    if &cost < item.priority() {
        // the cost is smaller than the current smallest cost, therefore we push it to the front
        queue.push_front(node, cost);
        return;
    }

    // the cost is larger than the current smallest cost, therefore we push it to the back
    queue.push_back(node, cost);
}

fn large_label_last<'graph, S, E>(
    node: Node<'graph, S>,
    cost: E::Value,
    queue: &mut DoubleEndedQueue<'graph, S, E::Value>,
) where
    S: GraphStorage,
    E: GraphCost<S>,
    E::Value: PartialOrd,
{
    if queue.is_empty() {
        // queue is empty, therefore we can just simply push the cost
        queue.push_back(node, cost);
        return;
    }

    // always push the item to the back of the queue
    queue.push_back(node, cost);

    let Some(average) = queue.average_priority() else {
        // we would divide by zero, this can only happen if the queue is empty, therefore we can
        // just proceed as normal.
        return;
    };

    loop {
        let Some(front) = queue.peek_front() else {
            // this should never happen, but if it does, we can just stop
            return;
        };

        if front.priority() <= &average {
            // the front item is smaller than the average, therefore we can stop
            return;
        }

        let Some(front) = queue.pop_front() else {
            // this should never happen, but if it does, we can just stop
            return;
        };

        let (node, priority) = front.into_parts();
        queue.push_back(node, priority);
    }
}

pub(super) struct ShortestPathFasterIter<'graph: 'parent, 'parent, S, E, G>
where
    S: GraphStorage,
    E: GraphCost<S>,
    E::Value: Ord,
{
    graph: &'graph Graph<S>,

    queue: DoubleEndedQueue<'graph, S, E::Value>,

    edge_cost: &'parent E,
    connections: G,

    source: Node<'graph, S>,

    num_nodes: usize,

    iteration: usize,
    next: Option<&'graph S::NodeId>,

    // candidate_order: SPFACandidateOrder,
    distances: HashMap<&'graph S::NodeId, E::Value, FxBuildHasher>,
    predecessors: HashMap<&'graph S::NodeId, Option<Node<'graph, S>>, FxBuildHasher>,
    in_queue: HashSet<&'graph S::NodeId, FxBuildHasher>,
}

impl<'graph: 'parent, 'parent, S, E, G> ShortestPathFasterIter<'graph, 'parent, S, E, G>
where
    S: GraphStorage,
    S::NodeId: PartialEq + Eq + Hash,
    E: GraphCost<S>,
    E::Value: PartialOrd + Ord + Zero + Bounded + Clone + 'graph,
    for<'a> &'a E::Value: Add<Output = E::Value>,
    G: Connections<'graph, S>,
{
    pub(super) fn new(
        graph: &'graph Graph<S>,

        edge_cost: &'parent E,
        connections: G,

        source: &'graph S::NodeId,
        // candidate_order: SPFACandidateOrder,
    ) -> Result<Self, ShortestPathFasterError> {
        let source_node = graph
            .node(source)
            .ok_or_else(|| Report::new(ShortestPathFasterError::NodeNotFound))?;

        let mut queue = DoubleEndedQueue::new();
        queue.push_back(source);

        let mut distances = HashMap::with_hasher(FxBuildHasher::default());
        distances.insert(source, E::Value::zero());

        let mut predecessors = HashMap::with_hasher(FxBuildHasher::default());
        predecessors.insert(source, None);

        let in_queue = HashSet::with_hasher(FxBuildHasher::default());

        Ok(Self {
            graph,
            queue,
            edge_cost,
            connections,
            source: source_node,
            num_nodes: graph.num_nodes(),
            iteration: 0,
            next: Some(source),
            // candidate_order,
            distances,
            predecessors,
            in_queue,
        })
    }

    fn has_cycle(&self) -> bool {
        let mut visited = HashSet::with_hasher(FxBuildHasher::default());
        let mut on_stack = HashSet::with_hasher(FxBuildHasher::default());
        let mut stack = Vec::new();

        for node in self.predecessors.keys() {
            if !visited.contains(*node) {
                while let Some(Some(pre)) = self.predecessors.get(*node) {
                    let predecessor_id = pre.id();
                    if !visited.contains(predecessor_id) {
                        visited.insert(predecessor_id);
                        on_stack.insert(predecessor_id);
                        stack.push(predecessor_id);
                    } else {
                        if on_stack.contains(predecessor_id) {
                            return true;
                        }
                        break;
                    }
                }

                while let Some(p) = stack.pop() {
                    on_stack.remove(p);
                }
            }
        }
        false
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
