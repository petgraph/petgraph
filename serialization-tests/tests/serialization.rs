
extern crate petgraph;
#[macro_use] extern crate quickcheck;
extern crate itertools;
extern crate serde_json;
extern crate bincode;
#[macro_use] extern crate defmac;

use std::collections::HashSet;

use itertools::{Itertools, repeat_n};
use itertools::assert_equal;

use std::fmt::Debug;
use std::iter::FromIterator;

use serde_json::{to_string, from_str};

use petgraph::prelude::*;
use petgraph::EdgeType;
use petgraph::stable_graph::StableDiGraph;
use petgraph::graph::{node_index, edge_index, IndexType};
use petgraph::visit::EdgeRef;
use petgraph::visit::NodeIndexable;
use petgraph::visit::IntoEdgeReferences;

// graphs are the equal, down to graph indices
// this is a strict notion of graph equivalence:
//
// * Requires equal node and edge indices, equal weights
// * Does not require: edge for node order
pub fn assert_graph_eq<N, N2, E, Ty, Ix>(g: &Graph<N, E, Ty, Ix>, h: &Graph<N2, E, Ty, Ix>)
    where N: PartialEq<N2> + Debug, N2: PartialEq<N2> + Debug, E: PartialEq + Debug,
          Ty: EdgeType,
          Ix: IndexType,
{
    assert_eq!(g.node_count(), h.node_count());
    assert_eq!(g.edge_count(), h.edge_count());

    // same node weigths
    assert_equal(g.raw_nodes().iter().map(|n| &n.weight),
                 h.raw_nodes().iter().map(|n| &n.weight));

    // same edge weigths
    assert_equal(g.raw_edges().iter().map(|n| &n.weight),
                 h.raw_edges().iter().map(|n| &n.weight));

    for e1 in g.edge_references() {
        let (a2, b2) = h.edge_endpoints(e1.id()).unwrap();
        assert_eq!(e1.source(), a2);
        assert_eq!(e1.target(), b2);
    }

    for index in g.node_indices() {
        let outgoing1 = <HashSet<_>>::from_iter(g.neighbors(index));
        let outgoing2 = <HashSet<_>>::from_iter(h.neighbors(index));
        assert_eq!(outgoing1, outgoing2);
        let incoming1 = <HashSet<_>>::from_iter(g.neighbors_directed(index, Incoming));
        let incoming2 = <HashSet<_>>::from_iter(h.neighbors_directed(index, Incoming));
        assert_eq!(incoming1, incoming2);
    }
}

// graphs are the equal, down to graph indices
// this is a strict notion of graph equivalence:
//
// * Requires equal node and edge indices, equal weights
// * Does not require: edge for node order
pub fn assert_stable_graph_eq<N, E>(g: &StableGraph<N, E>, h: &StableGraph<N, E>)
    where N: PartialEq + Debug, E: PartialEq + Debug,
{
    assert_eq!(g.node_count(), h.node_count());
    assert_eq!(g.edge_count(), h.edge_count());

    // same node weigths
    assert_equal(
        (0..g.node_bound()).map(|i| g.node_weight(node_index(i))),
        (0..h.node_bound()).map(|i| h.node_weight(node_index(i))));

    let last_edge_g = g.edge_references().next_back();
    let last_edge_h = h.edge_references().next_back();

    assert_eq!(last_edge_g.is_some(), last_edge_h.is_some());
    if let (Some(lg), Some(lh)) = (last_edge_g, last_edge_h) {
        let lgi = lg.id().index();
        let lhi = lh.id().index();
        // same edge weigths
        assert_equal(
            (0..lgi).map(|i| g.edge_weight(edge_index(i))),
            (0..lhi).map(|i| h.edge_weight(edge_index(i))));
    }

    for e1 in g.edge_references() {
        let (a2, b2) = h.edge_endpoints(e1.id()).unwrap();
        assert_eq!(e1.source(), a2);
        assert_eq!(e1.target(), b2);
    }

    for index in g.node_indices() {
        let outgoing1 = <HashSet<_>>::from_iter(g.neighbors(index));
        let outgoing2 = <HashSet<_>>::from_iter(h.neighbors(index));
        assert_eq!(outgoing1, outgoing2);
        let incoming1 = <HashSet<_>>::from_iter(g.neighbors_directed(index, Incoming));
        let incoming2 = <HashSet<_>>::from_iter(h.neighbors_directed(index, Incoming));
        assert_eq!(incoming1, incoming2);
    }
}


