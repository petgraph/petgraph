use alloc::{collections::BinaryHeap, vec, vec::Vec};
use core::{
    cell::Cell,
    cmp::{Ordering, Reverse},
    fmt::{Display, Formatter},
    hash::Hash,
    mem,
    ops::Add,
};

use error_stack::{Context, Report, Result};
use fxhash::FxBuildHasher;
use hashbrown::HashMap;
use indexmap::map::Entry;
use num_traits::Zero;
use petgraph_core::{
    deprecated::visit::{EdgeRef, IntoEdges, VisitMap, Visitable},
    edge::{
        marker::{Directed, Undirected},
        Direction,
    },
    Edge, Graph, GraphDirectionality, GraphStorage, Node,
};

use crate::{
    common::IndexMap,
    shortest_paths::{DirectRoute, Measure, Path, Route, ShortestDistance, ShortestPath},
    utilities::min_scored::MinScored,
};

struct QueueItem<'a, S, T> {
    node: Node<'a, S>,

    priority: T,

    skip: Cell<bool>,
}

impl<S, T> PartialEq for QueueItem<S, T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.priority.eq(&other.priority)
    }
}

impl<S, T> Eq for QueueItem<S, T> where T: Eq {}

impl<S, T> PartialOrd for QueueItem<S, T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.priority.partial_cmp(&other.priority)
    }
}

impl<S, T> Ord for QueueItem<S, T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

struct Queue<'a, S, T> {
    heap: BinaryHeap<Reverse<QueueItem<'a, S, T>>>,
}

impl<'a, S, T> Queue<'a, S, T> {
    fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
        }
    }

    fn push(&mut self, node: Node<'a, S>, priority: T) {
        self.heap.push(Reverse(QueueItem {
            node,
            priority,

            skip: Cell::new(false),
        }));
    }

    fn contains(&self, node: &Node<'a, S>) -> bool {
        self.heap.iter().any(|Reverse(item)| &item.node == node)
    }

    fn decrease_priority(&mut self, node: Node<'a, S>, priority: T) {
        for Reverse(item) in &self.heap {
            if &item.node == &node {
                item.skip.set(true);
                break;
            }
        }

        self.heap.push(Reverse(QueueItem {
            node,
            priority,

            skip: Cell::new(false),
        }));
    }

    fn pop_min(&mut self) -> Option<Node<'a, S>> {
        while let Some(Reverse(item)) = self.heap.pop() {
            if !item.skip.get() {
                return Some(item.node);
            }
        }

        None
    }
}

fn reconstruct_intermediates<'a, S>(
    previous: &HashMap<&'a S::NodeId, Option<Node<'a, S>>>,
    target: &'a S::NodeId,
) -> Vec<Node<'a, S>>
where
    S: GraphStorage,
{
    let mut current = target;

    let mut path = Vec::new();

    while let Some(node) = previous[current] {
        path.push(node);
        current = node.id();
    }

    // remove the source node (last one)
    path.pop();
    path.reverse();

    path
}

#[derive(Debug)]
enum DijkstraError {
    NodeNotFound,
}

impl Display for DijkstraError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NodeNotFound => write!(f, "node not found"),
        }
    }
}

impl Context for DijkstraError {}

struct DijkstraIter<'a, S, T, F, G>
where
    S: GraphStorage,
{
    queue: Queue<'a, S, T>,

    edge_cost: F,
    connections: G,

    source: &'a S::NodeId,

    discard_intermediates: bool,

    distances: HashMap<&'a S::NodeId, T, FxBuildHasher>,
    previous: HashMap<&'a S::NodeId, Option<Node<'a, S>>, FxBuildHasher>,
}

