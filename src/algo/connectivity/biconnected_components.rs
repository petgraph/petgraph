use alloc::{boxed::Box, vec::Vec};
use core::hash::Hash;
use hashbrown::{HashMap, HashSet};

use super::Color;
use crate::{algo::IntoNeighbors, visit::IntoNodeIdentifiers};

pub struct BiconnectedComponentsSearch<'a, N> {
    /// Map of node colors during search.
    /// If it hasn't any color, it wasn't visited yet,
    /// if it has gray color, it is being processed, and
    /// if it has black color, has finished being processed.
    pub color: HashMap<N, Color>,
    /// Preorder number of each node in DFS search.
    pub pre: HashMap<N, usize>,
    /// Lowest preorder number each node is reached in DFS search.
    pub low: HashMap<N, usize>,
    /// Neighbors of each node.
    pub neighbors: HashMap<N, Box<dyn Iterator<Item = N> + 'a>>,
    /// Stack of edges to be processed. Simulates a DFS search.
    pub edges_stack: Vec<(N, N)>,
    /// Stack of nodes visited in DFS search
    nodes_stack: Vec<N>,
}

/// Each call to `next` should return a graph's maximal biconnected component
/// if it exists, otherwise returns an empty set.
impl<'a, N> BiconnectedComponentsSearch<'a, N>
where
    N: Hash + Eq + Copy,
{
    pub fn new<G>(graph: G) -> Self
    where
        G: IntoNodeIdentifiers<NodeId = N>,
    {
        let mut edges_stack = Vec::new();
        let mut nodes_stack = Vec::new();

        let root = graph.node_identifiers().next();
        if let Some(start) = root {
            // Initial dummy edge
            edges_stack.push((start, start));
            nodes_stack.push(start);
        };

        BiconnectedComponentsSearch {
            color: HashMap::new(),
            pre: HashMap::new(),
            low: HashMap::new(),
            neighbors: HashMap::new(),
            edges_stack,
            nodes_stack,
        }
    }

    pub fn next<G>(&mut self, graph: G) -> Option<HashSet<N>>
    where
        G: 'a + IntoNeighbors<NodeId = N>,
    {
        while !self.edges_stack.is_empty() {
            let (parent, a) = *self.edges_stack.last().unwrap();

            if !self.color.contains_key(&a) {
                let cnt = self.color.len();
                self.color.insert(a, Color::Gray);
                self.pre.insert(a, cnt);
                self.low.insert(a, cnt);
                self.neighbors.insert(a, Box::new(graph.neighbors(a)));
            }

            if self.color.get(&a) == Some(&Color::Gray) {
                if let Some(b) = (*self.neighbors.get_mut(&a).unwrap()).next() {
                    if !self.color.contains_key(&b) {
                        self.edges_stack.push((a, b));
                        self.nodes_stack.push(b);
                    } else if b != parent {
                        let low_a = *self.low.get(&a).unwrap();
                        let pre_b = *self.pre.get(&b).unwrap();
                        if low_a > pre_b {
                            self.low.insert(a, pre_b);
                        }
                    }
                } else {
                    self.color.insert(a, Color::Black);
                }
            } else {
                self.edges_stack.pop();
                // Check if its initial dummy edge
                if parent == a {
                    return if self.color.len() == 1 {
                        // singleton graph
                        Some(HashSet::from([a]))
                    } else {
                        None
                    };
                }
                let low_parent = *self.low.get(&parent).unwrap();
                let low_a = *self.low.get(&a).unwrap();
                let pre_parent = *self.pre.get(&parent).unwrap();
                if low_a < low_parent {
                    self.low.insert(parent, low_a);
                }
                if low_a >= pre_parent {
                    let mut component = HashSet::new();
                    while self.nodes_stack.last() != Some(&parent) {
                        component.insert(self.nodes_stack.pop().unwrap());
                    }
                    component.insert(parent);
                    return Some(component);
                }
            }
        }

        None
    }
}
