use alloc::{collections::BinaryHeap, vec, vec::Vec};
use core::{
    cell::Cell,
    cmp::{Ordering, Reverse},
    fmt::{Display, Formatter},
    hash::{BuildHasher, Hash},
    mem,
    ops::Add,
};

use error_stack::{Context, Report, Result};
use fxhash::FxBuildHasher;
use hashbrown::HashMap;
use num_traits::Zero;
use petgraph_core::{
    base::MaybeOwned,
    edge::{
        marker::{Directed, Undirected},
        Direction,
    },
    DirectedGraphStorage, Edge, Graph, GraphDirectionality, GraphStorage, Node,
};

use crate::shortest_paths::{DirectRoute, Distance, Path, Route, ShortestDistance, ShortestPath};

macro_rules! fold {
    ($iter:expr => flatten) => {
        $iter
            .fold(Ok(vec![]), |acc, value| match (acc, value) {
                (Ok(mut acc), Ok(value)) => {
                    acc.extend(value);
                    Ok(acc)
                }
                (Err(mut acc), Err(error)) => {
                    acc.extend_one(error);
                    Err(acc)
                }
                (Err(err), _) | (_, Err(err)) => Err(err),
            })
            .map(|value| value.into_iter())
    };
}

struct QueueItem<'a, S, T>
where
    S: GraphStorage,
{
    node: Node<'a, S>,

    priority: T,

    skip: Cell<bool>,
}

impl<S, T> PartialEq for QueueItem<'_, S, T>
where
    S: GraphStorage,
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.priority.eq(&other.priority)
    }
}

impl<S, T> Eq for QueueItem<'_, S, T>
where
    S: GraphStorage,
    T: Eq,
{
}

impl<S, T> PartialOrd for QueueItem<'_, S, T>
where
    S: GraphStorage,
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.priority.partial_cmp(&other.priority)
    }
}

impl<S, T> Ord for QueueItem<'_, S, T>
where
    S: GraphStorage,
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

struct Queue<'a, S, T>
where
    S: GraphStorage,
    T: Ord,
{
    heap: BinaryHeap<Reverse<QueueItem<'a, S, T>>>,
}

