use std::arch::x86_64::__cpuid;
use std::io::ErrorKind::AddrNotAvailable;
use std::sync::Mutex;
use indexmap::map::Values;
use petgraph::{graph, Graph};
use petgraph::visit::IntoNodeIdentifiers;

fn  av(v: &mut Vec<i32>) {
    v.push(10); 
}


fn  main() {
    // let mut  g = Graph::new_undirected();
    // let a = g.add_node("A");
    // let b = g.add_node("B");
    // let c = g.add_node("C");
    // let d = g.add_node("D");
    // let e = g.add_node("E");
    // let f = g.add_node("F");
    // g.add_edge(a, b, 7);
    // g.add_edge(c, a, 9);
    // g.add_edge(a, d, 14);
    // g.add_edge(b, c, 10);
    // g.add_edge(d, c, 2);
    // g.add_edge(d, e, 9);
    // g.add_edge(b, f, 15);
    // g.add_edge(c, f, 11);
    // g.add_edge(e, f, 6);
    // 
    // let  ng = g.map2(|_, node| node.to_lowercase(), |_, edge | edge * 10);
    // let  nodes = ng.raw_nodes();
    // let  edges = ng.raw_edges();
    // 
    // for n in nodes {
    // 
    //     println!("{}", n.weight);
    // }

}