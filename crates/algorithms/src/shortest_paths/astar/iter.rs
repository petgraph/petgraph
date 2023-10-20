use core::{hash::Hash, ops::Add};

use error_stack::{Report, Result};
use fxhash::FxBuildHasher;
use hashbrown::HashMap;
use num_traits::Zero;
use petgraph_core::{base::MaybeOwned, Edge, Graph, GraphStorage, Node};

use crate::shortest_paths::{
    astar::error::AStarError,
    common::{queue::Queue, traits::ConnectionFn},
    Route,
};

pub(super) struct AStarIter<'a, S, T, E, H, C>
where
    S: GraphStorage,
    T: Ord,
{
    queue: Queue<'a, S, T>,

    edge_cost: E,
    heuristic: H,
    connections: C,

    source: Node<'a, S>,
    target: Node<'a, S>,

    distances: HashMap<&'a S::NodeId, T, FxBuildHasher>,
    heuristic_values: HashMap<&'a S::NodeId, MaybeOwned<'a, T>, FxBuildHasher>,
    previous: HashMap<&'a S::NodeId, Option<Node<'a, S>>, FxBuildHasher>,
}

impl<'a, S, T, E, H, C> AStarIter<'a, S, T, E, H, C>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    T: PartialOrd + Ord + Zero + Clone + 'a,
    for<'b> &'b T: Add<Output = T>,
    E: Fn(Edge<'a, S>) -> MaybeOwned<'a, T>,
    H: Fn(Node<'a, S>, Node<'a, S>) -> MaybeOwned<'a, T>,
    C: ConnectionFn<'a, S>,
{
    pub(super) fn new(
        graph: &'a Graph<S>,

        edge_cost: E,
        heuristic: H,
        connections: C,

        source: &'a S::NodeId,
        target: &'a S::NodeId,
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
            heuristic(source_node, target_node).into_owned(),
        );

        let mut distances = HashMap::with_hasher(FxBuildHasher::default());
        distances.insert(source, T::zero());

        let mut heuristic_values = HashMap::with_hasher(FxBuildHasher::default());
        heuristic_values.insert(source, heuristic(source_node, target_node));

        let mut previous = HashMap::with_hasher(FxBuildHasher::default());
        previous.insert(source, None);

        Ok(Self {
            queue,

            edge_cost,
            heuristic,
            connections,

            source: source_node,
            target: target_node,

            distances,
            heuristic_values,
            previous,
        })
    }
}

impl<'a, S, T, E, H, C> Iterator for AStarIter<'a, S, T, E, H, C>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    T: PartialOrd + Ord + Zero + Clone + 'a,
    for<'b> &'b T: Add<Output = T>,
    E: Fn(Edge<'a, S>) -> MaybeOwned<'a, T>,
    H: Fn(Node<'a, S>, Node<'a, S>) -> MaybeOwned<'a, T>,
    C: ConnectionFn<'a, S>,
{
    type Item = Route<'a, S, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.queue.pop_min()?;

        todo!()
    }
}