impl<'a, S, T> Queue<'a, S, T>
where
    S: GraphStorage,
    T: Ord,
{
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

    fn decrease_priority(&mut self, node: Node<'a, S>, priority: T) {
        for Reverse(item) in &self.heap {
            if item.node.id() == node.id() {
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

fn reconstruct_intermediates<'a, S, H>(
    previous: &HashMap<&'a S::NodeId, Option<Node<'a, S>>, H>,
    target: &'a S::NodeId,
) -> Vec<Node<'a, S>>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    H: BuildHasher,
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
pub enum DijkstraError {
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

fn outgoing_connections<'a, S>(node: &Node<'a, S>) -> impl Iterator<Item = Edge<'a, S>> + 'a
where
    S: DirectedGraphStorage,
{
    node.directed_connections(Direction::Outgoing)
}

trait ConnectionFn<'a, S>
where
    S: GraphStorage,
{
    fn connections(&self, node: &Node<'a, S>) -> impl Iterator<Item = Edge<'a, S>> + 'a;
}

impl<'a, S, I> ConnectionFn<'a, S> for fn(&Node<'a, S>) -> I
where
    S: GraphStorage,
    I: Iterator<Item = Edge<'a, S>> + 'a,
{
    fn connections(&self, node: &Node<'a, S>) -> impl Iterator<Item = Edge<'a, S>> + 'a {
        (*self)(node)
    }
}

struct DijkstraIter<'a, S, T, F, G>
where
    S: GraphStorage,
    T: Ord,
{
    queue: Queue<'a, S, T>,

    edge_cost: F,
    connections: G,

    source: Node<'a, S>,

    discard_intermediates: bool,

    distances: HashMap<&'a S::NodeId, T, FxBuildHasher>,
    previous: HashMap<&'a S::NodeId, Option<Node<'a, S>>, FxBuildHasher>,
}

impl<'a, S, T, F, G> DijkstraIter<'a, S, T, F, G>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    F: Fn(Edge<'a, S>) -> MaybeOwned<'a, T>,
    T: PartialOrd + Ord + Zero + Clone + 'a,
    for<'b> &'b T: Add<Output = T>,
    G: ConnectionFn<'a, S>,
{
    fn new(
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
        if !discard_intermediates {
            previous.insert(source, None);
        }

        queue.push(source_node, T::zero());

        Ok(Self {
            queue,
            edge_cost,
            connections,
            source: source_node,
            discard_intermediates,
            distances,
            previous,
        })
    }
}

impl<'a, S, T, F, G> Iterator for DijkstraIter<'a, S, T, F, G>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    F: Fn(Edge<'a, S>) -> MaybeOwned<'a, T>,
    T: PartialOrd + Ord + Zero + Clone + 'a,
    for<'b> &'b T: Add<Output = T>,
    G: ConnectionFn<'a, S>,
{
    type Item = Route<'a, S, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.queue.pop_min()?;

        let connections = self.connections.connections(&node);

        for edge in connections {
            let (u, v) = edge.endpoints();
            let target = if v.id() == node.id() { u } else { v };

            let alternative = &self.distances[node.id()] + (self.edge_cost)(edge).as_ref();

            let mut insert = false;
            if let Some(distance) = self.distances.get(target.id()) {
                if alternative < *distance {
                    insert = true;
                }
            } else {
                insert = true;
            }

            if insert {
                self.distances.insert(target.id(), alternative.clone());

                if !self.discard_intermediates {
                    self.previous.insert(target.id(), Some(node));
                }

                self.queue.decrease_priority(target, alternative);
            }
        }

        // we're currently visiting the node that has the shortest distance, therefore we know
        // that the distance is the shortest possible
        let distance = self.distances[node.id()].clone();
        let intermediates = if self.discard_intermediates {
            vec![]
        } else {
            reconstruct_intermediates(&self.previous, node.id())
        };

        let path = Path {
            source: self.source,
            target: node,
            intermediates,
        };

        Some(Route {
            path,
            distance: Distance { value: distance },
        })
    }
}

pub struct Dijkstra<D, F>
where
    D: GraphDirectionality,
{
    direction: D,

    edge_cost: F,
}

impl Dijkstra<Directed, ()> {
    pub fn directed() -> Self {
        Self {
            direction: Directed,
            edge_cost: (),
        }
    }
}

impl Dijkstra<Undirected, ()> {
    pub fn undirected() -> Self {
        Self {
            direction: Undirected,
            edge_cost: (),
        }
    }
}

impl<D, F> Dijkstra<D, F>
where
    D: GraphDirectionality,
{
    pub fn with_edge_cost<S, F2, T>(self, edge_cost: F2) -> Dijkstra<D, F2>
    where
        F2: Fn(Edge<S>) -> MaybeOwned<T>,
    {
        Dijkstra {
            direction: self.direction,
            edge_cost,
        }
    }

    pub fn without_edge_cost(self) -> Dijkstra<D, ()> {
        Dijkstra {
            direction: self.direction,
            edge_cost: (),
        }
    }
}

impl<S> ShortestPath<S> for Dijkstra<Undirected, ()>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    S::EdgeWeight: PartialOrd + Ord + Zero + Clone,
    for<'a> &'a S::EdgeWeight: Add<Output = S::EdgeWeight>,
{
    type Cost = S::EdgeWeight;
    type Error = DijkstraError;

    fn path_from<'a>(
        &self,
        graph: &'a Graph<S>,
        source: &'a S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'a, S, Self::Cost>>, Self::Error> {
        DijkstraIter::new(
            graph,
            |edge| MaybeOwned::Borrowed(edge.weight()),
            Node::<'a, S>::connections as fn(&Node<'a, S>) -> _,
            source,
            false,
        )
    }

    fn every_path<'a>(
        &self,
        graph: &'a Graph<S>,
    ) -> Result<impl Iterator<Item = Route<'a, S, Self::Cost>>, Self::Error> {
        let iter = graph
            .nodes()
            .map(move |node| self.path_from(graph, node.id()));

        fold!(iter => flatten)
    }
}

impl<S> ShortestPath<S> for Dijkstra<Directed, ()>
where
    S: DirectedGraphStorage,
    S::NodeId: Eq + Hash,
    S::EdgeWeight: PartialOrd + Ord + Zero + Clone,
    for<'a> &'a S::EdgeWeight: Add<Output = S::EdgeWeight>,
{
    type Cost = S::EdgeWeight;
    type Error = DijkstraError;

    fn path_from<'a>(
        &self,
        graph: &'a Graph<S>,
        source: &'a S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'a, S, Self::Cost>>, Self::Error> {
        DijkstraIter::new(
            graph,
            |edge| MaybeOwned::Borrowed(edge.weight()),
            outgoing_connections as fn(&Node<'a, S>) -> _,
            source,
            false,
        )
    }

    fn every_path<'a>(
        &self,
        graph: &'a Graph<S>,
    ) -> Result<impl Iterator<Item = Route<'a, S, Self::Cost>>, Self::Error> {
        let iter = graph
            .nodes()
            .map(move |node| self.path_from(graph, node.id()));

        fold!(iter => flatten)
    }
}

impl<S, F, T> ShortestPath<S> for Dijkstra<Undirected, F>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    F: Fn(Edge<S>) -> MaybeOwned<T>,
    for<'a> T: PartialOrd + Ord + Zero + Clone + 'a,
    for<'a> &'a T: Add<Output = T>,
{
    type Cost = T;
    type Error = DijkstraError;

    fn path_from<'a>(
        &self,
        graph: &'a Graph<S>,
        source: &'a S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'a, S, Self::Cost>>, Self::Error> {
        DijkstraIter::new(
            graph,
            &self.edge_cost,
            Node::<'a, S>::connections as fn(&Node<'a, S>) -> _,
            source,
            false,
        )
    }

    fn every_path<'a>(
        &self,
        graph: &'a Graph<S>,
    ) -> Result<impl Iterator<Item = Route<'a, S, Self::Cost>>, Self::Error> {
        let iter = graph
            .nodes()
            .map(move |node| self.path_from(graph, node.id()));

        fold!(iter => flatten)
    }
}

