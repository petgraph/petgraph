extern crate petgraph;
#[macro_use]
extern crate quickcheck;
extern crate bincode;
extern crate itertools;
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate defmac;

use std::{collections::HashSet, fmt::Debug, iter::FromIterator};

use itertools::{assert_equal, repeat_n, Itertools};
use petgraph::{
    graph::{edge_index, node_index, IndexType},
    prelude::*,
    visit::{EdgeRef, IntoEdgeReferences, NodeIndexable},
    EdgeType,
};

fn make_graph<Ty, Ix>() -> Graph<&'static str, i32, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    let mut g = Graph::default();
    let a = g.add_node("A");
    let b = g.add_node("B");
    let c = g.add_node("C");
    let d = g.add_node("D");
    let e = g.add_node("E");
    let f = g.add_node("F");
    g.extend_with_edges(&[
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
    // Remove a node to make the structure a bit more interesting
    g.remove_node(d);
    g
}

fn make_stable_graph<Ty, Ix>() -> StableGraph<String, i32, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    let mut g = StableGraph::default();
    let indices: Vec<_> = (0..1024).map(|i| g.add_node(format!("{}", i))).collect();
    for i in 1..256 {
        g.extend_with_edges((0..1024).map(|j| (indices[j], indices[(j + i) % 1024], i as i32)));
    }
    // Remove nodes to make the structure a bit more interesting
    for i in (0..1024).step_by(10) {
        g.remove_node(indices[i]);
    }
    g
}

// json macros
defmac!(tojson ref g => serde_json::to_string(g).unwrap());
defmac!(fromjson ref data => serde_json::from_str(data).unwrap());
defmac!(rejson ref g => fromjson!(tojson!(g)));

#[test]
fn json_stable_graph_str() {
    let g1 = make_stable_graph();

    let g2: StableGraph<String, i32> = rejson!(&g1);

    // map &str -> String
    let g1 = g1.map(|_, s| s.to_string(), |_, &w| w);
    assert_stable_graph_eq(&g1, &g2);
}

#[test]
fn json_stable_graph_nils() {
    let g1 = make_stable_graph().map(|_, _| (), |_, _| ());
    let g2 = rejson!(&g1);
    assert_stable_graph_eq(&g1, &g2);
}

// bincode macros
defmac!(encode ref g => bincode::serialize(g).unwrap());
defmac!(decode ref data => bincode::deserialize(data).unwrap());
defmac!(recode ref g => decode!(encode!(g)));

#[test]
fn bincode_stablegraph_to_graph_i32_0() {
    let g1 = StableGraph::<i32, i32>::new();
    let g2: Graph<i32, i32> = recode!(&g1);
    assert_graph_eq(&g2, &Graph::<i32, i32>::default());
}

#[test]
fn bincode_graph_to_stablegraph_i32_0() {
    let g1 = Graph::<i32, i32>::new();
    let g2: StableGraph<i32, i32> = recode!(&g1);
    assert_stable_graph_eq(&g2, &StableGraph::<i32, i32>::default());
}

#[test]
fn bincode_graph_to_graph_i32_1() {
    let mut g1 = Graph::<i32, i32>::new();
    let x = 1729;
    g1.add_node(x);
    let g2: Graph<i32, i32> = recode!(&g1);

    assert_graph_eq(&g1, &g2);
}

#[test]
fn bincode_stablegraph_added2_removed2() {
    // from quickcheck failure case:
    // StableGraph { Ty: "Directed", node_count: 4, edge_count: 1, edges: (0,
    // 2), node weights: {0: -55, 2: 83, 3: -12, 5: -2}, edge weights: {0: 75},
    //   free_node: NodeIndex(1), free_edge: EdgeIndex(4294967295) }
    let mut g1 = StableGraph::<i32, i32>::new();
    let x = 1729;
    let a = g1.add_node(x);
    let b = g1.add_node(x + 1);
    g1.remove_node(a);
    g1.remove_node(b);
    let g2: StableGraph<i32, i32> = recode!(&g1);

    assert_stable_graph_eq(&g1, &g2);
}

#[test]
fn bincode_stablegraph_added3_removed2() {
    // from quickcheck failure case:
    // StableGraph { Ty: "Directed", node_count: 1, edge_count: 0, node weights:
    // {2: -87}, edge weights: {}, free_node: NodeIndex(3), free_edge:
    // EdgeIndex(3) }
    let mut g1 = StableGraph::<i32, i32>::new();
    let x = 1729;
    let a = g1.add_node(x);
    let b = g1.add_node(x + 1);
    let _c = g1.add_node(x + 2);
    g1.remove_node(a);
    g1.remove_node(b);
    let g2: StableGraph<i32, i32> = recode!(&g1);

    assert_stable_graph_eq(&g1, &g2);
}

#[test]
fn bincode_stablegraph_to_graph_i32_1() {
    let mut g1 = StableGraph::<i32, i32>::new();
    let x = 1729;
    g1.add_node(x);
    let g2: Graph<i32, i32> = recode!(&g1);

    assert_eq!(g2.node_count(), 1);
    assert_eq!(g2.edge_count(), 0);
    assert_eq!(g2[node_index(0)], x);
}

