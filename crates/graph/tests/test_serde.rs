#![cfg(feature = "serde")]

use std::{collections::BTreeSet, fmt::Debug};

use insta::assert_json_snapshot;
use petgraph_core::{
    edge::{Directed, Direction, EdgeType, Undirected},
    index::{DefaultIx, IndexType},
    visit::EdgeRef,
};
use petgraph_graph::{DiGraph, Graph};
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::json;

fn assert_graph_eq<N, N2, E, Ty, Ix>(graph1: &Graph<N, E, Ty, Ix>, graph2: &Graph<N2, E, Ty, Ix>)
where
    N: PartialEq<N2> + Debug,
    N2: PartialEq<N2> + Debug,
    E: PartialEq + Debug,
    Ty: EdgeType,
    Ix: IndexType,
{
    assert_eq!(graph1.node_count(), graph2.node_count());
    assert_eq!(graph1.edge_count(), graph2.edge_count());

    // same node weights
    graph1
        .raw_nodes()
        .iter()
        .zip(graph2.raw_nodes().iter())
        .for_each(|(n1, n2)| assert_eq!(n1.weight, n2.weight));

    // same edge weights
    graph1
        .raw_edges()
        .iter()
        .zip(graph2.raw_edges().iter())
        .for_each(|(n1, n2)| assert_eq!(n1.weight, n2.weight));

    for edge in graph1.edge_references() {
        let (source, target) = graph2.edge_endpoints(edge.id()).expect("edge not found");

        assert_eq!(edge.source(), source);
        assert_eq!(edge.target(), target);
    }

    for index in graph1.node_indices() {
        let outgoing1: BTreeSet<_> = graph1
            .neighbors_directed(index, Direction::Outgoing)
            .collect();
        let outgoing2: BTreeSet<_> = graph2
            .neighbors_directed(index, Direction::Outgoing)
            .collect();

        assert_eq!(outgoing1, outgoing2);

        let incoming1: BTreeSet<_> = graph1
            .neighbors_directed(index, Direction::Incoming)
            .collect();
        let incoming2: BTreeSet<_> = graph2
            .neighbors_directed(index, Direction::Incoming)
            .collect();

        assert_eq!(incoming1, incoming2);
    }
}

