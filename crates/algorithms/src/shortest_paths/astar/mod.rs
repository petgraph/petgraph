mod error;
mod heuristic;
mod r#impl;
#[cfg(test)]
mod tests;

use alloc::vec::Vec;
use core::{hash::Hash, ops::Add};

use error_stack::Result;
use num_traits::Zero;
use petgraph_core::{
    base::MaybeOwned,
    edge::{
        marker::{Directed, Undirected},
        Direction,
    },
    DirectedGraphStorage, Edge, Graph, GraphDirectionality, GraphStorage, Node,
};

use self::{error::AStarError, r#impl::AStarImpl};
use super::{
    common::intermediates::Intermediates, Cost, DirectRoute, Route, ShortestDistance, ShortestPath,
};
use crate::{
    polyfill::IteratorExt,
    shortest_paths::{astar::heuristic::GraphHeuristic, common::cost::GraphCost},
};

fn outgoing_connections<'a, S>(node: &Node<'a, S>) -> impl Iterator<Item = Edge<'a, S>> + 'a
where
    S: DirectedGraphStorage,
{
    node.directed_connections(Direction::Outgoing)
}

pub struct AStar<D, E, H>
where
    D: GraphDirectionality,
{
    direction: D,

    edge_cost: E,
    heuristic: H,
}

impl AStar<Directed, (), ()> {
    pub fn directed() -> Self {
        Self {
            direction: Directed,
            edge_cost: (),
            heuristic: (),
        }
    }
}

impl AStar<Undirected, (), ()> {
    pub fn undirected() -> Self {
        Self {
            direction: Undirected,
            edge_cost: (),
            heuristic: (),
        }
    }
}

impl<D, E, H> AStar<D, E, H>
where
    D: GraphDirectionality,
{
    pub fn with_edge_cost<E2>(self, edge_cost: E2) -> AStar<D, E2, H> {
        AStar {
            direction: self.direction,
            edge_cost,
            heuristic: self.heuristic,
        }
    }

    pub fn without_edge_cost(self) -> AStar<D, (), H> {
        AStar {
            direction: self.direction,
            edge_cost: (),
            heuristic: self.heuristic,
        }
    }

    pub fn with_heuristic<H2>(self, heuristic: H2) -> AStar<D, E, H2> {
        AStar {
            direction: self.direction,
            edge_cost: self.edge_cost,
            heuristic,
        }
    }
}

macro_rules! call {
    (@impl direction($a:lifetime)undirected) => {
        Node::<$a, S>::connections as fn(&Node<$a, S>) -> _
    };
    (@impl direction($self:ident)directed) => {
        outgoing_connections as fn(&Node<$a, S>) -> _
    };
    (
        $a:lifetime,
        $self:ident,
        $graph:ident,
        $source:ident,
        $target:ident,edge = $edge:ident,direction = $direction:ident,intermediates =
        $intermediates:ident
    ) => {{
        AStarImpl::new(
            $graph,
            &$self.edge_cost,
            &$self.heuristic,
            call!(@impl direction($a) $direction),
            $source,
            $target,
            Intermediates::$intermediates,
        )
    }};
}

macro_rules! methods {
    (@impl any to(name=$name:ident, edge=$edge:ident, direction=$direction:ident, intermediates=$intermediates:ident, route=$route:ident, find=$find:ident)) => {
        fn $name<'a>(
            &self,
            graph: &'a Graph<S>,
            target: &'a S::NodeId,
        ) -> Result<impl Iterator<Item = $route<'a, S, Self::Cost>>, Self::Error> {
            let sources = graph.nodes().map(|node| node.id());

            let iter = sources
                .map(move |source| {
                    call!(
                        'a,
                        self,
                        graph,
                        source,
                        target,
                        edge = $edge,
                        direction = $direction,
                        intermediates = $intermediates
                    )
                })
                .collect_reports::<Vec<_>>()?;

            Ok(AStarImpl::$find(iter))
        }
    };
    (@impl path to(edge=$edge:ident, direction=$direction:ident)) => {
        methods!(@impl any to(name=path_to, edge=$edge, direction=$direction, intermediates=Discard, route=Route, find=find_all))
    };
    (@impl distance to(edge=$edge:ident, direction=$direction:ident)) => {
        methods!(@impl any to(name=distance_to, edge=$edge, direction=$direction, intermediates=Discard, route=DirectRoute, find=find_all_direct))
    };
    (@impl any from(name=$name:ident, edge=$edge:ident, direction=$direction:ident, intermediates=$intermediates:ident, route=$route:ident, find=$find:ident)) => {
        fn $name<'a>(
            &self,
            graph: &'a Graph<S>,
            source: &'a S::NodeId,
        ) -> Result<impl Iterator<Item = $route<'a, S, Self::Cost>>, Self::Error> {
            let targets = graph.nodes().map(|node| node.id());

            let iter = targets
                .map(move |target| {
                    call!(
                        'a,
                        self,
                        graph,
                        source,
                        target,
                        edge = $edge,
                        direction = $direction,
                        intermediates = $intermediates
                    )
                })
                .collect_reports::<Vec<_>>()?;

            Ok(AStarImpl::$find(iter))
        }
    };
    (@impl path from(edge=$edge:ident, direction=$direction:ident)) => {
        methods!(@impl any from(name=path_from, edge=$edge, direction=$direction, intermediates=Record, route=Route, find=find_all))
    };
    (@impl distance from(edge=$edge:ident, direction=$direction:ident)) => {
        methods!(@impl any from(name=distance_from, edge=$edge, direction=$direction, intermediates=Discard, route=DirectRoute, find=find_all_direct))
    };
    (@impl any between(edge=$edge:ident, direction=$direction:ident)) => {
        fn path_between<'a>(
            &self,
            graph: &'a Graph<S>,
            source: &'a S::NodeId,
            target: &'a S::NodeId,
        ) -> Option<Route<'a, S, Self::Cost>> {
            call!(
                'a,
                self,
                graph,
                source,
                target,
                edge = default,
                direction = undirected,
                intermediates = Discard
            )
            .ok()?
            .find()
        }
    }
}

