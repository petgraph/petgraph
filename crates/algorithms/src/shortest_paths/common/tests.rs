use alloc::vec::Vec;
use core::{fmt::Debug, hash::Hash};

use hashbrown::HashMap;
use petgraph_core::{Graph, GraphStorage, Node};
use petgraph_dino::NodeId;

use crate::shortest_paths::{ShortestDistance, ShortestPath};

pub(in crate::shortest_paths) fn assert_path<S, T>(
    mut received: HashMap<NodeId, (T, Vec<Node<S>>)>,
    expected: &[(NodeId, T, &[NodeId])],
) where
    S: GraphStorage<NodeId = NodeId>,
    T: PartialEq + Debug,
{
    assert_eq!(received.len(), expected.len());

    for (node, expected_distance, expected_route) in expected {
        let (distance, route) = received.remove(node).unwrap();
        let route: Vec<_> = route.into_iter().map(|node| *node.id()).collect();

        assert_eq!(distance, *expected_distance);
        assert_eq!(&route, expected_route);
    }
}

pub(in crate::shortest_paths) fn assert_distance<T>(
    mut received: HashMap<NodeId, T>,
    expected: &[(NodeId, T)],
) where
    T: PartialEq + Debug,
{
    assert_eq!(received.len(), expected.len());

    for (node, expected_distance) in expected {
        let distance = received.remove(node).unwrap();

        assert_eq!(distance, *expected_distance);
    }
}

pub(in crate::shortest_paths) fn path_from<'a, S, P>(
    graph: &'a Graph<S>,
    source: &'a S::NodeId,
    algorithm: &P,
) -> HashMap<S::NodeId, (P::Cost, Vec<Node<'a, S>>)>
where
    P: ShortestPath<S>,
    S: GraphStorage,
    S::NodeId: Copy + Eq + Hash,
{
    algorithm
        .path_from(graph, source)
        .unwrap()
        .map(|route| (*route.path.target.id(), (route.cost.0, route.path.to_vec())))
        .collect()
}

pub(in crate::shortest_paths) fn distance_from<'a, S, P>(
    graph: &'a Graph<S>,
    source: &'a S::NodeId,
    algorithm: &P,
) -> HashMap<S::NodeId, P::Cost>
where
    P: ShortestDistance<S>,
    S: GraphStorage,
    S::NodeId: Copy + Eq + Hash,
{
    algorithm
        .distance_from(graph, source)
        .unwrap()
        .map(|route| (*route.target.id(), route.cost.0))
        .collect()
}
