//! Tests for the DFS iterator.
//!
//! Even though this is a unit test, it is an integration tests, as I didn't want to muck around the
//! already deprecated visit code.
//!
//! This is basically a 1:1 port of the tests from the `petgraph` crate (which already used
//! integration tests)
use petgraph_core::deprecated::{
    edge::Directed,
    visit::{Dfs, Walker},
};
use petgraph_dino::DiDinoGraph;

/// Graph:
///
/// ```text
/// A → B → C
///       ↘
///         D
/// ```
///
/// DFS from A should yield A, B, C, D
#[test]
fn simple() {
    let mut graph = DiDinoGraph::new();

    let a = graph.insert_node("A").id();
    let b = graph.insert_node("B").id();
    let c = graph.insert_node("C").id();
    let d = graph.insert_node("D").id();

    graph.insert_edge("A → B", a, b);
    graph.insert_edge("B → C", b, c);
    graph.insert_edge("B → D", b, d);

    let dfs = Dfs::new(&graph, a);

    let received = dfs.iter(&graph).collect::<Vec<_>>();
    let expected = vec![a, b, d, c];
    assert_eq!(received, expected);
}

/// Graph:
///
/// ```text
/// A → B → C
/// ```
///
/// DFS from B should yield B, C
///
/// A is connected via a directed edge, but it is not reachable from B.
#[test]
fn unreachable() {
    let mut graph = DiDinoGraph::new();

    let a = graph.insert_node("A").id();
    let b = graph.insert_node("B").id();
    let c = graph.insert_node("C").id();

    graph.insert_edge("A → B", a, b);
    graph.insert_edge("B → C", b, c);

    let dfs = Dfs::new(&graph, b);

    let received = dfs.iter(&graph).collect::<Vec<_>>();
    let expected = vec![b, c];

    assert_eq!(received, expected);
}

/// Graph:
///
/// ```text
/// A → B
///
/// C
/// ```
///
/// DFS from A should yield A, B
/// C is completely disconnected from A
#[test]
fn disconnected() {
    let mut graph = DiDinoGraph::new();

    let a = graph.insert_node("A").id();
    let b = graph.insert_node("B").id();
    let c = graph.insert_node("C").id();

    graph.insert_edge("A → B", a, b);

    let dfs = Dfs::new(&graph, a);

    let received = dfs.iter(&graph).collect::<Vec<_>>();
    let expected = vec![a, b];

    assert_eq!(received, expected);

    // if we have a disconnected node, we should be able to start a DFS from it as well and get only
    // that node
    let dfs = Dfs::new(&graph, c);

    let received = dfs.iter(&graph).collect::<Vec<_>>();
    let expected = vec![c];

    assert_eq!(received, expected);
}

/// Verify that the order of the nodes returned is consistent with the order of edges inserted.
///
/// Meaning that if `B → C` is inserted before `B → D`, then `C` should be visited before `D`.
///
/// Graph:
///
/// ```text
/// B → C
///   ↘
///     D
/// ```
#[test]
fn order() {
    let mut graph = DiDinoGraph::new();

    let b = graph.insert_node("B").id();
    let c = graph.insert_node("C").id();
    let d = graph.insert_node("D").id();

    graph.insert_edge("B → C", b, c);
    graph.insert_edge("B → D", b, d);

    let dfs = Dfs::new(&graph, b);

    let received = dfs.iter(&graph).collect::<Vec<_>>();
    let expected = vec![b, d, c];

    assert_eq!(received, expected);
}

/// Verify that the order of the nodes returned is consistent with the order of edges inserted.
///
/// This should still be true if we have a deeper graph.
///
/// ```text
/// B → C → G
///   ↘   ↘
///     D   H
///     ↓ ↘
///     F   E
/// ```
#[test]
fn order_deep() {
    let mut graph = DiDinoGraph::new();

    let b = graph.insert_node("B").id();
    let c = graph.insert_node("C").id();
    let d = graph.insert_node("D").id();
    let e = graph.insert_node("E").id();
    let f = graph.insert_node("F").id();
    let g = graph.insert_node("G").id();
    let h = graph.insert_node("H").id();

    graph.insert_edge("B → C", b, c);
    graph.insert_edge("B → D", b, d);
    graph.insert_edge("C → G", c, g);
    graph.insert_edge("C → H", c, h);
    graph.insert_edge("H → E", d, e);
    graph.insert_edge("D → F", d, f);

    let dfs = Dfs::new(&graph, b);

    let received = dfs.iter(&graph).collect::<Vec<_>>();
    let expected = vec![b, d, f, e, c, h, g];

    assert_eq!(received, expected);
}

/// Test against `GraphMap`, which (unlike any other built-in graph) does not use `NodeIndex` as
/// node identifiers.
#[test]
#[ignore = "GraphMap hasn't been ported yet"]
fn graphmap() {
    // use petgraph::graphmap::GraphMap;
    // let mut graph = GraphMap::<_, _, Directed>::new();
    //
    // let a = graph.add_node("A");
    // let b = graph.add_node("B");
    // let c = graph.add_node("C");
    // let d = graph.add_node("D");
    //
    // // disconnected node shouldn't be included in the DFS
    // graph.add_node("E");
    //
    // graph.add_edge(a, b, "A → B");
    // graph.add_edge(a, c, "A → C");
    // graph.add_edge(b, c, "B → C");
    // graph.add_edge(b, d, "B → D");
    //
    // let mut dfs = Dfs::new(&graph, a);
    // let mut order = vec![];
    //
    // while let Some(node) = dfs.next(&graph) {
    //     order.push(node);
    // }
    //
    // assert_eq!(order, vec![a, c, b, d]);
}

#[test]
#[ignore = "GraphMap hasn't been ported yet"]
fn graphmap_disconnected() {
    // use petgraph::graphmap::GraphMap;
    // let mut graph = GraphMap::<_, _, Directed>::new();
    //
    // let a = graph.add_node("A");
    // let b = graph.add_node("B");
    //
    // let c = graph.add_node("C");
    //
    // graph.add_edge(a, b, "A → B");
    //
    // let mut dfs = Dfs::new(&graph, c);
    // let mut order = vec![];
    //
    // while let Some(node) = dfs.next(&graph) {
    //     order.push(node);
    // }
    //
    // assert_eq!(order, vec![c]);
}