impl<S, E, H> ShortestPath<S> for AStar<Undirected, E, H>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
    E: GraphCost<S>,
    for<'a> E::Value: PartialOrd + Ord + Zero + Clone + 'a,
    for<'a> &'a E::Value: Add<Output = E::Value>,
    H: GraphHeuristic<S, Value = E::Value>,
{
    type Cost = E::Value;
    type Error = AStarError;

    fn path_to<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        target: &'graph <S as GraphStorage>::NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let sources = graph.nodes().map(|node| node.id());

        let iter = sources
            .map(move |source| {
                call!(
                    'graph,
                    self,
                    graph,
                    source,
                    target,
                    edge = default,
                    direction = undirected,
                    intermediates = Record
                )
            })
            .collect_reports::<Vec<_>>()?;

        Ok(AStarImpl::find_all(iter))
    }

    fn path_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: &'graph <S as GraphStorage>::NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let targets = graph.nodes().map(|node| node.id());

        let iter = targets
            .map(move |target| {
                call!(
                    'graph,
                    self,
                    graph,
                    source,
                    target,
                    edge = default,
                    direction = undirected,
                    intermediates = Record
                )
            })
            .collect_reports::<Vec<_>>()?;

        Ok(AStarImpl::find_all(iter))
    }

    fn path_between<'graph>(
        &self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
        target: &'graph S::NodeId,
    ) -> Option<Route<'graph, S, Self::Cost>> {
        call!(
            'graph,
            self,
            graph,
            source,
            target,
            edge = default,
            direction = undirected,
            intermediates = Discard
        )
        .ok()?
        .find()
    }

    fn every_path<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let sources = graph.nodes().map(|node| node.id());

        let iter = sources
            .flat_map(move |source| {
                graph.nodes().map(|node| node.id()).map(move |target| {
                    call!(
                        'graph,
                        self,
                        graph,
                        source,
                        target,
                        edge = default,
                        direction = undirected,
                        intermediates = Record
                    )
                })
            })
            .collect_reports::<Vec<_>>()?;

        Ok(iter.into_iter().filter_map(|iter| iter.find()))
    }
}

// impl<S, H> ShortestDistance<S> for AStar<Undirected, (), H>
// where
//     S: GraphStorage,
//     S::NodeId: Eq + Hash,
//     S::EdgeWeight: PartialOrd + Ord + Zero + Clone,
//     for<'a> &'a S::EdgeWeight: Add<Output = S::EdgeWeight>,
//     H: for<'a> Fn(Node<'a, S>, Node<'a, S>) -> MaybeOwned<'a, S::EdgeWeight>,
// {
//     type Cost = S::EdgeWeight;
//     type Error = AStarError;
//
//     fn distance_to<'a>(
//         &self,
//         graph: &'a Graph<S>,
//         target: &'a S::NodeId,
//     ) -> Result<impl Iterator<Item = DirectRoute<'a, S, Self::Cost>>, Self::Error> { let sources
//       = graph.nodes().map(|node| node.id());
//
//         let iter = sources
//             .map(move |source| {
//                 call!(
//                     'a,
//                     self,
//                     graph,
//                     source,
//                     target,
//                     edge = default,
//                     direction = undirected,
//                     intermediates = Discard
//                 )
//             })
//             .collect_reports::<Vec<_>>()?;
//
//         Ok(AStarImpl::find_all_direct(iter))
//     }
//
//     fn distance_from<'a>(
//         &self,
//         graph: &'a Graph<S>,
//         source: &'a S::NodeId,
//     ) -> Result<impl Iterator<Item = DirectRoute<'a, S, Self::Cost>>, Self::Error> { let targets
//       = graph.nodes().map(|node| node.id());
//
//         let iter = targets
//             .map(move |target| {
//                 call!(
//                     'a,
//                     self,
//                     graph,
//                     source,
//                     target,
//                     edge = default,
//                     direction = undirected,
//                     intermediates = Discard
//                 )
//             })
//             .collect_reports::<Vec<_>>()?;
//
//         Ok(AStarImpl::find_all_direct(iter))
//     }
//
//     fn distance_between<'a>(
//         &self,
//         graph: &'a Graph<S>,
//         source: &'a S::NodeId,
//         target: &'a S::NodeId,
//     ) -> Option<Cost<Self::Cost>> { call!( 'a, self, graph, source, target, edge = default,
//       direction = undirected, intermediates = Discard ) .ok()? .find() .map(|route| route.cost)
//     }
//
//     fn every_distance<'a>(
//         &self,
//         graph: &'a Graph<S>,
//     ) -> Result<impl Iterator<Item = DirectRoute<'a, S, Self::Cost>>, Self::Error> { let sources
//       = graph.nodes().map(|node| node.id());
//
//         let iter = sources
//             .flat_map(move |source| {
//                 graph.nodes().map(|node| node.id()).map(move |target| {
//                     call!(
//                         'a,
//                         self,
//                         graph,
//                         source,
//                         target,
//                         edge = default,
//                         direction = undirected,
//                         intermediates = Discard
//                     )
//                 })
//             })
//             .collect_reports::<Vec<_>>()?;
//
//         Ok(AStarImpl::find_all_direct(iter))
//     }
// }