impl<S, F, T> ShortestPath<S> for Dijkstra<Directed, F>
where
    S: DirectedGraphStorage,
    S::NodeId: Eq + Hash,
    F: Fn(Edge<S>) -> MaybeOwned<T>,
    for<'a> T: PartialOrd + Ord + Zero + Clone + 'a,
    for<'a> &'a T: Add<Output = T>,
{
    type Cost = T;
    type Error = DijkstraError;

    fn path_from<'a>(
        &self,
        graph: &'a Graph<S>,
        source: &'a S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'a, S, Self::Cost>>, Self::Error> {
        DijkstraIter::new(
            graph,
            &self.edge_cost,
            outgoing_connections as fn(&Node<'a, S>) -> _,
            source,
            false,
        )
    }

    fn every_path<'a>(
        &self,
        graph: &'a Graph<S>,
    ) -> Result<impl Iterator<Item = Route<'a, S, Self::Cost>>, Self::Error> {
        let iter = graph
            .nodes()
            .map(move |node| self.path_from(graph, node.id()));

        fold!(iter => flatten)
    }
}

#[cfg(test)]
pub(super) mod tests {
    use hashbrown::HashMap;
    use petgraph_core::{base::MaybeOwned, edge::marker::Directed, Edge, GraphStorage};
    use petgraph_dino::{DiDinoGraph, DinoStorage, EdgeId, NodeId};
    use petgraph_utils::{graph, GraphCollection};

    use crate::shortest_paths::{dijkstra::Dijkstra, ShortestPath};

