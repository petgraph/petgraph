use alloc::vec::Vec;
use core::{hash::Hash, ops::Add};

use error_stack::{Report, Result};
use fxhash::FxBuildHasher;
use hashbrown::HashMap;
use num_traits::Zero;
use petgraph_core::{base::MaybeOwned, Edge, Graph, GraphStorage, Node};

use crate::shortest_paths::{
    astar::{error::AStarError, heuristic::GraphHeuristic},
    common::{
        cost::GraphCost,
        intermediates::{self, reconstruct_intermediates, Intermediates},
        queue::Queue,
        traits::ConnectionFn,
    },
    Cost, DirectRoute, Path, Route,
};

// 'a: lifetime of the graph
// 'b: lifetime of the A* instance
// The graph must outlive the A* instance
pub(super) struct AStarImpl<'graph: 'parent, 'parent, S, E, H, C>
where
    S: GraphStorage,
    E: GraphCost<S>,
    E::Value: Ord,
{
    queue: Queue<'graph, S, E::Value>,

    edge_cost: &'parent E,
    heuristic: &'parent H,
    connections: C,

    source: Node<'graph, S>,
    target: Node<'graph, S>,

    intermediates: Intermediates,

    distances: HashMap<&'graph S::NodeId, E::Value, FxBuildHasher>,
    previous: HashMap<&'graph S::NodeId, Option<Node<'graph, S>>, FxBuildHasher>,
}

impl<'graph: 'parent, 'parent, S, E, H, C> AStarImpl<'graph, 'parent, S, E, H, C>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    E: GraphCost<S>,
    E::Value: PartialOrd + Ord + Zero + Clone + 'graph,
    for<'c> &'c E::Value: Add<Output = E::Value>,
    H: GraphHeuristic<S, Value = E::Value>,
    C: ConnectionFn<'graph, S> + 'parent,
{
    pub(super) fn new(
        graph: &'graph Graph<S>,

        edge_cost: &'parent E,
        heuristic: &'parent H,
        connections: C,

        source: &'graph S::NodeId,
        target: &'graph S::NodeId,

        intermediates: Intermediates,
    ) -> Result<Self, AStarError> {
        let source_node = graph
            .node(source)
            .ok_or_else(|| Report::new(AStarError::NodeNotFound))?;

        let target_node = graph
            .node(target)
            .ok_or_else(|| Report::new(AStarError::NodeNotFound))?;

        let mut queue = Queue::new();
        queue.push(
            source_node,
            heuristic.estimate(source_node, target_node).into_owned(),
        );

        let mut distances = HashMap::with_hasher(FxBuildHasher::default());
        distances.insert(source, E::Value::zero());

        let mut previous = HashMap::with_hasher(FxBuildHasher::default());
        if intermediates == Intermediates::Record {
            previous.insert(source, None);
        }

        Ok(Self {
            queue,

            edge_cost,
            heuristic,
            connections,

            source: source_node,
            target: target_node,

            intermediates,

            distances,
            previous,
        })
    }

    pub(super) fn find(mut self) -> Option<Route<'graph, S, E::Value>> {
        while let Some(node) = self.queue.pop_min() {
            if node.id() == self.target.id() {
                let intermediates = if self.intermediates == Intermediates::Record {
                    reconstruct_intermediates(&self.previous, node.id())
                } else {
                    Vec::new()
                };

                let distance = self.distances[node.id()].clone();

                let path = Path {
                    source: self.source,
                    target: self.target,
                    intermediates,
                };

                return Some(Route {
                    path,
                    cost: Cost(distance),
                });
            }

            let connections = self.connections.connections(&node);
            for edge in connections {
                let alternative = &self.distances[node.id()] + self.edge_cost.cost(edge).as_ref();

                let (u, v) = edge.endpoints();
                let neighbour = if u.id() == node.id() { v } else { u };

                if let Some(distance) = self.distances.get(neighbour.id()) {
                    if alternative >= *distance {
                        continue;
                    }
                }

                let guess = &alternative + self.heuristic.estimate(neighbour, self.target).as_ref();
                self.distances.insert(neighbour.id(), alternative);

                if self.intermediates == Intermediates::Record {
                    self.previous.insert(neighbour.id(), Some(node));
                }

                self.queue.decrease_priority(neighbour, guess);
            }
        }

        None
    }

    #[inline]
    pub(super) fn find_all(
        items: Vec<Self>,
    ) -> impl Iterator<Item = Route<'graph, S, E::Value>> + 'parent {
        items.into_iter().filter_map(|item| item.find())
    }

    #[inline]
    pub(super) fn find_all_direct(
        items: Vec<Self>,
    ) -> impl Iterator<Item = DirectRoute<'graph, S, E::Value>> + 'parent {
        Self::find_all(items).map(|route| DirectRoute {
            source: route.path.source,
            target: route.path.target,
            cost: route.cost,
        })
    }
}
