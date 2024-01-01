use alloc::vec::Vec;
use core::{
    fmt::{Debug, Display},
    hash::Hash,
};

use error_stack::Result;
use hashbrown::HashMap;
use petgraph_core::{Graph, GraphStorage, Node};

use crate::shortest_paths::{DirectRoute, Route, ShortestDistance, ShortestPath};

/// Helper Macro to create a map of expected results
///
/// Technically this macro is not necessarily needed, but it makes the test code much more
/// readable
macro_rules! expected {
    (@rule $nodes:ident; $source:ident; ($($path:ident),*) ;$target:ident; $cost:literal) => {
        $crate::shortest_paths::common::tests::Expect {
            source: $nodes.$source,
            target: $nodes.$target,
            transit: alloc::vec![$($nodes.$path),*],
            cost: $cost,
        }
    };
    (@rule $nodes:ident; $source:literal; ($($path:literal),*) ;$target:literal; $cost:literal) => {
        $crate::shortest_paths::common::tests::Expect {
            source: $nodes[$source],
            target: $nodes[$target],
            transit: alloc::vec![$($nodes[$path]),*],
            cost: $cost,
        }
    };
    ($nodes:ident; [$($source:tt -( $($path:tt),* )> $target:tt : $cost:literal),* $(,)?]) => {
        alloc::vec![
            $(expected!(@rule $nodes; $source; ($($path),*) ;$target; $cost)),*
        ]
    };
}

pub(in crate::shortest_paths) use expected;
use petgraph_core::node::NodeId;

pub(in crate::shortest_paths) struct Expect<T> {
    pub(in crate::shortest_paths) source: NodeId,
    pub(in crate::shortest_paths) target: NodeId,

    pub(in crate::shortest_paths) transit: Vec<NodeId>,

    pub(in crate::shortest_paths) cost: T,
}

pub(in crate::shortest_paths) struct TestCase<'a, S, A, T>
where
    S: GraphStorage,
{
    graph: &'a Graph<S>,
    algorithm: &'a A,
    expected: &'a [Expect<T>],
}

impl<'a, S, A, T> TestCase<'a, S, A, T>
where
    S: GraphStorage,
{
    pub(crate) const fn new(
        graph: &'a Graph<S>,
        algorithm: &'a A,
        expected: &'a [Expect<T>],
    ) -> Self {
        Self {
            graph,
            algorithm,
            expected,
        }
    }
}

impl<'a, S, A, T> TestCase<'a, S, A, T>
where
    S: GraphStorage,
    A: ShortestPath<S, Cost = T>,
    T: PartialEq + Debug,
{
    #[track_caller]
    fn assert_path_routes(&self, routes: Result<impl Iterator<Item = Route<'a, S, T>>, A::Error>) {
        let mut routes: HashMap<_, _> = routes
            .unwrap()
            .map(|route| {
                (
                    (route.path().source().id(), route.path().target().id()),
                    route,
                )
            })
            .collect();

        for expect in self.expected {
            let source = expect.source;
            let target = expect.target;

            let route = routes.remove(&(source, target)).expect("route not found");

            let (path, cost) = route.into_parts();

            assert_eq!(path.source().id(), source, "source of {source} -> {target}");
            assert_eq!(path.target().id(), target, "target of {source} -> {target}");
            assert_eq!(
                path.transit().iter().map(Node::id).collect::<Vec<_>>(),
                expect.transit.iter().copied().collect::<Vec<_>>(),
                "transit of {source} -> {target}"
            );
            assert_eq!(*cost.value(), expect.cost, "cost of {source} -> {target}");
        }

        assert!(routes.is_empty());
    }

    pub(in crate::shortest_paths) fn assert_every_path(&self) {
        self.assert_path_routes(self.algorithm.every_path(self.graph));
    }

    #[track_caller]
    pub(in crate::shortest_paths) fn assert_path_from(&self, source: NodeId) {
        self.assert_path_routes(self.algorithm.path_from(self.graph, source));
    }
}

impl<'a, S, A, T> TestCase<'a, S, A, T>
where
    S: GraphStorage,
    A: ShortestDistance<S, Cost = T>,
    T: PartialEq + Debug,
{
    #[track_caller]
    fn assert_distance_routes(
        &self,
        routes: Result<impl Iterator<Item = DirectRoute<'a, S, T>>, A::Error>,
    ) {
        let mut routes: HashMap<_, _> = routes
            .unwrap()
            .map(|route| ((route.source().id(), route.target().id()), route))
            .collect();

        for expect in self.expected {
            let source = expect.source;
            let target = expect.target;

            let route = routes.remove(&(source, target)).expect("route not found");

            assert_eq!(
                route.source().id(),
                source,
                "source of {source} -> {target}"
            );
            assert_eq!(
                route.target().id(),
                target,
                "target of {source} -> {target}"
            );

            assert_eq!(
                *route.cost().value(),
                expect.cost,
                "cost of {source} -> {target}"
            );
        }

        assert!(routes.is_empty());
    }

    pub(in crate::shortest_paths) fn assert_every_distance(&self) {
        self.assert_distance_routes(self.algorithm.every_distance(self.graph));
    }

    #[track_caller]
    pub(in crate::shortest_paths) fn assert_distance_from(&self, source: NodeId) {
        self.assert_distance_routes(self.algorithm.distance_from(self.graph, source));
    }
}
