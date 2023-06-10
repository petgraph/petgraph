#![cfg(feature = "serde")]

use insta::assert_json_snapshot;
use petgraph_core::{edge::EdgeType, index::IndexType};
use petgraph_graph::{DiGraph, Graph};

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
fn node_str_edges_i32() {
    let graph: DiGraph<_, _> = example();

    assert_json_snapshot!(&graph);
}