impl<'a, S, T, U, F, G> DijkstraIter<'a, S, T, F, G>
where
    S: GraphStorage,
    F: FnMut(Edge<S>) -> T,
    T: PartialOrd<&T> + Zero + Clone,
    &T: Add<Output = T>,
    G: FnMut(Node<'a, S>) -> U,
    U: Iterator<Item = Edge<S>>,
{
    pub fn new(
        graph: &'a Graph<S>,
        edge_cost: F,
        connections: G,
        source: &'a S::NodeId,
        discard_intermediates: bool,
    ) -> Result<Self, DijkstraError> {
        let source_node = graph
            .node(source)
            .ok_or_else(|| Report::new(DijkstraError::NodeNotFound))?;

        let mut distances = HashMap::with_hasher(FxBuildHasher::default());
        let mut previous = HashMap::with_hasher(FxBuildHasher::default());

        let mut queue = Queue::new();

        distances.insert(source, T::zero());
        previous.insert(source, None);

        queue.push(source_node, T::zero());

        Ok(Self {
            queue,
            edge_cost,
            connections,
            source,
            discard_intermediates,
            distances,
            previous,
        })
    }
}

impl<'a, S, T, U, F, G> Iterator for DijkstraIter<'a, S, T, F, G>
where
    S: GraphStorage,
    F: FnMut(Edge<S>) -> T,
    T: PartialOrd<&T> + Clone,
    &T: Add<Output = T>,
    G: FnMut(Node<'a, S>) -> U,
    U: Iterator<Item = Edge<S>>,
{
    type Item = Route<'a, S, T>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(node) = self.queue.pop_min() {
            let connections = (self.connections)(node);

            for edge in connections {
                let (u, v) = edge.endpoints();
                let target = if v.id() == node.id() { u } else { v };

                let alternative = &self.distances[node.id()] + self.edge_cost(edge.weight());

                let mut insert = false;
                if let Some(distance) = self.distances.get(target.id()) {
                    if alternative < distance {
                        insert = true;
                    }
                } else {
                    insert = true;
                }

                if insert {
                    self.distances.insert(edge.id(), alternative.clone());
                    self.previous.insert(edge.id(), Some(target));
                    self.queue.decrease_priority(target, alternative);
                }
            }

            // we're currently visiting the node that has the shortest distance, therefore we know
            // that the distance is the shortest possible
            let distance = self.distances[node.id()].clone();
            let intermediates = if self.discard_intermediates {
                vec![]
            } else {
                reconstruct_intermediates(&self.previous, node.id());
            };

            let path = Path {
                source: self.source,
                target: node,
                intermediates,
            };

            return Some(Route { path, distance });
        }

        None
    }
}

#[non_exhaustive]
struct Dijkstra<D, F>
where
    D: GraphDirectionality,
{
    direction: D,

    edge_cost: F,
}

impl<S> ShortestPath<S> for Dijkstra<Undirected, ()>
where
    S: GraphStorage,
    S::NodeId: Hash,
    S::EdgeWeight: Zero + Clone,
    &S::EdgeWeight: Add<Output = S::EdgeWeight>,
{
    type Cost = S::EdgeWeight;

    fn from(
        self,
        graph: &Graph<S>,
        source: &S::NodeId,
    ) -> impl Iterator<Item = Route<S, Self::Cost>> {
        DijkstraIter::new(
            graph,
            |edge| edge.weight(),
            |node| node.connections(),
            source,
            false,
        )
    }

    fn every(self, graph: &Graph<S>) -> impl Iterator<Item = Route<S, Self::Cost>> {
        graph
            .nodes()
            .flat_map(move |node| self.from(graph, node.id()))
    }
}

impl<S> ShortestPath<S> for Dijkstra<Directed, ()>
where
    S: GraphStorage,
    S::NodeId: Hash,
    S::EdgeWeight: Zero + Clone,
    &S::EdgeWeight: Add<Output = S::EdgeWeight>,
{
    type Cost = S::EdgeWeight;

    fn from(
        self,
        graph: &Graph<S>,
        source: &S::NodeId,
    ) -> impl Iterator<Item = Route<S, Self::Cost>> {
        DijkstraIter::new(
            graph,
            |edge| edge.weight(),
            |node| node.directed_connections(Direction::Outgoing),
            source,
            false,
        )
    }

    fn every(self, graph: &Graph<S>) -> impl Iterator<Item = Route<S, Self::Cost>> {
        graph
            .nodes()
            .flat_map(move |node| self.from(graph, node.id()))
    }
}

impl<S, F, T> ShortestPath<S> for Dijkstra<Undirected, F>
where
    S: GraphStorage,
    F: FnMut(Edge<S>) -> T,
    T: Zero + Clone,
    &T: Add<Output = T>,
{
    type Cost = T;

    fn from(
        self,
        graph: &Graph<S>,
        source: &S::NodeId,
    ) -> impl Iterator<Item = Route<S, Self::Cost>> {
        DijkstraIter::new(
            graph,
            self.edge_cost,
            |node| node.connections(),
            source,
            false,
        )
    }

    fn every(self, graph: &Graph<S>) -> impl Iterator<Item = Route<S, Self::Cost>> {
        graph
            .nodes()
            .flat_map(move |node| self.from(graph, node.id()))
    }
}

