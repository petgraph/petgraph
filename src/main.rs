// Playing around with the library
use std::fmt::Debug;
use std::collections::VecDeque;
use petgraph::graph::{NodeIndex};
use petgraph::dot::{Dot, Config};
use petgraph::Graph;
use petgraph::visit::depth_first_search;
use petgraph::visit::{DfsEvent, Control, Bfs, VisitMap, GraphRef, Visitable, IntoNeighbors};
use petgraph::visit::{NodeCount, NodeIndexable};
use petgraph::graph::DefaultIx;


fn main() {
    let mut graph = Graph::<_, u32>::new();
    let v0 = graph.add_node(0);
    let v1 = graph.add_node(1);
    let v2 = graph.add_node(2);
    let v3 = graph.add_node(3);
    // 1 ---> 2
    // |   /  |
    // v L    v
    // 3 ---> 0

    graph.extend_with_edges(&[
        (v1, v2, 1), (v1, v3, 1), (v2, v3, 1),
        (v2, v0, 1), (v3, v0, 1)
    ]);

    println!("{:?}", Dot::with_config(&graph, &[]));

    let path = BfsPath::shortest_path(&graph, v1, v0);

    println!("Path {:?}", path);
    let max_flow = ford_fulkerson(graph.clone(), v1, v0);
    println!("First try {}", max_flow);
}

fn min_weight<V>(mut graph: Graph<V, u32>, path: Vec<NodeIndex>) -> u32 {
    let mut weight = 0;
    let mut iter = path.into_iter();
    let mut second = iter.next();
    for first in iter {
        ();
    }
    0
}

/// TODO: Allow edge weights
/// 
/// Computes the max flow in the graph.
/// WARNING: The algorithm will change a input graph. 
/// Input a copy of the graph if you still need it.
fn ford_fulkerson<V>(mut graph: Graph<V, u32>, start: NodeIndex, end: NodeIndex) -> u32
where
    V: Clone + Debug,
{
    let mut max_flow = 0;
    let mut second = end;
    let mut first;
    let mut path;

    // Runs BfsPath::shortest_path and assigned the output to path. 
    // Checks path isn't length 1.
    while (path = BfsPath::shortest_path(&graph, start, end)) == () && path.len() != 1 {
        max_flow += 1;
        for node in path.into_iter().skip(1) {
            first = node;
            if let Some(edge) = graph.find_edge(first, second) {
                graph.remove_edge(edge);
            } else {
                panic!("Error in search.");
            }
            // Add reverse edge to make the residual graph.
            graph.add_edge(second, first, 1);
            second = first;
        }
        second = end;
    }

    max_flow
}

// Same as Bfs but can return the path
#[derive(Clone)]
pub struct BfsPath<N, VM> {
    /// The queue of nodes to visit
    pub stack: VecDeque<N>,
    /// The map of discovered nodes
    pub discovered: VM, 
}

impl<N, VM> Default for BfsPath<N, VM>
where
    VM: Default,
{
    fn default() -> Self {
        BfsPath {
            stack: VecDeque::new(),
            discovered: VM::default(),
        }
    }
}

impl<N, VM> BfsPath<N, VM>
where
    N: Copy + PartialEq,
    VM: VisitMap<N>,
{
    /// Create a new **Bfs**, using the graph's visitor map, and put **start**
    /// in the stack of nodes to visit.
    pub fn new<G>(graph: G, start: N) -> Self
    where
        G: GraphRef + Visitable<NodeId = N, Map = VM>,
    {
        let mut discovered = graph.visit_map();
        discovered.visit(start);
        let mut stack = VecDeque::new();
        stack.push_front(start);
        BfsPath { stack, discovered }
    }

    /// Return the next node in the bfs, or **None** if the traversal is done.
    pub fn next<G>(&mut self, graph: G) -> Option<N>
    where
        G: IntoNeighbors<NodeId = N>,
    {
        if let Some(node) = self.stack.pop_front() {
            for succ in graph.neighbors(node) {
                if self.discovered.visit(succ) {
                    self.stack.push_back(succ);
                }
            }

            return Some(node);
        }
        None
    }

    // Path is in reverse order.
    pub fn shortest_path<G>(graph: G, start: N, end: N) -> Vec<N> 
    where
        G: GraphRef + Visitable<NodeId = N, Map = VM> + NodeCount,
        G: IntoNeighbors<NodeId = N> + NodeIndexable,
        N: Debug,
    {
        let mut predecessor: Vec<Option<N>> = vec![None; graph.node_count()];
        let mut path = vec![end];
        let mut bfs = BfsPath::new(graph, start);

        while let Some(node) = bfs.stack.pop_front() {
            if node == end {
                break;
            }
            let mut neighbors = graph.neighbors(node);
            for succ in graph.neighbors(node) {
                if bfs.discovered.visit(succ) {
                    bfs.stack.push_back(succ);
                }
                predecessor[graph.to_index(succ)] = Some(node);
            }
        } 
        println!("pred {:?}", predecessor);
        let mut next = end;
        while let Some(node) = predecessor[graph.to_index(next)] {
            path.push(node);
            next = node;
        }
        path
    }
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
            (v0, v1, 1), (v0, v2, 1),
            (v2, v3, 1), (v3, v4, 1), (v2, v4, 1),
        ]);
        // 0 ---> 1
        // |      
        // v
        // 2 ---> 4
        // |     7
        // v   /
        // 3
        assert_eq!(1, ford_fulkerson(graph.clone(), v0, v4));

        graph.add_edge(v1, v4, 1);
        assert_eq!(2, ford_fulkerson(graph.clone(), v0, v4));

        graph.add_edge(v0, v3, 1);
        assert_eq!(3, ford_fulkerson(graph.clone(), v0, v4));

        graph.clear();
        graph.extend_with_edges(&[
            (v0, v1, 1), (v0, v2, 1), (v1, v4, 1),
            (v2, v3, 1), (v4, v3, 1), (v2, v4, 1),
        ]);
        assert_eq!(2, ford_fulkerson(graph.clone(), v0, v4));
    }
}