#[test]
fn test_ser_graph_str() {
    let mut g = Graph::new();
    let a = g.add_node("A");
    let b = g.add_node("B");
    let c = g.add_node("C");
    let d = g.add_node("D");
    let e = g.add_node("E");
    let f = g.add_node("F");
    g.add_edge(a, b, 7);
    g.add_edge(c, a, 9);
    g.add_edge(a, d, 14);
    g.add_edge(b, c, 10);
    g.add_edge(d, c, 2);
    g.add_edge(d, e, 9);
    g.add_edge(b, f, 15);
    g.add_edge(c, f, 11);
    g.add_edge(e, f, 6);
    // Remove a node to make the structure a bit more interesting
    g.remove_node(d);
    println!("{:#?}", g);

    println!("{:?}",
             serde_json::to_string(&g));
    println!("{}",
             serde_json::to_string(&g).unwrap());

    let json = serde_json::to_string(&g).unwrap();

    let g2: Result<Graph<String, i32>, _> = serde_json::from_str(&json);
    println!("{:?}", g2);
    let g2 = g2.unwrap();
    assert_graph_eq(&g, &g2);
    assert_graph_eq(&g2, &g);
}

#[test]
fn test_ser_graph_nils() {
    let mut g: Graph<(), ()> = Graph::new();
    let a = g.add_node(());
    let b = g.add_node(());
    let c = g.add_node(());
    let d = g.add_node(());
    let e = g.add_node(());
    let f = g.add_node(());
    g.extend_with_edges(&[
    (a, b),
    (c, a),
    (a, d),
    (b, c),
    (d, c),
    (d, e),
    (b, f),
    (c, f),
    (e, f)]);

    // Remove a node to make the structure a bit more interesting
    g.remove_node(d);
    println!("{:?}", g);

    println!("{:?}",
             serde_json::to_string(&g));
    println!("{}",
             serde_json::to_string(&g).unwrap());

    let json = serde_json::to_string(&g).unwrap();

    let g2: Result<Graph<(), ()>, _> = serde_json::from_str(&json);
    println!("{:?}", g2);

    let g2 = g2.unwrap();
    assert_graph_eq(&g, &g2);
    assert_graph_eq(&g2, &g);
}

const DIGRAPH_NILS: &str = r#"{
    "nodes":[null,null,null,null,null],
    "edge_property": "directed",
    "edges":[[0,1,null],[2,0,null],[1,3,null],[1,2,null],[2,3,null],[4,3,null]]
    }"#;

const DIGRAPH_NILS_INDEX_OOB: &str = r#"{
    "nodes":[null,null,null,null,null],
    "edge_property": "directed",
    "edges":[[0,1,null],[2,5,null],[1,3,null],[1,2,null],[2,3,null],[4,3,null]]
    }"#;

const DIGRAPH_NILS_INDEX_OUTSIDE_U8: &str = r#"{
    "nodes":[null,null,null,null,null],
    "edge_property": "directed",
    "edges":[[0,1,null],[2,300,null],[1,3,null],[1,2,null],[2,3,null],[4,3,null]]
    }"#;

const DIGRAPH_STRI32: &str = r#"{
    "nodes":["A","B","C","D","E","F"],
    "edge_property": "directed",
    "edges":[[0,1,7],[2,0,9],[0,3,14],[1,2,10],[3,2,2],[3,4,9],[1,5,15],[2,5,11],[4,5,6]]
    }"#;


type DiGraphNils = DiGraph<(), ()>;
type UnGraphNils = UnGraph<(), ()>;
type DiGraphNilsU8 = DiGraph<(), (), u8>;
type DiGraphStrI32 = DiGraph<String, i32>;

#[test]
fn test_from_string_directed_ok() {
    serde_json::from_str::<DiGraphNils>(DIGRAPH_NILS).unwrap();
}

#[test]
#[should_panic(expected="edge property mismatch")]
fn test_from_string_directed_mismatch() {
    serde_json::from_str::<UnGraphNils>(DIGRAPH_NILS).unwrap();
}

#[test]
#[should_panic(expected="does not exist")]
fn test_from_string_index_oob() {
    serde_json::from_str::<DiGraphNils>(DIGRAPH_NILS_INDEX_OOB).unwrap();
}

#[test]
#[should_panic(expected="expected u8")]
fn test_from_string_index_too_large() {
    serde_json::from_str::<DiGraphNilsU8>(DIGRAPH_NILS_INDEX_OUTSIDE_U8).unwrap();
}

#[test]
fn test_from_string_directed_str_i32() {
    serde_json::from_str::<DiGraphStrI32>(DIGRAPH_STRI32).unwrap();
}

#[test]
#[should_panic(expected="expected unit")]
fn test_from_string_directed_wrong_weight_type() {
    serde_json::from_str::<DiGraphNils>(DIGRAPH_STRI32).unwrap();
}

