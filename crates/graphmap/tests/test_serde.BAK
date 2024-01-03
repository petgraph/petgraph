#![cfg(feature = "serde")]

use std::fmt::Debug;

use insta::assert_json_snapshot;
use petgraph_core::edge::{Directed, EdgeType};
use petgraph_graph::Graph;
use petgraph_graphmap::{EntryStorage, NodeTrait};
#[cfg(feature = "proptest")]
use proptest::prelude::*;
use serde::{Deserialize, Serialize};

fn example() -> EntryStorage<i32, u32, Directed> {
    let [a, b, c, d, e, f] = [0, 1, 2, 3, 4, 5];

    let mut graph = EntryStorage::from_edges([
        (a, b, 7),
        (c, a, 9),
        (a, d, 14),
        (b, c, 10),
        (d, c, 2),
        (d, e, 9),
        (b, f, 15),
        (c, f, 11),
        (e, f, 6),
    ]);

    // add disconnected node
    graph.add_node(42);

    graph
}

fn assert_graph_eq<N, E, Ty>(graph1: &EntryStorage<N, E, Ty>, graph2: &EntryStorage<N, E, Ty>)
where
    N: NodeTrait + PartialEq + Debug,
    E: PartialEq + Debug,
    Ty: EdgeType,
{
    let nodes1 = graph1.nodes().collect::<Vec<_>>();
    let nodes2 = graph2.nodes().collect::<Vec<_>>();

    assert_eq!(nodes1, nodes2);

    let edges1 = graph1.all_edges().collect::<Vec<_>>();
    let edges2 = graph2.all_edges().collect::<Vec<_>>();

    assert_eq!(edges1, edges2);
}

#[test]
#[cfg(not(miri))]
fn serialize() {
    let graph = example();

    assert_json_snapshot!(graph);
}

#[test]
fn deserialize() {
    let graph = example();
    let serialized = serde_value::to_value(&graph).unwrap();
    let deserialized: EntryStorage<i32, u32, Directed> =
        EntryStorage::deserialize(serialized).unwrap();

    assert_graph_eq(&graph, &deserialized);
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct Point {
    pub x: u32,
    pub y: i32,
}

fn example_struct() -> EntryStorage<Point, i32, Directed> {
    let a = Point { x: 0, y: 0 };
    let b = Point { x: 1, y: 1 };
    let c = Point { x: 2, y: 2 };

    let d = Point { x: 2, y: 1 };

    let mut graph = EntryStorage::from_edges([
        (a, b, 1), //
        (b, c, 2),
        (c, a, 3),
    ]);

    graph.add_node(d);

    graph
}

#[test]
#[cfg(not(miri))]
fn serialize_struct() {
    let graph = example_struct();

    assert_json_snapshot!(graph);
}

#[test]
fn deserialize_struct() {
    let graph = example_struct();

    let serialized = serde_value::to_value(&graph).unwrap();
    let deserialized: EntryStorage<Point, i32, Directed> =
        EntryStorage::deserialize(serialized).unwrap();

    assert_graph_eq(&graph, &deserialized);
}

#[cfg(feature = "proptest")]
#[cfg(not(miri))]
proptest! {
    #[test]
    fn roundtrip(graph in any::<GraphMap<i32, u32, Directed>>()) {
        let serialized = serde_value::to_value(&graph).unwrap();
        let deserialized: GraphMap<i32, u32, Directed> = GraphMap::deserialize(serialized).unwrap();

        assert_graph_eq(&graph, &deserialized);
    }

    #[test]
    fn roundtrip_graphmap_to_graph_to_graphmap(graph in any::<GraphMap<i32, u32, Directed>>()) {
        let serialized = serde_value::to_value(&graph).unwrap();
        let deserialized: Graph<i32, u32, Directed> = Graph::deserialize(serialized).unwrap();

        let serialized = serde_value::to_value(deserialized).unwrap();
        let reserialized: GraphMap<i32, u32, Directed> = GraphMap::deserialize(serialized).unwrap();

        assert_graph_eq(&graph, &reserialized);
    }
}
