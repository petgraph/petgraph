/// Playing around with the library
/// 
use petgraph::graph::{NodeIndex};
use petgraph::dot::{Dot, Config};
use petgraph::Graph;
use petgraph::visit::{DfsPostOrder, Dfs};



fn main() {
    // Create an undirected graph with `i32` nodes and edges with `()` associated data.
    let mut deps = Graph::<_, i32>::new();
    let pg = deps.add_node("pg");
    let fb = deps.add_node("fb");
    let qc = deps.add_node("qc");
    let rand = deps.add_node("rand");
    let libc = deps.add_node("libc");
    deps.extend_with_edges(&[
        (pg, fb), (pg, qc),
        (qc, rand), (rand, libc), (qc, libc),
    ]);

    // Output the tree to `graphviz` `DOT` format
    println!("Before {:?}", Dot::with_config(&deps, &[Config::EdgeNoLabel]));

    ford_fulkerson(deps, pg, libc);
}

// Implemement my own DFS.
fn ford_fulkerson<V>(mut graph: Graph<V, i32>, start: NodeIndex, end: NodeIndex) -> i32
{
    let mut dfs = DfsPostOrder::new(&graph, start);
    println!("Running DFS");
    while let Some(nx) = dfs.next(&graph) {
        println!("Intermediate stack {:?}", dfs.stack);
        println!("{:?}", nx);
        if nx == end {
            break;
        }
    }
    println!("DFS finished.");
    let stack = dfs.stack;
    println!("{:?}", stack);
    0
}