#![feature(macro_rules)]
#![feature(default_type_params)]
extern crate arena;
extern crate copygraph;

use arena::TypedArena;
use std::collections::RingBuf;
use std::collections::HashSet;
use std::cell::Cell;
use std::fmt;

pub use copygraph::{
    MinScored,
    DiGraph,
    Graph,
    Ptr,
    Node,
    NodeCell,
};



fn make_graph() {
    let root = TypedArena::new();
    let mut g: DiGraph<_, f32> = DiGraph::new();
    let an = g.add_node(Ptr(root.alloc(Node("A"))));
    let bn = g.add_node(Ptr(root.alloc(Node("B"))));
    let cn = g.add_node(Ptr(root.alloc(Node("C"))));
    g.add_edge(an, bn, 1.);
    g.add_edge(an, cn, 2.);
    /*
    println!("{}", g.nodes);

    {
        for node in g.nodes() {
            println!("Node= {}", node);
        }
    }

    for next in g.edges(an) {
        println!("{} is a successor of {}", next, an);
    }

    g.remove_node(bn);
    println!("Removed B, {}", g.nodes);

    g.add_edge(cn, bn, 2.);
    println!("Added edge C to B, {}", g.nodes);
    g.add_edge(bn, an, 1.);
    println!("Added edge B to A, {}", g.nodes);
    g.add_edge(bn, cn, 3.);
    println!("Added edge B to C, {}", g.nodes);
    g.remove_edge(bn, an);
    println!("Removed edge B to A, {}", g.nodes);
    g.remove_edge(bn, an);
    println!("Removed edge B to A, {}", g.nodes);

    println!("Reversed, {}", g.reverse().nodes);

    */
    // Wikipedia example
    let root = TypedArena::<NodeCell<_>>::new();
    let mut g: DiGraph<_, f32> = DiGraph::new();
    let node = |name: &'static str| Ptr(root.alloc(NodeCell(Cell::new(name))));
    let a = g.add_node(node("A"));
    let b = g.add_node(node("B"));
    let c = g.add_node(node("C"));
    let d = g.add_node(node("D"));
    let e = g.add_node(node("E"));
    let f = g.add_node(node("F"));
    g.add_diedge(a, b, 7.);
    g.add_diedge(a, c, 9.);
    g.add_diedge(a, d, 14.);
    g.add_diedge(b, c, 10.);
    g.add_diedge(c, d, 2.);
    g.add_diedge(d, e, 9.);
    g.add_diedge(b, f, 15.);
    g.add_diedge(c, f, 11.);
    g.add_diedge(e, f, 6.);
    println!("{}", g);

    f.set("F'");

    println!("Scores= {}", 
        copygraph::dijkstra(&g, a, |gr, n| gr.edges(n).map(|&x|x))
    );

    let mut rb = RingBuf::new();
    rb.push_back(a);
    let mut it = copygraph::graph::BFT{
        graph: &g,
        stack: rb,
        visited: HashSet::new(),
        neighbors: |g, n| g.neighbors(n).map(|&x| x),
    };
    for node in it {
        println!("Visit {}", node);
    }

    let mut g: DiGraph<_, f32> = DiGraph::new();
    let node = |name: &'static str| name;
    let a = g.add_node(node("A"));
    let b = g.add_node(node("B"));
    let c = g.add_node(node("C"));
    let d = g.add_node(node("D"));
    let e = g.add_node(node("E"));
    let f = g.add_node(node("F"));
    g.add_diedge(a, b, 7.);
    g.add_diedge(a, c, 9.);
    g.add_diedge(a, d, 14.);
    g.add_diedge(b, c, 10.);
    g.add_diedge(c, d, 2.);
    g.add_diedge(d, e, 9.);
    g.add_diedge(b, f, 15.);
    g.add_diedge(c, f, 11.);
    g.add_diedge(e, f, 6.);

    println!("{}", g);

    let root = TypedArena::<Node<_>>::new();
    let mut g: Graph<_, f32> = Graph::new();
    let node = |name: &'static str| Ptr(root.alloc(Node(name.to_string())));
    let a = g.add_node(node("A"));
    let b = g.add_node(node("B"));
    let c = g.add_node(node("C"));
    let d = g.add_node(node("D"));
    let e = g.add_node(node("E"));
    let f = g.add_node(node("F"));
    g.add_edge(a, b, 7.);
    g.add_edge(a, c, 9.);
    g.add_edge(a, d, 14.);
    g.add_edge(b, c, 10.);
    g.add_edge(c, d, 2.);
    g.add_edge(d, e, 9.);
    g.add_edge(b, f, 15.);
    g.add_edge(c, f, 11.);
    g.add_edge(e, f, 6.);
    println!("{}", g);
    println!("{}", copygraph::dijkstra(&g, a, |gr, n| gr.edges(n).map(|(n, &e)| (n, e))));
    for node in g.traverse_depth_first(a) {
        println!("Visit {}", node);
    }
    println!("");
    for node in g.traverse_breadth_first(a) {
        println!("Visit {}", node);
    }

    let mut g: Graph<int, int> = Graph::new();
    g.add_node(1);
    g.add_node(2);
    g.add_edge(1, 2, -1);

    println!("{}", g);
    *g.edge_mut(1, 2).unwrap() = 3;
    for elt in g.edges(1) {
        println!("Edge {} => {}", 1i, elt);
    }
    for elt in g.edges(2) {
        println!("Edge {} => {}", 2i, elt);
    }
    //g.remove_node(2);
    g.remove_edge(2, 1);
    println!("{}", g);
}


fn main() {
    make_graph();
}