    graph!(
        /// Uses the graph from networkx
        ///
        /// <https://github.com/networkx/networkx/blob/main/networkx/algorithms/shortest_paths/tests/test_weighted.py>
        factory(networkx) => DiDinoGraph<&'static str, i32>;
        [
            a: "A",
            b: "B",
            c: "C",
            d: "D",
            e: "E",
        ] as NodeId, [
            ab: a -> b: 10,
            ac: a -> c: 5,
            bd: b -> d: 1,
            bc: b -> c: 2,
            de: d -> e: 1,
            cb: c -> b: 3,
            cd: c -> d: 5,
            ce: c -> e: 2,
            ea: e -> a: 7,
            ed: e -> d: 6,
        ] as EdgeId
    );

    graph!(
        /// Uses a randomly generated graph
        factory(random) => DiDinoGraph<&'static str, &'static str>;
        [
            a: "A",
            b: "B",
            c: "C",
            d: "D",
            e: "E",
            f: "F",
        ] as NodeId, [
            ab: a -> b: "apple",
            bc: b -> c: "cat",
            cd: c -> d: "giraffe",
            de: d -> e: "is",
            ef: e -> f: "banana",
            fa: f -> a: "bear",
            ad: a -> d: "elephant",
        ] as EdgeId
    );

    #[test]
    fn every_directed_default_edge_cost() {
        let GraphCollection {
            graph,
            nodes,
            edges,
        } = networkx::create();

        let dijkstra = Dijkstra::directed();

        let routes: HashMap<_, _> = dijkstra
            .path_from(&graph, &nodes.a)
            .unwrap()
            .map(|route| (*route.path.target.id(), route.distance.value))
            .collect();

        let expected = [
            (nodes.a, 0),
            (nodes.b, 8),
            (nodes.c, 5),
            (nodes.d, 9),
            (nodes.e, 7),
        ];

        // TODO: path

        assert_eq!(routes.len(), expected.len());

        for (node, weight) in expected {
            assert_eq!(routes[&node], weight);
        }
    }

    fn edge_cost<S>(edge: Edge<S>) -> MaybeOwned<'_, usize>
    where
        S: GraphStorage,
        S::EdgeWeight: AsRef<[u8]>,
    {
        MaybeOwned::Owned(edge.weight().as_ref().len())
    }

    #[test]
    fn every_directed_custom_edge_cost() {
        let GraphCollection {
            graph,
            nodes,
            edges,
        } = random::create();

        let dijkstra = Dijkstra::directed().with_edge_cost(edge_cost);

        let routes: HashMap<_, _> = dijkstra
            .path_from(&graph, &nodes.a)
            .unwrap()
            .map(|route| (*route.path.target.id(), route.distance.value))
            .collect();

        let expected = [
            (nodes.a, 0),
            (nodes.b, 5),
            (nodes.c, 8),
            (nodes.d, 8),
            (nodes.e, 10),
            (nodes.f, 16),
        ];

        // TODO: path

        assert_eq!(routes.len(), expected.len());

        for (node, weight) in expected {
            assert_eq!(routes[&node], weight);
        }
    }

    #[test]
    fn every_undirected_default_edge_cost() {
        let GraphCollection {
            graph,
            nodes,
            edges,
        } = networkx::create();

        let dijkstra = Dijkstra::undirected();
        let result: HashMap<_, _> = dijkstra
            .path_from(&graph, &nodes.a)
            .unwrap()
            .map(|route| (*route.path.target.id(), route.distance.value))
            .collect();

        let expected = [
            (nodes.a, 0),
            (nodes.b, 7),
            (nodes.c, 5),
            (nodes.d, 8),
            (nodes.e, 7),
        ];

        // TODO: path

        assert_eq!(result.len(), expected.len());

        for (node, weight) in expected {
            assert_eq!(result[&node], weight);
        }
    }

    //
    // fn non_empty_graph() -> impl Strategy<Value = Graph<(), u8, Directed, u8>> {
    //     any::<Graph<(), u8, Directed, u8>>()
    //         .prop_filter("graph is empty", |graph| graph.node_count() > 0)
    // }

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