impl<S, F, T> ShortestPath<S> for Dijkstra<Directed, F>
where
    S: GraphStorage,
    F: FnMut(Edge<S>) -> T,
    T: Zero + Clone,
    &T: Add<Output = T>,
{
    type Cost = T;

    fn from(
        self,
        graph: &Graph<S>,
        source: &S::NodeId,
    ) -> impl Iterator<Item = Route<S, Self::Cost>> {
        DijkstraIter::new(
            graph,
            self.edge_cost,
            |node| node.directed_connections(Direction::Outgoing),
            source,
            false,
        )
    }

    fn every(self, graph: &Graph<S>) -> impl Iterator<Item = Route<S, Self::Cost>> {
        graph
            .nodes()
            .flat_map(move |node| self.from(graph, node.id()))
    }
}

#[cfg(test)]
pub(super) mod tests {
    use petgraph_core::{
        edge::{Directed, Undirected},
        visit::{EdgeRef, IntoNodeReferences},
    };
    use petgraph_graph::{Graph, NodeIndex};
    use proptest::{prelude::*, sample::Index};

    use super::dijkstra;

    /// Uses the graph from networkx
    ///
    /// <https://github.com/networkx/networkx/blob/main/networkx/algorithms/shortest_paths/tests/test_weighted.py>
    pub fn setup() -> Graph<&'static str, i32> {
        let mut graph = Graph::new();

        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");
        let d = graph.add_node("D");
        let e = graph.add_node("E");

        graph.extend_with_edges([
            (a, b, 10),
            (a, c, 5),
            (b, d, 1),
            (b, c, 2),
            (d, e, 1),
            (c, b, 3),
            (c, d, 5),
            (c, e, 2),
            (e, a, 7),
            (e, d, 6),
        ]);

        graph
    }

    #[test]
    fn no_goal_directed() {
        let graph = setup();

        let node = |weight: &'static str| {
            graph
                .node_references()
                .find(|(_, &node_weight)| node_weight == weight)
                .unwrap()
                .0
        };

        let result = dijkstra(&graph, node("A"), None, |edge| *edge.weight());

        let expected = [
            (node("A"), 0),
            (node("B"), 8),
            (node("C"), 5),
            (node("D"), 9),
            (node("E"), 7),
        ];

        assert_eq!(result.len(), expected.len());

        for (node, weight) in expected {
            assert_eq!(result[&node], weight);
        }
    }

    #[test]
    fn no_goal_undirected() {
        let graph = setup().into_edge_type::<Undirected>();

        let node = |weight: &'static str| {
            graph
                .node_references()
                .find(|(_, &node_weight)| node_weight == weight)
                .unwrap()
                .0
        };

        let result = dijkstra(&graph, node("A"), None, |edge| *edge.weight());

        let expected = [
            (node("A"), 0),
            (node("B"), 7),
            (node("C"), 5),
            (node("D"), 8),
            (node("E"), 7),
        ];

        assert_eq!(result.len(), expected.len());

        for (node, weight) in expected {
            assert_eq!(result[&node], weight);
        }
    }

    #[test]
    fn goal_directed() {
        let graph = setup();

        let node = |weight: &'static str| {
            graph
                .node_references()
                .find(|(_, &node_weight)| node_weight == weight)
                .unwrap()
                .0
        };

        let result = dijkstra(&graph, node("A"), Some(node("D")), |edge| *edge.weight());

        // we only guarantee that A - D exists in the result
        assert_eq!(result[&node("D")], 9);
    }

    #[test]
    fn goal_undirected() {
        let graph = setup().into_edge_type::<Undirected>();

        let node = |weight: &'static str| {
            graph
                .node_references()
                .find(|(_, &node_weight)| node_weight == weight)
                .unwrap()
                .0
        };

        let result = dijkstra(&graph, node("A"), Some(node("D")), |edge| *edge.weight());

        // we only guarantee that A - D exists in the result
        assert_eq!(result[&node("D")], 8);
    }

    fn non_empty_graph() -> impl Strategy<Value = Graph<(), u8, Directed, u8>> {
        any::<Graph<(), u8, Directed, u8>>()
            .prop_filter("graph is empty", |graph| graph.node_count() > 0)
    }

    // #[cfg(not(miri))]
    // proptest! {
    //     #[test]
    //     fn triangle_inequality(
    //         graph in non_empty_graph(),
    //         node in any::<Index>()
    //     ) { let node = NodeIndex::new(node.index(graph.node_count())); let result =
    //       dijkstra(&graph, node, None, |edge| *edge.weight() as u32);
    //
    //         // triangle inequality:
    //         // d(v,u) <= d(v,v2) + d(v2,u)
    //         for (node, weight) in &result {
    //             for edge in graph.edges(*node) {
    //                 let next = edge.target();
    //                 let next_weight = *edge.weight() as u32;
    //
    //                 if result.contains_key(&next) {
    //                     assert!(result[&next] <= *weight + next_weight);
    //                 }
    //             }
    //         }
    //     }
    // }
}
