use alloc::vec::Vec;
use core::hash::Hash;

use error_stack::{Report, Result};
use fxhash::FxBuildHasher;
use hashbrown::HashMap;
use numi::num::{identity::Zero, ops::AddRef};
use petgraph_core::{Graph, GraphStorage, Node};

use crate::shortest_paths::{
    astar::{error::AStarError, heuristic::GraphHeuristic, AStarMeasure},
    common::{
        connections::Connections,
        cost::{Cost, GraphCost},
        queue::priority::PriorityQueue,
        route::{DirectRoute, Route},
        transit::{reconstruct_path_to, PredecessorMode},
    },
    Path,
};

// The graph must outlive the A* instance
pub(super) struct AStarImpl<'graph: 'parent, 'parent, S, E, H, C>
where
    S: GraphStorage,
    S::NodeId: FlaggableGraphId<S>,
    E: GraphCost<S>,
    E::Value: Ord,
{
    queue: PriorityQueue<'graph, S, E::Value>,

    edge_cost: &'parent E,
    heuristic: &'parent H,
    connections: C,

    source: Node<'graph, S>,
    target: Node<'graph, S>,

    predecessor_mode: PredecessorMode,

    distances: HashMap<&'graph S::NodeId, E::Value, FxBuildHasher>,
    predecessors: HashMap<&'graph S::NodeId, Option<Node<'graph, S>>, FxBuildHasher>,
}

impl<'graph: 'parent, 'parent, S, E, H, C> AStarImpl<'graph, 'parent, S, E, H, C>
where
    S: GraphStorage,
    S::NodeId: FlaggableGraphId<S> + Eq + Hash,
    E: GraphCost<S>,
    E::Value: AStarMeasure,
    H: GraphHeuristic<S, Value = E::Value>,
    C: Connections<'graph, S> + 'parent,
{
    pub(super) fn new(
        graph: &'graph Graph<S>,

        edge_cost: &'parent E,
        heuristic: &'parent H,
        connections: C,

        source: &'graph S::NodeId,
        target: &'graph S::NodeId,

        predecessor_mode: PredecessorMode,
    ) -> Result<Self, AStarError> {
        let source_node = graph
            .node(source)
            .ok_or_else(|| Report::new(AStarError::NodeNotFound))?;

        let target_node = graph
            .node(target)
            .ok_or_else(|| Report::new(AStarError::NodeNotFound))?;

        let mut queue = PriorityQueue::new();
        queue.push(
            source_node,
            heuristic.estimate(source_node, target_node).into_owned(),
        );

        let mut distances = HashMap::with_hasher(FxBuildHasher::default());
        distances.insert(source, E::Value::zero());

        let mut predecessors = HashMap::with_hasher(FxBuildHasher::default());
        if predecessor_mode == PredecessorMode::Record {
            predecessors.insert(source, None);
        }

        Ok(Self {
            queue,

            edge_cost,
            heuristic,
            connections,

            source: source_node,
            target: target_node,

            predecessor_mode,

            distances,
            predecessors,
        })
    }

    pub(super) fn find(mut self) -> Option<Route<'graph, S, E::Value>> {
        while let Some(QueueItem { node, .. }) = self.queue.pop_min() {
            if node.id() == self.target.id() {
                let transit = if self.predecessor_mode == PredecessorMode::Record {
                    reconstruct_path_to(&self.predecessors, node.id())
                } else {
                    Vec::new()
                };

                let distance = self.distances[node.id()].clone();

                return Some(Route::new(
                    Path::new(self.source, transit, self.target),
                    Cost::new(distance),
                ));
            }

            let connections = self.connections.connections(&node);
            for edge in connections {
                let alternative =
                    self.distances[node.id()].add_ref(self.edge_cost.cost(edge).as_ref());

                let (u, v) = edge.endpoints();
                let neighbour = if u.id() == node.id() { v } else { u };

                if let Some(distance) = self.distances.get(neighbour.id()) {
                    if alternative >= *distance {
                        continue;
                    }
                }

                let guess =
                    alternative.add_ref(self.heuristic.estimate(neighbour, self.target).as_ref());
                self.distances.insert(neighbour.id(), alternative);

                if self.predecessor_mode == PredecessorMode::Record {
                    self.predecessors.insert(neighbour.id(), Some(node));
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
        items.into_iter().filter_map(Self::find)
    }

    #[inline]
    pub(super) fn find_all_direct(
        items: Vec<Self>,
    ) -> impl Iterator<Item = DirectRoute<'graph, S, E::Value>> + 'parent {
        Self::find_all(items).map(From::from)
    }
}