fn example<Ty, Ix>() -> Graph<&'static str, i32, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    let mut graph = Graph::default();

    let a = graph.add_node("A");
    let b = graph.add_node("B");
    let c = graph.add_node("C");
    let d = graph.add_node("D");
    let e = graph.add_node("E");
    let f = graph.add_node("F");

    graph.extend_with_edges([
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

    // we remove `d` to ensure that holes are handled correctly
    graph.remove_node(d);

    graph
}

#[test]
fn node_str_edges_i32_serialize() {
    let graph: DiGraph<_, _> = example();

    assert_json_snapshot!(&graph);
}

#[test]
fn node_str_edges_i32_deserialize() {
    let graph: DiGraph<_, _> = example();

    let value = serde_value::to_value(graph.clone()).expect("failed to serialize");

    let graph2: DiGraph<String, i32> = DiGraph::deserialize(value).expect("failed to deserialize");

    assert_graph_eq(&graph, &graph2);
}

#[test]
fn node_null_edges_null_serialize() {
    let graph: DiGraph<(), ()> = example().map(|_, _| (), |_, _| ());

    assert_json_snapshot!(&graph);
}

#[test]
fn node_null_edges_null_deserialize() {
    let graph: DiGraph<(), ()> = example().map(|_, _| (), |_, _| ());
    let value = serde_value::to_value(graph.clone()).expect("failed to serialize");

    let graph2: DiGraph<(), ()> = DiGraph::deserialize(value).expect("failed to deserialize");

    assert_graph_eq(&graph, &graph2);
}

#[derive(Debug, Copy, Clone)]
enum Expected {
    Ok,
    Error { error: &'static str },
}

fn assert_inputs<N, E, Ty, Ix>(expected: &[Expected])
where
    N: DeserializeOwned + Debug,
    E: DeserializeOwned + Debug,
    Ty: EdgeType,
    Ix: IndexType + DeserializeOwned,
{
    insta::glob!("inputs/serde/*.json", |path| {
        let content = std::fs::read_to_string(path).expect("failed to read file");
        let index = path
            .file_stem()
            .expect("failed to get file stem")
            .to_string_lossy()
            .split('-')
            .next()
            .expect("failed to get file stem")
            .parse::<usize>()
            .expect("failed to parse file stem");

        let expected = expected[index];

        let graph = serde_json::from_str::<Graph<N, E, Ty, Ix>>(&content);

        match expected {
            Expected::Ok => {
                graph.expect("failed to deserialize");
            }
            Expected::Error {
                error: expected, ..
            } => {
                let error = graph.expect_err("expected error");
                assert_eq!(error.to_string(), *expected);
            }
        }
    });
}

#[test]
fn snapshot_deserialize_default_null() {
    assert_inputs::<(), (), Directed, DefaultIx>(&[
        Expected::Ok,
        Expected::Error {
            error: "invalid value: node index `5` does not exist in graph with length `5`",
        },
        Expected::Error {
            error: "invalid value: node index `300` does not exist in graph with length `5`",
        },
        Expected::Error {
            error: r#"invalid type: string "A", expected unit at line 3 column 7"#,
        },
    ]);
}

#[test]
fn snapshot_deserialize_default_null_undirected() {
    assert_inputs::<(), (), Undirected, DefaultIx>(&[
        Expected::Error {
            error: "invalid value: expected undirected graph, but received directed graph",
        },
        Expected::Error {
            error: "invalid value: expected undirected graph, but received directed graph",
        },
        Expected::Error {
            error: "invalid value: expected undirected graph, but received directed graph",
        },
        Expected::Error {
            error: r#"invalid type: string "A", expected unit at line 3 column 7"#,
        },
    ]);
}

#[test]
fn snapshot_deserialize_u8_null() {
    assert_inputs::<(), (), Directed, u8>(&[
        Expected::Ok,
        Expected::Error {
            error: "invalid value: node index `5` does not exist in graph with length `5`",
        },
        Expected::Error {
            error: "invalid value: integer `300`, expected u8 at line 18 column 9",
        },
        Expected::Error {
            error: r#"invalid type: string "A", expected unit at line 3 column 7"#,
        },
    ]);
}

#[test]
fn snapshot_deserialize_default_str_i32() {
    assert_inputs::<String, i32, Directed, DefaultIx>(&[
        Expected::Error {
            error: "invalid type: null, expected a string at line 3 column 8",
        },
        Expected::Error {
            error: "invalid type: null, expected a string at line 3 column 8",
        },
        Expected::Error {
            error: "invalid type: null, expected a string at line 3 column 8",
        },
        Expected::Ok,
    ]);
}

#[test]
fn snapshot_deserialize_default_str_i32_undirected() {
    assert_inputs::<String, i32, Undirected, DefaultIx>(&[
        Expected::Error {
            error: "invalid type: null, expected a string at line 3 column 8",
        },
        Expected::Error {
            error: "invalid type: null, expected a string at line 3 column 8",
        },
        Expected::Error {
            error: "invalid type: null, expected a string at line 3 column 8",
        },
        Expected::Error {
            error: "invalid value: expected undirected graph, but received directed graph",
        },
    ]);
}

#[test]
fn snapshot_deserialize_u8_str_i32() {
    assert_inputs::<String, u8, Directed, DefaultIx>(&[
        Expected::Error {
            error: "invalid type: null, expected a string at line 3 column 8",
        },
        Expected::Error {
            error: "invalid type: null, expected a string at line 3 column 8",
        },
        Expected::Error {
            error: "invalid type: null, expected a string at line 3 column 8",
        },
        Expected::Ok,
    ]);
}

#[test]
fn error_on_too_many_nodes() {
    // The last index (0xFF) is reserved, and cannot be used as a node index.
    let max = u8::MAX;
    let nodes = (0..=max).collect::<Vec<_>>();

    let value = json!({
        "nodes": nodes,
        "edge_property": "directed",
        "edges": []
    });

    let error =
        DiGraph::<u64, (), u8>::deserialize(value).expect_err("expected deserialization to fail");

    assert_eq!(
        error.to_string(),
        "invalid value: node length `256` exceeds maximum of `255`"
    );
}

#[test]
fn error_on_too_many_edges() {
    // The last index (0xFF) is reserved, and cannot be used as an edge index.
    let max = u8::MAX;
    let edges = (0..=max).map(|_| (0, 1, ())).collect::<Vec<_>>();

    let value = json!({
        "nodes": [0, 1],
        "edge_property": "directed",
        "edges": edges
    });

    let error =
        DiGraph::<u64, (), u8>::deserialize(value).expect_err("expected deserialization to fail");

    assert_eq!(
        error.to_string(),
        "invalid value: edge length `256` exceeds maximum of `255`"
    );
}
