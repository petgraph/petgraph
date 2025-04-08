use petgraph::{
    algo::connectivity::TwoEdgeConnectedComponentsSearch,
    dot::Dot,
    visit::{GraphProp, IntoEdgeReferences, IntoNeighbors, IntoNodeReferences, NodeIndexable},
    Graph, Undirected,
};

use core::{
    fmt::{Debug, Display},
    hash::Hash,
};
use hashbrown::HashSet;

#[test]
fn two_edge_connected_components_test_empty() {
    let gr: Graph<(), (), Undirected> = Graph::new_undirected();

    let mut iter = TwoEdgeConnectedComponentsSearch::new(&gr);

    assert_eq!(iter.next(&gr), None);
    assert_eq!(iter.next(&gr), None);
}

#[test]
fn two_edge_connected_components_test_k1() {
    let mut gr: Graph<&str, (), Undirected> = Graph::new_undirected();
    let a = gr.add_node("A");

    let mut iter = TwoEdgeConnectedComponentsSearch::new(&gr);

    assert_eq!(iter.next(&gr), Some(HashSet::from([a])));
    assert_eq!(iter.next(&gr), None);
}

#[test]
fn two_edge_connected_components_test_k2() {
    let mut gr = Graph::new_undirected();
    let a = gr.add_node("A");
    let b = gr.add_node("B");

    gr.add_edge(a, b, 1.);

    let expected_two_edge_connected_components = vec![HashSet::from([a]), HashSet::from([b])];

    test_two_edge_connected_components(&gr, expected_two_edge_connected_components);
}

#[test]
fn two_edge_connected_components_test_k3() {
    let mut gr = Graph::new_undirected();
    let a = gr.add_node("A");
    let b = gr.add_node("B");
    let c = gr.add_node("C");

    gr.add_edge(a, b, 1.);
    gr.add_edge(a, c, 2.);
    gr.add_edge(b, c, 3.);

    let mut iter = TwoEdgeConnectedComponentsSearch::new(&gr);
    assert_eq!(iter.next(&gr), Some(HashSet::from([a, b, c])));
    assert_eq!(iter.next(&gr), None);
}

#[test]
// A - B - C - D
//         | /
//         E
fn two_edge_connected_components_test_a() {
    let mut gr = Graph::new_undirected();
    let a = gr.add_node("A");
    let b = gr.add_node("B");
    let c = gr.add_node("C");
    let d = gr.add_node("D");
    let e = gr.add_node("E");

    gr.add_edge(a, b, 1.);
    gr.add_edge(b, c, 2.);
    gr.add_edge(c, d, 3.);
    gr.add_edge(c, e, 4.);
    gr.add_edge(d, e, 5.);

    let expected_two_edge_connected_components = vec![
        HashSet::from([a]),
        HashSet::from([b]),
        HashSet::from([c, d, e]),
    ];

    test_two_edge_connected_components(&gr, expected_two_edge_connected_components);
}

#[test]
// A - B - C - D
//         | /
//     F - E
fn two_edge_connected_components_test_b() {
    let mut gr = Graph::new_undirected();
    let a = gr.add_node("A");
    let b = gr.add_node("B");
    let c = gr.add_node("C");
    let d = gr.add_node("D");
    let e = gr.add_node("E");
    let f = gr.add_node("F");

    gr.add_edge(a, b, 1.);
    gr.add_edge(b, c, 2.);
    gr.add_edge(c, d, 3.);
    gr.add_edge(c, e, 4.);
    gr.add_edge(d, e, 5.);
    gr.add_edge(e, f, 6.);

    let expected_two_edge_connected_components = vec![
        HashSet::from([a]),
        HashSet::from([b]),
        HashSet::from([c, d, e]),
        HashSet::from([f]),
    ];

    test_two_edge_connected_components(&gr, expected_two_edge_connected_components);
}

#[test]
// A - B - C
// | /   \ |
// D      E
fn two_edge_connected_components_test_c() {
    let mut gr = Graph::new_undirected();
    let a = gr.add_node("A");
    let b = gr.add_node("B");
    let c = gr.add_node("C");
    let d = gr.add_node("D");
    let e = gr.add_node("E");

    gr.add_edge(a, b, 1.);
    gr.add_edge(a, d, 2.);
    gr.add_edge(b, c, 3.);
    gr.add_edge(b, d, 4.);
    gr.add_edge(b, e, 4.);
    gr.add_edge(c, e, 5.);

    let mut iter = TwoEdgeConnectedComponentsSearch::new(&gr);

    assert_eq!(iter.next(&gr), Some(HashSet::from([a, b, c, d, e])));
    assert_eq!(iter.next(&gr), None);
}

