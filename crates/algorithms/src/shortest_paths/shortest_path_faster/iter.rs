use core::{hash::Hash, ops::Add};

use error_stack::{Report, Result};
use fxhash::FxBuildHasher;
use hashbrown::HashMap;
use num_traits::Zero;
use petgraph_core::{base::MaybeOwned, Edge, Graph, GraphStorage, Node};

use super::error::ShortestPathFasterError;
use crate::shortest_paths::{
    common::{intermediates::Intermediates, traits::ConnectionFn},
    Route,
};

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum SPFACandidateOrder {
    SmallFirst,
    LargeLast,
}

pub(super) struct ShortestPathFasterIter<'graph: 'parent, 'parent, S, E, G>
where
    S: GraphStorage,
    E: GraphCost<S>,
    E::Value: Ord,
{
    queue: Queue<'graph, S, E::Value>,

    edge_cost: F,
    connections: G,

    source: Node<'a, S>,

    num_nodes: usize,

    init: bool,
    next: Option<Node<'a, S>>,

    intermediates: Intermediates,
    candidate_order: SPFACandidateOrder,

    distances: HashMap<&'a S::NodeId, T, FxBuildHasher>,
    predecessors: HashMap<&'a S::NodeId, Option<Node<'a, S>>, FxBuildHasher>,
}

impl<'graph: 'parent, 'parent, S, E, G> ShortestPathFasterIter<'graph, 'parent, S, E, G>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    E: GraphCost<S>,
    E::Value: PartialOrd + Ord + Zero + Clone + 'graph,
    for<'a> &'a E::Value: Add<Output = E::Value>,
    G: Connections<'graph, S>,
{
    pub(super) fn new(
        graph: &'a Graph<S>,

        edge_cost: F,
        connections: G,

        source: &'a S::NodeId,

        intermediates: Intermediates,
        candidate_order: SPFACandidateOrder,
    ) -> Result<Self, ShortestPathFasterError> {
        let source_node = graph
            .node(source)
            .ok_or_else(|| Report::new(ShortestPathFasterError::NodeNotFound))?;

        let mut queue: Queue<'_, S, <E as GraphCost<S>>::Value> = Queue::new();
        queue.push(source_node, T::zero());

        let mut distances = HashMap::with_hasher(FxBuildHasher::default());
        distances.insert(source, T::zero());

        let mut predecessors = HashMap::with_hasher(FxBuildHasher::default());
        if intermediates == Intermediates::Record {
            predecessors.insert(source, None);
        }

        Ok(Self {
            queue,
            edge_cost,
            connections,
            source: source_node,
            num_nodes: graph.num_nodes(),
            init: true,
            next: None,
            intermediates,
            candidate_order,
            distances,
            predecessors,
        })
    }
}

impl<'graph: 'parent, 'parent, S, E, G> Iterator
    for ShortestPathFasterIter<'graph, 'parent, S, E, G>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
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
            self.next = Some(self.source);

            return Some(Route {
                path: Path {
                    source: self.source,
                    target: self.source,
                    intermediates: Vec::new(),
                },
                cost: Cost(E::Value::zero()),
            });
        }

        let node = self.next?;
        let connections = self.connections.connections(&node);

        for edge in connections {
            let (u, v) = edge.endpoints();
            let target = if v.id() == node.id() { u } else { v };

            let next_distance_cost = self.distances[&node.id()] + (self.edge_cost)(edge);

            if next_distance_cost < self.distances[&target.id()] {
                self.distances.insert(target.id(), next_distance_cost);

                if self.intermediates == Intermediates::Record {
                    self.predecessors.insert(target.id(), Some(node));
                }

                self.queue.decrease_priority(target, next_distance_cost);
            }
        }

        let Some(node) = self.queue.pop_min() else {
            // No more elements in the queue, we're done.
            self.next = None;
            return None;
        };

        self.next = Some(node);

        // we're currently visiting the node that has the shortest distance, therefore we know
        // that the distance is the shortest possible
        let distance = self.distances[node.id()].clone();
        let intermediates = if self.intermediates == Intermediates::Discard {
            Vec::new()
        } else {
            reconstruct_intermediates(&self.predecessors, node.id())
        };

        let path = Path {
            source: self.source,
            target: node,
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
