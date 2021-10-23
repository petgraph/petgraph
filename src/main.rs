// Playing around with the library
use std::fmt::Debug;
use petgraph::graph::{NodeIndex, EdgeIndex};
use petgraph::dot::{Dot, Config};
use petgraph::Graph;
use petgraph::visit::depth_first_search;
use petgraph::visit::{DfsEvent, Control, EdgeRef, GraphBase};
use petgraph::graph::DefaultIx;


fn main() {
    // Create an undirected graph with `i32` nodes and edges with `()` associated data.
    let mut graph = Graph::<_, u32>::new();
    let v0 = graph.add_node(0);
    let v1 = graph.add_node(1);
    let v2 = graph.add_node(2);
    let v3 = graph.add_node(3);
    let v4 = graph.add_node(4);
    // 0 ---> 1
    // |      
    // v      
    // 2 ---> 4
    // |     7
    // v   /
    // 3

    graph.extend_with_edges(&[
        (v0, v1), (v0, v2),
        (v2, v3), (v3, v4), (v2, v4),
    ]);

    println!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));

    let max_flow = ford_fulkerson(graph.clone(), v0, v4);
    println!("First try {}", max_flow);

    graph.add_edge(v1, v4, 0);

    let max_flow = ford_fulkerson(graph, v0, v4);
    println!("Second run {}", max_flow);
}

fn find_path<V, E>(graph: &Graph<V, E>, 
                   start: NodeIndex, 
                   goal: NodeIndex) 
                   -> Vec<NodeIndex>
{    
    let empty: NodeIndex<DefaultIx> = NodeIndex::end();
    let mut predecessor = vec![empty; graph.node_count()];
    depth_first_search(graph, Some(start), |event| {
        if let DfsEvent::TreeEdge(u, v) = event {
            predecessor[v.index()] = u;
            if v == goal {
                return Control::Break(v);
            }
        }
        Control::Continue
    });

    let mut next = goal;
    let mut path = vec![next];
    while next != start {
        let pred = predecessor[next.index()];
        if pred.index() == empty.index() {
            break;
        }
        path.push(pred);
        next = pred;
    }
    path
}


// Don't use edge indices. Use endpoints.
fn ford_fulkerson<V>(mut graph: Graph<V, u32>, start: NodeIndex, end: NodeIndex) -> u32
where
    V: Clone + Debug,
{
    let mut max_flow = 0;
    let mut second = end;
    let mut first;
    let mut path = find_path(&graph, start, end);
    while path.len() != 1 {
        max_flow += 1;
        for node in path.into_iter().skip(1) {
            first = node;
            if let Some(edge) = graph.find_edge(first, second) {
                graph.remove_edge(edge);
            } else {
                panic!("Error in dfs.");
            }
            second = first;
        }
        path = find_path(&graph, start, end);
        second = end;
    }

    max_flow
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_flow_unweighted() {
        let mut graph = Graph::<_, u32>::new();
        let v0 = graph.add_node(0);
        let v1 = graph.add_node(1);
        let v2 = graph.add_node(2);
        let v3 = graph.add_node(3);
        let v4 = graph.add_node(4);

        graph.extend_with_edges(&[
            (v0, v1), (v0, v2),
            (v2, v3), (v3, v4), (v2, v4),
        ]);
        // 0 ---> 1
        // |      
        // v
        // 2 ---> 4
        // |     7
        // v   /
        // 3
        assert_eq!(1, ford_fulkerson(graph.clone(), v0, v4));

        graph.add_edge(v1, v4, 0);
        assert_eq!(2, ford_fulkerson(graph.clone(), v0, v4));

        graph.add_edge(v0, v3, 0);
        assert_eq!(3, ford_fulkerson(graph.clone(), v0, v4));
    }
}