#[test]
// A - B - D - E - F
//     | /   \
//     C       G - H
//             | /
//             I
fn two_edge_connected_components_test_d() {
    let mut gr = Graph::new_undirected();
    let a = gr.add_node("A");
    let b = gr.add_node("B");
    let c = gr.add_node("C");
    let d = gr.add_node("D");
    let e = gr.add_node("E");
    let f = gr.add_node("F");
    let g = gr.add_node("G");
    let h = gr.add_node("H");
    let i = gr.add_node("I");

    gr.add_edge(a, b, 1.);
    gr.add_edge(b, c, 2.);
    gr.add_edge(b, d, 3.);
    gr.add_edge(c, d, 4.);
    gr.add_edge(d, e, 5.);
    gr.add_edge(d, g, 6.);
    gr.add_edge(e, f, 7.);
    gr.add_edge(g, h, 8.);
    gr.add_edge(g, i, 9.);
    gr.add_edge(h, i, 10.);

    let expected_two_edge_connected_components = vec![
        HashSet::from([a]),
        HashSet::from([b, c, d]),
        HashSet::from([e]),
        HashSet::from([f]),
        HashSet::from([g, h, i]),
    ];

    test_two_edge_connected_components(&gr, expected_two_edge_connected_components);
}

#[test]
// A - B ---- C
// |   | \  / |
// |   |  D - E
// |   |
// G - F - H
//      \ /
//       I - J
fn two_edge_connected_components_test_e() {
    let mut gr = Graph::new_undirected();
    let a = gr.add_node("A");
    let b = gr.add_node("B");
    let c = gr.add_node("C");
    let d = gr.add_node("D");
    let e = gr.add_node("E");
    let f = gr.add_node("F");
    let g = gr.add_node("G");
    let h = gr.add_node("H");
    let i = gr.add_node("I");
    let j = gr.add_node("J");

    gr.add_edge(a, b, 1.);
    gr.add_edge(a, g, 2.);
    gr.add_edge(b, c, 3.);
    gr.add_edge(b, d, 4.);
    gr.add_edge(b, f, 5.);
    gr.add_edge(c, d, 6.);
    gr.add_edge(c, e, 7.);
    gr.add_edge(d, e, 8.);
    gr.add_edge(f, g, 9.);
    gr.add_edge(f, h, 10.);
    gr.add_edge(f, i, 11.);
    gr.add_edge(h, i, 12.);
    gr.add_edge(i, j, 13.);

    let expected_two_edge_connected_components = vec![
        HashSet::from([a, b, c, d, e, f, g, h, i]),
        HashSet::from([j]),
    ];

    test_two_edge_connected_components(&gr, expected_two_edge_connected_components);
}

fn test_two_edge_connected_components<G, N>(
    gr: G,
    expected_two_edge_connected_components: Vec<HashSet<N>>,
) where
    N: Debug + Hash + Eq + Copy,
    G: IntoNodeReferences
        + IntoEdgeReferences
        + IntoNeighbors<NodeId = N>
        + NodeIndexable
        + GraphProp,
    G::NodeWeight: Display,
    G::EdgeWeight: Display,
{
    println!("{}", Dot::new(&gr));

    let mut iter = TwoEdgeConnectedComponentsSearch::new(&gr);
    let mut two_edge_connected_components = Vec::new();
    while let Some(two_edge_connected_component) = iter.next(&gr) {
        two_edge_connected_components.push(two_edge_connected_component);
    }
    assert_eq!(iter.next(&gr), None);

    println!("actual: {:?}", two_edge_connected_components);
    println!("expected: {:?}", expected_two_edge_connected_components);
    let expected_len = expected_two_edge_connected_components.len();
    for expected_two_edge_connected_component in expected_two_edge_connected_components {
        assert!(two_edge_connected_components.contains(&expected_two_edge_connected_component));
    }

    assert_eq!(two_edge_connected_components.len(), expected_len);
}