#[test]
fn test_from_string_digraph_str_i32() {
    let g4nodes = ["A","B","C","D","E","F"];
    let g4edges = [[0,1,7],[2,0,9],[0,3,14],[1,2,10],[3,2,2],[3,4,9],[1,5,15],[2,5,11],[4,5,6]];

    type GSI = DiGraph<String, i32>;
    type GSISmall = DiGraph<String, i32, u8>;

    let g4 = serde_json::from_str::<GSI>(DIGRAPH_STRI32);
    println!("{:?}", g4);
    let g4 = g4.unwrap(); // assert

    for ni in g4.node_indices() {
        assert_eq!(&g4nodes[ni.index()], &g4[ni]);
    }
    for e in g4.edge_references() {
        let edge_data = g4edges[e.id().index()];

        let (s, t) = g4.edge_endpoints(e.id()).unwrap();
        assert_eq!(edge_data[0] as usize, s.index());
        assert_eq!(edge_data[1] as usize, t.index());

        assert_eq!(edge_data[2], g4[e.id()]);
    }

    let g4small = serde_json::from_str::<GSISmall>(DIGRAPH_STRI32);
    println!("{:?}", g4small);
    g4small.unwrap(); // assert
}

#[test]
fn test_nodes_too_big() {
    // ensure we fail if node or edge count exceeds index max
    use serde_json::from_str;

    let j1_big = &format!("{}{}{}",
                          r#"
                          {"nodes": [
                          "#,
                          repeat_n(0, 300).format(", "),
                          r#"
                          ],
                          "edge_property": "directed",
                          "edges": []
                          }
                          "#);

    type G8 = DiGraph<i32, (), u8>;
    type G16 = DiGraph<i32, (), u16>;
    type G32 = DiGraph<i32, (), u32>;
    type G64 = DiGraph<i32, (), usize>;

    type H1 = DiGraph<i32, i32>;

    assert!(from_str::<G8>(j1_big).is_err());
    println!("{:?}", from_str::<G8>(j1_big));
    from_str::<G16>(j1_big).unwrap(); // assert
    from_str::<G32>(j1_big).unwrap(); // assert
    from_str::<G64>(j1_big).unwrap(); // assert

    // other edge weight is also ok -- because it has no edges
    from_str::<H1>(j1_big).unwrap(); // assert
}

#[test]
fn test_edges_too_big() {
    // ensure we fail if node or edge count exceeds index max
    use serde_json::from_str;

    let j1_big = format!("{}{}{}",
                         r#"
                         {"nodes": [0],
                         "edge_property": "directed",
                         "edges": ["#,
                         repeat_n("[0, 0, 1]", (1 << 16) - 1).format(", "),
                         "]}");

    type G8 = DiGraph<i32, i32, u8>;
    type G16 = DiGraph<i32, i32, u16>;
    type G32 = DiGraph<i32, i32, u32>;
    type G64 = DiGraph<i32, i32, usize>;

    assert!(from_str::<G8>(&j1_big).is_err());
    assert!(from_str::<G16>(&j1_big).is_err());
    assert!(from_str::<G32>(&j1_big).is_ok());
    assert!(from_str::<G64>(&j1_big).is_ok());
}

use petgraph::stable_graph::StableGraph;


#[test]
fn test_stable_graph_str() {
    let mut g = StableGraph::new();
    let a = g.add_node("A");
    let b = g.add_node("B");
    let c = g.add_node("C");
    let d = g.add_node("D");
    let e = g.add_node("E");
    let f = g.add_node("F");
    g.add_edge(a, b, 7);
    g.add_edge(c, a, 9);
    g.add_edge(a, d, 14);
    g.add_edge(b, c, 10);
    g.add_edge(d, c, 2);
    g.add_edge(d, e, 9);
    g.add_edge(b, f, 15);
    g.add_edge(c, f, 11);
    g.add_edge(e, f, 6);

    // Remove a node to make the structure a bit more interesting
    g.remove_node(d);

    println!("{:#?}", g);

    println!("{:?}",
             serde_json::to_string(&g));
    println!("{}",
             serde_json::to_string(&g).unwrap());

    let json = serde_json::to_string(&g).unwrap();

    let g2: Result<StableGraph<String, i32>, _> = serde_json::from_str(&json);
    let g2 = g2.unwrap();
    println!("{:?}", g2);
    // map &str -> String
    let g1 = g.map(|_, s| s.to_string(), |_, &w| w);
    assert_stable_graph_eq(&g1, &g2);
}

#[test]
fn test_stable_graph_nils() {
    let mut g: StableGraph<(), ()> = StableGraph::new();

    let a = g.add_node(());
    let b = g.add_node(());
    let c = g.add_node(());
    let d = g.add_node(());
    let e = g.add_node(());
    let f = g.add_node(());
    g.extend_with_edges(&[
    (a, b),
    (c, a),
    (a, d),
    (b, c),
    (d, c),
    (d, e),
    (b, f),
    (c, f),
    (e, f)]);

    // Remove a node to make the structure a bit more interesting
    g.remove_node(d);

    println!("{:?}", g);

    println!("{:?}",
             serde_json::to_string(&g));
    println!("{}",
             serde_json::to_string(&g).unwrap());

    let json = serde_json::to_string(&g).unwrap();
    let g2: Result<StableGraph<(), ()>, _> = serde_json::from_str(&json);
    let g2 = g2.unwrap();
    println!("{:?}", g2);
    assert_stable_graph_eq(&g, &g2);
}


// bincode macros
defmac!(encode ref g => bincode::serialize(g, bincode::Infinite).unwrap());
defmac!(decode ref data => bincode::deserialize(data).unwrap());
defmac!(recode ref g => decode!(encode!(g)));


#[test]
fn bincode_stablegraph_to_graph_i32_0() {
    let g1 = StableGraph::<i32, i32>::new();
    let _g2: Graph<i32, i32> = recode!(g1);
}

#[test]
fn bincode_graph_to_graph_i32_1() {
    let mut g1 = Graph::<i32, i32>::new();
    let x = 1729;
    g1.add_node(x);
    let data = encode!(g1);
    println!("");
    println!("{:02x}", data.iter().format(" "));

    let g2: Graph<i32, i32> = bincode::deserialize(&data).unwrap();

    assert_graph_eq(&g1, &g2);
}

#[test]
fn bincode_stablegraph_to_graph_i32_1() {
    let mut g1 = StableGraph::<i32, i32>::new();
    let x = 1729;
    g1.add_node(x);
    let g2: Graph<i32, i32> = recode!(g1);

    assert_eq!(g2.node_count(), 1);
    assert_eq!(g2.edge_count(), 0);
    assert_eq!(g2[node_index(0)], x);
}

quickcheck! {
    fn json_graph_to_bigger_graph(g1: DiGraph<i32, i32, u16>) -> () {
        let json1 = to_string(&g1).unwrap();
        let g2: DiGraph<i32, i32, usize> = from_str(&json1).unwrap();
        let json2 = to_string(&g2).unwrap();
        let g3: DiGraph<i32, i32, u16> = from_str(&json2).unwrap();
        assert_graph_eq(&g1, &g3);
    }
    fn json_graph_to_stablegraph_to_graph(g1: Graph<i32, i32>) -> () {
        let json1 = to_string(&g1).unwrap();
        let sg: StableGraph<i32, i32> = from_str(&json1).unwrap();
        let json2 = to_string(&sg).unwrap();
        let g2: Graph<i32, i32> = from_str(&json2).unwrap();
        assert_graph_eq(&g1, &g2);
    }

    fn json_stablegraph_to_stablegraph(g1: StableGraph<i32, i32>) -> () {
        let json1 = to_string(&g1).unwrap();
        let sg: StableGraph<i32, i32> = from_str(&json1).unwrap();
        assert_stable_graph_eq(&g1, &sg);
    }

    fn bincode_graph_to_graph_nils(g1: Graph<(), ()>) -> () {
        let g2: Graph<(), ()> = recode!(g1);
        assert_graph_eq(&g1, &g2);
    }

    fn bincode_graph_to_stablegraph_to_graph_nils(g1: Graph<(), ()>) -> () {
        let data = encode!(g1);
        let sg: StableGraph<(), ()> = decode!(data);
        let data2 = encode!(sg);
        let g2: Graph<(), ()> = decode!(data2);
        assert_eq!(data, data2);
        assert_graph_eq(&g1, &g2);
    }

    fn bincode_graph_to_stablegraph_to_graph_u16(g1: DiGraph<i32, i32, u16>) -> () {
        let data = encode!(g1);
        let sg: StableDiGraph<i32, i32, u16> = decode!(data);
        let data2 = encode!(sg);
        let g2: DiGraph<i32, i32, u16> = decode!(data2);
        assert_eq!(data, data2);
        assert_graph_eq(&g1, &g2);
    }

    fn bincode_stablegraph_to_stablegraph(g1: StableGraph<i32, i32>) -> () {
        let g2: StableGraph<i32, i32> = recode!(g1);
        assert_stable_graph_eq(&g1, &g2);
    }
}
