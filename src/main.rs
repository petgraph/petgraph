// Playing around with the library
use std::fmt::Debug;
use std::collections::VecDeque;
use std::cmp;
use petgraph::graph::{NodeIndex};
use petgraph::dot::{Dot};
use petgraph::Graph;
use petgraph::visit::{VisitMap, GraphRef, Visitable, IntoNeighbors};
use petgraph::visit::{NodeCount, NodeIndexable};

fn main() {
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

    println!("{:?}", Dot::with_config(&graph, &[]));
    let max_flow = ford_fulkerson(graph.clone(), v0, v4);
    println!("First try {}", max_flow);
}

// Finds the minimum edge weight along the path.
fn min_weight<V>(graph: &Graph<V, u32>, path: Vec<NodeIndex>) -> u32 {
    let mut iter = path.into_iter();
    if let Some(first) = iter.next() {
        if let Some(second) = iter.next() {
            if let Some(edge) = graph.find_edge(first, second) {
                let mut weight = graph.edge_weight(edge).expect("Edge should be in graph.");
                let mut first = second;
                for second in iter {
                    if let Some(edge) = graph.find_edge(first, second) {
                        weight = cmp::min(weight, graph.edge_weight(edge).expect("Edge should be in graph."));
                        first = second;
                    } else {
                        return 0;
                    }
                }
                return *weight;
            }
        }
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
    println!("Run FF {:?} -> {:?}", start, end);
    let mut max_flow = 0;
    
    loop {
        let mut second = end;
        let path = BfsPath::shortest_path(&graph, start, end);
        if path.len() == 1 {
            break;
        }

        let path_flow = min_weight(&graph, path.clone());
        println!("path {:?} flow {:?}", path, path_flow);
        max_flow += path_flow;

        for node in path.into_iter().rev().skip(1) {
            let first = node;
            // Expect or something
            let edge = graph.find_edge(first, second).expect("Edge should be in graph");
            let mut weight = graph[edge];
            weight = weight - path_flow;
            if weight == 0 {
                graph.remove_edge(edge);
            }

            // Add reverse edge to make the residual graph.
            match graph.find_edge(second, first) {
                None => {
                    graph.add_edge(second, first, path_flow);
                }
                Some(edge) => {
                    graph.update_edge(second, first, path_flow + graph[edge]);
                }
            }
            second = first;
        }
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

    // Finds a path from start to end with Bfs. Edge weights are ignored.
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
            for succ in graph.neighbors(node) {
                if bfs.discovered.visit(succ) {
                    bfs.stack.push_back(succ);
                }
                predecessor[graph.to_index(succ)] = Some(node);
            }
        } 
        let mut next = end;
        while let Some(node) = predecessor[graph.to_index(next)] {
            path.push(node);
            if node == start {
                break;
            }
            next = node;
        }
        path.reverse();
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_flow_unweighted() {
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

    #[test]
    fn test_min_weight() {
        let mut graph = Graph::<i32, u32>::new();
        let v0 = graph.add_node(0);
        let v1 = graph.add_node(0);
        let v2 = graph.add_node(0);
        let v3 = graph.add_node(0);
        graph.extend_with_edges(&[
            (v0, v1, 3), (v1, v2, 3),
            (v2, v3, 4)
        ]);
        let path = vec![v0, v1, v2, v3];
        assert_eq!(3, min_weight(&graph, path));
        let path = vec![v0, v2, v1, v3];
        assert_eq!(0, min_weight(&graph, path));
    }

    #[test]
    fn test_max_flow_weighted() {
        let mut graph = Graph::<_, u32>::new();
        let v0 = graph.add_node(0);
        let v1 = graph.add_node(1);
        let v2 = graph.add_node(2);
        let v3 = graph.add_node(3);
        graph.extend_with_edges(&[
            (v1, v2, 3), (v1, v3, 1), (v2, v3, 3),
            (v2, v0, 1), (v3, v0, 3)
        ]);
        let max_flow = ford_fulkerson(graph.clone(), v1, v0);
        assert_eq!(4, max_flow);
    }
}