quickcheck! {
    fn json_graph_to_stablegraph_to_graph(g1: Graph<i32, i32>) -> () {
        let sg: StableGraph<i32, i32> = rejson!(&g1);
        let g2: Graph<i32, i32> = rejson!(&sg);
        assert_graph_eq(&g1, &g2);
    }

    fn json_stablegraph_to_stablegraph(g1: StableGraph<i32, i32>) -> () {
        let sg: StableGraph<i32, i32> = rejson!(&g1);
        assert_stable_graph_eq(&g1, &sg);
    }

    fn json_graph_to_bigger_graph(g1: DiGraph<i32, i32, u16>) -> () {
        let g2: DiGraph<i32, i32, usize> = rejson!(&g1);
        let g3: DiGraph<i32, i32, u16> = rejson!(&g2);
        assert_graph_eq(&g1, &g3);
    }

    fn bincode_graph_to_graph_nils(g1: Graph<(), ()>) -> () {
        let g2: Graph<(), ()> = recode!(&g1);
        assert_graph_eq(&g1, &g2);
    }

    fn bincode_graph_to_stablegraph_to_graph_nils(g1: Graph<(), ()>) -> () {
        let data = encode!(&g1);
        let sg: StableGraph<(), ()> = decode!(&data);
        let data2 = encode!(&sg);
        let g2: Graph<(), ()> = decode!(&data2);
        assert_eq!(data, data2);
        assert_graph_eq(&g1, &g2);
    }

    fn bincode_graph_to_stablegraph_to_graph_u16(g1: DiGraph<i32, i32, u16>) -> () {
        let data = encode!(&g1);
        let sg: StableDiGraph<i32, i32, u16> = decode!(&data);
        let data2 = encode!(&sg);
        let g2: DiGraph<i32, i32, u16> = decode!(&data2);
        assert_eq!(data, data2);
        assert_graph_eq(&g1, &g2);
    }

    fn bincode_stablegraph_to_stablegraph(g1: StableGraph<i32, i32>) -> () {
        let g2: StableGraph<i32, i32> = recode!(&g1);
        assert_stable_graph_eq(&g1, &g2);
    }

    fn json_graphmap_to_graphmap(g1: DiGraphMap<i32, i32>) -> () {
        let g2: DiGraphMap<i32, i32> = rejson!(&g1);
        assert!(petgraph::algo::is_isomorphic(&g1, &g2));
    }

    fn bincode_graphmap_to_graphmap(g1: DiGraphMap<i8, ()>) -> () {
        let g2: DiGraphMap<i8, ()> = recode!(&g1);
        assert!(petgraph::algo::is_isomorphic(&g1, &g2));
    }

    fn bincode_graphmap_to_graph(g1: DiGraphMap<i8, i8>) -> () {
        let g2: DiGraph<i8, i8> = recode!(&g1);
        assert!(petgraph::algo::is_isomorphic(&g1, &g2));
    }

    // graph to graphmap is not always possible because of parallel edges
}

#[test]
fn json_graphmap_integer() {
    let mut gr: GraphMap<i32, u32, Directed> = GraphMap::from_edges(&[
        (6, 0, 0),
        (0, 3, 1),
        (3, 6, 2),
        (8, 6, 3),
        (8, 2, 4),
        (2, 5, 5),
        (5, 8, 6),
        (7, 5, 7),
        (1, 7, 8),
        (7, 4, 9),
        (4, 1, 10),
    ]);
    // unconnected node
    gr.add_node(42);

    let gr_deser: GraphMap<i32, u32, Directed> = rejson!(&gr);
    assert!(petgraph::algo::is_isomorphic(&gr, &gr_deser));
    assert_eq!(gr_deser[(4, 1)], 10);
}

#[test]
fn json_graphmap_struct() {
    use serde_derive::{Deserialize, Serialize};

    #[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
    struct TestingNode {
        pub a: u32,
        pub b: i32,
    }
    let mut gr: GraphMap<TestingNode, (u8, f32), Undirected> = GraphMap::from_edges(&[
        (
            TestingNode { a: 42, b: -1 },
            TestingNode { a: 12, b: -2 },
            (1, 2.),
        ),
        (
            TestingNode { a: 12, b: -2 },
            TestingNode { a: 13, b: -3 },
            (99, 99.),
        ),
        (
            TestingNode { a: 13, b: -3 },
            TestingNode { a: 42, b: -1 },
            (99, 98.),
        ),
    ]);
    gr.add_node(TestingNode { a: 0, b: 0 });

    let gr_deser: GraphMap<TestingNode, (u8, f32), Undirected> = rejson!(&gr);
    assert!(petgraph::algo::is_isomorphic(&gr, &gr_deser));
    assert_eq!(
        gr_deser[(TestingNode { a: 42, b: -1 }, TestingNode { a: 12, b: -2 })],
        (1, 2.)
    );
}
