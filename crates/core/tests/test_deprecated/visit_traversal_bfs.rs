//! Tests for the BFS iterator.
//!
//! Even though this is a unit test, it is an integration tests, as I didn't want to muck around the
//! already deprecated visit code.
//!
//! This is basically a 1:1 port of the tests from the `petgraph` crate (which already used
//! integration tests)

use petgraph_core::deprecated::visit::{Bfs, Walker};
use petgraph_dino::DiDinoGraph;

/// Graph:
///
/// ```text
/// A → B → C
///       ↘
///         D
/// ```
///
/// BFS from A should yield A, B, C, D
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

    let dfs = Bfs::new(&graph, a);

    let received = dfs.iter(&graph).collect::<Vec<_>>();
    let expected = vec![a, b, c, d];
    assert_eq!(received, expected);
}

/// Graph:
///
/// ```text
/// A → B → C
/// ```
///
/// BFS from B should yield B, C
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

    let dfs = Bfs::new(&graph, b);

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
/// BFS from A should yield A, B
/// C is completely disconnected from A
#[test]
fn disconnected() {
    let mut graph = DiDinoGraph::new();

    let a = graph.insert_node("A").id();
    let b = graph.insert_node("B").id();
    let c = graph.insert_node("C").id();

    graph.insert_edge("A → B", a, b);

    let dfs = Bfs::new(&graph, a);

    let received = dfs.iter(&graph).collect::<Vec<_>>();
    let expected = vec![a, b];

    assert_eq!(received, expected);

    // if we have a disconnected node, we should be able to start a DFS from it as well and get only
    // that node
    let dfs = Bfs::new(&graph, c);

    let received = dfs.iter(&graph).collect::<Vec<_>>();
    let expected = vec![c];

    assert_eq!(received, expected);
}

/// Verify that the order of the nodes returned is consistent with the order of edges inserted.
///
/// In contrast to DFS the order is different, we start with the newest edge first instead of the
/// oldest.
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

    let dfs = Bfs::new(&graph, b);

    let received = dfs.iter(&graph).collect::<Vec<_>>();
    let expected = vec![b, c, d];

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
    graph.insert_edge("D → E", d, e);
    graph.insert_edge("D → F", d, f);

    let dfs = Bfs::new(&graph, b);

    let received = dfs.iter(&graph).collect::<Vec<_>>();
    let expected = vec![b, c, d, g, h, e, f];

    assert_eq!(received, expected);
}
