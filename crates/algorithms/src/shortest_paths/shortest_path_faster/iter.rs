use core::{hash::Hash, ops::Add};

use error_stack::{Report, Result};
use fxhash::FxBuildHasher;
use hashbrown::{HashMap, HashSet};
use num_traits::Zero;
use petgraph_core::{Graph, GraphStorage, Node};

use super::error::ShortestPathFasterError;
use crate::shortest_paths::{
    common::{
        connections::Connections, cost::GraphCost, double_ended_queue::DoubleEndedQueue,
        intermediates::reconstruct_intermediates,
    },
    Cost, Path, Route,
};

pub(super) struct ShortestPathFasterIter<'graph: 'parent, 'parent, S, E, G>
where
    S: GraphStorage,
    E: GraphCost<S>,
    E::Value: Ord,
{
    graph: &'graph Graph<S>,

    queue: DoubleEndedQueue<&'graph S::NodeId>,

    edge_cost: &'parent E,
    connections: G,

    source: Node<'graph, S>,

    num_nodes: usize,

    iteration: usize,
    init: bool,
    next: Option<&'graph S::NodeId>,

    // candidate_order: SPFACandidateOrder,
    distances: HashMap<&'graph S::NodeId, E::Value, FxBuildHasher>,
    predecessors: HashMap<&'graph S::NodeId, Option<Node<'graph, S>>, FxBuildHasher>,
}

impl<'graph: 'parent, 'parent, S, E, G> ShortestPathFasterIter<'graph, 'parent, S, E, G>
where
    S: GraphStorage,
    S::NodeId: PartialEq + Eq + Hash,
    E: GraphCost<S>,
    E::Value: PartialOrd + Ord + Zero + Clone + 'graph,
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

        Ok(Self {
            graph,
            queue,
            edge_cost,
            connections,
            source: source_node,
            num_nodes: graph.num_nodes(),
            iteration: 0,
            init: true,
            next: None,
            // candidate_order,
            distances,
            predecessors,
        })
    }

    fn detect_cycle(&self) -> bool {
        let mut visited =
            HashSet::with_capacity_and_hasher(self.num_nodes, FxBuildHasher::default());
        let mut on_stack =
            HashSet::with_capacity_and_hasher(self.num_nodes, FxBuildHasher::default());
        let mut stack = Vec::with_capacity(self.num_nodes);

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
    E::Value: PartialOrd + Ord + Zero + Clone + 'graph,
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
        // the first iteration is special, as we immediately return the source node
        // and then begin with the actual iteration loop.
        if self.init {
            self.init = false;
            self.next = Some(self.source.id());

            return Some(Route {
                path: Path {
                    source: self.source,
                    target: self.source,
                    intermediates: Vec::new(),
                },
                cost: Cost(E::Value::zero()),
            });
        }

        let node_id = self.next?;
        let node = self.graph.node(&node_id).expect("node to be present");
        let connections = self.connections.connections(&node);

        for edge in connections {
            let (u, v) = edge.endpoints();
            let target = if v.id() == node_id { u.id() } else { v.id() };

            let next_distance_cost = &self.distances[&node_id] + self.edge_cost.cost(edge).as_ref();

            if let Some(distance) = self.distances.get(target) {
                if next_distance_cost < *distance {
                    self.distances.insert(target, next_distance_cost);

                    self.iteration += 1;

                    if self.iteration == self.num_nodes {
                        self.iteration = 0;
                        if self.detect_cycle() {
                            // We've reached the maximum number of iterations, which means that
                            // we've detected a negative cycle. We
                            // terminate early.
                            return None;
                        }
                    }

                    self.predecessors.insert(
                        target,
                        Some(self.graph.node(node_id).expect("node to exist")),
                    );

                    if !self.queue.contains(&target) {
                        self.queue.push_back(target);
                    }
                }
            }
        }

        let Some(node) = self.queue.pop_front() else {
            // No more elements in the queue, we're done.
            self.next = None;
            return None;
        };

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
